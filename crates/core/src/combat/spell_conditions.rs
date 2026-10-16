use crate::{RandomState, RuleError};
use centurions_content::{Warrior, combat::MonsterRecord, span, word};

/// Recovered conditions retain signed additions at their original record widths.
pub fn apply(
    target: &mut [u8],
    monster: bool,
    effect: i16,
    duration: i16,
    random: &mut RandomState,
) -> Result<(), RuleError> {
    if target.len() != if monster { 288 } else { 3138 } {
        return Err(RuleError::Invalid("spell condition record size".into()));
    }
    if !(1..=40).contains(&effect) {
        return Err(RuleError::Unsupported {
            action: "combat.spell.condition",
            dependency: "spell condition outside recovered party and monster effects",
        });
    }
    add_duration(target, monster, effect, duration)?;
    if !monster && word(target, 0x9a)? != 0 {
        random.bound(100);
    }
    follow_up(target, monster, effect)
}

pub(crate) fn add_duration(
    target: &mut [u8],
    monster: bool,
    effect: i16,
    duration: i16,
) -> Result<(), RuleError> {
    if target.len() != if monster { 288 } else { 3138 } || !(1..=40).contains(&effect) {
        return Err(RuleError::Invalid("condition duration input".into()));
    }
    if monster {
        let offset = 0xb2 + (effect as usize - 1);
        let previous = i16::from(target[offset] as i8);
        let total = previous.wrapping_add(duration);
        if total.wrapping_abs() < 125 && previous >= 0 {
            target[offset] = (previous as u8).wrapping_add(duration as u8);
        }
    } else {
        let offset = 0x5e + (effect as usize - 1) * 2;
        let previous = word(target, offset)?;
        let total = previous.wrapping_add(duration);
        if total < 100 && previous >= 0 {
            target[offset..offset + 2].copy_from_slice(&total.to_be_bytes());
        }
    }
    Ok(())
}

/// spelllist15c8dc..15ca62 applies immediate effects after the duration writer.
pub(crate) fn follow_up(target: &mut [u8], monster: bool, effect: i16) -> Result<(), RuleError> {
    match effect {
        2 => {
            if monster {
                target[0x106] = 0;
            } else {
                target[0x3c..0x3e].fill(0);
            }
        }
        3 | 7 => {
            if monster {
                target[0x106] = ((target[0x106] as i8) / 2) as u8;
            } else {
                let movement = word(target, 0x3c)? / 2;
                target[0x3c..0x3e].copy_from_slice(&movement.to_be_bytes());
            }
        }
        10 => {
            if monster {
                if (target[0x90] as i8) > 100 || target[0xcb] != 0 {
                    target[0xb5] = 0;
                }
            } else if word(target, 0x90)? < 0 {
                target[0x64..0x66].fill(0);
            }
        }
        26 if !monster && word(target, 0x50)? < -9 => {
            target[0x90..0x92].copy_from_slice(&(-1_i16).to_be_bytes());
            let health = word(target, 0x52)? / 4;
            target[0x50..0x52].copy_from_slice(&health.to_be_bytes());
        }
        64 if !monster && (word(target, 0x50)? < -9 || word(target, 0x90)? != 0) => {
            target[0x90..0x92].fill(0);
            target[0x50..0x52].copy_from_slice(&(-9_i16).to_be_bytes());
        }
        _ => {}
    }
    Ok(())
}

/// spelllist 0x15c76a changes allegiance after the party reflection draw.
/// The returned delta belongs to byte442739, not the hostile-body cache4435c4.
pub(crate) fn change_allegiance(
    warriors: &mut [Warrior],
    enemies: &mut [MonsterRecord],
    target: usize,
    caster: usize,
) -> Result<i8, RuleError> {
    let side = if caster < 10 {
        word(
            &warriors.get(caster).ok_or("charm caster missing")?.bytes,
            0x10,
        )?
    } else {
        word(
            &enemies
                .get(caster - 10)
                .ok_or("charm caster missing")?
                .bytes,
            0xe,
        )?
    };
    let monster = target >= 10;
    let bytes = if monster {
        &mut enemies
            .get_mut(target - 10)
            .ok_or("charm target missing")?
            .bytes
    } else {
        &mut warriors
            .get_mut(target)
            .ok_or("charm target missing")?
            .bytes
    };
    let offset = if monster { 0xe } else { 0x10 };
    let previous = word(bytes, offset)?;
    if side != 0 || previous != 0 {
        if monster {
            bytes[0x10c..0x10e].copy_from_slice(&(-1_i16).to_be_bytes());
        } else if side != 0 && previous != side {
            bytes[0x18..0x1a].copy_from_slice(&(-1_i16).to_be_bytes());
        }
        bytes[offset..offset + 2].copy_from_slice(&side.to_be_bytes());
    }
    Ok(if monster {
        0
    } else {
        i8::from(previous == 0 && side != 0) - i8::from(previous != 0 && side == 0)
    })
}

pub struct ConditionOutcome {
    pub warriors: Vec<Warrior>,
    pub enemies: Vec<MonsterRecord>,
    pub duration: i16,
    pub sounds: Vec<i16>,
}

/// The authored 1703 packet bypasses resist saving throws and causes no damage.
pub fn resolve_1703(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    packet: &[u8],
    count: i16,
    mask: &[i16],
    random: &mut RandomState,
) -> Result<ConditionOutcome, RuleError> {
    if warriors.len() > 6 || enemies.len() > 100 || mask.len() < 10 + enemies.len() || count < 1 {
        return Err(RuleError::Invalid("spell condition body inputs".into()));
    }
    span(packet, 0, 58)?;
    if packet.len() != 58
        || packet[9] != 3
        || packet[19] != 8
        || packet[20] != 8
        || packet[12] != 0
        || packet[15] != 0
        || packet[23..30].iter().any(|value| *value != 0)
        || word(packet, 34)? != 0
        || word(packet, 40)? != 36
        || word(packet, 50)? != 0
    {
        return Err(RuleError::Unsupported {
            action: "combat.spell.condition",
            dependency: "other spell condition packet",
        });
    }
    for warrior in warriors {
        span(&warrior.bytes, 0, 3138)?;
    }
    for enemy in enemies {
        span(&enemy.bytes, 0, 288)?;
    }
    let mut next = random.clone();
    let duration = next.range(i16::from(packet[13] as i8), i16::from(packet[14] as i8));
    let mut outcome = ConditionOutcome {
        warriors: warriors.to_vec(),
        enemies: enemies.to_vec(),
        duration,
        sounds: Vec::new(),
    };
    for (body, warrior) in outcome.warriors.iter_mut().enumerate() {
        if mask[body] > 0 {
            apply(&mut warrior.bytes, false, 36, duration, &mut next)?;
            outcome.sounds.push(word(packet, 38)?);
        }
    }
    for (index, enemy) in outcome.enemies.iter_mut().enumerate() {
        if mask[index + 10] > 0 {
            apply(&mut enemy.bytes, true, 36, duration, &mut next)?;
            outcome.sounds.push(word(packet, 38)?);
        }
    }
    *random = next;
    Ok(outcome)
}
