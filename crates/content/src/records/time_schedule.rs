use super::{StoryRecord, put_word};
use crate::{ContentError, span, word};

/// One 40-byte entry of the mission's daily event schedule, checked each time
/// the day turns over. Bytes after 22 are unknown.
pub struct ScheduleEntry<'a>(pub &'a [u8]);

impl<'a> ScheduleEntry<'a> {
    /// Size of one schedule entry.
    pub const BYTES: usize = 40;

    /// How many entries the schedule holds.
    pub fn count(schedule: &[u8]) -> usize {
        schedule.len() / Self::BYTES
    }
    /// Entry `slot` of the schedule.
    pub fn at(schedule: &'a [u8], slot: usize) -> Self {
        Self(&schedule[slot * Self::BYTES..(slot + 1) * Self::BYTES])
    }
    /// The day the entry activates; an entry with day zero ends the scan of the schedule.
    pub fn due_day(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// Days between one activation and the next.
    pub fn interval(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// Percentage compared against a 0..100 draw.
    pub fn chance(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// The story script the entry runs when it activates.
    pub fn script(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    /// The map the party must be on for the location test.
    pub fn map(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
    /// The region the party must be in for the location test; negative tests none.
    pub fn region(&self) -> Result<i16, ContentError> {
        word(self.0, 10)
    }
    /// Column the party must stand on for the location test; negative tests none.
    pub fn x(&self) -> Result<i16, ContentError> {
        word(self.0, 12)
    }
    /// Row the party must stand on for the location test.
    pub fn y(&self) -> Result<i16, ContentError> {
        word(self.0, 14)
    }
    /// An item some warrior must carry for the entry to activate; zero or less needs none.
    pub fn required_item(&self) -> Result<i16, ContentError> {
        word(self.0, 16)
    }
    /// A mission flag that must be set for the entry to activate; negative needs none.
    pub fn flag(&self) -> Result<i16, ContentError> {
        word(self.0, 18)
    }
    /// Positive values enable the location test; 1 requires a map, region and
    /// position, and other positive values have no known meaning.
    pub fn location_mode(&self) -> Result<i16, ContentError> {
        word(self.0, 20)
    }
}

/// One entry of the daily event schedule as a view that can change it.
pub struct ScheduleEntryMut<'a>(pub &'a mut [u8]);

impl<'a> ScheduleEntryMut<'a> {
    /// Entry `slot` of the schedule.
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
    /// The day the entry next comes due.
    pub fn due_day(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// Sets the day the entry next comes due.
    pub fn set_due_day(&mut self, day: i16) {
        put_word(self.0, 0, day);
    }
    /// Sets the days between one activation and the next.
    pub fn set_interval(&mut self, interval: i16) {
        put_word(self.0, 2, interval);
    }
    /// Sets the percent chance the entry activates when due.
    pub fn set_chance(&mut self, chance: i16) {
        put_word(self.0, 4, chance);
    }
}

/// An eight-byte rectangle at the start of the active region state, as the
/// schedule's location test reads it: top, left, bottom, right.
pub struct ScheduleRegion<'a>(pub &'a [u8]);

impl<'a> ScheduleRegion<'a> {
    /// Reads the rectangle of region `region`.
    pub fn read(regions: &'a [u8], region: usize) -> Result<Self, ContentError> {
        Ok(Self(span(regions, region * 8, 8)?))
    }
    /// The rectangle's top row.
    pub fn top(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// The rectangle's left column.
    pub fn left(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// The rectangle's bottom row, which is outside it.
    pub fn bottom(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// The rectangle's right column, which is outside it.
    pub fn right(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
}

/// Scheduled scripts (Data ED3): 42-byte event records indexed by the
/// magnitude of a schedule entry's script.
pub struct ScheduledScripts<'a>(pub &'a [u8]);

impl<'a> ScheduledScripts<'a> {
    /// The event record for the script `script`.
    pub fn record(&self, script: u16) -> Result<&'a [u8], ContentError> {
        span(
            self.0,
            usize::from(script) * StoryRecord::BYTES,
            StoryRecord::BYTES,
        )
    }
}
