//! Victory in the original combat execution order.
use crate::{RandomState, RuleError, items};
use centurions_content::combat::MonsterRecord;
use centurions_content::{
    Content, InventoryRecord, Warrior,
    records::{CharacterInformation, Condition, ItemDefinition},
};

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
        crate::story_inventory::normalize_warrior_after_battle(w)?;
    }
    let mut experience = 0_i32;
    for m in enemies.iter_mut() {
        let level = m.record().level()?;
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
        experience = experience.wrapping_add(
            base + flying
                + multiplier * i32::from(m.record().maximum_stamina()?)
                + i32::from(m.record().experience_bonus()?),
        );
        for slot in 0..6 {
            let id = m.record().loot_item(slot)?;
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
            if random.bound(150) < 70 || matches!(ItemDefinition(definition).category()?, 23..=25) {
                loot_ids.push(id);
            }
        }
        // The older victory entry also clears DrainPsi; ordinary booty does not.
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
        m.record_mut().clear_damage_marker();
        m.record_mut().set_faction(1);
    }
    let eligible = warriors
        .iter()
        .filter(|w| {
            w.stamina().unwrap_or(0) > 0
                && w.record().automatic_mode().unwrap_or(-1) >= 0
                && w.record().active_flag() != 0
        })
        .count() as i32;
    let share = if eligible != 0 {
        (experience as f32 as i32 / eligible).abs()
    } else {
        0
    };
    for w in warriors {
        if w.stamina()? > 0 && w.record().automatic_mode()? >= 0 && w.record().active_flag() != 0 {
            let points = CharacterInformation(&w.bytes)
                .upgrade_points()?
                .wrapping_add(share);
            w.record_mut().set_upgrade_points(points);
            let remaining = w.record().remaining_experience()?.wrapping_sub(share);
            w.record_mut().set_remaining_experience(remaining);
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
