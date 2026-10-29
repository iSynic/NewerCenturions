//! Support rules for combat/spells.
use super::{
    AttackResult, BodySpellResult, DamageFlags, DamagePacket, DirectSpell, FieldUsage, RandomState,
    RestrictionMemoryView, RuleError, monster_race_is_excluded, race_is_excluded,
    resolve_damage_with_amount,
};
use centurions_content::combat::MonsterRecord;
use centurions_content::{
    Warrior,
    records::{
        AttackPacket, Condition, EnemyRecord, EnemyRecordMut, SavingThrow, WarriorRecord,
        WarriorRecordMut,
    },
};

/// resist150432..150580 runs the mental save before the general save-mode gate.
fn mental_resisted(
    target: &[u8],
    monster: bool,
    packet: &[u8],
    count: i16,
    memory: Option<RestrictionMemoryView<'_>>,
    random: &mut RandomState,
) -> Result<bool, RuleError> {
    let category = AttackPacket(packet).defense_category();
    // resist150420: a negative animation byte other than -9 skips the category test.
    let animation = AttackPacket(packet).effect_animation();
    if category != 0 && (animation >= 0 || animation == -9) {
        return Ok(false);
    }
    let threshold = if category != 0 {
        let level = if monster {
            EnemyRecord::from_bytes(target).level()?
        } else {
            WarriorRecord::from_bytes(target).level()?
        };
        let caster = memory
            .and_then(RestrictionMemoryView::resist_caster_level)
            .ok_or(RuleError::Unsupported {
                action: "combat.spell",
                dependency: "negative effect animation reads a caster level outside the represented records",
            })?;
        level
            .wrapping_mul(5)
            .wrapping_add(35)
            .wrapping_sub(caster.wrapping_mul(5))
    } else if monster {
        let record = EnemyRecord::from_bytes(target);
        record.level()?.wrapping_mul(4).wrapping_add(
            if record.has_creature_category(EnemyRecord::MECHANICAL_CATEGORY) {
                1035
            } else {
                35
            },
        )
    } else {
        let record = WarriorRecord::from_bytes(target);
        record
            .saving_throw(SavingThrow::Mental)?
            .wrapping_add(if record.race()? == 3 { 1000 } else { 0 })
    };
    Ok(random.bound(100)
        <= threshold.wrapping_add(
            AttackPacket(packet)
                .save_bonus_per_attack()
                .wrapping_mul(count),
        ))
}

/// resist1501a0..15040e: category 9 replaces every save with the missile hit test.
fn physical_resisted(
    target: &[u8],
    monster: bool,
    memory: Option<RestrictionMemoryView<'_>>,
    random: &mut RandomState,
) -> Result<bool, RuleError> {
    let (mut hit, difficulty) = memory
        .and_then(RestrictionMemoryView::physical_accuracy)
        .ok_or(RuleError::Unsupported {
            action: "combat.spell",
            dependency: "category-9 resist reads the accuracy of an inactive warrior or unloaded NPC",
        })?;
    if monster {
        let record = EnemyRecord::from_bytes(target);
        if record.turn_skip_duration() != 0 {
            return Ok(false);
        }
        let level = record.level()?;
        hit = hit
            .wrapping_sub(record.missile_dodge_bonus()?)
            .wrapping_sub(random.bound(record.speed()? / 3))
            .wrapping_sub(level.wrapping_mul(3))
            .wrapping_sub(i16::from(record.guarding()).wrapping_mul(level))
            .wrapping_add(match record.size()?.wrapping_abs() {
                0 => -5,
                1 | 2 => 5,
                3 => 10,
                _ => 0,
            });
    } else {
        let record = WarriorRecord::from_bytes(target);
        if record.turn_skip_duration()? != 0 {
            return Ok(false);
        }
        hit = hit
            .wrapping_sub(record.missile_dodge()?)
            .wrapping_sub(5)
            .wrapping_sub(random.bound(record.quickness()? / 3))
            .wrapping_sub(record.guarding()?.wrapping_mul(record.level()?));
    }
    let hit = crate::rules::pin(
        hit,
        15_i16.wrapping_sub(difficulty.wrapping_mul(5)),
        90_i16.wrapping_sub(difficulty.wrapping_mul(5)),
    );
    Ok(random.bound(100) > hit)
}

/// resist's ordered saves for condition and healing packets.
pub(super) fn status_resisted(
    target: &[u8],
    monster: bool,
    context: &DirectSpell<'_>,
    memory: Option<RestrictionMemoryView<'_>>,
    random: &mut RandomState,
) -> Result<bool, RuleError> {
    status_resisted_with_feedback(target, monster, context, memory, random, &mut false)
}

