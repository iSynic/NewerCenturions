//! EXTRA and table records read by story effects. Constructors make the
//! callers' bounds checks; accessors return signed words.
use super::{ExtraTable, ItemDefinition};
use crate::bytes::{long, put_long};
use crate::{ContentError, span, word};

/// Encounter EXTRA for opcode 120, which replaces a monster.
pub struct ActorReplacementExtra<'a>(pub &'a [u8]);

impl<'a> ActorReplacementExtra<'a> {
    /// The EXTRA record for the signed parameter `id`, picked by its magnitude.
    pub fn select(table: &'a [u8], id: i16) -> Result<Self, ContentError> {
        Ok(Self(ExtraTable(table).by_magnitude(id)?))
    }
    /// -1 removes the selected monster; other values name the replacement.
    pub fn replacement(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// Sound played as the monster is replaced.
    pub fn sound(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// Dialogue shown before the replacement.
    pub fn introduction(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    /// Dialogue shown after the replacement.
    pub fn conclusion(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}

/// Encounter EXTRA for opcode 123: five signed monster id words.
pub struct RetreatClassesExtra<'a>(pub &'a [u8]);

impl RetreatClassesExtra<'_> {
    /// The five monster ids that retreat; zero is unused.
    pub fn classes(&self) -> impl Iterator<Item = i16> + '_ {
        self.0
            .as_chunks::<2>()
            .0
            .iter()
            .map(|bytes| i16::from_be_bytes(*bytes))
    }
}

/// EXTRA for opcodes 15 and 16: health of the picked warrior or of the
/// whole party.
pub struct PartyHealthExtra<'a>(pub &'a [u8]);

impl<'a> PartyHealthExtra<'a> {
    /// Views an EXTRA record, which must hold all ten bytes.
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        Ok(Self(span(extra, 0, ExtraTable::RECORD_BYTES)?))
    }
    /// Factor the drawn amount is multiplied by.
    pub fn multiplier(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// Lowest amount drawn for each member.
    pub fn low(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// Highest amount drawn for each member.
    pub fn high(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// Sound played as the health changes.
    pub fn sound(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    /// Dialogue shown as the health changes.
    pub fn dialogue(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}

/// EXTRA for the warrior-choice skill predicate.
pub struct SkillCheckExtra<'a>(pub &'a [u8]);

impl<'a> SkillCheckExtra<'a> {
    /// Views an EXTRA record, which must hold all ten bytes.
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        Ok(Self(span(extra, 0, ExtraTable::RECORD_BYTES)?))
    }
    /// A special-skill index or an attribute index, depending on the check kind.
    pub fn subject(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// The value checked against: a special skill with its bonus must reach it, an attribute
    /// must exceed it.
    pub fn threshold(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// Whether the check is against a special skill rather than an attribute.
    pub fn checks_special_skill(&self) -> Result<bool, ContentError> {
        Ok(word(self.0, 4)? == 0)
    }
    /// The script that follows a success or a failure.
    pub fn script(&self, success: bool) -> Result<i16, ContentError> {
        word(self.0, if success { 6 } else { 8 })
    }
}

/// EXTRA for opcodes 17/18 scripted spells.
pub struct ScriptedSpellExtra<'a>(pub &'a [u8]);

impl<'a> ScriptedSpellExtra<'a> {
    /// Views an EXTRA record, which must hold all ten bytes.
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        Ok(Self(span(extra, 0, ExtraTable::RECORD_BYTES)?))
    }
    /// The scan the story casts.
    pub fn spell(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// How many attacks the scan makes.
    pub fn attack_count(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// Bonus to the target's saving throw.
    pub fn save_bonus(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// Nonzero replaces the packet's saving-throw mode with mode 3.
    pub fn overrides_saving_throw(&self) -> Result<bool, ContentError> {
        Ok(word(self.0, 6)? != 0)
    }
}

/// Encounter EXTRA for opcode 56 NPC battles. Callers check its exact length.
pub struct NpcBattleExtra<'a>(pub &'a [u8]);

impl NpcBattleExtra<'_> {
    /// Size of an NPC battle EXTRA record.
    pub const BYTES: usize = ExtraTable::RECORD_BYTES;

    /// The battle the party fights; with `maximum_battle`, the lowest number drawn.
    pub fn battle(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// Highest battle number to draw from; zero fights `battle` itself.
    pub fn maximum_battle(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// The script that runs if the party escapes; -1 steps the party back instead.
    pub fn escape_script(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// Sound played as the battle begins.
    pub fn sound(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    /// Message shown as the battle begins.
    pub fn text(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}

/// An 88-byte map treasure record (`treasure`), handed out as booty.
pub struct TreasureRecord<'a>(pub &'a [u8]);

/// A treasure record as a view that can change it.
pub struct TreasureRecordMut<'a>(pub &'a mut [u8]);

impl TreasureRecordMut<'_> {
    /// Uses up the positive item slots, leaving the other signed entries and
    /// the awards.
    pub fn consume_items(&mut self) -> Result<(), ContentError> {
        span(self.0, 0, TreasureRecord::BYTES)?;
        for slot in 0..TreasureRecord::SLOTS {
            if word(self.0, slot * 2)? > 0 {
                super::put_word(self.0, slot * 2, 0);
                super::put_word(self.0, 40 + slot * 2, 0);
            }
        }
        Ok(())
    }
}

impl<'a> TreasureRecord<'a> {
    /// Size of a treasure record.
    pub const BYTES: usize = 88;
    /// Number of item slots in a treasure record.
    pub const SLOTS: usize = 20;

    /// A treasure record that gives only `amount` experience.
    pub fn experience_award(amount: i32) -> [u8; 88] {
        let mut record = [0; Self::BYTES];
        put_long(&mut record, 80, amount);
        record
    }

    /// Views a treasure record, which must be complete.
    pub fn new(record: &'a [u8]) -> Result<Self, ContentError> {
        Ok(Self(span(record, 0, Self::BYTES)?))
    }
    /// Whether the record gives experience or credits.
    pub fn has_awards(&self) -> bool {
        self.0[80..88].iter().any(|value| *value != 0)
    }
    /// The item in slot `slot`; a negative entry is not a plain item.
    pub fn item(&self, slot: usize) -> Result<i16, ContentError> {
        word(self.0, slot * 2)
    }
    /// The enhancement of the item in slot `slot`.
    pub fn enhancement(&self, slot: usize) -> Result<i16, ContentError> {
        word(self.0, 40 + slot * 2)
    }
    /// A negative award keeps a random bound in its high word.
    pub fn experience(&self) -> Result<i32, ContentError> {
        long(self.0, 80)
    }
    /// Credits the treasure gives.
    pub fn credits(&self) -> Result<i32, ContentError> {
        long(self.0, 84)
    }
}

impl ItemDefinition<'_> {
    /// The word shared with `story_script`, which spell restrictions read from
    /// the loaded item record.
    pub fn restriction_alias(&self) -> Result<i16, ContentError> {
        word(self.0, 0x32)
    }
}
