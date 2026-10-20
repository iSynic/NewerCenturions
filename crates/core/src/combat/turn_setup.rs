//! Turn setup for combat.
use super::put_word;
use crate::{RandomState, RuleError, items};
use centurions_content::combat::MonsterRecord;
use centurions_content::{Warrior, word};

pub fn direction_offset(direction: i16, distance: i16) -> (i16, i16) {
    let radians = (f64::from(direction) / 57.3) as f32 as f64;
    (
        ((radians.cos() * f64::from(distance) - 5.0).trunc() as i32) as i16,
        ((radians.sin() * f64::from(distance) - 5.0).trunc() as i32) as i16,
    )
}

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
        let bytes = &mut monster.bytes;
        if bytes[0xb3] != 0 || word(bytes, 0x110)? <= 0 {
            return Err(RuleError::Unsupported {
                action: "combat.turn",
                dependency: "inactive enemy",
            });
        }
        let chance = word(bytes, 0x5c)?;
        if random.bound(100) <= chance {
            return Err(RuleError::Unsupported {
                action: "combat.turn",
                dependency: "enemy turn script",
            });
        }
        initialize_enemy_record(bytes, random)?;
    } else {
        return Err(RuleError::Invalid("invalid active body".into()));
    }
    Ok(())
}

/// getup12667e..127090: turn markers, confusion, signed faction and AP/movement.
pub(crate) fn initialize_warrior_record(
    warrior: &mut Warrior,
    race_moves: &[i8],
    random: &mut RandomState,
) -> Result<(bool, i8), RuleError> {
    centurions_content::span(&warrior.bytes, 0, 3138)?;
    let bytes = &mut warrior.bytes;
    if (word(bytes, 0x5e)? != 0 || word(bytes, 0x10)? != 0) && word(bytes, 0x90)? == 0 {
        put_word(bytes, 0x90, 1);
    }
    let faction = word(bytes, 0x10)?;
    if word(bytes, 0x90)? >= 0 && word(bytes, 0x98)? != 0 {
        let roll = random.bound(100);
        if roll < 30 {
            if random.bound(2) == 1 && faction == 0 {
                put_word(bytes, 0x10, faction.wrapping_add(1));
                if word(bytes, 0x90)? == 0 {
                    put_word(bytes, 0x90, 1);
                }
            } else if random.bound(2) == 2 && faction != 0 {
                put_word(bytes, 0x10, 0);
            }
        } else if roll > 70 {
            let fleeing = word(bytes, 0x5e)?.wrapping_add(30);
            put_word(bytes, 0x5e, fleeing);
        } else {
            let sleeping = word(bytes, 0x60)?.wrapping_add(1);
            put_word(bytes, 0x60, sleeping);
            return Ok((true, 0));
        }
    }
    let unavailable_delta = i8::from(word(bytes, 0x10)? != 0) - i8::from(faction != 0);
    items::compact(warrior)?;
    let bytes = &mut warrior.bytes;
    let carry = word(bytes, 0x40)?.clamp(0, 1);
    let mut points = carry
        .wrapping_add(word(bytes, 0x12)?)
        .wrapping_add(word(bytes, 0x1e)?);
    if word(bytes, 0x8c)? != 0 {
        points = points.wrapping_add(4);
    }
    put_word(bytes, 0x40, points);
    let race = word(bytes, 0x32)? - 1;
    let race_move = *race_moves
        .get(race as usize)
        .ok_or_else(|| RuleError::Invalid("race movement missing".into()))?;
    calculate_movement(warrior, race_move)?;
    let maximum = word(&warrior.bytes, 0x3e)?;
    put_word(&mut warrior.bytes, 0x3c, maximum);
    put_word(&mut warrior.bytes, 0xacc, 0);
    put_word(&mut warrior.bytes, 0xac8, 0);
    Ok((false, unavailable_delta))
}

