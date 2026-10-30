//! How booty decides what a battle pays.
use crate::{RandomState, RuleError, items};
use centurions_content::{Content, InventoryRecord, records::TreasureRecord};

/// What a treasure record pays before the monsters do: items, victory points and credits.
#[derive(Default)]
pub(super) struct RewardPrefix {
    pub items: Vec<InventoryRecord>,
    pub victory_points: i32,
    pub credits: i32,
    pub last_item: Option<i16>,
}

/// How a battle pays: by its monsters, nothing, or from a treasure record.
#[derive(Clone, Copy)]
pub enum RewardMode<'a> {
    Ordinary,
    NoLoot,
    Treasure(&'a [u8]),
}

impl<'a> RewardMode<'a> {
    /// Chooses how a battle pays from its mode number: 0 by the monsters, 5 nothing, 2 and 10 are
    /// reported unsupported, and any other mode a treasure record.
    pub(crate) fn select(mode: i16, record: &'a [u8]) -> Result<Self, RuleError> {
        match mode {
            0 => Ok(Self::Ordinary),
            5 => Ok(Self::NoLoot),
            2 | 10 => Err(RuleError::Unsupported {
                action: "combat.rewards",
                dependency: "alternate ordinary battle cleanup mode",
            }),
            _ => Ok(Self::Treasure(record)),
        }
    }

    /// Rolls what a treasure record pays: each item with its enhancement raised by the difficulty,
    /// and its victory points and credits, a negative amount being a random draw up to that bound.
    pub(super) fn prefix(
        self,
        content: &Content,
        difficulty: i16,
        random: &mut RandomState,
    ) -> Result<RewardPrefix, RuleError> {
        let Self::Treasure(bytes) = self else {
            return Ok(RewardPrefix::default());
        };
        let record = TreasureRecord::new(bytes)?;
        let mut loot = vec![];
        let mut last = None;
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
                    random,
                )?);
                last = Some(id);
            }
        }
        let mut award = |amount: i32| {
            if amount < 0 {
                i32::from(random.bound(((amount >> 16) as i16).wrapping_abs()))
            } else {
                amount
            }
        };
        let victory_points = award(record.victory_points()?);
        let credits = award(record.credits()?);
        Ok(RewardPrefix {
            items: loot,
            victory_points,
            credits,
            last_item: last,
        })
    }
}

/// What the booty needs to pay a battle: the difficulty, the deduction, the party's currency and
/// how the battle pays.
pub struct RewardContext<'a> {
    pub difficulty: i16,
    pub deduction: i16,
    pub currency: i32,
    pub mode: RewardMode<'a>,
}
