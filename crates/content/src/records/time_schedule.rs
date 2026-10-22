use super::{StoryRecord, put_word};
use crate::{ContentError, span, word};

/// One 40-byte entry of the mission's daily event schedule, scanned by the
/// CODE 1 timeclick day rollover. Bytes after offset 22 remain opaque.
pub struct ScheduleEntry<'a>(pub &'a [u8]);

impl<'a> ScheduleEntry<'a> {
    pub const BYTES: usize = 40;

    pub fn count(schedule: &[u8]) -> usize {
        schedule.len() / Self::BYTES
    }
    pub fn at(schedule: &'a [u8], slot: usize) -> Self {
        Self(&schedule[slot * Self::BYTES..(slot + 1) * Self::BYTES])
    }
    /// Zero ends the scan.
    pub fn due_day(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn interval(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// Percentage compared against a 0..100 draw.
    pub fn chance(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    pub fn script(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    pub fn map(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
    pub fn region(&self) -> Result<i16, ContentError> {
        word(self.0, 10)
    }
    pub fn x(&self) -> Result<i16, ContentError> {
        word(self.0, 12)
    }
    pub fn y(&self) -> Result<i16, ContentError> {
        word(self.0, 14)
    }
    pub fn required_item(&self) -> Result<i16, ContentError> {
        word(self.0, 16)
    }
    pub fn flag(&self) -> Result<i16, ContentError> {
        word(self.0, 18)
    }
    /// Positive values enable the location test; only 1 is recovered as a
    /// map/region/position requirement.
    pub fn location_mode(&self) -> Result<i16, ContentError> {
        word(self.0, 20)
    }
}

pub struct ScheduleEntryMut<'a>(pub &'a mut [u8]);

impl<'a> ScheduleEntryMut<'a> {
    pub fn at(schedule: &'a mut [u8], slot: usize) -> Self {
        Self(&mut schedule[slot * ScheduleEntry::BYTES..(slot + 1) * ScheduleEntry::BYTES])
    }
    /// Opcode 54 edits an entry only when it lies inside the loaded table.
    pub fn in_table(schedule: &'a mut [u8], slot: usize) -> Option<Self> {
        let offset = slot * ScheduleEntry::BYTES;
        schedule
            .get_mut(offset..offset + ScheduleEntry::BYTES)
            .map(Self)
    }
    pub fn due_day(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn set_due_day(&mut self, day: i16) {
        put_word(self.0, 0, day);
    }
    pub fn set_interval(&mut self, interval: i16) {
        put_word(self.0, 2, interval);
    }
    pub fn set_chance(&mut self, chance: i16) {
        put_word(self.0, 4, chance);
    }
}

/// An eight-byte rectangle at the start of the active region state, as the
/// schedule's location test reads it: top, left, bottom, right.
pub struct ScheduleRegion<'a>(pub &'a [u8]);

impl<'a> ScheduleRegion<'a> {
    pub fn read(regions: &'a [u8], region: usize) -> Result<Self, ContentError> {
        Ok(Self(span(regions, region * 8, 8)?))
    }
    pub fn top(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn left(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    pub fn bottom(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    pub fn right(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
}

/// Scheduled scripts (Data ED3): 42-byte event records indexed by the
/// magnitude of a schedule entry's script.
pub struct ScheduledScripts<'a>(pub &'a [u8]);

impl<'a> ScheduledScripts<'a> {
    pub fn record(&self, script: u16) -> Result<&'a [u8], ContentError> {
        span(
            self.0,
            usize::from(script) * StoryRecord::BYTES,
            StoryRecord::BYTES,
        )
    }
}