/// getup126a18..126dd6: signed counters, confusion draws and haste attack copies.
pub(crate) fn initialize_enemy_record(
    bytes: &mut [u8],
    random: &mut RandomState,
) -> Result<(bool, i16), RuleError> {
    put_word(bytes, 0x5c, word(bytes, 0x5c)?.wrapping_abs());
    bytes[0x109] = (bytes[0x109] as i8).min(1).wrapping_add(bytes[0x8a] as i8) as u8;
    let faction = word(bytes, 0xe)?;
    if bytes[0xcf] != 0 {
        match random.bound(100) {
            1..40 => {
                if random.bound(2) == 1 && faction == 0 {
                    put_word(bytes, 0xe, 1);
                } else if random.bound(2) == 1 && faction != 0 {
                    put_word(bytes, 0xe, 0);
                }
            }
            40..61 => {
                bytes[0xb3] = bytes[0xb3].wrapping_add(1);
                return Ok((true, 0));
            }
            _ => bytes[0xb2] = bytes[0xb2].wrapping_add(1),
        }
    }
    let mut movement = (word(bytes, 6)? as i8).wrapping_sub(bytes[0xb4] as i8);
    if bytes[0xb8] != 0 {
        movement /= 2;
    }
    if bytes[0xc9] != 0 {
        movement = movement.wrapping_mul(2);
        let points = (bytes[0x109] as i8).wrapping_add(4).min(11);
        bytes[0x109] = points as u8;
        for slot in 1..points / 2 {
            let offset = 0x93 + slot as usize * 4;
            if bytes[offset] == 0 {
                bytes.copy_within(0x93..0x97, offset);
            }
        }
    }
    bytes[0x106] = movement as u8;
    let hostile_change = i16::from(word(bytes, 0xe)? != 0) - i16::from(faction != 0);
    Ok((false, hostile_change))
}

#[derive(Debug, PartialEq, Eq)]
pub struct EncounterBattle {
    pub battle: i16,
    pub sound: i16,
    pub message: i16,
    pub treasure: i16,
}

/// CODE 1 movecalc. Each SANE-to-single conversion is an explicit f32 rounding.
pub fn calculate_movement(warrior: &mut Warrior, race_movement: i8) -> Result<(), RuleError> {
    let bytes = &warrior.bytes;
    let maximum = 1000_i16
        .wrapping_add(word(bytes, 0x9e2)?.wrapping_mul(100))
        .wrapping_add(word(bytes, 0xace)?.wrapping_mul(10));
    let available = maximum.wrapping_sub(word(bytes, 0x9d6)?) as u16;
    if maximum == 0 {
        return Err(RuleError::Unsupported {
            action: "combat.movement",
            dependency: "SANE division by zero",
        });
    }
    let mut movement =
        ((available as f64 / (maximum as u16) as f64) as f32 as f64 * race_movement as f64) as f32;
    movement = (movement as f64 - word(bytes, 0x62)? as f64) as f32;
    if word(bytes, 0x6a)? != 0 {
        movement = (movement as f64 / 2.0) as f32;
    }
    if word(bytes, 0x8c)? != 0 {
        movement = (movement as f64 * 2.0) as f32;
    }
    movement = (movement as f64 + word(bytes, 0x30)? as f64) as f32;
    movement = ((movement as f64 + 0.5) as f32).max(5.0);
    if !movement.is_finite() || movement as f64 > i32::MAX as f64 {
        return Err(RuleError::Unsupported {
            action: "combat.movement",
            dependency: "SANE integer overflow",
        });
    }
    let movement = (movement.trunc() as i32) as i16;
    warrior.bytes[0x9d8..0x9da].copy_from_slice(&maximum.to_be_bytes());
    warrior.bytes[0x3e..0x40].copy_from_slice(&movement.to_be_bytes());
    Ok(())
}

/// CODE 1 getbattleorder, including its ordered adjacent comparisons.
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
