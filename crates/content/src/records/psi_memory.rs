use super::put_word;

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
