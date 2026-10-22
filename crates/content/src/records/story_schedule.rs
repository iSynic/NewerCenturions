//! Opcode 54 schedule edit; the entries it changes use `ScheduleEntryMut`.
use crate::{ContentError, span, word};

/// EXTRA record for opcode 54 schedule edit.
pub struct ScheduleEditExtra<'a>(pub &'a [u8]);

impl<'a> ScheduleEditExtra<'a> {
    pub fn new(extra: &'a [u8]) -> Result<Self, ContentError> {
        span(extra, 0, 10).map(Self)
    }
    pub fn entry(&self) -> Result<u16, ContentError> {
        Ok(word(self.0, 0)? as u16)
    }
    pub fn chance(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    pub fn interval(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    pub fn reset(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    pub fn addition(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}
