use crate::{ContentError, word};

/// Ten-byte EXTRA record for a story tile change.
pub struct TileMutationExtra<'a>(pub &'a [u8]);

impl TileMutationExtra<'_> {
    /// Size of the record.
    pub const BYTES: usize = 10;

    /// The map the tile is on.
    pub fn map(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// The column of the tile.
    pub fn x(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// The row of the tile.
    pub fn y(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// The tile put in its place.
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
    /// Size of the record.
    pub const BYTES: usize = 10;

    /// The map whose event is replaced; -1 and -2 name the menu and typed encounters
    /// instead.
    pub fn map(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// The event or encounter whose script is replaced.
    pub fn slot(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// The script whose actions replace the event's.
    pub fn script(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// Which of the encounter's branches is replaced.
    pub fn branch(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}

/// Ten-byte EXTRA record setting map event chances: a single slot and an
/// inclusive slot range, where zero leaves either unused.
pub struct EventChanceExtra<'a>(pub &'a [u8]);

impl EventChanceExtra<'_> {
    /// Size of the record.
    pub const BYTES: usize = 10;

    /// The one event to set.
    pub fn slot(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// The new chance.
    pub fn chance(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// First event of the range to set.
    pub fn first_slot(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    /// Last event of the range to set.
    pub fn last_slot(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}
