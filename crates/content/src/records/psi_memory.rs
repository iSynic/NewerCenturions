use super::put_word;
use crate::{ContentError, word};

/// A warrior's 80-byte block of the party's Psi selection memory
/// (`definespells`). The session owns the bytes; the view only changes the
/// remembered selections it names.
pub struct PsiSelectionMemoryMut<'a>(pub &'a mut [u8]);

impl PsiSelectionMemoryMut<'_> {
    /// Remembers a heal cast from the warrior's portrait at `power`.
    pub fn remember_portrait_heal(&mut self, actor: usize, power: i16) {
        self.remember_cast(actor, [0, 0, 0], power);
    }
    /// Remembers that warrior `actor` cast the scan `identity` (caste, level and
    /// number) at `power`, so the scan menu offers it again.
    pub fn remember_cast(&mut self, actor: usize, identity: [i16; 3], power: i16) {
        for (index, value) in identity.into_iter().chain([power]).enumerate() {
            put_word(self.0, actor * 80 + 72 + index * 2, value);
        }
    }
}

/// A read-only view of the party's Psi selection memory (`definespells`).
pub struct PsiSelectionMemory<'a>(pub &'a [u8]);

impl PsiSelectionMemory<'_> {
    /// The scan kept for warrior `actor` in preset `slot` (1 to 10): its caste, level, number and
    /// power. Slot 10 is the scan the warrior cast last.
    ///
    /// Original: checkkeypad
    pub fn preset(&self, actor: usize, slot: usize) -> Result<[i16; 4], ContentError> {
        let base = actor * 80 + (slot - 1) * 8;
        Ok([
            word(self.0, base)?,
            word(self.0, base + 2)?,
            word(self.0, base + 4)?,
            word(self.0, base + 6)?,
        ])
    }
}
