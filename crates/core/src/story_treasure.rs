use crate::{RandomState, RuleError, items, story_inventory};
use centurions_content::{Content, InventoryRecord, Warrior, records::TreasureRecord};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Treasure {
    pub warriors: Vec<Warrior>,
    pub items: Vec<InventoryRecord>,
    pub share: i32,
    pub credits: i32,
    /// The awarded experience, booty's LONG 0x443392.
    pub experience: i32,
    pub inventory_record: Option<InventoryRecord>,
}

pub fn prepare(
    warriors: &[Warrior],
    record: &[u8],
    content: &Content,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<Treasure, RuleError> {
    let record = TreasureRecord::new(record)?;
    let mut result = story_inventory::normalize_story_loot(warriors)?;
    let mut next_random = random.clone();
    let mut loot = Vec::new();
    for slot in 0..TreasureRecord::SLOTS {
        let id = record.item(slot)?;
        if id > 0 {
            let enhancement = record.enhancement(slot)?;
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
    // booty105c68..105f18 randomizes items, then experience, then credits.
    let mut award = |amount: i32| {
        if amount < 0 {
            i32::from(next_random.bound(((amount >> 16) as i16).wrapping_abs()))
        } else {
            amount
        }
    };
    let amount = award(record.experience()?);
    let credits = award(record.credits()?);
    let eligible = result
        .iter()
        .filter(|warrior| {
            warrior.stamina().unwrap_or(0) > 0
                && warrior.record().automatic_mode().unwrap_or(-1) >= 0
                && warrior.record().active_flag() != 0
        })
        .count() as i32;
    let rounded = crate::rules::sane_long(f64::from(amount as f32));
    let share = if eligible == 0 { 0 } else { rounded / eligible };
    for warrior in &mut result {
        if warrior.stamina()? <= 0
            || warrior.record().automatic_mode()? < 0
            || warrior.record().active_flag() == 0
        {
            continue;
        }
        // The active-flag byte above already bounds this unchecked read.
        // The sum is exact in extended precision; only its LONG store saturates.
        let experience = crate::rules::sane_long(
            f64::from(warrior.record().upgrade_points()) + f64::from(share),
        );
        let experience = if experience < 0 {
            i32::from(((experience >> 16) as i16).wrapping_abs())
        } else {
            experience
        };
        let remaining = crate::rules::sane_long(
            f64::from(warrior.record().remaining_experience()?) - f64::from(share),
        );
        warrior.record_mut().set_upgrade_points(experience);
        warrior.record_mut().set_remaining_experience(remaining);
    }
    let inventory_record = items::sort_loot(&mut loot);
    *random = next_random;
    Ok(Treasure {
        warriors: result,
        items: loot,
        share,
        credits,
        experience: amount,
        inventory_record,
    })
}
