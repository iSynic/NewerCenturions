//! Source-derived targeting and displacement rules with explicit work globals.

use crate::{RandomState, RuleError};
use centurions_content::{Warrior, combat::MonsterRecord, span, word};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SaveContext {
    pub combat: bool,
    pub delivery_kind: i8,
    pub modifier_factor: i8,
    pub modifier_scale: i16,
    pub modifier_base: i8,
    pub resistance_bonus: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveResult {
    pub saved: bool,
    pub roll: Option<i16>,
    pub random: RandomState,
}

/// CODE 1 savevs, 0x13a3ae. A disabled save consumes no random number.
pub fn save_against(
    warriors: &[Warrior],
    monsters: &[MonsterRecord],
    context: SaveContext,
    selector: i16,
    body: i16,
    random: &RandomState,
) -> Result<SaveResult, RuleError> {
    if context.combat && context.delivery_kind > 1 {
        return Ok(SaveResult {
            saved: false,
            roll: None,
            random: random.clone(),
        });
    }
    let mut modifier = i16::from(context.modifier_factor)
        .wrapping_mul(context.modifier_scale)
        .wrapping_add(i16::from(context.modifier_base));
    // Validate addressed memory before consuming randomness; body 9 and negative
    // bodies still take the original unsuccessful branch after one draw.
    let resistance = if (0..9).contains(&body) {
        if selector == 0 && context.resistance_bonus {
            modifier = modifier.wrapping_add(50);
        }
        let record = warriors
            .get(body as usize)
            .ok_or_else(|| RuleError::Invalid("save warrior record unavailable".into()))?;
        Some((
            word(
                &record.bytes,
                checked_offset(0x112, i32::from(selector) * 2)?,
            )?,
            false,
        ))
    } else if body > 9 {
        let record = monsters
            .get((body - 10) as usize)
            .ok_or_else(|| RuleError::Invalid("save monster record unavailable".into()))?;
        let value = span(&record.bytes, checked_offset(0x8b, i32::from(selector))?, 1)?[0] as i8;
        Some((i16::from(value), value > 99))
    } else {
        None
    };
    let mut next_random = random.clone();
    let roll = next_random.bound(100);
    let saved = resistance
        .is_some_and(|(value, automatic)| roll <= value.wrapping_add(modifier) || automatic);
    Ok(SaveResult {
        saved,
        roll: Some(roll),
        random: next_random,
    })
}

fn checked_offset(base: i32, delta: i32) -> Result<usize, RuleError> {
    usize::try_from(base + delta)
        .map_err(|_| RuleError::Invalid("source record address precedes supplied memory".into()))
}

/// CODE 1 randrange, 0x13849e. Reversed bounds and WORD overflow are retained.
pub fn random_range(minimum: i16, maximum: i16, random: &mut RandomState) -> i16 {
    minimum
        .wrapping_add(random.bound(maximum.wrapping_sub(minimum).wrapping_add(1)))
        .wrapping_sub(1)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Displacement {
    pub dx: i8,
    pub dy: i8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShiftResult {
    pub displacement: Displacement,
    pub random: RandomState,
    /// The native routine leaves its last arithmetic/service result in D0.W.
    pub work_word: i16,
}

/// CODE 1 shift, 0x153e44. The coordinate table WORDs are truncated to BYTEs.
pub fn shift(
    position: (i16, i16),
    target: (i8, i8),
    displacement: Displacement,
    random: &RandomState,
) -> ShiftResult {
    let distance_x = (i16::from(position.0 as i8) - i16::from(target.0)).wrapping_abs();
    let distance_y = (i16::from(position.1 as i8) - i16::from(target.1)).wrapping_abs();
    let mut next = displacement;
    let mut rng = random.clone();
    let mut work = distance_y;
    if (distance_y as i8) < (distance_x as i8) {
        if next.dy != 0 {
            work = rng.bound(2).wrapping_sub(1);
            if work == 0 {
                next.dy = 0;
                return ShiftResult {
                    displacement: next,
                    random: rng,
                    work_word: work,
                };
            }
        }
        next.dy = -1;
        work = rng.bound(3);
        match work {
            1 => next.dy = 1,
            2 => next.dy = 0,
            _ => {}
        }
        work = rng.bound(3);
        match work {
            1 => next.dx = -1,
            2 => next.dx = 1,
            _ => {}
        }
        if work == 3 && next.dy != 0 {
            next.dx = 0;
        } else {
            next.dx = 1;
            work = rng.bound(2).wrapping_sub(1);
            if work == 0 {
                work = -i16::from(next.dx);
                next.dx = work as i8;
            }
        }
    } else if next.dx != 0 {
        next.dx = 0;
    } else {
        next.dx = -1;
        work = rng.bound(3);
        match work {
            1 => next.dx = 1,
            2 => next.dx = 0,
            _ => {}
        }
        work = rng.bound(3);
        match work {
            1 => next.dy = -1,
            2 => next.dy = 1,
            3 => next.dy = 0,
            _ => {}
        }
    }
    ShiftResult {
        displacement: next,
        random: rng,
        work_word: work,
    }
}

pub struct TargetingInput<'a> {
    /// Signed WORD at 0x4435d6 for each of the 110 candidate bodies.
    pub eligibility: &'a [i16],
    pub turn: i16,
    pub last_warrior: i8,
    pub warriors: &'a [Warrior],
    pub monsters: &'a [MonsterRecord],
    pub warrior_positions: &'a [(i16, i16)],
    pub monster_positions: &'a [(i16, i16)],
    pub origin: (i32, i32),
    /// Native column-major occupancy WORD table, normally 90 by 90.
    pub occupancy: &'a [i16],
    /// One 7 by 7 BYTE mask selected by the original one-based parameter.
    pub mask: &'a [u8; 49],
    pub faction: i16,
    pub last_occupant: i16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetingResult {
    pub choice: i16,
    pub same_faction: i16,
    pub other_faction: i16,
    pub last_occupant: i16,
    /// Every write to the scratch occupancy global, including duplicates.
    pub occupant_writes: Vec<i16>,
}

/// CODE 1 bestchoice, 0x1363e4. Candidate order and asymmetric tie rules matter.
pub fn best_choice(input: TargetingInput<'_>) -> Result<TargetingResult, RuleError> {
    let mut result = TargetingResult {
        choice: -1,
        same_faction: 0,
        other_faction: 0,
        last_occupant: input.last_occupant,
        occupant_writes: Vec::new(),
    };
    for candidate in 0..110 {
        let eligible = *input
            .eligibility
            .get(candidate)
            .ok_or_else(|| RuleError::Invalid("target eligibility table incomplete".into()))?;
        if eligible > input.turn {
            continue;
        }
        let position =
            if candidate >= 10 && word(&monster(input.monsters, candidate - 10)?.bytes, 0x110)? > 0
            {
                *input.monster_positions.get(candidate - 10).ok_or_else(|| {
                    RuleError::Invalid("target monster position unavailable".into())
                })?
            } else {
                if i16::from(input.last_warrior) < candidate as i16 {
                    continue;
                }
                let warrior = warrior(input.warriors, candidate)?;
                if span(&warrior.bytes, 0x9df, 1)?[0] == 0 {
                    continue;
                }
                *input.warrior_positions.get(candidate).ok_or_else(|| {
                    RuleError::Invalid("target warrior position unavailable".into())
                })?
            };
        let left = (i32::from(position.0.wrapping_sub(3)).wrapping_add(input.origin.0)) as i16;
        let top = (i32::from(position.1.wrapping_sub(3)).wrapping_add(input.origin.1)) as i16;
        let mut same = 0_i16;
        let mut other = 0_i16;
        let mut seen = [0_i16; 110];
        for row in 0..7_i16 {
            for column in 0..7_i16 {
                if input.mask[(row * 7 + column) as usize] == 0 {
                    continue;
                }
                let x = left.wrapping_add(column);
                let y = top.wrapping_add(row);
                // The native unsigned column multiplication must not alias an
                // out-of-bounds coordinate into a different supplied cell.
                if !(0..90).contains(&x) || !(0..90).contains(&y) {
                    return Err(RuleError::Invalid(
                        "target mask addresses outside occupancy map".into(),
                    ));
                }
                let occupant = input
                    .occupancy
                    .get(x as usize * 90 + y as usize)
                    .ok_or_else(|| RuleError::Invalid("target occupancy table incomplete".into()))?
                    .wrapping_abs();
                result.last_occupant = occupant;
                result.occupant_writes.push(occupant);
                if occupant >= 110 {
                    continue;
                }
                let allied = if occupant < 9 {
                    let index = usize::try_from(occupant).map_err(|_| {
                        RuleError::Invalid("wrapped occupancy has negative body".into())
                    })?;
                    word(&warrior(input.warriors, index)?.bytes, 0x10)? == input.faction
                } else {
                    let index = usize::try_from(occupant - 10).map_err(|_| {
                        RuleError::Invalid("reserved body 9 addresses before monster table".into())
                    })?;
                    let record = monster(input.monsters, index)?;
                    let allied = word(&record.bytes, 0xe)? == input.faction
                        || span(&record.bytes, 0xb0, 1)?[0] == 0xff;
                    let already = seen[index];
                    seen[index] = already.wrapping_add(1);
                    if already != 0 {
                        continue;
                    }
                    allied
                };
                if allied {
                    same = same.wrapping_add(1);
                } else {
                    other = other.wrapping_add(1);
                }
            }
        }
        let replace = if input.faction == 0 {
            let competitive = other >= result.other_faction
                || other >= result.other_faction.wrapping_sub(result.same_faction);
            competitive
                && ((other >= result.other_faction && same <= result.same_faction && same < other)
                    || (result.other_faction < other.wrapping_sub(same) && same < other))
        } else {
            result.other_faction <= other
                && (same <= result.same_faction || result.other_faction < other)
        };
        if replace {
            result.choice = candidate as i16;
            result.same_faction = same;
            result.other_faction = other;
        }
    }
    Ok(result)
}

fn warrior(records: &[Warrior], index: usize) -> Result<&Warrior, RuleError> {
    records
        .get(index)
        .ok_or_else(|| RuleError::Invalid("target warrior record unavailable".into()))
}
fn monster(records: &[MonsterRecord], index: usize) -> Result<&MonsterRecord, RuleError> {
    records
        .get(index)
        .ok_or_else(|| RuleError::Invalid("target monster record unavailable".into()))
}
