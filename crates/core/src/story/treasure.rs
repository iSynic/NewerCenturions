use crate::{RandomState, RuleError, items, story_inventory};
use centurions_content::{
    Content, InventoryRecord, Warrior,
    records::{RandomTreasureExtra, TreasureRecord},
};

/// What a treasure record gave: the warriors afterward, the items, the victory points share of each
/// eligible warrior and the credits and victory points awarded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Treasure {
    pub warriors: Vec<Warrior>,
    pub items: Vec<InventoryRecord>,
    pub share: i32,
    pub credits: i32,
    /// The awarded victory points.
    pub victory_points: i32,
    pub inventory_record: Option<InventoryRecord>,
}

/// Works out a treasure record: its items are generated, a negative victory point or credit amount is
/// rolled, and the victory points are divided among the warriors in the battle who can take it.
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
    // booty randomizes items, then victory points, then credits.
    let mut award = |amount: i32| {
        if amount < 0 {
            i32::from(next_random.bound(((amount >> 16) as i16).wrapping_abs()))
        } else {
            amount
        }
    };
    let amount = award(record.victory_points()?);
    let credits = award(record.credits()?);
    let eligible = result
        .iter()
        .filter(|warrior| {
            warrior.stamina().unwrap_or(0) > 0
                && warrior.record().computer_control().unwrap_or(-1) >= 0
                && warrior.record().in_battle() != 0
        })
        .count() as i32;
    let rounded = crate::rules::sane_long(f64::from(amount as f32));
    let share = if eligible == 0 { 0 } else { rounded / eligible };
    for warrior in &mut result {
        if warrior.stamina()? <= 0
            || warrior.record().computer_control()? < 0
            || warrior.record().in_battle() == 0
        {
            continue;
        }
        // The in-battle test above already bounds this read. The sum is exact;
        // only storing it as a 32-bit value saturates.
        let victory_points = crate::rules::sane_long(
            f64::from(warrior.record().upgrade_points()) + f64::from(share),
        );
        let victory_points = if victory_points < 0 {
            i32::from(((victory_points >> 16) as i16).wrapping_abs())
        } else {
            victory_points
        };
        let remaining = crate::rules::sane_long(
            f64::from(warrior.record().remaining_victory_points()?) - f64::from(share),
        );
        warrior.record_mut().set_upgrade_points(victory_points);
        warrior.record_mut().set_remaining_victory_points(remaining);
    }
    let inventory_record = items::sort_loot(&mut loot);
    *random = next_random;
    Ok(Treasure {
        warriors: result,
        items: loot,
        share,
        credits,
        victory_points: amount,
        inventory_record,
    })
}

/// Random Treasure fills the treasure record with |e1| items from
/// randrange(e2, e3) (Rand(|e1|) of them for negative e1). It returns the count
/// and the record; items past the item list overflow into the record's
/// enhancement and award fields.
///
/// Original: newland
pub(crate) fn random_items(
    random: &mut RandomState,
    extra: RandomTreasureExtra<'_>,
) -> Result<(i16, [u8; TreasureRecord::BYTES]), RuleError> {
    let requested = extra.count()?;
    if !(-44..=44).contains(&requested) {
        return Err(RuleError::Unsupported {
            action: "story.opcode",
            dependency: "random items past the record",
        });
    }
    let count = if requested < 0 {
        random.bound(requested.wrapping_abs())
    } else {
        requested
    };
    let mut record = [0; TreasureRecord::BYTES];
    for item in 0..count.max(0) as usize {
        let id = random.range(extra.minimum_item()?, extra.maximum_item()?);
        record[item * 2..item * 2 + 2].copy_from_slice(&id.to_be_bytes());
    }
    Ok((count, record))
}
