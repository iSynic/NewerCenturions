use crate::{RandomState, RuleError};
use centurions_content::{
    Warrior,
    combat::MonsterRecord,
    records::{Condition, EnemyRecord, EnemyRecordMut, WarriorRecord, WarriorRecordMut},
    span, word,
};

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
    if !monster && WarriorRecord::from_bytes(target).condition(Condition::ReflectingScans)? != 0 {
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
        let index = effect as usize - 1;
        let previous = EnemyRecord::from_bytes(target).condition_duration(index);
        let total = previous.wrapping_add(duration);
        if total.wrapping_abs() < 125 && previous >= 0 {
            EnemyRecordMut::from_bytes(target).set_condition_duration(index, total);
        }
    } else {
        let index = effect as usize - 1;
        let previous = WarriorRecord::from_bytes(target).condition_duration(index)?;
        let total = previous.wrapping_add(duration);
        if total < 100 && previous >= 0 {
            WarriorRecordMut::from_bytes(target).set_condition_duration(index, total);
        }
    }
    Ok(())
}

/// spelllist15c8dc..15ca62 applies immediate effects after the duration writer.
pub(crate) fn follow_up(target: &mut [u8], monster: bool, effect: i16) -> Result<(), RuleError> {
    match effect {
        2 => {
            if monster {
                EnemyRecordMut::from_bytes(target).set_movement_points(0);
            } else {
                WarriorRecordMut::from_bytes(target).set_movement_points(0);
            }
        }
        3 | 7 => {
            if monster {
                let movement = EnemyRecord::from_bytes(target).movement_points() / 2;
                EnemyRecordMut::from_bytes(target).set_movement_points(movement);
            } else {
                let movement = WarriorRecord::from_bytes(target).movement_points()? / 2;
                WarriorRecordMut::from_bytes(target).set_movement_points(movement);
            }
        }
        10 => {
            if monster {
                if EnemyRecord::from_bytes(target).saving_throw(5) > 100
                    || EnemyRecord::from_bytes(target).condition(Condition::ComputerControl) != 0
                {
                    EnemyRecordMut::from_bytes(target).set_condition(Condition::Poisoned, 0);
                }
            } else if WarriorRecord::from_bytes(target).automatic_mode()? < 0 {
                WarriorRecordMut::from_bytes(target).set_condition(Condition::Poisoned, 0);
            }
        }
        26 if !monster && WarriorRecord::from_bytes(target).stamina()? < -9 => {
            WarriorRecordMut::from_bytes(target).set_automatic_mode(-1);
            let health = WarriorRecord::from_bytes(target).maximum_stamina()? / 4;
            WarriorRecordMut::from_bytes(target).set_stamina(health);
        }
        64 if !monster
            && (WarriorRecord::from_bytes(target).stamina()? < -9
                || WarriorRecord::from_bytes(target).automatic_mode()? != 0) =>
        {
            WarriorRecordMut::from_bytes(target).set_automatic_mode(0);
            WarriorRecordMut::from_bytes(target).set_stamina(-9);
        }
        // spelllist15ca1c clears condition indices 28 and 3.
        110 => {
            clear_duration(target, monster, 28);
            clear_duration(target, monster, 3);
        }
        _ => {}
    }
    Ok(())
}

/// spelllist15bc8e adds the duration to the turn-skip condition without a cap.
pub(crate) fn stun(target: &mut [u8], monster: bool, duration: i16) -> Result<(), RuleError> {
    if monster {
        let skip = EnemyRecord::from_bytes(target).condition_duration(1) as i8;
        EnemyRecordMut::from_bytes(target)
            .set_condition_duration(1, skip.wrapping_add(duration as i8).into());
        EnemyRecordMut::from_bytes(target).set_movement_points(0);
    } else {
        let skip = WarriorRecord::from_bytes(target).turn_skip_duration()?;
        let mut record = WarriorRecordMut::from_bytes(target);
        record.set_turn_skip_duration(skip.wrapping_add(duration));
        record.set_action_points(0);
        record.set_movement_points(0);
    }
    Ok(())
}

