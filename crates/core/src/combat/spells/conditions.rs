use crate::combat::spells::effects;
use crate::{RandomState, RuleError};
use centurions_content::WARRIOR_BYTES;
use centurions_content::combat::MONSTER_BYTES;
use centurions_content::{
    Warrior,
    combat::MonsterRecord,
    records::{
        Condition, EnemyRecord, EnemyRecordMut, SpellInfoRecord, WarriorRecord, WarriorRecordMut,
    },
    span,
};

/// Adds a spell's condition, signed, at the field's own width.
///
/// Original: spelllist
pub fn apply(
    target: &mut [u8],
    monster: bool,
    effect: i16,
    duration: i16,
    random: &mut RandomState,
) -> Result<(), RuleError> {
    if target.len()
        != if monster {
            MONSTER_BYTES
        } else {
            WARRIOR_BYTES
        }
    {
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

/// Adds a condition's duration to a body's. Nothing changes when the body already carries a
/// negative duration, or the total would reach 100 for a warrior or 125 for a monster.
pub(crate) fn add_duration(
    target: &mut [u8],
    monster: bool,
    effect: i16,
    duration: i16,
) -> Result<(), RuleError> {
    if target.len()
        != if monster {
            MONSTER_BYTES
        } else {
            WARRIOR_BYTES
        }
        || !(1..=40).contains(&effect)
    {
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

/// A spell's immediate effects, after its duration is written.
///
/// Original: spelllist
pub(crate) fn follow_up(target: &mut [u8], monster: bool, effect: i16) -> Result<(), RuleError> {
    match effect {
        effects::HELPLESS => {
            if monster {
                EnemyRecordMut::from_bytes(target).set_movement(0);
            } else {
                WarriorRecordMut::from_bytes(target).set_movement(0);
            }
        }
        effects::ENTANGLED | effects::MOVING_SLOWLY => {
            if monster {
                let movement = EnemyRecord::from_bytes(target).movement() / 2;
                EnemyRecordMut::from_bytes(target).set_movement(movement);
            } else {
                let movement = WarriorRecord::from_bytes(target).movement()? / 2;
                WarriorRecordMut::from_bytes(target).set_movement(movement);
            }
        }
        10 => {
            if monster {
                if EnemyRecord::from_bytes(target).save(5) > 100
                    || EnemyRecord::from_bytes(target).condition(Condition::ComputerControl) != 0
                {
                    EnemyRecordMut::from_bytes(target).set_condition(Condition::Poisoned, 0);
                }
            } else if WarriorRecord::from_bytes(target).computer_control()? < 0 {
                WarriorRecordMut::from_bytes(target).set_condition(Condition::Poisoned, 0);
            }
        }
        effects::QUARTER_REVIVAL
            if !monster && WarriorRecord::from_bytes(target).stamina()? < -9 =>
        {
            WarriorRecordMut::from_bytes(target).set_computer_control(-1);
            let health = WarriorRecord::from_bytes(target).stamina_max()? / 4;
            WarriorRecordMut::from_bytes(target).set_stamina(health);
        }
        effects::REVIVAL
            if !monster
                && (WarriorRecord::from_bytes(target).stamina()? < -9
                    || WarriorRecord::from_bytes(target).computer_control()? != 0) =>
        {
            WarriorRecordMut::from_bytes(target).set_computer_control(0);
            WarriorRecordMut::from_bytes(target).set_stamina(-9);
        }
        // Clears conditions 28 and 3.
        effects::SCENARIO_TAIL => {
            clear_duration(target, monster, 28);
            clear_duration(target, monster, 3);
        }
        _ => {}
    }
    Ok(())
}

/// Stuns: the duration adds to the turn-skip condition without a cap.
///
/// Original: spelllist
pub(crate) fn stun(target: &mut [u8], monster: bool, duration: i16) -> Result<(), RuleError> {
    if monster {
        let skip = EnemyRecord::from_bytes(target).condition_duration(1) as i8;
        EnemyRecordMut::from_bytes(target)
            .set_condition_duration(1, skip.wrapping_add(duration as i8).into());
        EnemyRecordMut::from_bytes(target).set_movement(0);
    } else {
        let skip = WarriorRecord::from_bytes(target).turn_skip_duration()?;
        let mut record = WarriorRecordMut::from_bytes(target);
        record.set_turn_skip_duration(skip.wrapping_add(duration));
        record.set_attacks(0);
        record.set_movement(0);
    }
    Ok(())
}

/// Party Psi stays within 0..maximum; monster Psi is only floored at zero.
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
            .min(view.psi_points_max()?);
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

/// Effects above 99 clear condition `effect - 101`, with no upper bound: 100 clears
/// the field before the conditions and 141 and up clear later record fields.
///
/// Original: resolvespell
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

/// Ends a condition on a body.
pub(crate) fn clear_duration(target: &mut [u8], monster: bool, index: usize) {
    if monster {
        EnemyRecordMut::from_bytes(target).set_condition_duration(index, 0);
    } else {
        WarriorRecordMut::from_bytes(target).set_condition_duration(index, 0);
    }
}

/// Changes a body's side, after the party's reflection draw. The returned change
/// applies to `killparty`, not the hostile count.
///
/// Original: spelllist
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
        let monster = enemies.get_mut(target - 10).ok_or("charm target missing")?;
        let previous = monster.record().faction()?;
        if side != 0 || previous != 0 {
            monster.record_mut().set_target(-1);
            monster.record_mut().set_faction(side);
        }
        previous
    } else {
        let warrior = warriors.get_mut(target).ok_or("charm target missing")?;
        let previous = warrior.record().faction()?;
        if side != 0 || previous != 0 {
            if side != 0 && previous != side {
                warrior.record_mut().set_target(-1);
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

/// What a condition spell left: the bodies, the duration, and the sounds.
pub struct ConditionOutcome {
    pub warriors: Vec<Warrior>,
    pub enemies: Vec<MonsterRecord>,
    pub duration: i16,
    pub sounds: Vec<i16>,
}

/// Whether a packet is spell 1703's.
pub(crate) fn is_1703_packet(packet: &[u8]) -> Result<bool, RuleError> {
    if packet.len() != 58 {
        return Ok(false);
    }
    let attack = SpellInfoRecord(packet);
    Ok(attack.cannot() == 3
        && attack.defense_category() == 8
        && attack.effect_animation() == 8
        && attack.unidentified_byte12() == 0
        && attack.power_duration_range().0 == 0
        && attack.elemental_damage().iter().all(|value| *value == 0)
        && attack.damage_maximum()? == 0
        && attack.effect()? == 36
        && attack.race_restriction()? == 0)
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
    if !is_1703_packet(packet)? {
        return Err(RuleError::Invalid("not the 1703 condition packet".into()));
    }
    for warrior in warriors {
        span(&warrior.bytes, 0, WARRIOR_BYTES)?;
    }
    for monster in enemies {
        span(&monster.bytes, 0, MONSTER_BYTES)?;
    }
    let mut next = random.clone();
    let (minimum, maximum) = SpellInfoRecord(packet).duration_range();
    let duration = next.range(minimum, maximum);
    let mut outcome = ConditionOutcome {
        warriors: warriors.to_vec(),
        enemies: enemies.to_vec(),
        duration,
        sounds: Vec::new(),
    };
    for (body, warrior) in outcome.warriors.iter_mut().enumerate() {
        if mask[body] > 0 {
            apply(&mut warrior.bytes, false, 36, duration, &mut next)?;
            outcome.sounds.push(SpellInfoRecord(packet).impact_sound()?);
        }
    }
    for (index, monster) in outcome.enemies.iter_mut().enumerate() {
        if mask[index + 10] > 0 {
            apply(&mut monster.bytes, true, 36, duration, &mut next)?;
            outcome.sounds.push(SpellInfoRecord(packet).impact_sound()?);
        }
    }
    *random = next;
    Ok(outcome)
}
