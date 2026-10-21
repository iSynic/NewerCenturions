//! Support for combat/spells.
use super::{
    AttackResult, DamageFlags, DamagePacket, DirectSpell, RandomState, RestrictionMemoryView,
    RuleError, SupportSpellResult, monster_race_is_excluded, put_word, race_is_excluded,
    resolve_damage_with_amount, spell_conditions, status_resisted,
};
use centurions_content::combat::MonsterRecord;
use centurions_content::{Warrior, word};

pub fn resolve_player_support(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    packet: &[u8],
    id: i16,
    count: i16,
    random: &mut RandomState,
) -> Result<SupportSpellResult, RuleError> {
    resolve_support_with_context(
        warriors,
        enemies,
        mask,
        id,
        &DirectSpell {
            packet,
            level: id % 1000 / 100 - 1,
            count,
            armor_filter: false,
            weapon_sound: 0,
        },
        random,
    )
}

pub fn resolve_support_with_context(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    id: i16,
    context: &DirectSpell<'_>,
    random: &mut RandomState,
) -> Result<SupportSpellResult, RuleError> {
    resolve_support_for_caster(warriors, enemies, mask, id, context, (None, None), random)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_support_for_caster(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    _id: i16,
    context: &DirectSpell<'_>,
    caller: (Option<i16>, Option<RestrictionMemoryView<'_>>),
    random: &mut RandomState,
) -> Result<SupportSpellResult, RuleError> {
    let (caster, memory) = caller;
    let DirectSpell {
        packet,
        count,
        armor_filter,
        weapon_sound,
        ..
    } = *context;
    if !(1..=6).contains(&warriors.len())
        || enemies.len() > 100
        || mask.len() < 10 + enemies.len()
        || packet.len() != 58
        || !matches!(count, 1..=7)
        || !(0..7).contains(&context.level)
    {
        return Err(RuleError::Invalid("support spell inputs".into()));
    }
    let effect = word(packet, 40)?;
    if !(matches!(effect, 0..=41 | 51 | 57 | 64))
        || packet[9] > 3
        || !matches!(packet[19], 7 | 8 | 10)
        || (packet[20] as i8) < 0
        || word(packet, 34)? != 0
        || packet[23..30].iter().any(|v| *v != 0)
    {
        return Err(RuleError::Unsupported {
            action: "combat.spell.support",
            dependency: "other support packet",
        });
    }
    for warrior in warriors {
        centurions_content::span(&warrior.bytes, 0, 3138)?;
    }
    let mut next_random = random.clone();
    let mut amount = 0;
    let mut duration = next_random.range(i16::from(packet[13] as i8), i16::from(packet[14] as i8));
    if packet[15] != 0 {
        for _ in 0..count {
            duration = duration.wrapping_add(
                next_random.range(i16::from(packet[15] as i8), i16::from(packet[16] as i8)),
            );
        }
    }
    let mut result = SupportSpellResult {
        warriors: warriors.to_vec(),
        enemies: enemies.to_vec(),
        sounds: if matches!(packet[1], 9 | 10) {
            vec![6001]
        } else {
            vec![]
        },
        targets: vec![],
        healed: false,
        recovered: 0,
    };
    for (member, selected) in mask.iter().enumerate().take(result.warriors.len()) {
        if *selected <= 0 {
            continue;
        }
        let w = &mut result.warriors[member].bytes;
        if race_is_excluded(packet, w, "combat.spell.support")? {
            result.sounds.push(651);
            continue;
        }
        if status_resisted(w, false, context, &mut next_random)? {
            continue;
        }
        let damage = resolve_damage_with_amount(
            w,
            false,
            &DamagePacket {
                bytes: packet.to_vec(),
                attack_count: count,
            },
            DamageFlags {
                shield: true,
                armor: true,
                saves: true,
                armor_filter,
            },
            amount,
            &mut next_random,
        )?
        .total;
        let damage = if effect == 28 { duration } else { damage };
        amount = damage;
        let mut sounds = vec![];
        let mut follow_up = None;
        // Authored effect0 has no spelllist work; effect41 returns presentation only.
        if matches!(effect, 0 | 41) {
            if effect == 41 && word(w, 0x9a)? != 0 {
                next_random.bound(100);
            }
            result.targets.push((
                member,
                AttackResult {
                    feedback: None,
                    damage: 0,
                    killed: false,
                    sounds: if effect == 41 {
                        vec![word(packet, 38)?]
                    } else {
                        vec![]
                    },
                },
            ));
            continue;
        }
        if matches!(effect, 51 | 64) {
            let target = if word(w, 0x9a)? != 0 && next_random.bound(100) < 34 {
                usize::try_from(caster.unwrap_or(member as i16))
                    .map_err(|_| RuleError::Invalid("negative special-effect caster".into()))?
            } else {
                member
            };
            if effect == 51 {
                let caster = usize::try_from(caster.ok_or("charm caller missing")?)
                    .map_err(|_| RuleError::Invalid("negative charm caster".into()))?;
                let delta = spell_conditions::change_allegiance(
                    &mut result.warriors,
                    &mut result.enemies,
                    target,
                    caster,
                )?;
                result.recovered = result.recovered.wrapping_sub(delta as u8);
            } else if target < result.warriors.len() {
                spell_conditions::follow_up(&mut result.warriors[target].bytes, false, effect)?;
            } else {
                return Err(RuleError::Unsupported {
                    action: "combat.spell.support",
                    dependency: "revival caller outside the party",
                });
            }
            result.targets.push((
                member,
                AttackResult {
                    feedback: None,
                    damage: 0,
                    killed: false,
                    sounds: vec![word(packet, 38)?],
                },
            ));
            continue;
        }
        let mut healing_death = false;
        if (1..=40).contains(&effect) {
            spell_conditions::add_duration(w, false, effect, duration)?;
            let reflected = word(w, 0x9a)? != 0 && next_random.bound(100) < 34;
            follow_up = Some(if reflected {
                caster.unwrap_or(member as i16)
            } else {
                member as i16
            });
            if damage == 0 {
                sounds.push(word(packet, 38)?);
            }
        } else {
            let previous = word(w, 0x50)?;
            if damage < 0 && previous > -10 {
                let after = previous
                    .wrapping_add(damage.wrapping_abs())
                    .max(-10)
                    .min(word(w, 0x52)?);
                // heal1287c2 calls killbody after the signed WORD addition wraps.
                healing_death = previous > 0 && after < 1;
                if previous <= 0 && after > 0 {
                    result.recovered = result.recovered.wrapping_add(1);
                }
                if previous <= 0 && after < -9 {
                    sounds.push(next_random.bound(2).wrapping_add(506));
                }
                put_word(w, 0x50, after);
                sounds.insert(0, word(packet, 38)?);
                result.healed = true;
            }
        }
        let previous = word(w, 0x50)?;
        if effect == 28 && damage > 0 && previous >= 0 {
            put_word(w, 0x50, previous.wrapping_sub(damage));
            put_word(w, 0x14, 1);
            sounds.push(word(packet, 38)?);
        }
        result.targets.push((
            member,
            AttackResult {
                feedback: None,
                damage,
                killed: healing_death || (effect == 28 && previous > 0 && word(w, 0x50)? < 1),
                sounds,
            },
        ));
        if let Some(body) = follow_up {
            if body < 10 {
                let target = result
                    .warriors
                    .get_mut(
                        usize::try_from(body)
                            .map_err(|_| RuleError::Invalid("negative condition body".into()))?,
                    )
                    .ok_or("condition body outside party")?;
                spell_conditions::follow_up(&mut target.bytes, false, effect)?;
            } else {
                let target = result
                    .enemies
                    .get_mut((body - 10) as usize)
                    .ok_or("condition body outside enemies")?;
                spell_conditions::follow_up(&mut target.bytes, true, effect)?;
            }
        }
    }
    for index in 0..result.enemies.len() {
        if mask[index + 10] <= 0 {
            continue;
        }
        if matches!(effect, 0 | 41 | 51)
            && word(packet, 50)? != 0
            && monster_race_is_excluded(packet, index + 10, memory)?
        {
            result.sounds.push(651);
            continue;
        }
        let m = &mut result.enemies[index].bytes;
        centurions_content::span(m, 0, 288)?;
        if status_resisted(m, true, context, &mut next_random)? {
            continue;
        }
        let damage = resolve_damage_with_amount(
            m,
            true,
            &DamagePacket {
                bytes: packet.to_vec(),
                attack_count: count,
            },
            DamageFlags {
                shield: true,
                armor: true,
                saves: true,
                armor_filter,
            },
            amount,
            &mut next_random,
        )?;
        let total = if effect == 28 { duration } else { damage.total };
        amount = total;
        let mut sounds = if damage.shield_sound {
            vec![206]
        } else {
            vec![]
        };
        if matches!(effect, 0 | 41) {
            if effect == 41 {
                sounds.push(word(packet, 38)?);
            }
            result.targets.push((
                index + 10,
                AttackResult {
                    feedback: None,
                    damage: 0,
                    killed: false,
                    sounds,
                },
            ));
            continue;
        }
        if effect == 51 {
            let caster = usize::try_from(caster.ok_or("charm caller missing")?)
                .map_err(|_| RuleError::Invalid("negative charm caster".into()))?;
            spell_conditions::change_allegiance(
                &mut result.warriors,
                &mut result.enemies,
                index + 10,
                caster,
            )?;
            result.targets.push((
                index + 10,
                AttackResult {
                    feedback: None,
                    damage: 0,
                    killed: false,
                    sounds: vec![word(packet, 38)?],
                },
            ));
            continue;
        }
        if effect == 64 {
            return Err(RuleError::Unsupported {
                action: "combat.spell.support",
                dependency: "revival caller outside the party",
            });
        }
        if (1..=40).contains(&effect) {
            spell_conditions::apply(m, true, effect, duration, &mut next_random)?;
            if total == 0 {
                sounds.push(word(packet, 38)?);
            }
        }
        if damage.total < 0 && word(m, 0x110)? > 0 {
            let health = word(m, 0x110)?.wrapping_sub(damage.total);
            put_word(m, 0x110, health);
            m[0x105] = 1;
            sounds.push(word(packet, 38)?);
            sounds.push(weapon_sound);
            sounds.push(word(packet, 38)?);
        }
        if effect == 28 && total != 0 {
            let health = word(m, 0x110)?.wrapping_sub(total);
            put_word(m, 0x110, health);
            m[0x105] = 1;
            sounds.push(word(packet, 38)?);
            if health > 0 {
                sounds.push(weapon_sound);
            }
        }
        result.targets.push((
            index + 10,
            AttackResult {
                feedback: None,
                damage: total,
                killed: effect == 28 && total != 0 && word(m, 0x110)? < 1,
                sounds,
            },
        ));
    }
    *random = next_random;
    Ok(result)
}
