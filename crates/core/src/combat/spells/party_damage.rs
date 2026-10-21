//! Party damage for combat/spells.
use super::{
    AttackResult, DamageFlags, DamagePacket, DirectSpell, FieldUsage, RandomState, RuleError,
    WarriorSpellResult, race_is_excluded, resolve_damage_with_amount, spell_conditions,
    validate_direct,
};
use centurions_content::{
    Warrior,
    records::{AttackPacket, Condition, SavingThrow, WarriorRecord, WarriorRecordMut},
};

pub fn resolve_warriors(
    warriors: &[Warrior],
    packet: &[u8],
    level: i16,
    count: i16,
    mask: &[i16],
    armor_filter: bool,
    random: &mut RandomState,
) -> Result<WarriorSpellResult, RuleError> {
    resolve_party(
        warriors,
        mask,
        &DirectSpell {
            packet,
            level,
            count,
            armor_filter,
            weapon_sound: 0,
        },
        None,
        &mut 0,
        random,
    )
    .map(|(party, _)| party)
}

pub(super) fn resolve_party(
    warriors: &[Warrior],
    mask: &[i16],
    context: &DirectSpell<'_>,
    mut usage: Option<&mut FieldUsage>,
    amount: &mut i16,
    random: &mut RandomState,
) -> Result<(WarriorSpellResult, i16), RuleError> {
    let DirectSpell {
        packet,
        level,
        count,
        armor_filter,
        ..
    } = *context;
    validate_direct(packet)?;
    if mask.len() < warriors.len() || !(0..7).contains(&level) || count < 0 {
        return Err(RuleError::Invalid("spell resolution inputs".into()));
    }
    let poison = AttackPacket(packet).effect()? == 4;
    if poison && (!(1..=7).contains(&count) || usage.is_some()) {
        return Err(RuleError::Unsupported {
            action: "combat.spell",
            dependency: "other poison count, defense or field collision",
        });
    }
    let mut next_random = random.clone();
    let mut result = WarriorSpellResult {
        warriors: warriors.to_vec(),
        targets: vec![],
        race_excluded: vec![],
    };
    let (minimum, maximum) = AttackPacket(packet).duration_bounds();
    let mut duration = next_random.range(minimum, maximum);
    let (additional_minimum, additional_maximum) =
        AttackPacket(packet).additional_duration_bounds();
    if additional_minimum != 0 {
        for _ in 0..count {
            duration =
                duration.wrapping_add(next_random.range(additional_minimum, additional_maximum));
        }
    }
    for (member, warrior) in result.warriors.iter_mut().enumerate() {
        if mask[member] <= 0 {
            continue;
        }
        let w = &mut warrior.bytes;
        if race_is_excluded(packet, w, "combat.spell")? {
            result.race_excluded.push(member);
            continue;
        }
        if !armor_filter && matches!(AttackPacket(packet).saving_throw_mode(), 0 | 2) {
            if AttackPacket(packet).defense_category() == 0 {
                let threshold = WarriorRecord::from_bytes(w)
                    .saving_throw(SavingThrow::Mental)?
                    .wrapping_add(if WarriorRecord::from_bytes(w).race()? == 3 {
                        1000
                    } else {
                        0
                    })
                    .wrapping_add(
                        AttackPacket(packet)
                            .save_bonus_per_attack()
                            .wrapping_mul(count),
                    );
                if next_random.bound(100) <= threshold {
                    continue;
                }
            }
            if matches!(AttackPacket(packet).defense_category(), 0 | 5)
                && WarriorRecord::from_bytes(w).condition(Condition::ComputerControl)? != 0
            {
                continue;
            }
            let immune = (level..7).any(|school_level| {
                WarriorRecord::from_bytes(w)
                    .level_protection(school_level as usize)
                    .unwrap_or(0)
                    != 0
            });
            let threshold = WarriorRecord::from_bytes(w)
                .base_psionic_resistance()?
                .wrapping_add(
                    AttackPacket(packet)
                        .resistance_bonus_per_attack()
                        .wrapping_mul(count),
                );
            if next_random.bound(100) > threshold && immune {
                continue;
            }
        }
        if let Some(usage) = usage.as_deref_mut() {
            if usage.used[usage.slot + 1] != 0 {
                continue;
            }
            usage.used[usage.slot + 1] = 1;
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
            *amount,
            &mut next_random,
        )?;
        *amount = damage.total;
        let mut sounds = vec![];
        if damage.shield_sound {
            sounds.push(206);
        }
        if poison || AttackPacket(packet).effect()? == 23 {
            spell_conditions::apply(
                w,
                false,
                AttackPacket(packet).effect()?,
                duration,
                &mut next_random,
            )?;
            if poison || damage.total == 0 {
                sounds.push(AttackPacket(packet).impact_sound()?);
            }
        }
        let previous = WarriorRecord::from_bytes(w).stamina()?;
        if damage.total != 0 && previous > -10 {
            sounds.push(AttackPacket(packet).impact_sound()?);
            if damage.total < 0 {
                return Err(RuleError::Unsupported {
                    action: "combat.spell",
                    dependency: "spell healing",
                });
            }
            if previous >= 0 {
                WarriorRecordMut::from_bytes(w).set_stamina(previous.wrapping_sub(damage.total));
                if damage.total > 0 {
                    WarriorRecordMut::from_bytes(w).mark_damaged();
                }
            }
        }
        result.targets.push((
            member,
            AttackResult {
                feedback: None,
                damage: damage.total,
                killed: previous > 0 && WarriorRecord::from_bytes(w).stamina()? < 1,
                sounds,
            },
        ));
    }
    *random = next_random;
    Ok((result, duration))
}
