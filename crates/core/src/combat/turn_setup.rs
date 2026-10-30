//! Starting a turn: conditions, attacks, movement and the battle order.
use crate::{RandomState, RuleError, items};
use centurions_content::WARRIOR_BYTES;
use centurions_content::combat::MonsterRecord;
use centurions_content::{
    Warrior,
    records::{Condition, EnemyRecord, EnemyRecordMut},
};

/// Turns a bearing in degrees and a distance into the battle-map offset where the monsters'
/// formation starts.
pub fn direction_offset(direction: i16, distance: i16) -> (i16, i16) {
    let radians = (f64::from(direction) / 57.3) as f32 as f64;
    (
        ((radians.cos() * f64::from(distance) - 5.0).trunc() as i32) as i16,
        ((radians.sin() * f64::from(distance) - 5.0).trunc() as i32) as i16,
    )
}

/// Starts the turn of the body that holds it, a warrior or a monster. A monster that cannot act, or
/// whose turn script runs, is not supported here.
pub fn initialize_turn(
    body: i16,
    warriors: &mut [Warrior],
    enemies: &mut [MonsterRecord],
    race_moves: &[i8],
    random: &mut RandomState,
) -> Result<(), RuleError> {
    if (0..6).contains(&body) {
        let warrior = &mut warriors[body as usize];
        initialize_warrior_record(warrior, race_moves, random)?;
    } else if body >= 10 {
        let monster = enemies
            .get_mut((body - 10) as usize)
            .ok_or_else(|| RuleError::Invalid("enemy outside battle".into()))?;
        if monster.record().turn_skip_duration() != 0 || monster.record().stamina()? <= 0 {
            return Err(RuleError::Unsupported {
                action: "combat.turn",
                dependency: "inactive enemy",
            });
        }
        let chance = monster.record().turn_macro_chance()?;
        if random.bound(100) <= chance {
            return Err(RuleError::Unsupported {
                action: "combat.turn",
                dependency: "enemy turn script",
            });
        }
        initialize_enemy_record(&mut monster.bytes, random)?;
    } else {
        return Err(RuleError::Invalid("invalid active body".into()));
    }
    Ok(())
}

/// Starts a warrior's turn: markers, confusion, side, attacks and movement.
///
/// Original: getup
pub(crate) fn initialize_warrior_record(
    warrior: &mut Warrior,
    race_moves: &[i8],
    random: &mut RandomState,
) -> Result<(bool, i8), RuleError> {
    centurions_content::span(&warrior.bytes, 0, WARRIOR_BYTES)?;
    if (warrior.record().fleeing_duration()? != 0 || warrior.record().faction()? != 0)
        && warrior.record().computer_control()? == 0
    {
        warrior.record_mut().set_computer_control(1);
    }
    let faction = warrior.record().faction()?;
    if warrior.record().computer_control()? >= 0 && warrior.record().confusion_duration()? != 0 {
        let roll = random.bound(100);
        if roll < 30 {
            if random.bound(2) == 1 && faction == 0 {
                warrior.record_mut().set_faction(faction.wrapping_add(1));
                if warrior.record().computer_control()? == 0 {
                    warrior.record_mut().set_computer_control(1);
                }
            } else if random.bound(2) == 2 && faction != 0 {
                warrior.record_mut().set_faction(0);
            }
        } else if roll > 70 {
            let fleeing = warrior.record().fleeing_duration()?.wrapping_add(30);
            warrior
                .record_mut()
                .set_condition(Condition::Retreating, fleeing);
        } else {
            let sleeping = warrior.record().turn_skip_duration()?.wrapping_add(1);
            warrior.record_mut().set_turn_skip_duration(sleeping);
            return Ok((true, 0));
        }
    }
    let unavailable_delta = i8::from(warrior.record().faction()? != 0) - i8::from(faction != 0);
    items::compact(warrior)?;
    let carry = warrior.record().attacks()?.clamp(0, 1);
    let mut points = carry
        .wrapping_add(warrior.record().normal_attacks()?)
        .wrapping_add(warrior.record().attack_bonus()?);
    if warrior.record().condition(Condition::Speedy)? != 0 {
        points = points.wrapping_add(4);
    }
    warrior.record_mut().set_attacks(points);
    let race = warrior.record().race()? - 1;
    let race_move = *race_moves
        .get(race as usize)
        .ok_or_else(|| RuleError::Invalid("race movement missing".into()))?;
    calculate_movement(warrior, race_move)?;
    let maximum = warrior.record().movement_max()?;
    warrior.record_mut().set_movement(maximum);
    warrior.record_mut().set_attacks_used(0);
    warrior.record_mut().set_spells_so_far(0);
    Ok((false, unavailable_delta))
}

