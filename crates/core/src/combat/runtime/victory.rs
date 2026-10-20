//! Victory in the original combat execution order.
use crate::combat::put_word;
use crate::{RandomState, RuleError, items};
use centurions_content::combat::MonsterRecord;
use centurions_content::{Content, InventoryRecord, Warrior, word};

pub fn victory_records(
    warriors: &mut [Warrior],
    enemies: &mut [MonsterRecord],
    random: &mut RandomState,
) -> Result<i32, RuleError> {
    Ok(victory_loot(warriors, enemies, random, None)?.0)
}

pub fn victory_loot(
    warriors: &mut [Warrior],
    enemies: &mut [MonsterRecord],
    random: &mut RandomState,
    content: Option<&Content>,
) -> Result<(i32, Vec<InventoryRecord>), RuleError> {
    let mut loot_ids = vec![];
    for w in warriors.iter_mut() {
        let b = &mut w.bytes;
        put_word(b, 0x10, 0);
        b[0x9de] = 0;
        put_word(b, 0x60, 0);
        for o in (0x5e..0x66).step_by(2) {
            if word(b, o)? > 0 {
                put_word(b, o, 0);
            }
        }
        if word(b, 0x90)? > 0 {
            put_word(b, 0x90, 0);
        }
    }
    let mut experience = 0_i32;
    for m in enemies.iter_mut() {
        let b = &mut m.bytes;
        let level = word(b, 0)?;
        if !matches!(level, 0..=3 | 7) || word(b, 0x10e)? != 0 || word(b, 0x3a)? > 0 {
            return Err(RuleError::Unsupported {
                action: "combat.rewards",
                dependency: "other monster rewards",
            });
        }
        random.range(0, word(b, 0x3a)?);
        let flying = if b[0x7f] != 0 {
            30 * i32::from(level)
        } else {
            0
        };
        let (base, multiplier) = if level == 7 {
            (200, 24)
        } else {
            [(15, 3), (30, 6), (45, 9), (65, 12)][level as usize]
        };
        experience = experience.wrapping_add(
            base + flying + multiplier * i32::from(word(b, 0x112)?) + i32::from(word(b, 0x54)?),
        );
        for offset in (0x26..0x32).step_by(2) {
            let id = word(b, offset)?;
            if id == 0 {
                continue;
            }
            let content = content.ok_or(RuleError::Unsupported {
                action: "combat.rewards",
                dependency: "monster loot content",
            })?;
            let definition = content
                .items
                .get(id.unsigned_abs() as usize)
                .ok_or_else(|| RuleError::Invalid("monster loot outside item bands".into()))?;
            if random.bound(150) < 70 || matches!(word(definition, 0x1a)?, 23..=25) {
                loot_ids.push(id);
            }
        }
        for offset in [0xb2, 0xb3, 0xb4, 0xb7, 0xcf, 0xd8, 0xd4, 0x105] {
            b[offset] = 0;
        }
        put_word(b, 0xe, 1);
    }
    let eligible = warriors
        .iter()
        .filter(|w| {
            w.stamina().unwrap_or(0) > 0
                && word(&w.bytes, 0x90).unwrap_or(-1) >= 0
                && w.bytes[0x9df] != 0
        })
        .count() as i32;
    let share = if eligible != 0 {
        (experience as f32 as i32 / eligible).abs()
    } else {
        0
    };
    for w in warriors {
        if w.stamina()? > 0 && word(&w.bytes, 0x90)? >= 0 && w.bytes[0x9df] != 0 {
            for (offset, sign) in [(0x9d2, 1_i32), (0x9ce, -1)] {
                let current = i32::from_be_bytes(w.bytes[offset..offset + 4].try_into().unwrap());
                let next = current.wrapping_add(share.wrapping_mul(sign));
                w.bytes[offset..offset + 4].copy_from_slice(&next.to_be_bytes());
            }
        }
    }
    let mut loot = vec![];
    if let Some(content) = content {
        for id in loot_ids {
            loot.push(items::initialize_at_difficulty(id, 0, 0, content, random)?);
        }
        loot.sort_by_key(|item| std::cmp::Reverse(item.0[10] as i8));
    }
    Ok((share, loot))
}
