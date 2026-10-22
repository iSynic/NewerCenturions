//! Ten-byte EXTRA records read by the party story opcodes.
use crate::{ContentError, span, word};

/// EXTRA record for newland 108 (0x1414d8..0x141918) statistic adjustment.
pub struct StatisticAdjustmentExtra<'a>(pub &'a [u8]);

impl<'a> StatisticAdjustmentExtra<'a> {
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        span(extra, 0, 10).map(Self)
    }
    pub fn selector(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn amount(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
}

/// EXTRA record for newland 30 (0x1447e0..0x144a5c) party filtering.
pub struct PartyFilterExtra<'a>(pub &'a [u8]);

impl<'a> PartyFilterExtra<'a> {
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        span(extra, 0, 10).map(Self)
    }
    /// Skill index for the skill test, or attribute index otherwise.
    pub fn selector(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// Zero selects the skill test, which also adds this word to its chance.
    pub fn test_word(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// 1 adds every member to the mask; 2 adds living members.
    pub fn inclusion(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    pub fn invert(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    pub fn attribute_threshold(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}

/// EXTRA record for newland 52 party selection.
pub struct PartySelectionExtra<'a>(pub &'a [u8]);

impl<'a> PartySelectionExtra<'a> {
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        span(extra, 0, 10).map(Self)
    }
    pub fn kind(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// Comparison operand whose meaning depends on `kind`.
    pub fn value(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    pub fn eligibility(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
}
