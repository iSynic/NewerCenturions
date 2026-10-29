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
    /// BitTst with a signed index: -1 reads bit 7 of the preceding byte 113.
    pub fn permits_required_school_bit(&self, index: i16) -> Result<bool, ContentError> {
        let byte = 114 + isize::from(index >> 3);
        let byte = usize::try_from(byte)
            .map_err(|_| ContentError::Invalid("required school bit before record".into()))?;
        Ok(span(self.0, byte, 1)?[0] & (0x80 >> (index & 7)) != 0)
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
