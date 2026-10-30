//! Story item EXTRA records and the 88-byte treasure record (Data TD).
use crate::{ContentError, span, word};

/// EXTRA record for opcode 38 item prerequisite branch.
pub struct ItemPrerequisiteExtra<'a>(pub &'a [u8]);

impl<'a> ItemPrerequisiteExtra<'a> {
    /// Views an EXTRA record, which must hold all ten bytes.
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        span(extra, 0, 10).map(Self)
    }
    /// The item to look for.
    pub fn item(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// 0 requires absence, 1 presence and 2 always matches.
    pub fn mode(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
}

/// EXTRA record for opcode 22 party item consumption.
pub struct ItemConsumptionExtra<'a>(pub &'a [u8]);

impl<'a> ItemConsumptionExtra<'a> {
    /// Views an EXTRA record, which must hold all ten bytes.
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        span(extra, 0, 10).map(Self)
    }
    /// The item to change.
    pub fn item(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// How many of the item to affect.
    pub fn maximum(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// 1 removes, 2 changes quantity and 3 replaces the item.
    pub fn mode(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// Charges to add; negative drains them.
    pub fn quantity_change(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    /// The item that replaces it.
    pub fn replacement(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}