pub(super) fn status_resisted_with_feedback(
    target: &[u8],
    monster: bool,
    context: &DirectSpell<'_>,
    memory: Option<RestrictionMemoryView<'_>>,
    random: &mut RandomState,
    mental_feedback: &mut bool,
) -> Result<bool, RuleError> {
    *mental_feedback = false;
    if context.armor_filter {
        return Ok(false);
    }
    if AttackPacket(context.packet).defense_category() == 9 {
        return physical_resisted(target, monster, memory, random);
    }
    if mental_resisted(
        target,
        monster,
        context.packet,
        context.count,
        memory,
        random,
    )? {
        *mental_feedback = true;
        return Ok(true);
    }
    // resist150588 compares the category as a signed BYTE; a negative one indexes
    // the NPC save bytes backwards, still inside the record.
    let category = AttackPacket(context.packet).defense_category() as i8;
    if category <= 6
        && monster
        && (target[(0x8b + isize::from(category)) as usize] as i8 > 100
            || EnemyRecord::from_bytes(target).magic_immunity()? > 100)
    {
        return Ok(true);
    }
    // The mode is signed too: only 1 and 3..127 skip the save, except category -9.
    let mode = AttackPacket(context.packet).saving_throw_mode() as i8;
    if (mode == 1 || mode > 2) && category != -9 {
        return Ok(false);
    }
    if matches!(AttackPacket(context.packet).defense_category(), 0 | 5)
        && if monster {
            EnemyRecord::from_bytes(target).condition(Condition::ComputerControl) != 0
        } else {
            WarriorRecord::from_bytes(target).automatic_mode()? != 0
        }
    {
        return Ok(true);
    }
    let mut immune = false;
    for level in context.level..7 {
        let protected = if monster {
            EnemyRecord::from_bytes(target)
                .signed_level_protection(level)
                .map(i16::from)
        } else {
            WarriorRecord::from_bytes(target).signed_level_protection(level)
        }
        // resolve_support_inner admits levels 0..6, so 0x7e + 2 * level stays inside.
        .map_err(|_| RuleError::Invalid("signed level protection level".into()))?;
        immune |= protected != 0;
    }
    let resistance = if monster {
        EnemyRecord::from_bytes(target).magic_immunity()?
    } else {
        WarriorRecord::from_bytes(target).base_psionic_resistance()?
    };
    let threshold = resistance.wrapping_add(
        AttackPacket(context.packet)
            .resistance_bonus_per_attack()
            .wrapping_mul(context.count),
    );
    let roll = random.bound(100);
    Ok(if monster {
        roll <= threshold || immune
    } else {
        roll > threshold && immune
    })
}

