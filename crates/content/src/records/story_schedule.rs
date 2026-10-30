//! Opcode 54 schedule edit; the entries it changes use `ScheduleEntryMut`.
use crate::{ContentError, span, word};

/// EXTRA record for opcode 54 schedule edit.
pub struct ScheduleEditExtra<'a>(pub &'a [u8]);

impl<'a> ScheduleEditExtra<'a> {
    /// Views an EXTRA record, which must hold all ten bytes.
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        span(extra, 0, 10).map(Self)
    }
    /// The schedule entry to change.
    pub fn entry(&self) -> Result<u16, ContentError> {
        Ok(word(self.0, 0)? as u16)
    }
    /// The entry's new percent chance of activating; -1 keeps it.
    pub fn chance(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// The entry's new day increment; -1 keeps it.
    pub fn interval(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// Whether to reset the entry to the current date.
    pub fn reset(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    /// Days to add to the entry's next activation; -1 adds none.
    pub fn addition(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}
