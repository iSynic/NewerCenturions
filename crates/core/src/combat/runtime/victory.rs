//! Winning a battle.
use crate::{RandomState, RuleError, items};
use centurions_content::combat::MonsterRecord;
use centurions_content::{
    Content, InventoryRecord, Warrior,
    records::{CharacterInformation, Condition, ItemDefinition},
};

/// Awards a won battle to the warriors and returns the share of victory points each gets.
///
/// Original: booty
pub fn victory_records(
    warriors: &mut [Warrior],
    enemies: &mut [MonsterRecord],
    random: &mut RandomState,
) -> Result<i32, RuleError> {
    Ok(victory_loot(warriors, enemies, random, None)?.0)
}

/// Awards a won battle. Each monster is worth victory points by its level, flying, stamina and own
/// victory points; the total is divided among the standing warriors still in the battle, added to
/// their upgrade points and taken from their remaining victory points. Each item a monster carries
/// drops by chance, always for item classes 23 to 25. The monsters' conditions are cleared and
/// they are set to faction 1. Returns the share and the loot.
///
/// Original: booty
pub fn victory_loot(
    warriors: &mut [Warrior],
    enemies: &mut [MonsterRecord],
    random: &mut RandomState,
    content: Option<&Content>,
) -> Result<(i32, Vec<InventoryRecord>), RuleError> {
    let mut loot_ids = vec![];
    for w in warriors.iter_mut() {
        crate::story::inventory::normalize_warrior_after_battle(w)?;
    }
    let mut victory_points = 0_i32;
    for m in enemies.iter_mut() {
        let level = m.record().hit_dice()?;
        if !matches!(level, 0..=3 | 7)
            || m.record().psi_points()? != 0
            || m.record().credit_drop_bound()? > 0
        {
            return Err(RuleError::Unsupported {
                action: "combat.rewards",
                dependency: "other monster rewards",
            });
        }
        random.range(0, m.record().credit_drop_bound()?);
        let flying = if m.record().flying_flag() != 0 {
            30 * i32::from(level)
        } else {
            0
        };
        let (base, multiplier) = if level == 7 {
            (200, 24)
        } else {
            [(15, 3), (30, 6), (45, 9), (65, 12)][level as usize]
        };
        victory_points = victory_points.wrapping_add(
            base + flying
                + multiplier * i32::from(m.record().stamina_max()?)
                + i32::from(m.record().victory_points()?),
        );
        for slot in 0..6 {
            let id = m.record().item(slot)?;
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
            if random.bound(150) < 70 || matches!(ItemDefinition(definition).item_type()?, 23..=25)
            {
                loot_ids.push(id);
            }
        }
        // This victory path also clears DrainPsi; ordinary booty does not.
        for condition in [
            Condition::Retreating,
            Condition::Helpless,
            Condition::Entangled,
            Condition::ScanBlocked,
            Condition::Confused,
            Condition::IncreasedDefense,
            Condition::DrainPsi,
        ] {
            m.record_mut().set_condition(condition, 0);
        }
        m.record_mut().clear_been_attacked();
        m.record_mut().set_faction(1);
    }
    let eligible = warriors
        .iter()
        .filter(|w| {
            w.stamina().unwrap_or(0) > 0
                && w.record().computer_control().unwrap_or(-1) >= 0
                && w.record().in_battle() != 0
        })
        .count() as i32;
    let share = if eligible != 0 {
        (victory_points as f32 as i32 / eligible).abs()
    } else {
        0
    };
    for w in warriors {
        if w.stamina()? > 0 && w.record().computer_control()? >= 0 && w.record().in_battle() != 0 {
            let points = CharacterInformation(&w.bytes)
                .upgrade_points()?
                .wrapping_add(share);
            w.record_mut().set_upgrade_points(points);
            let remaining = w.record().remaining_victory_points()?.wrapping_sub(share);
            w.record_mut().set_remaining_victory_points(remaining);
        }
    }
    let mut loot = vec![];
    if let Some(content) = content {
        for id in loot_ids {
            loot.push(items::initialize_at_difficulty(id, 0, 0, content, random)?);
        }
        loot.sort_by_key(|item| std::cmp::Reverse(item.category_code() as i8));
    }
    Ok((share, loot))
}
