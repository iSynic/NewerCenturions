use crate::{RandomState, RuleError, items, story_inventory};
use centurions_content::{Content, InventoryRecord, Warrior, mission::long, span, word};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Treasure {
    pub warriors: Vec<Warrior>,
    pub items: Vec<InventoryRecord>,
    pub share: i32,
}

pub fn prepare(
    warriors: &[Warrior],
    record: &[u8],
    content: &Content,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<Treasure, RuleError> {
    let record = span(record, 0, 88)?;
    let amount = long(record, 80)?;
    if amount < 0 || long(record, 84)? != 0 {
        return Err(RuleError::Unsupported {
            action: "story.treasure",
            dependency: "random experience or treasure credit awards",
        });
    }
    let mut result = story_inventory::normalize_story_loot(warriors)?;
    let mut next_random = random.clone();
    let mut loot = Vec::new();
    for slot in 0..20 {
        let id = word(record, slot * 2)?;
        if id > 0 {
            let enhancement = word(record, 40 + slot * 2)?;
            let enhancement = if enhancement == 0 {
                0
            } else {
                enhancement.wrapping_add(difficulty).max(1)
            };
            loot.push(items::initialize_at_difficulty(
                id,
                enhancement,
                difficulty,
                content,
                &mut next_random,
            )?);
        }
    }
    let eligible = result
        .iter()
        .filter(|warrior| {
            warrior.stamina().unwrap_or(0) > 0
                && word(&warrior.bytes, 0x90).unwrap_or(-1) >= 0
                && warrior.bytes[0x9df] != 0
        })
        .count() as i32;
    let rounded = amount as f32;
    if rounded as f64 >= i32::MAX as f64 {
        return Err(RuleError::Unsupported {
            action: "story.treasure",
            dependency: "SANE reward conversion overflow",
        });
    }
    let share = if eligible == 0 {
        0
    } else {
        rounded as i32 / eligible
    };
    for warrior in &mut result {
        if warrior.stamina()? <= 0 || word(&warrior.bytes, 0x90)? < 0 || warrior.bytes[0x9df] == 0 {
            continue;
        }
        let experience =
            long(&warrior.bytes, 0x9d2)?
                .checked_add(share)
                .ok_or(RuleError::Unsupported {
                    action: "story.treasure",
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
                    action: "story.treasure",
                    dependency: "SANE experience balance overflow",
                })?;
        warrior.bytes[0x9d2..0x9d6].copy_from_slice(&experience.to_be_bytes());
        warrior.bytes[0x9ce..0x9d2].copy_from_slice(&remaining.to_be_bytes());
    }
    *random = next_random;
    Ok(Treasure {
        warriors: result,
        items: loot,
        share,
    })
}
