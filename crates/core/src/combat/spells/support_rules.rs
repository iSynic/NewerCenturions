//! Support rules for combat/spells.
use super::{
    AttackResult, BodySpellResult, DamageFlags, DamagePacket, DirectSpell, RandomState,
    RestrictionMemoryView, RuleError, monster_race_is_excluded, race_is_excluded,
    resolve_damage_with_amount,
};
use centurions_content::combat::MonsterRecord;
use centurions_content::{
    Warrior,
    records::{
        AttackPacket, Condition, EnemyRecord, EnemyRecordMut, WarriorRecord, WarriorRecordMut,
    },
};

/// resist150588..150742 for nonphysical status categories7,8,10.
pub(super) fn status_resisted(
    target: &[u8],
    monster: bool,
    context: &DirectSpell<'_>,
    random: &mut RandomState,
) -> Result<bool, RuleError> {
    if context.armor_filter || !matches!(AttackPacket(context.packet).saving_throw_mode(), 0 | 2) {
        return Ok(false);
    }
    if AttackPacket(context.packet).defense_category() <= 6
        && monster
        && (EnemyRecord::from_bytes(target)
            .saving_throw(usize::from(AttackPacket(context.packet).defense_category()))
            > 100
            || EnemyRecord::from_bytes(target).magic_immunity()? > 100)
    {
        return Ok(true);
    }
    if AttackPacket(context.packet).defense_category() == 5
        && if monster {
            EnemyRecord::from_bytes(target).condition(Condition::ComputerControl) != 0
        } else {
            WarriorRecord::from_bytes(target).automatic_mode()? != 0
        }
    {
        return Ok(true);
    }
    let immune = (context.level..7).any(|level| {
        if monster {
            EnemyRecord::from_bytes(target).level_protection(level as usize) != 0
        } else {
            WarriorRecord::from_bytes(target)
                .level_protection(level as usize)
                .unwrap_or(1)
                != 0
        }
    });
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
#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_psi_drain(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    caster: i16,
    memory: RestrictionMemoryView<'_>,
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
    if !matches!(effect, 60 | 61) || AttackPacket(packet).defense_category() != 5 {
        return Err(RuleError::Unsupported {
            action: "combat.spell.drain",
            dependency: "other Psi-drain packet category",
        });
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
            race_is_excluded(packet, &result.warriors[body].bytes, "combat.spell.drain")?
        };
        if excluded {
            result.race_excluded.push(body);
            continue;
        }
        let target = spell_body(&mut result.warriors, &mut result.enemies, body)?;
        if status_resisted(target, monster, context, &mut next_random)? {
            continue;
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
            let target = spell_body(&mut result.warriors, &mut result.enemies, caster)?;
            let points = psi_points(target, caster >= 10)?.wrapping_add(amount / 2);
            set_psi_points(target, caster >= 10, points);
        }
        result.targets.push((
            body,
            AttackResult {
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
