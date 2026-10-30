pub use super::reward_mode::{RewardContext, RewardMode};
use crate::{RandomState, RuleError, items, story_inventory};
use centurions_content::{
    Content, InventoryRecord, Warrior,
    combat::MonsterRecord,
    records::{CharacterInformation, Condition, ItemDefinition},
    span,
};

/// What a battle paid: the warriors and monsters after it, the items, the shares of experience, the
/// credits and currency, and the party deaths.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rewards {
    pub warriors: Vec<Warrior>,
    pub monsters: Vec<MonsterRecord>,
    pub items: Vec<InventoryRecord>,
    pub share: i32,
    pub total: i32,
    pub currency: i32,
    /// The credits this award pays.
    pub credits: i32,
    pub kill_party: i8,
    pub last_item_id: Option<i16>,
    pub inventory_record: Option<InventoryRecord>,
}

/// Prepares an ordinary battle's booty; sharing and promotion happen when the
/// screen closes.
///
/// Original: booty
pub fn prepare(
    warriors: &[Warrior],
    monsters: &[MonsterRecord],
    content: &Content,
    difficulty: i16,
    deduction: i16,
    currency: i32,
    random: &mut RandomState,
) -> Result<Rewards, RuleError> {
    prepare_mode(
        warriors,
        monsters,
        content,
        RewardContext {
            difficulty,
            deduction,
            currency,
            mode: RewardMode::Ordinary,
        },
        random,
    )
}

/// Prepares a battle's booty under the given mode: the monsters' experience, credits and item
/// drops, any treasure record's items, the experience share per eligible warrior, and the count of
/// fallen warriors.
///
/// Original: booty
pub fn prepare_mode(
    warriors: &[Warrior],
    monsters: &[MonsterRecord],
    content: &Content,
    context: RewardContext<'_>,
    random: &mut RandomState,
) -> Result<Rewards, RuleError> {
    let RewardContext {
        difficulty,
        deduction,
        currency,
        mode,
    } = context;
    if !(1..=6).contains(&warriors.len()) || monsters.len() > 100 {
        return Err(RuleError::Invalid("invalid battle reward capacity".into()));
    }
    let kill_party = warriors
        .iter()
        .filter(|warrior| {
            warrior.stamina().unwrap_or(0) < 1
                || warrior.record().computer_control().unwrap_or(0) < 0
        })
        .count() as i8;
    let mut warriors = story_inventory::normalize_story_loot(warriors)?;
    let mut monsters = monsters.to_vec();
    let mut next_random = random.clone();
    let super::reward_mode::RewardPrefix {
        items: mut loot,
        experience: mut total,
        credits: mut cash,
        last_item: mut last_item_id,
    } = mode.prefix(content, difficulty, &mut next_random)?;
    let mut ids = vec![];
    for (index, monster) in monsters.iter_mut().enumerate() {
        span(&monster.bytes, 0, 288)?;
        for condition in [
            Condition::IncreasedDefense,
            Condition::Confused,
            Condition::ScanBlocked,
            Condition::Entangled,
            Condition::Helpless,
            Condition::Retreating,
        ] {
            monster.record_mut().set_condition(condition, 0);
        }
        monster.record_mut().clear_been_attacked();
        let captured =
            monster.record().mechanical_state() == 255 && monster.record().capture_flag() == 1;
        if captured {
            monster.record_mut().set_mechanical_state(1);
        }
        // booty 0x106064 reuses the award cursor, so mode 5 visits only NPC 0.
        if matches!(mode, RewardMode::NoLoot) && index > 0 {
            if captured || monster.record().survivor_marker()? < 0 {
                monster.record_mut().set_faction(0);
            }
            continue;
        }
        monster.record_mut().set_faction(1);
        cash = cash.wrapping_add(i32::from(
            next_random.range(0, monster.record().credit_drop_bound()?),
        ));
        let level = monster.record().hit_dice()?;
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
        if monster.record().psi_points()? != 0 {
            base = (f64::from(base) * 1.5) as i16;
        }
        if monster.record().flying_flag() != 0 {
            base = base.wrapping_add(level.wrapping_mul(30));
        }
        total = total.wrapping_add(i32::from(base));
        total = total.wrapping_add(i32::from(monster.record().experience()?));
        total = total.wrapping_add(i32::from(
            multiplier.wrapping_mul(monster.record().stamina_max()?),
        ));
        if matches!(mode, RewardMode::NoLoot) {
            cash = 0;
            continue;
        }
        for slot in 0..6 {
            let id = monster.record().item(slot)?;
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
            if draw < threshold || matches!(ItemDefinition(definition).item_type()?, 23..=25) {
                ids.push(id);
            }
        }
    }
    let eligible = warriors
        .iter()
        .filter(|w| {
            w.stamina().unwrap_or(0) > 0
                && w.record().computer_control().unwrap_or(-1) >= 0
                && w.record().in_battle() != 0
        })
        .count() as i32;
    let denominator = eligible + i32::from(deduction);
    let share = if denominator == 0 {
        0
    } else {
        // A negative NPC denominator can make LONG_MIN / -1, which wraps.
        crate::rules::sane_long(f64::from(total as f32))
            .wrapping_div(denominator)
            .wrapping_abs()
    };
    for id in ids {
        if matches!(mode, RewardMode::Treasure(_)) {
            // booty skips randomitem for NPC drops after consuming treasure.
            // Its cold 22-byte crate template at41fc32 contains only zeroes.
            let mut item = InventoryRecord([0; 22]);
            item.set_id(id);
            loot.push(item);
            continue;
        }
        loot.push(items::initialize_at_difficulty(
            id,
            0,
            difficulty,
            content,
            &mut next_random,
        )?);
        last_item_id = Some(id);
    }
    // The native sort count is populated only by the later randomitem pass.
    let inventory_record = if matches!(mode, RewardMode::Treasure(_)) {
        None
    } else {
        items::sort_loot(&mut loot)
    };
    let rounded_share = f64::from(share as f32);
    for warrior in &mut warriors {
        if warrior.stamina()? <= 0
            || warrior.record().computer_control()? < 0
            || warrior.record().in_battle() == 0
        {
            continue;
        }
        let experience = crate::rules::sane_long(
            f64::from(CharacterInformation(&warrior.bytes).upgrade_points()?) + rounded_share,
        );
        let experience = if experience < 0 {
            i32::from(((experience >> 16) as i16).wrapping_abs())
        } else {
            experience
        };
        let remaining = crate::rules::sane_long(
            f64::from(warrior.record().remaining_experience()?) - rounded_share,
        );
        warrior.record_mut().set_upgrade_points(experience);
        warrior.record_mut().set_remaining_experience(remaining);
    }
    *random = next_random;
    Ok(Rewards {
        warriors,
        monsters,
        items: loot,
        share,
        total,
        currency: currency.wrapping_add(cash),
        credits: cash,
        kill_party,
        last_item_id,
        inventory_record,
    })
}

pub fn close_without_promotion(warriors: &[Warrior]) -> Result<Vec<Warrior>, RuleError> {
    let mut result = warriors.to_vec();
    for warrior in &mut result {
        span(&warrior.bytes, 0, 3138)?;
        if warrior.stamina()? > 0 && warrior.record().remaining_experience()? < 0 {
            return Err(RuleError::Unsupported {
                action: "combat.rewards.close",
                dependency: "warrior level promotion",
            });
        }
        warrior.record_mut().set_in_battle(1);
    }
    Ok(result)
}
