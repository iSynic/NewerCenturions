//! Table selection and scan fields used by levelup and scanselect callers.
use super::{AttackPacket, LearnedScans, ProfessionProfile, PsiSelectionMemoryMut, RaceProfile};
use crate::{ContentError, span};

impl<'a> RaceProfile<'a> {
    /// One-based 408-byte race profile; callers reject indexes below one.
    pub fn from_table(races: &'a [u8], race: i16) -> Result<Self, ContentError> {
        Ok(Self(span(races, (race as usize - 1) * 408, 408)?))
    }
}

impl<'a> ProfessionProfile<'a> {
    /// One-based 576-byte MOS profile; callers reject indexes below one.
    pub fn from_table(profiles: &'a [u8], profession: i16) -> Result<Self, ContentError> {
        Ok(Self(span(profiles, (profession as usize - 1) * 576, 576)?))
    }
}

impl<'a> AttackPacket<'a> {
    /// 58-byte scan packet: each zero-based school holds a 7x7 tier/column matrix.
    pub fn scan_entry(attacks: &'a [u8], school: usize, scan: usize) -> Result<Self, ContentError> {
        Ok(Self(span(attacks, (school * 49 + scan) * 58, 58)?))
    }
}

impl LearnedScans<'_> {
    /// Raw learned-scan byte; a well-formed draft stores only 0 or 1.
    pub fn scan_marker(&self, row: usize, column: usize) -> Option<u8> {
        self.0.get(0x9f0 + row * 7 + column).copied()
    }
}

impl PsiSelectionMemoryMut<'_> {
    /// Replaces one warrior's 80-byte block with its saved scan shortcuts.
    pub fn store_shortcuts(&mut self, member: usize, shortcuts: &[u8]) {
        self.0[member * 80..(member + 1) * 80].copy_from_slice(shortcuts);
    }
}
