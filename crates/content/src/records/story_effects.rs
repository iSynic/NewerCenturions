//! EXTRA and table records consumed by newland story effects. Constructors
//! keep the callers' original bounds checks; accessors return signed WORDs.
use super::{ExtraTable, ItemDefinition};
use crate::{ContentError, mission::long, span, word};

/// Encounter EXTRA for opcode 120 actor replacement (newland 0x1404ba..0x140734).
pub struct ActorReplacementExtra<'a>(pub &'a [u8]);

impl<'a> ActorReplacementExtra<'a> {
    pub fn select(table: &'a [u8], id: i16) -> Result<Self, ContentError> {
        Ok(Self(ExtraTable(table).by_magnitude(id)?))
    }
    /// -1 removes the selected NPC; other values name the replacement.
    pub fn replacement(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn sound(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    pub fn introduction(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    pub fn conclusion(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}

/// Encounter EXTRA for opcode 123: five signed NPC class words.
pub struct RetreatClassesExtra<'a>(pub &'a [u8]);

impl RetreatClassesExtra<'_> {
    pub fn classes(&self) -> impl Iterator<Item = i16> + '_ {
        self.0
            .as_chunks::<2>()
            .0
            .iter()
            .map(|bytes| i16::from_be_bytes(*bytes))
    }
}

/// EXTRA for opcodes 15/16 selected and party health.
pub struct PartyHealthExtra<'a>(pub &'a [u8]);

impl<'a> PartyHealthExtra<'a> {
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        Ok(Self(span(extra, 0, ExtraTable::RECORD_BYTES)?))
    }
    pub fn multiplier(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn low(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    pub fn high(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    pub fn sound(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    pub fn dialogue(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}

/// EXTRA for the warrior-choice skill predicate.
pub struct SkillCheckExtra<'a>(pub &'a [u8]);

impl<'a> SkillCheckExtra<'a> {
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        Ok(Self(span(extra, 0, ExtraTable::RECORD_BYTES)?))
    }
    /// A special-skill index or an attribute index, depending on the check kind.
    pub fn subject(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn threshold(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    pub fn checks_special_skill(&self) -> Result<bool, ContentError> {
        Ok(word(self.0, 4)? == 0)
    }
    pub fn script(&self, success: bool) -> Result<i16, ContentError> {
        word(self.0, if success { 6 } else { 8 })
    }
}

/// EXTRA for opcodes 17/18 scripted spells.
pub struct ScriptedSpellExtra<'a>(pub &'a [u8]);

impl<'a> ScriptedSpellExtra<'a> {
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        Ok(Self(span(extra, 0, ExtraTable::RECORD_BYTES)?))
    }
    pub fn spell(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn attack_count(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
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
    pub const BYTES: usize = ExtraTable::RECORD_BYTES;

    pub fn battle(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn maximum_battle(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    pub fn escape_script(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    pub fn sound(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    pub fn text(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}

/// 88-byte map treasure record read by booty (0x105c68..0x105f18).
pub struct TreasureRecord<'a>(pub &'a [u8]);

pub struct TreasureRecordMut<'a>(pub &'a mut [u8]);

impl TreasureRecordMut<'_> {
    /// booty105c68 consumes positive item slots, leaving signed nonitems and awards.
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
    pub const BYTES: usize = 88;
    pub const SLOTS: usize = 20;

    pub fn experience_award(amount: i32) -> [u8; 88] {
        let mut record = [0; Self::BYTES];
        record[80..84].copy_from_slice(&amount.to_be_bytes());
        record
    }

    pub fn new(record: &'a [u8]) -> Result<Self, ContentError> {
        Ok(Self(span(record, 0, Self::BYTES)?))
    }
    pub fn has_awards(&self) -> bool {
        self.0[80..88].iter().any(|value| *value != 0)
    }
    pub fn item(&self, slot: usize) -> Result<i16, ContentError> {
        word(self.0, slot * 2)
    }
    pub fn enhancement(&self, slot: usize) -> Result<i16, ContentError> {
        word(self.0, 40 + slot * 2)
    }
    /// Negative awards are random bounds in their high WORD.
    pub fn experience(&self) -> Result<i32, ContentError> {
        long(self.0, 80)
    }
    pub fn credits(&self) -> Result<i32, ContentError> {
        long(self.0, 84)
    }
}

impl ItemDefinition<'_> {
    /// The shared WORD at 0x32 (see story_script), read by spell restrictions
    /// as the item work alias.
    pub fn restriction_alias(&self) -> Result<i16, ContentError> {
        word(self.0, 0x32)
    }
}
