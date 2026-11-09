//! The level's "Data RI" party restrictions, checked when a squad is chosen.
use crate::{ContentError, span, word};

/// What the level allows in a squad.
pub struct PartyRestrictions<'a>(pub &'a [u8]);

impl PartyRestrictions<'_> {
    /// Size of the restrictions record.
    pub const BYTES: usize = 0x140;

    /// The scenario's own description of the restrictions.
    ///
    /// Original: viewrestrictions
    pub fn description(&self) -> Result<String, ContentError> {
        let text = span(self.0, 0, 0x100)?;
        Ok(encoding_rs::MACINTOSH
            .decode(span(text, 1, usize::from(text[0]))?)
            .0
            .into_owned())
    }
    /// The stored flag of race `index` (counting from zero): nonzero when banned.
    ///
    /// Original: viewrestrictions
    pub fn race_flag(&self, index: usize) -> Option<u8> {
        self.listed(0x104, index)
    }
    /// The stored flag of M.O.S. `index` (counting from zero): nonzero when banned.
    ///
    /// Original: viewrestrictions
    pub fn mos_flag(&self, index: usize) -> Option<u8> {
        self.listed(0x122, index)
    }
    /// The stored byte at `index` of the thirty-entry list starting at `base`.
    fn listed(&self, base: usize, index: usize) -> Option<u8> {
        (index < 30)
            .then(|| self.0.get(base + index).copied())
            .flatten()
    }
    /// The most warriors the squad may hold, or 0 for no limit.
    pub fn maximum_warriors(&self) -> Result<i16, ContentError> {
        word(self.0, 0x100)
    }
    /// The highest level admitted, or 0 for no limit.
    pub fn maximum_level(&self) -> Result<i16, ContentError> {
        word(self.0, 0x102)
    }
    /// Whether a one-based race is banned; `None` when it lies outside the record.
    pub fn race_banned(&self, race: i16) -> Option<bool> {
        self.flag(0x104, race)
    }
    /// Whether a one-based M.O.S. is banned; `None` when it lies outside the record.
    pub fn mos_banned(&self, mos: i16) -> Option<bool> {
        self.flag(0x122, mos)
    }
    /// Whether the byte for the one-based `value`, counted from `base`, is set; `None`
    /// past the record.
    fn flag(&self, base: usize, value: i16) -> Option<bool> {
        let index = usize::try_from(base as isize + isize::from(value) - 1).ok()?;
        (index < Self::BYTES).then(|| self.0.get(index).is_some_and(|byte| *byte != 0))
    }
}