/// resolvespell151022 and spelllist15bd50 retain the original Psi-drain ordering.
/// Party targets clear the amount before Siphon's transfer; NPC targets transfer
/// the negative half before clearing it. Neither branch applies stamina damage.
/// `caster` is the current initiative body; a collision passes its usage bytes.
#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_psi_drain(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    caster: i16,
    (memory, mut usage): (RestrictionMemoryView<'_>, Option<&mut FieldUsage>),
    random: &mut RandomState,
) -> Result<BodySpellResult, RuleError> {
    let packet = context.packet;
    if packet.len() != 58
        || warriors.len() > 6
        || enemies.len() > 100
        || mask.len() < 10 + enemies.len()
        || !(1..=7).contains(&context.count)
        || !(0..7).contains(&context.level)
    {
        return Err(RuleError::Invalid("Psi drain inputs".into()));
    }
    let effect = AttackPacket(packet).effect()?;
    // Drains in other categories are support packets (is_support).
    if !matches!(effect, 60 | 61) || AttackPacket(packet).defense_category() != 5 {
        return Err(RuleError::Invalid("not a category-5 Psi drain".into()));
    }
    let caster =
        usize::try_from(caster).map_err(|_| RuleError::Invalid("negative Psi caster".into()))?;
    if caster >= warriors.len() && caster < 10
        || caster >= 10 + enemies.len()
        || warriors.iter().any(|record| record.bytes.len() != 3138)
        || enemies.iter().any(|record| record.bytes.len() != 288)
        || mask[warriors.len()..10].iter().any(|flag| *flag > 0)
    {
        return Err(RuleError::Invalid("Psi drain body records".into()));
    }
    let mut next_random = random.clone();
    let mut result = BodySpellResult {
        total_delta: 0,
        warriors: warriors.to_vec(),
        enemies: enemies.to_vec(),
        sounds: vec![],
        targets: vec![],
        race_excluded: vec![],
    };
    let (minimum, maximum) = AttackPacket(packet).duration_bounds();
    next_random.range(minimum, maximum);
    let (additional_minimum, additional_maximum) =
        AttackPacket(packet).additional_duration_bounds();
    if additional_minimum != 0 {
        for _ in 0..context.count {
            next_random.range(additional_minimum, additional_maximum);
        }
    }
    for body in (0..warriors.len()).chain(10..10 + enemies.len()) {
        if mask[body] <= 0 {
            continue;
        }
        let monster = body >= 10;
        let excluded = if monster {
            AttackPacket(packet).race_restriction()? != 0
                && monster_race_is_excluded(packet, body, Some(memory))?
        } else {
            race_is_excluded(packet, &result.warriors[body].bytes, Some(memory))?
        };
        if excluded {
            result.race_excluded.push(body);
            continue;
        }
        let target = spell_body(&mut result.warriors, &mut result.enemies, body)?;
        if status_resisted(target, monster, context, Some(memory), &mut next_random)? {
            continue;
        }
        // Resistance precedes collide's usage byte WORD4358c2, as in the body loop.
        if let Some(usage) = usage.as_deref_mut() {
            if usage.used[usage.slot + 1] != 0 {
                continue;
            }
            usage.used[usage.slot + 1] = 1;
        }
        let damage = resolve_damage_with_amount(
            target,
            monster,
            &DamagePacket {
                bytes: packet.to_vec(),
                attack_count: context.count,
            },
            DamageFlags {
                shield: true,
                armor: true,
                saves: true,
                armor_filter: context.armor_filter,
            },
            0,
            &mut next_random,
        )?;
        let reflected = !monster
            && WarriorRecord::from_bytes(target).condition(Condition::ReflectingScans)? != 0
            && next_random.bound(100) < 34;
        let actual = if reflected { caster } else { body };
        let target = spell_body(&mut result.warriors, &mut result.enemies, actual)?;
        let amount = damage.total.wrapping_neg();
        let mut points = psi_points(target, actual >= 10)?
            .wrapping_add(amount)
            .max(0);
        if actual < 10 {
            points = points.min(WarriorRecord::from_bytes(target).maximum_psi()?);
        }
        set_psi_points(target, actual >= 10, points);
        if effect == 61 && actual >= 10 {
            // 15bb0c: a party initiative body credits WORD4435d2; an NPC one credits
            // WORD447196, which live NPC turns hold at the acting NPC.
            let recipient = if caster < 9 {
                memory
                    .selected_warrior()
                    .map(usize::try_from)
                    .transpose()
                    .map_err(|_| RuleError::Invalid("negative Siphon recipient".into()))?
                    .unwrap_or(caster)
            } else {
                caster
            };
            let target = spell_body(&mut result.warriors, &mut result.enemies, recipient)?;
            let points = psi_points(target, recipient >= 10)?.wrapping_add(amount / 2);
            set_psi_points(target, recipient >= 10, points);
        }
        result.targets.push((
            body,
            AttackResult {
                icon: 0,
                feedback: None,
                damage: 0,
                killed: false,
                sounds: if damage.shield_sound {
                    vec![206]
                } else {
                    vec![]
                },
            },
        ));
    }
    *random = next_random;
    Ok(result)
}

pub(super) fn spell_body<'a>(
    warriors: &'a mut [Warrior],
    enemies: &'a mut [MonsterRecord],
    body: usize,
) -> Result<&'a mut [u8], RuleError> {
    if body < 10 {
        Ok(&mut warriors
            .get_mut(body)
            .ok_or("Psi party body missing")?
            .bytes)
    } else {
        Ok(&mut enemies
            .get_mut(body - 10)
            .ok_or("Psi enemy body missing")?
            .bytes)
    }
}

fn psi_points(target: &[u8], enemy: bool) -> Result<i16, RuleError> {
    Ok(if enemy {
        EnemyRecord::from_bytes(target).psi_points()?
    } else {
        WarriorRecord::from_bytes(target).psi_points()?
    })
}

fn set_psi_points(target: &mut [u8], enemy: bool, points: i16) {
    if enemy {
        EnemyRecordMut::from_bytes(target).set_psi_points(points);
    } else {
        WarriorRecordMut::from_bytes(target).set_psi_points(points);
    }
}
