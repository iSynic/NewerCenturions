//! Story item EXTRA records and the 88-byte treasure record (Data TD).
use crate::{ContentError, span, word};

/// EXTRA record for opcode 38 item prerequisite branch.
pub struct ItemPrerequisiteExtra<'a>(pub &'a [u8]);

impl<'a> ItemPrerequisiteExtra<'a> {
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        span(extra, 0, 10).map(Self)
    }
    pub fn item(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// 0 requires absence, 1 presence and 2 always matches.
    pub fn mode(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
}

/// EXTRA record for opcode 22 party item consumption (newland 0x14666e).
pub struct ItemConsumptionExtra<'a>(pub &'a [u8]);

impl<'a> ItemConsumptionExtra<'a> {
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        span(extra, 0, 10).map(Self)
    }
    pub fn item(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn maximum(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// 1 removes, 2 changes quantity and 3 replaces the item.
    pub fn mode(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    pub fn quantity_change(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    pub fn replacement(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}
