//! Profile lookups and scan fields used on promotion and when choosing scans.
use super::warrior_layout::KNOWN_SCANS;
use super::{
    LearnedScans, ProfessionProfile, ProfileTable, PsiSelectionMemoryMut, RaceProfile,
    SpellInfoRecord,
};
use crate::{ContentError, span};

impl<'a> RaceProfile<'a> {
    /// One-based 408-byte race profile; callers reject indexes below one.
    pub fn from_table(races: &'a [u8], race: i16) -> Result<Self, ContentError> {
        Ok(Self(ProfileTable::races(races).row(race as usize - 1)?))
    }
}

impl<'a> ProfessionProfile<'a> {
    /// One-based 576-byte M.O.S. profile; callers reject indexes below one.
    pub fn from_table(profiles: &'a [u8], profession: i16) -> Result<Self, ContentError> {
        Ok(Self(
            ProfileTable::professions(profiles).row(profession as usize - 1)?,
        ))
    }
}

impl<'a> SpellInfoRecord<'a> {
    /// 58-byte scan packet: each zero-based school holds a 7x7 tier/column matrix.
    pub fn scan_entry(attacks: &'a [u8], school: usize, scan: usize) -> Result<Self, ContentError> {
        Ok(Self(span(attacks, (school * 49 + scan) * 58, 58)?))
    }
}

impl LearnedScans<'_> {
    /// The learned-scan byte as stored; a well-formed draft holds only 0 or 1.
    pub fn scan_marker(&self, row: usize, column: usize) -> Option<u8> {
        self.0.get(KNOWN_SCANS + row * 7 + column).copied()
    }
}

impl PsiSelectionMemoryMut<'_> {
    /// Replaces one warrior's 80-byte block with its saved scan shortcuts.
    pub fn store_shortcuts(&mut self, member: usize, shortcuts: &[u8]) {
        self.0[member * 80..(member + 1) * 80].copy_from_slice(shortcuts);
    }
}
