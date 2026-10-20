//! Body damage for combat/spells.
use super::{
    AttackResult, BodySpellResult, DamageFlags, DamagePacket, DirectSpell, FieldUsage, RandomState,
    RestrictionMemoryView, RuleError, monster_race_is_excluded, put_word,
    resolve_damage_with_amount, resolve_party, spell_conditions, validate_direct,
};
use centurions_content::combat::MonsterRecord;
use centurions_content::{Warrior, word};

pub fn resolve_bodies(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    random: &mut RandomState,
) -> Result<BodySpellResult, RuleError> {
    resolve_bodies_inner(warriors, enemies, mask, context, None, None, random)
}

pub fn resolve_bodies_with_item_cache(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    item_cache: &[u8],
    random: &mut RandomState,
) -> Result<BodySpellResult, RuleError> {
    resolve_bodies_inner(
        warriors,
        enemies,
        mask,
        context,
        None,
        Some(RestrictionMemoryView::item_cache_only(item_cache)),
        random,
    )
}

pub fn resolve_bodies_with_memory(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    memory: RestrictionMemoryView<'_>,
    random: &mut RandomState,
) -> Result<BodySpellResult, RuleError> {
    resolve_bodies_inner(warriors, enemies, mask, context, None, Some(memory), random)
}

pub fn resolve_collision(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    usage: &mut FieldUsage,
    random: &mut RandomState,
) -> Result<BodySpellResult, RuleError> {
    resolve_collision_inner(warriors, enemies, mask, context, usage, None, random)
}

pub fn resolve_collision_with_item_cache(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    usage: &mut FieldUsage,
    item_cache: &[u8],
    random: &mut RandomState,
) -> Result<BodySpellResult, RuleError> {
    resolve_collision_inner(
        warriors,
        enemies,
        mask,
        context,
        usage,
        Some(RestrictionMemoryView::item_cache_only(item_cache)),
        random,
    )
}

pub(super) fn resolve_collision_inner(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    usage: &mut FieldUsage,
    memory: Option<RestrictionMemoryView<'_>>,
    random: &mut RandomState,
) -> Result<BodySpellResult, RuleError> {
    if usage.slot >= 60 || usage.used.len() != 128 {
        return Err(RuleError::Invalid("spell field collision usage".into()));
    }
    let mut next_usage = FieldUsage {
        slot: usage.slot,
        used: usage.used.clone(),
    };
    let result = resolve_bodies_inner(
        warriors,
        enemies,
        mask,
        context,
        Some(&mut next_usage),
        memory,
        random,
    )?;
    *usage = next_usage;
    Ok(result)
}

