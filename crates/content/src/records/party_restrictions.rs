//! The level's "Data RI" party restrictions, read by partyselect 0x147d3a into
//! the 0x140-byte buffer at 0x41f71a.
use crate::{ContentError, word};

pub struct PartyRestrictions<'a>(pub &'a [u8]);

impl PartyRestrictions<'_> {
    pub const BYTES: usize = 0x140;

    /// WORD 0x41f81a: the most warriors the squad may hold, or 0 for no limit.
    pub fn maximum_warriors(&self) -> Result<i16, ContentError> {
        word(self.0, 0x100)
    }
    /// WORD 0x41f81c: the highest admitted level, or 0 for no limit.
    pub fn maximum_level(&self) -> Result<i16, ContentError> {
        word(self.0, 0x102)
    }
    /// BYTE 0x41f81e + race - 1 (0x148c32). `None` when the index leaves the buffer.
    pub fn race_banned(&self, race: i16) -> Option<bool> {
        self.flag(0x104, race)
    }
    /// BYTE 0x41f83c + M.O.S. - 1 (0x148c6a). `None` when the index leaves the buffer.
    pub fn profession_banned(&self, profession: i16) -> Option<bool> {
        self.flag(0x122, profession)
    }
    fn flag(&self, base: usize, value: i16) -> Option<bool> {
        let index = usize::try_from(base as isize + isize::from(value) - 1).ok()?;
        (index < Self::BYTES).then(|| self.0.get(index).is_some_and(|byte| *byte != 0))
    }
}