/// Party Psi stays within 0..maximum; NPC Psi is only floored at zero.
pub(crate) fn move_psi(
    warriors: &mut [Warrior],
    enemies: &mut [MonsterRecord],
    body: usize,
    amount: i16,
) -> Result<(), RuleError> {
    if body < 10 {
        let record = &mut warriors.get_mut(body).ok_or("Psi target missing")?.bytes;
        let view = WarriorRecord::from_bytes(record);
        let points = view
            .psi_points()?
            .wrapping_add(amount)
            .max(0)
            .min(view.maximum_psi()?);
        WarriorRecordMut::from_bytes(record).set_psi_points(points);
    } else {
        let record = &mut enemies
            .get_mut(body - 10)
            .ok_or("Psi target missing")?
            .bytes;
        let points = EnemyRecord::from_bytes(record)
            .psi_points()?
            .wrapping_add(amount)
            .max(0);
        EnemyRecordMut::from_bytes(record).set_psi_points(points);
    }
    Ok(())
}

/// The siphon transfer is unclamped.
pub(crate) fn add_psi(
    warriors: &mut [Warrior],
    enemies: &mut [MonsterRecord],
    body: usize,
    amount: i16,
) -> Result<(), RuleError> {
    if body < 10 {
        let record = &mut warriors.get_mut(body).ok_or("Psi caster missing")?.bytes;
        let points = WarriorRecord::from_bytes(record)
            .psi_points()?
            .wrapping_add(amount);
        WarriorRecordMut::from_bytes(record).set_psi_points(points);
    } else {
        let record = &mut enemies
            .get_mut(body - 10)
            .ok_or("Psi caster missing")?
            .bytes;
        let points = EnemyRecord::from_bytes(record)
            .psi_points()?
            .wrapping_add(amount);
        EnemyRecordMut::from_bytes(record).set_psi_points(points);
    }
    Ok(())
}

/// resolvespell1519a8 clears slot effect-101 for every selector above 99, without a
/// bound: 100 clears the word (party) or byte (NPC) before the table, and 141..
/// clear later record fields.
pub(crate) fn clear_selector_slot(
    target: &mut [u8],
    monster: bool,
    effect: i16,
) -> Result<(), RuleError> {
    let index = i32::from(effect) - 101;
    let (offset, width) = if monster {
        (0xb2 + index, 1)
    } else {
        (0x5e + index * 2, 2)
    };
    let slot = usize::try_from(offset)
        .ok()
        .and_then(|offset| target.get_mut(offset..offset + width))
        .ok_or(RuleError::Unsupported {
            action: "combat.spell",
            dependency: "condition clear outside the target record",
        })?;
    slot.fill(0);
    Ok(())
}

pub(crate) fn clear_duration(target: &mut [u8], monster: bool, index: usize) {
    if monster {
        EnemyRecordMut::from_bytes(target).set_condition_duration(index, 0);
    } else {
        WarriorRecordMut::from_bytes(target).set_condition_duration(index, 0);
    }
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
        warriors
            .get(caster)
            .ok_or("charm caster missing")?
            .record()
            .faction()?
    } else {
        enemies
            .get(caster - 10)
            .ok_or("charm caster missing")?
            .record()
            .faction()?
    };
    let monster = target >= 10;
    let previous = if monster {
        let enemy = enemies.get_mut(target - 10).ok_or("charm target missing")?;
        let previous = enemy.record().faction()?;
        if side != 0 || previous != 0 {
            enemy.record_mut().set_target_body(-1);
            enemy.record_mut().set_faction(side);
        }
        previous
    } else {
        let warrior = warriors.get_mut(target).ok_or("charm target missing")?;
        let previous = warrior.record().faction()?;
        if side != 0 || previous != 0 {
            if side != 0 && previous != side {
                warrior.record_mut().set_target_body(-1);
            }
            warrior.record_mut().set_faction(side);
        }
        previous
    };
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

/// Packet contract of the authored 1703 fast path.
pub(crate) fn is_1703_packet(packet: &[u8]) -> Result<bool, RuleError> {
    if packet.len() != 58 {
        return Ok(false);
    }
    Ok(packet[9] == 3
        && packet[19] == 8
        && packet[20] == 8
        && packet[12] == 0
        && packet[15] == 0
        && packet[23..30].iter().all(|value| *value == 0)
        && word(packet, 34)? == 0
        && word(packet, 40)? == 36
        && word(packet, 50)? == 0)
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
    // Callers route only this packet here; every other shape takes the support resolver.
    if !is_1703_packet(packet)? {
        return Err(RuleError::Invalid("not the 1703 condition packet".into()));
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
