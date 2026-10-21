//! Bit tables consumed by the original equipment admission caller. Bit zero
//! is the high bit, and the sentinel ability 58 still participates in lookup.
use crate::{ContentError, span, word};

pub struct EquipmentEligibility<'a>(pub &'a [u8]);
pub struct RaceEquipment<'a>(pub &'a [u8]);
pub struct ProfessionEquipment<'a>(pub &'a [u8]);

fn bit(bytes: &[u8], offset: usize, index: usize) -> Result<bool, ContentError> {
    Ok(span(bytes, offset + index / 8, 1)?[0] & (0x80 >> (index % 8)) != 0)
}

impl EquipmentEligibility<'_> {
    pub fn required_race(&self) -> Result<i16, ContentError> {
        word(self.0, 108)
    }
    pub fn required_profession(&self) -> Result<i16, ContentError> {
        word(self.0, 110)
    }
    pub fn ability(&self, index: usize) -> Result<bool, ContentError> {
        bit(self.0, 128, index)
    }
    pub fn forbids_trait(&self, index: usize) -> Result<bool, ContentError> {
        bit(self.0, 104, index)
    }
    pub fn requires_trait(&self, index: usize) -> Result<bool, ContentError> {
        bit(self.0, 112, index)
    }
    pub fn forbids_school(&self, index: usize) -> Result<bool, ContentError> {
        bit(self.0, 106, index)
    }
    pub fn has_required_schools(&self) -> Result<bool, ContentError> {
        Ok(word(self.0, 114)? != 0)
    }
    pub fn permits_required_school(&self, index: usize) -> Result<bool, ContentError> {
        bit(self.0, 114, index)
    }
}

impl RaceEquipment<'_> {
    pub fn permits_ability(&self, index: usize) -> Result<bool, ContentError> {
        bit(self.0, 336, index)
    }
    pub fn has_trait(&self, index: usize) -> Result<bool, ContentError> {
        bit(self.0, 344, index)
    }
}

impl ProfessionEquipment<'_> {
    pub fn permits_ability(&self, index: usize) -> Result<bool, ContentError> {
        bit(self.0, 442, index)
    }
    pub fn psi_school(&self) -> Result<i16, ContentError> {
        word(self.0, 254)
    }
}
