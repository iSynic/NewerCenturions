use crate::{RandomState, RuleError, combat, items};
use centurions_content::{Content, InventoryRecord, Warrior, mission::long, span, word};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartyItem {
    pub warrior: usize,
    pub slot: usize,
}

pub fn query_party_item(warriors: &[Warrior], id: i16) -> Result<Option<PartyItem>, RuleError> {
    for (warrior, record) in warriors.iter().enumerate() {
        for (slot, item) in record.inventory()?.iter().enumerate() {
            if item.id() == id {
                return Ok(Some(PartyItem { warrior, slot }));
            }
        }
    }
    Ok(None)
}

pub fn item_prerequisite_matches(warriors: &[Warrior], extra: &[u8]) -> Result<bool, RuleError> {
    let extra = span(extra, 0, 10)?;
    let found = query_party_item(warriors, word(extra, 0)?)?.is_some();
    Ok(match word(extra, 2)? {
        0 => !found,
        1 => found,
        2 => true,
        _ => false,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Consumption {
    pub warriors: Vec<Warrior>,
    pub consumed: usize,
    pub sounds: Vec<i16>,
}

pub fn consume_party_items(
    warriors: &[Warrior],
    content: &Content,
    race_movements: &[i8],
    extra: &[u8],
) -> Result<Consumption, RuleError> {
    let extra = span(extra, 0, 10)?;
    let id = word(extra, 0)?;
    let maximum = word(extra, 2)?;
    let mode = word(extra, 4)?;
    for warrior in warriors {
        span(&warrior.bytes, 0, 3138)?;
    }
    let mut result = Consumption {
        warriors: warriors.to_vec(),
        consumed: 0,
        sounds: Vec::new(),
    };
    if maximum <= 0 {
        return Ok(result);
    }
    for warrior in &mut result.warriors {
        let mut slot = 0;
        while slot < warrior.inventory()?.len() && result.consumed < maximum as usize {
            let item = warrior.inventory()?[slot].clone();
            if item.id() != id {
                slot += 1;
                continue;
            }
            if mode != 1 {
                return Err(RuleError::Unsupported {
                    action: "story.consume_item",
                    dependency: "quantity changes and replacement of equipped items",
                });
            }
            let count = warrior.inventory()?.len();
            if count > 30 || item.0[9] != 0 {
                return Err(RuleError::Unsupported {
                    action: "story.consume_item",
                    dependency: "equipped items or extended inventory",
                });
            }
            let definition = content
                .items
                .get(id.unsigned_abs() as usize)
                .ok_or_else(|| {
                    RuleError::Invalid("consumed item definition is unavailable".into())
                })?;
            let weight = word(definition, 0x10)?
                .wrapping_add(word(definition, 0x24)?.wrapping_mul(word(&item.0, 2)?));
            let carried = word(&warrior.bytes, 0x9d6)?.wrapping_sub(weight);
            warrior.bytes[0x9d6..0x9d8].copy_from_slice(&carried.to_be_bytes());
            for destination in slot..29 {
                let start = 0x132 + (destination + 1) * 22;
                warrior
                    .bytes
                    .copy_within(start..start + 22, 0x132 + destination * 22);
            }
            warrior.bytes[0x1a..0x1c].copy_from_slice(&((count - 1) as i16).to_be_bytes());
            warrior.bytes[0x132 + (count - 1) * 22..0x132 + 100 * 22].fill(0);
            let race = word(&warrior.bytes, 0x32)?.wrapping_sub(1);
            let movement = race_movements.get(race as usize).ok_or_else(|| {
                RuleError::Invalid("consumed item race movement is unavailable".into())
            })?;
            combat::calculate_movement(warrior, *movement)?;
            result.consumed += 1;
            result.sounds.push(655);
        }
    }
    Ok(result)
}

pub fn normalize_story_loot(warriors: &[Warrior]) -> Result<Vec<Warrior>, RuleError> {
    let mut result = warriors.to_vec();
    for warrior in &mut result {
        span(&warrior.bytes, 0, 3138)?;
        warrior.bytes[0x10..0x12].fill(0);
        warrior.bytes[0x9de] = 0;
        warrior.bytes[0x60..0x62].fill(0);
        for offset in (0x5e..0x66).step_by(2) {
            if word(&warrior.bytes, offset)? > 0 {
                warrior.bytes[offset..offset + 2].fill(0);
            }
        }
        if word(&warrior.bytes, 0x90)? > 0 {
            warrior.bytes[0x90..0x92].fill(0);
        }
    }
    Ok(result)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExperienceAward {
    pub warriors: Vec<Warrior>,
    pub share: i32,
}

pub fn award_story_experience(
    warriors: &[Warrior],
    amount: i32,
) -> Result<ExperienceAward, RuleError> {
    let mut result = normalize_story_loot(warriors)?;
    for warrior in &mut result {
        warrior.bytes[0x9df] = 1;
    }
    let eligible = result
        .iter()
        .filter(|warrior| {
            warrior.stamina().unwrap_or(0) > 0 && word(&warrior.bytes, 0x90).unwrap_or(-1) >= 0
        })
        .count() as i32;
    let rounded = amount as f32;
    if rounded as f64 >= i32::MAX as f64 || (rounded as f64) < i32::MIN as f64 {
        return Err(RuleError::Unsupported {
            action: "story.experience",
            dependency: "SANE reward conversion overflow",
        });
    }
    let share = if eligible == 0 {
        0
    } else {
        (rounded as i32 / eligible).wrapping_abs()
    };
    for warrior in &mut result {
        if warrior.stamina()? <= 0 || word(&warrior.bytes, 0x90)? < 0 {
            continue;
        }
        let experience =
            long(&warrior.bytes, 0x9d2)?
                .checked_add(share)
                .ok_or(RuleError::Unsupported {
                    action: "story.experience",
                    dependency: "SANE experience overflow",
                })?;
        let experience = if experience < 0 {
            i32::from(((experience >> 16) as i16).wrapping_abs())
        } else {
            experience
        };
        let remaining =
            long(&warrior.bytes, 0x9ce)?
                .checked_sub(share)
                .ok_or(RuleError::Unsupported {
                    action: "story.experience",
                    dependency: "SANE experience balance overflow",
                })?;
        warrior.bytes[0x9d2..0x9d6].copy_from_slice(&experience.to_be_bytes());
        warrior.bytes[0x9ce..0x9d2].copy_from_slice(&remaining.to_be_bytes());
    }
    Ok(ExperienceAward {
        warriors: result,
        share,
    })
}

pub fn grant_story_treasure(
    treasure: &[u8],
    content: &Content,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<Vec<InventoryRecord>, RuleError> {
    let treasure = span(treasure, 0, 88)?;
    if treasure[80..88].iter().any(|value| *value != 0) {
        return Err(RuleError::Unsupported {
            action: "story.treasure",
            dependency: "treasure credit and experience awards",
        });
    }
    let mut result = Vec::new();
    let mut candidate_random = random.clone();
    for slot in 0..20 {
        let id = word(treasure, slot * 2)?;
        if id > 0 {
            let enhancement = word(treasure, 40 + slot * 2)?;
            let enhancement = if enhancement == 0 {
                0
            } else {
                enhancement.wrapping_add(difficulty).max(1)
            };
            result.push(items::initialize_at_difficulty(
                id,
                enhancement,
                difficulty,
                content,
                &mut candidate_random,
            )?);
        }
    }
    *random = candidate_random;
    Ok(result)
}
