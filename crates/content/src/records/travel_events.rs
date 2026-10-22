use crate::{ContentError, word};

/// Ten-byte EXTRA record for a story tile change.
pub struct TileMutationExtra<'a>(pub &'a [u8]);

impl TileMutationExtra<'_> {
    pub const BYTES: usize = 10;

    pub fn map(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn x(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    pub fn y(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    pub fn tile(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    /// 2 selects the object layer; every other value the floor.
    pub fn layer(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}

/// Ten-byte EXTRA record replacing one map event's script.
pub struct EventReplacementExtra<'a>(pub &'a [u8]);

impl EventReplacementExtra<'_> {
    pub const BYTES: usize = 10;

    pub fn map(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn slot(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    pub fn script(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
}

/// Ten-byte EXTRA record setting map event chances: a single slot and an
/// inclusive slot range, where zero leaves either unused.
pub struct EventChanceExtra<'a>(pub &'a [u8]);

impl EventChanceExtra<'_> {
    pub const BYTES: usize = 10;

    pub fn slot(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    pub fn chance(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    pub fn first_slot(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    pub fn last_slot(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}
