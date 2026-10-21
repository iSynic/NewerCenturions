use super::put_word;

/// Retained 80-byte per-warrior Psi selection work. Stored bytes and untouched
/// selection fields remain owned by the session; this view is never serialized.
pub struct PsiSelectionMemoryMut<'a>(pub &'a mut [u8]);

impl PsiSelectionMemoryMut<'_> {
    pub fn remember_portrait_heal(&mut self, actor: usize, power: i16) {
        self.remember_cast(actor, [0, 0, 0], power);
    }
    pub fn remember_cast(&mut self, actor: usize, identity: [i16; 3], power: i16) {
        for (index, value) in identity.into_iter().chain([power]).enumerate() {
            put_word(self.0, actor * 80 + 72 + index * 2, value);
        }
    }
}
