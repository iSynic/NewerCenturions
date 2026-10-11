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
    if !matches!(effect, 4 | 13 | 36) {
        return Err(RuleError::Unsupported {
            action: "combat.spell.condition",
            dependency: "spell condition outside recovered party and monster effects",
        });
    }
    if !monster && effect == 4 && word(target, 0x9a)? != 0 {
        return Err(RuleError::Unsupported {
            action: "combat.spell.condition",
            dependency: "poison ward reflection",
        });
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
        if word(target, 0x9a)? != 0 {
            random.bound(100);
        }
    }
    Ok(())
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
