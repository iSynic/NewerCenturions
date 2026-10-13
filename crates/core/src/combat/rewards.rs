use crate::{RandomState, RuleError, items, story_inventory};
use centurions_content::{
    Content, InventoryRecord, Warrior, combat::MonsterRecord, mission::long, span, word,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rewards {
    pub warriors: Vec<Warrior>,
    pub enemies: Vec<MonsterRecord>,
    pub items: Vec<InventoryRecord>,
    pub share: i32,
    pub total: i32,
    pub currency: i32,
    pub unavailable: i8,
    pub last_item_id: Option<i16>,
}

/// Original ordinary booty preparation; allocation and level promotion occur at Close.
pub fn prepare(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    content: &Content,
    difficulty: i16,
    npc_denominator: i16,
    currency: i32,
    random: &mut RandomState,
) -> Result<Rewards, RuleError> {
    if !(1..=6).contains(&warriors.len()) || enemies.len() > 100 {
        return Err(RuleError::Invalid("invalid battle reward capacity".into()));
    }
    let unavailable = warriors
        .iter()
        .filter(|warrior| {
            warrior.stamina().unwrap_or(0) < 1 || word(&warrior.bytes, 0x90).unwrap_or(0) < 0
        })
        .count() as i8;
    let mut warriors = story_inventory::normalize_story_loot(warriors)?;
    let mut enemies = enemies.to_vec();
    let mut next_random = random.clone();
    let mut total = 0_i32;
    let mut cash = 0_i32;
    let mut ids = vec![];
    let mut last_item_id = None;
    for enemy in &mut enemies {
        span(&enemy.bytes, 0, 288)?;
        let bytes = &mut enemy.bytes;
        for offset in [0xd8, 0xcf, 0xb7, 0xb4, 0xb3, 0xb2, 0x105] {
            bytes[offset] = 0;
        }
        if bytes[0xb0] == 255 && bytes[0xb1] == 1 {
            bytes[0xb0] = 1;
        }
        bytes[0xe..0x10].copy_from_slice(&1_i16.to_be_bytes());
        cash = cash.wrapping_add(i32::from(next_random.range(0, word(bytes, 0x3a)?)));
        let level = word(bytes, 0)?;
        let (mut base, multiplier): (i16, i16) = match level {
            0..=20 => {
                let bases = [
                    15, 30, 45, 65, 80, 100, 140, 200, 300, 450, 700, 1100, 1800, 2300, 2800, 3200,
                    3700, 4200, 4700, 5200, 5700,
                ];
                let multipliers = [
                    3, 6, 9, 12, 15, 18, 21, 24, 27, 30, 33, 36, 39, 42, 45, 50, 55, 60, 65, 70, 75,
                ];
                (bases[level as usize], multipliers[level as usize])
            }
            _ => (6200, 80),
        };
        if word(bytes, 0x10e)? != 0 {
            base = (f64::from(base) * 1.5) as i16;
        }
        if bytes[0x7f] != 0 {
            base = base.wrapping_add(level.wrapping_mul(30));
        }
        total = total.wrapping_add(i32::from(base));
        total = total.wrapping_add(i32::from(word(bytes, 0x54)?));
        total = total.wrapping_add(i32::from(multiplier.wrapping_mul(word(bytes, 0x112)?)));
        for offset in (0x26..0x32).step_by(2) {
            let id = word(bytes, offset)?;
            if id == 0 {
                continue;
            }
            let definition = content
                .items
                .get(usize::from(id.unsigned_abs()))
                .ok_or_else(|| RuleError::Invalid("monster loot outside item bands".into()))?;
            last_item_id = Some(id);
            let threshold = 70_i16.wrapping_sub(difficulty.wrapping_mul(20));
            let draw = next_random.bound(150);
            if draw < threshold || matches!(word(definition, 0x1a)?, 23..=25) {
                ids.push(id);
            }
        }
    }
    let eligible = warriors
        .iter()
        .filter(|w| {
            w.stamina().unwrap_or(0) > 0
                && word(&w.bytes, 0x90).unwrap_or(-1) >= 0
                && w.bytes[0x9df] != 0
        })
        .count() as i32;
    let denominator = eligible + i32::from(npc_denominator);
    let share = if denominator == 0 {
        0
    } else {
        sane_integer(f64::from(total as f32))?
            .checked_div(denominator)
            .ok_or(RuleError::Unsupported {
                action: "combat.rewards",
                dependency: "signed reward division overflow",
            })?
            .wrapping_abs()
    };
    let mut loot = vec![];
    for id in ids {
        loot.push(items::initialize_at_difficulty(
            id,
            0,
            difficulty,
            content,
            &mut next_random,
        )?);
        last_item_id = Some(id);
    }
    loot.sort_by_key(|item| std::cmp::Reverse(item.0[10] as i8));
    let rounded_share = f64::from(share as f32);
    for warrior in &mut warriors {
        if warrior.stamina()? <= 0 || word(&warrior.bytes, 0x90)? < 0 || warrior.bytes[0x9df] == 0 {
            continue;
        }
        let experience = sane_integer(f64::from(long(&warrior.bytes, 0x9d2)?) + rounded_share)?;
        let experience = if experience < 0 {
            i32::from(((experience >> 16) as i16).wrapping_abs())
        } else {
            experience
        };
        let remaining = sane_integer(f64::from(long(&warrior.bytes, 0x9ce)?) - rounded_share)?;
        warrior.bytes[0x9d2..0x9d6].copy_from_slice(&experience.to_be_bytes());
        warrior.bytes[0x9ce..0x9d2].copy_from_slice(&remaining.to_be_bytes());
    }
    *random = next_random;
    Ok(Rewards {
        warriors,
        enemies,
        items: loot,
        share,
        total,
        currency: currency.wrapping_add(cash),
        unavailable,
        last_item_id,
    })
}

fn sane_integer(value: f64) -> Result<i32, RuleError> {
    if !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&value) {
        return Err(RuleError::Unsupported {
            action: "combat.rewards",
            dependency: "SANE integer conversion overflow",
        });
    }
    Ok(value as i32)
}

pub fn close_without_promotion(warriors: &[Warrior]) -> Result<Vec<Warrior>, RuleError> {
    let mut result = warriors.to_vec();
    for warrior in &mut result {
        span(&warrior.bytes, 0, 3138)?;
        if warrior.stamina()? > 0 && long(&warrior.bytes, 0x9ce)? < 0 {
            return Err(RuleError::Unsupported {
                action: "combat.rewards.close",
                dependency: "warrior level promotion",
            });
        }
        warrior.bytes[0x9df] = 1;
    }
    Ok(result)
}
