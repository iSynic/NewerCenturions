//! A warrior's learned scans (`cspells`) and the ten scan shortcuts
//! (`definespells`) used when choosing a scan.
use super::put_word;
use super::warrior_layout::{DEFINE_SPELLS, KNOWN_SCANS, NAME, PSI_MEMORY_END};
use crate::{ContentError, word};

/// A warrior's learned scans and scan shortcuts as a read-only view.
pub struct LearnedScans<'a>(pub &'a [u8]);
/// A warrior's learned scans and scan shortcuts as a view that can change them.
pub struct LearnedScansMut<'a>(pub &'a mut [u8]);

impl LearnedScans<'_> {
    /// Whether the warrior has learned the scan at `row` and `column`.
    pub fn selected(&self, row: usize, column: usize) -> bool {
        self.0[KNOWN_SCANS + row * 7 + column] != 0
    }
    /// The position of the last learned scan, counting row by row.
    pub fn last_selected(&self) -> Option<usize> {
        self.0[KNOWN_SCANS..NAME]
            .iter()
            .rposition(|value| *value != 0)
    }
    /// How many scans of the tier `row` the warrior has learned.
    pub fn tier_count(&self, row: usize) -> Result<i16, ContentError> {
        word(self.0, 0x42 + row * 2)
    }
    /// The ten scan shortcuts as stored.
    pub fn shortcuts(&self) -> &[u8] {
        &self.0[DEFINE_SPELLS..PSI_MEMORY_END]
    }
    /// The scan remembered in shortcut `index`, as its row and column.
    pub fn shortcut(&self, index: usize) -> Result<(i16, i16), ContentError> {
        let start = DEFINE_SPELLS + index * 8;
        Ok((word(self.0, start + 2)?, word(self.0, start + 4)?))
    }
}

impl LearnedScansMut<'_> {
    /// Puts back the shortcuts saved by `shortcuts`.
    pub fn restore_selection_memory(&mut self, memory: &[u8]) {
        self.0[DEFINE_SPELLS..PSI_MEMORY_END].copy_from_slice(memory);
    }

    /// Marks the scan at `row` and `column` learned or not.
    pub fn set_selected(&mut self, row: usize, column: usize, selected: bool) {
        self.0[KNOWN_SCANS + row * 7 + column] = u8::from(selected);
    }
    /// Sets how many scans of the tier `row` the warrior has learned.
    pub fn set_tier_count(&mut self, row: usize, count: i16) {
        put_word(self.0, 0x42 + row * 2, count);
    }
    /// Sets whether the warrior has learned a healing scan.
    pub fn set_healing_available(&mut self, available: bool) {
        self.0[0x9dc] = u8::from(available);
    }
    /// Forgets shortcut `index`.
    pub fn clear_shortcut(&mut self, index: usize) {
        self.0[DEFINE_SPELLS + index * 8..DEFINE_SPELLS + (index + 1) * 8].fill(0);
    }
}
