//! Saved scan matrix and ten shortcuts used by scanselect (0x15a9cc).
use super::put_word;
use crate::{ContentError, word};

pub struct LearnedScans<'a>(pub &'a [u8]);
pub struct LearnedScansMut<'a>(pub &'a mut [u8]);

impl LearnedScans<'_> {
    pub fn selected(&self, row: usize, column: usize) -> bool {
        self.0[0x9f0 + row * 7 + column] != 0
    }
    pub fn last_selected(&self) -> Option<usize> {
        self.0[0x9f0..0xa21].iter().rposition(|value| *value != 0)
    }
    pub fn tier_count(&self, row: usize) -> Result<i16, ContentError> {
        word(self.0, 0x42 + row * 2)
    }
    pub fn shortcuts(&self) -> &[u8] {
        &self.0[0xa76..0xac6]
    }
    pub fn shortcut(&self, index: usize) -> Result<(i16, i16), ContentError> {
        let start = 0xa76 + index * 8;
        Ok((word(self.0, start + 2)?, word(self.0, start + 4)?))
    }
}

impl LearnedScansMut<'_> {
    pub fn restore_selection_memory(&mut self, memory: &[u8]) {
        self.0[0xa76..0xac6].copy_from_slice(memory);
    }

    pub fn set_selected(&mut self, row: usize, column: usize, selected: bool) {
        self.0[0x9f0 + row * 7 + column] = u8::from(selected);
    }
    pub fn set_tier_count(&mut self, row: usize, count: i16) {
        put_word(self.0, 0x42 + row * 2, count);
    }
    pub fn set_healing_available(&mut self, available: bool) {
        self.0[0x9dc] = u8::from(available);
    }
    pub fn clear_shortcut(&mut self, index: usize) {
        self.0[0xa76 + index * 8..0xa76 + (index + 1) * 8].fill(0);
    }
}
