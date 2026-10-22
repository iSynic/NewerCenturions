//! Destination fields of the 42-byte event record used by script returns.
use super::{StoryRecord, StoryRecordMut};

impl StoryRecord<'_> {
    pub const BYTES: usize = 42;

    pub fn destination_map(&self) -> i8 {
        self.0[4] as i8
    }
    pub fn destination_x(&self) -> i8 {
        self.0[5] as i8
    }
    pub fn destination_y(&self) -> i8 {
        self.0[6] as i8
    }
    /// The original compares this byte as signed against an unsigned current level.
    pub fn destination_level(&self) -> u8 {
        self.0[40]
    }
}

impl StoryRecordMut<'_> {
    pub fn set_destination_position(&mut self, x: i32, y: i32) {
        self.0[5] = x as u8;
        self.0[6] = y as u8;
    }
}
