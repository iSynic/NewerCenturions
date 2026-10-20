//! Party damage for combat/spells.
use super::{
    AttackResult, DamageFlags, DamagePacket, DirectSpell, FieldUsage, RandomState, RuleError,
    WarriorSpellResult, put_word, race_is_excluded, resolve_damage_with_amount, spell_conditions,
    validate_direct,
};
use centurions_content::{Warrior, word};

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
    let poison = word(packet, 40)? == 4;
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
    let mut duration = next_random.range(i16::from(packet[13] as i8), i16::from(packet[14] as i8));
    if packet[15] != 0 {
        for _ in 0..count {
            duration = duration.wrapping_add(
                next_random.range(i16::from(packet[15] as i8), i16::from(packet[16] as i8)),
            );
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
        if !armor_filter && matches!(packet[9], 0 | 2) {
            if packet[19] == 0 {
                let threshold = word(w, 0x112)?
                    .wrapping_add(if word(w, 0x32)? == 3 { 1000 } else { 0 })
                    .wrapping_add(i16::from(packet[8] as i8).wrapping_mul(count));
                if next_random.bound(100) <= threshold {
                    continue;
                }
            }
            if matches!(packet[19], 0 | 5) && word(w, 0x90)? != 0 {
                continue;
            }
            let immune = (level..7).any(|school_level| {
                word(w, 0x5e + (school_level as usize + 16) * 2).unwrap_or(0) != 0
            });
            let threshold =
                word(w, 0x22)?.wrapping_add(i16::from(packet[10] as i8).wrapping_mul(count));
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
        if poison || word(packet, 40)? == 23 {
            spell_conditions::apply(w, false, word(packet, 40)?, duration, &mut next_random)?;
            if poison || damage.total == 0 {
                sounds.push(word(packet, 38)?);
            }
        }
        let previous = word(w, 0x50)?;
        if damage.total != 0 && previous > -10 {
            sounds.push(word(packet, 38)?);
            if damage.total < 0 {
                return Err(RuleError::Unsupported {
                    action: "combat.spell",
                    dependency: "spell healing",
                });
            }
            if previous >= 0 {
                put_word(w, 0x50, previous.wrapping_sub(damage.total));
                if damage.total > 0 {
                    put_word(w, 0x14, 1);
                }
            }
        }
        result.targets.push((
            member,
            AttackResult {
                feedback: None,
                damage: damage.total,
                killed: previous > 0 && word(w, 0x50)? < 1,
                sounds,
            },
        ));
    }
    *random = next_random;
    Ok((result, duration))
}