/// Starts a monster's turn: condition counters, confusion draws and haste.
///
/// Original: getup
pub(crate) fn initialize_enemy_record(
    bytes: &mut [u8],
    random: &mut RandomState,
) -> Result<(bool, i16), RuleError> {
    let chance = EnemyRecord::from_bytes(bytes)
        .turn_macro_chance()?
        .wrapping_abs();
    EnemyRecordMut::from_bytes(bytes).set_turn_macro_chance(chance);
    let record = EnemyRecord::from_bytes(bytes);
    let actions = record
        .action_points()
        .min(1)
        .wrapping_add(record.attack_count());
    EnemyRecordMut::from_bytes(bytes).set_action_points(actions);
    let faction = EnemyRecord::from_bytes(bytes).faction()?;
    if EnemyRecord::from_bytes(bytes).condition(Condition::Confused) != 0 {
        match random.bound(100) {
            1..40 => {
                if random.bound(2) == 1 && faction == 0 {
                    EnemyRecordMut::from_bytes(bytes).set_faction(1);
                } else if random.bound(2) == 1 && faction != 0 {
                    EnemyRecordMut::from_bytes(bytes).set_faction(0);
                }
            }
            40..61 => {
                let duration = EnemyRecord::from_bytes(bytes)
                    .condition(Condition::Helpless)
                    .wrapping_add(1);
                EnemyRecordMut::from_bytes(bytes).set_condition(Condition::Helpless, duration);
                return Ok((true, 0));
            }
            _ => {
                let duration = EnemyRecord::from_bytes(bytes)
                    .condition(Condition::Retreating)
                    .wrapping_add(1);
                EnemyRecordMut::from_bytes(bytes).set_condition(Condition::Retreating, duration);
            }
        }
    }
    let record = EnemyRecord::from_bytes(bytes);
    let mut movement =
        (record.movement_max()? as i8).wrapping_sub(record.condition(Condition::Entangled) as i8);
    if record.condition(Condition::MovingSlowly) != 0 {
        movement /= 2;
    }
    if record.condition(Condition::Speedy) != 0 {
        movement = movement.wrapping_mul(2);
        let points = record.action_points().wrapping_add(4).min(11);
        let mut record = EnemyRecordMut::from_bytes(bytes);
        record.set_action_points(points);
        for slot in 1..points / 2 {
            record.fill_missing_natural_attack(slot as usize);
        }
    }
    EnemyRecordMut::from_bytes(bytes).set_movement(movement);
    let hostile_change =
        i16::from(EnemyRecord::from_bytes(bytes).faction()? != 0) - i16::from(faction != 0);
    Ok((false, hostile_change))
}

/// The battle an encounter record picks, with its sound, message and treasure.
#[derive(Debug, PartialEq, Eq)]
pub struct EncounterBattle {
    pub battle: i16,
    pub sound: i16,
    pub message: i16,
    pub treasure: i16,
}

/// A warrior's movement points, rounded to single precision at each step.
///
/// Original: movecalc
pub fn calculate_movement(warrior: &mut Warrior, race_movement: i8) -> Result<(), RuleError> {
    let record = warrior.record();
    let maximum = 1000_i16
        .wrapping_add(record.brawn()?.wrapping_mul(100))
        .wrapping_add(record.brawn_upgrades()?.wrapping_mul(10));
    let available = maximum.wrapping_sub(record.load()?) as u16;
    // A zero capacity divides by zero: 65535/0 is infinite and 0/0 is NaN.
    let mut movement =
        ((available as f64 / (maximum as u16) as f64) as f32 as f64 * race_movement as f64) as f32;
    movement = (movement as f64 - record.condition(Condition::Entangled)? as f64) as f32;
    if record.condition(Condition::MovingSlowly)? != 0 {
        movement = (movement as f64 / 2.0) as f32;
    }
    if warrior.record().condition(Condition::Speedy)? != 0 {
        movement = (movement as f64 * 2.0) as f32;
    }
    movement = (movement as f64 + record.movement_bonus()? as f64) as f32;
    movement = ((movement as f64 + 0.5) as f32).max(5.0);
    // The record keeps the low word.
    let movement = crate::rules::sane_long(f64::from(movement.trunc())) as i16;
    warrior.record_mut().set_weight_capacity(maximum);
    warrior.record_mut().set_movement_max(movement);
    Ok(())
}

/// Deals the battle order. A surprise puts the squad first or last; otherwise
/// every body draws a random free slot.
///
/// Original: getbattleorder
pub fn initiative(
    warrior_speeds: &[i16],
    enemy_speeds: &[i16],
    surprise: i16,
    random: &mut RandomState,
) -> Result<Vec<i16>, RuleError> {
    if !(1..=6).contains(&warrior_speeds.len())
        || enemy_speeds.len() > 100
        || !(-1..=1).contains(&surprise)
    {
        return Err(RuleError::Invalid(
            "initiative requires the living starter squad and at most 100 enemies".into(),
        ));
    }
    let total = warrior_speeds.len() + enemy_speeds.len();
    let mut order = vec![-1; 110];
    match surprise {
        1 => {
            for index in 0..warrior_speeds.len() {
                order[index + 1] = index as i16;
            }
        }
        -1 => {
            for index in 0..warrior_speeds.len() {
                order[109 - index] = index as i16;
            }
        }
        _ => {
            let mut left = warrior_speeds.len();
            while left != 0 {
                let slot = random.bound(total as i16) as usize;
                if order[slot] == -1 {
                    left -= 1;
                    order[slot] = left as i16;
                }
            }
        }
    }
    let mut left = enemy_speeds.len();
    while left != 0 {
        let slot = random.bound(total as i16) as usize;
        if order[slot] == -1 {
            order[slot] = left as i16 + 9;
            left -= 1;
        }
    }
    if surprise == 0 {
        let speed = |body: i16| {
            if body < 10 {
                warrior_speeds[body as usize]
            } else {
                enemy_speeds[(body - 10) as usize]
            }
        };
        for start in 1..total {
            for slot in start..total {
                if order[slot] >= 0
                    && order[slot + 1] >= 0
                    && speed(order[slot]) < speed(order[slot + 1])
                {
                    order.swap(slot, slot + 1);
                }
            }
        }
    }
    Ok(order)
}
