//! Ten-byte EXTRA records read by the party story opcodes.
use crate::{ContentError, span, word};

/// EXTRA record for opcode 108 statistic adjustment.
pub struct StatisticAdjustmentExtra<'a>(pub &'a [u8]);

impl<'a> StatisticAdjustmentExtra<'a> {
    /// Views an EXTRA record, which must hold all ten bytes.
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        span(extra, 0, 10).map(Self)
    }
    /// Which statistic to change.
    pub fn selector(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// How much to raise it; negative lowers it.
    pub fn amount(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
}

/// EXTRA record for opcode 30 party filtering.
pub struct PartyFilterExtra<'a>(pub &'a [u8]);

impl<'a> PartyFilterExtra<'a> {
    /// Views an EXTRA record, which must hold all ten bytes.
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        span(extra, 0, 10).map(Self)
    }
    /// Skill index for the skill test, or attribute index otherwise.
    pub fn selector(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// Zero selects the skill test; any other value selects the attribute test.
    pub fn test_word(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// 1 adds every member to the mask; 2 adds living members.
    pub fn inclusion(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// Nonzero picks the members the test does not match.
    pub fn invert(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    /// An attribute check passes above this value.
    pub fn attribute_threshold(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}

/// EXTRA record for opcode 52 party selection.
pub struct PartySelectionExtra<'a>(pub &'a [u8]);

impl<'a> PartySelectionExtra<'a> {
    /// Views an EXTRA record, which must hold all ten bytes.
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        span(extra, 0, 10).map(Self)
    }
    /// The kind of check that picks members.
    pub fn kind(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// Comparison operand whose meaning depends on `kind`.
    pub fn value(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// Which members are considered: 0 every member, 1 only those with stamina above zero.
    pub fn eligibility(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
}
