//! Ten-byte EXTRA records read by the story variable and item branch opcodes.
use crate::{ContentError, span, word};

/// EXTRA record for opcode 76 variable update.
pub struct VariableUpdateExtra<'a>(pub &'a [u8]);

impl<'a> VariableUpdateExtra<'a> {
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        span(extra, 0, 10).map(Self)
    }
    pub fn variable(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn amount(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// One more than the branch mode.
    pub fn branch(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    pub fn threshold(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    pub fn target(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}

/// EXTRA record for opcode 77 variable predicate.
pub struct VariablePredicateExtra<'a>(pub &'a [u8]);

impl<'a> VariablePredicateExtra<'a> {
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        span(extra, 0, 10).map(Self)
    }
    pub fn variable(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn threshold(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    pub fn branch(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    pub fn target(&self, matched: bool) -> Result<i16, ContentError> {
        word(self.0, if matched { 8 } else { 6 })
    }
}

/// EXTRA record for opcode 21 party item condition.
pub struct PartyItemConditionExtra<'a>(pub &'a [u8]);

impl<'a> PartyItemConditionExtra<'a> {
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        span(extra, 0, 10).map(Self)
    }
    pub fn item(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn found_branch(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// 0 replaces the script and 3 ends it.
    pub fn missing_action(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    pub fn found_target(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    pub fn missing_script(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}