pub(super) fn resolve_bodies_inner(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    mut usage: Option<&mut FieldUsage>,
    memory: Option<RestrictionMemoryView<'_>>,
    random: &mut RandomState,
) -> Result<BodySpellResult, RuleError> {
    let DirectSpell {
        packet,
        level,
        count,
        armor_filter,
        weapon_sound,
    } = *context;
    if warriors.len() > 6 || enemies.len() > 100 || mask.len() < 10 + enemies.len() {
        return Err(RuleError::Invalid("spell body inputs".into()));
    }
    if word(packet, 40)? == 36 {
        if usage.is_some() {
            return Err(RuleError::Unsupported {
                action: "combat.spell.collision",
                dependency: "lingering condition effects",
            });
        }
        let outcome =
            spell_conditions::resolve_1703(warriors, enemies, packet, count, mask, random)?;
        let targets = (0..warriors.len())
            .chain(10..10 + enemies.len())
            .filter(|body| mask[*body] > 0)
            .map(|body| {
                (
                    body,
                    AttackResult {
                        feedback: None,
                        damage: 0,
                        killed: false,
                        sounds: vec![word(packet, 38).unwrap_or(0)],
                    },
                )
            })
            .collect();
        return Ok(BodySpellResult {
            warriors: outcome.warriors,
            enemies: outcome.enemies,
            sounds: vec![],
            targets,
            race_excluded: vec![],
        });
    }
    validate_direct(packet)?;
    let poison = word(packet, 40)? == 4;
    if poison && mask[6..10].iter().any(|flag| *flag > 0) {
        return Err(RuleError::Unsupported {
            action: "combat.spell",
            dependency: "poison target outside the represented body records",
        });
    }
    let mut excluded_monsters = vec![false; enemies.len()];
    if word(packet, 50)? != 0 {
        for (body, flag) in mask.iter().enumerate().skip(10) {
            if *flag <= 0 {
                continue;
            }
            let excluded = excluded_monsters
                .get_mut(body - 10)
                .ok_or(RuleError::Unsupported {
                    action: "combat.spell",
                    dependency: "restricted monster target outside the current records",
                })?;
            *excluded = monster_race_is_excluded(packet, body, memory)?;
        }
    }
    let mut next_random = random.clone();
    let mut amount = 0;
    let (party, duration) = resolve_party(
        warriors,
        mask,
        context,
        usage.as_deref_mut(),
        &mut amount,
        &mut next_random,
    )?;
    let mut result = BodySpellResult {
        warriors: party.warriors,
        enemies: enemies.to_vec(),
        sounds: if matches!(packet[1], 9 | 10) {
            vec![6001]
        } else {
            vec![]
        },
        targets: party.targets,
        race_excluded: party.race_excluded,
    };
    for (index, enemy) in result.enemies.iter_mut().enumerate() {
        if mask[index + 10] <= 0 {
            continue;
        }
        let m = &mut enemy.bytes;
        centurions_content::span(m, 0, 288)?;
        if excluded_monsters[index] {
            result.race_excluded.push(index + 10);
            continue;
        }
        if !armor_filter {
            if packet[19] == 0 {
                let threshold = word(m, 0)?
                    .wrapping_mul(4)
                    .wrapping_add(if m[0xab] != 0 { 1035 } else { 35 })
                    .wrapping_add(i16::from(packet[8] as i8).wrapping_mul(count));
                if next_random.bound(100) <= threshold {
                    continue;
                }
            }
            // Categories above six skip both indexed and general NPC immunity checks.
            if packet[19] <= 6
                && (m[0x8b + usize::from(packet[19])] as i8 >= 101 || word(m, 0xa)? >= 101)
            {
                continue;
            }
            if matches!(packet[9], 0 | 2) {
                if matches!(packet[19], 0 | 5) && m[0xcb] != 0 {
                    continue;
                }
                let immune =
                    (level..7).any(|school_level| m[0xb2 + school_level as usize + 16] != 0);
                let threshold =
                    word(m, 0xa)?.wrapping_add(i16::from(packet[10] as i8).wrapping_mul(count));
                if next_random.bound(100) <= threshold || immune {
                    continue;
                }
            }
        }
        if let Some(usage) = usage.as_deref_mut() {
            if usage.used[usage.slot + 1] != 0 {
                continue;
            }
            usage.used[usage.slot + 1] = 1;
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
        amount = damage.total;
        let mut sounds = vec![];
        if damage.shield_sound {
            sounds.push(206);
        }
        if poison || word(packet, 40)? == 23 {
            spell_conditions::apply(m, true, word(packet, 40)?, duration, &mut next_random)?;
            if poison || damage.total == 0 {
                sounds.push(word(packet, 38)?);
            }
        }
        let previous = word(m, 0x110)?;
        if damage.total != 0 {
            if damage.total < 0 {
                return Err(RuleError::Unsupported {
                    action: "combat.spell",
                    dependency: "spell healing",
                });
            }
            sounds.push(word(packet, 38)?);
            m[0x105] = 1;
            put_word(m, 0x110, previous.wrapping_sub(damage.total));
            if word(m, 0x110)? > 0 {
                sounds.push(weapon_sound);
            }
        }
        result.targets.push((
            index + 10,
            AttackResult {
                feedback: None,
                damage: damage.total,
                killed: word(m, 0x110)? < 1 && damage.total != 0,
                sounds,
            },
        ));
    }
    *random = next_random;
    Ok(result)
}
