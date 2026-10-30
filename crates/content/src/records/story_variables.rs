//! Ten-byte EXTRA records read by the story variable and item branch opcodes.
use crate::{ContentError, span, word};

/// EXTRA record for opcode 76 variable update.
pub struct VariableUpdateExtra<'a>(pub &'a [u8]);

impl<'a> VariableUpdateExtra<'a> {
    /// Views an EXTRA record, which must hold all ten bytes.
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        span(extra, 0, 10).map(Self)
    }
    /// The mission flag to change.
    pub fn variable(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// Added to the flag, which stays within -127 and 127.
    pub fn amount(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// One more than the branch mode.
    pub fn branch(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// Flag value from which the branch is taken; zero never branches.
    pub fn threshold(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    /// The script or encounter number the branch goes to.
    pub fn target(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}

/// EXTRA record for opcode 77 variable predicate.
pub struct VariablePredicateExtra<'a>(pub &'a [u8]);

impl<'a> VariablePredicateExtra<'a> {
    /// Views an EXTRA record, which must hold all ten bytes.
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        span(extra, 0, 10).map(Self)
    }
    /// The mission flag to test.
    pub fn variable(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// The flag matches when it is at least this value.
    pub fn threshold(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// Where the branch goes: 0 another script, 1 an encounter, 2 a typed encounter;
    /// any other value carries on.
    pub fn branch(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// The script or encounter number the branch goes to when the flag matches or
    /// not; zero carries on.
    pub fn target(&self, matched: bool) -> Result<i16, ContentError> {
        word(self.0, if matched { 8 } else { 6 })
    }
}

/// EXTRA record for opcode 21 party item condition.
pub struct PartyItemConditionExtra<'a>(pub &'a [u8]);

impl<'a> PartyItemConditionExtra<'a> {
    /// Views an EXTRA record, which must hold all ten bytes.
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        span(extra, 0, 10).map(Self)
    }
    /// The item to look for in the party.
    pub fn item(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// Where the branch goes when the item is found: 0 another script, 1 an encounter, 2 a typed
    /// encounter; any other value carries on.
    pub fn found_branch(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// 0 replaces the script and 3 ends it.
    pub fn missing_action(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// The script or encounter number taken when the item is found.
    pub fn found_target(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    /// The script that replaces this one when the item is missing and the missing
    /// action is 0.
    pub fn missing_script(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}
