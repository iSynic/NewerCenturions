//! The race, MOS and item bit tables that decide who may use an item.
//! Bit zero is the high bit, and the end marker, ability 58, is looked up like
//! any other.
use crate::{ContentError, span, word};

/// The restriction tables of an item definition.
pub struct EquipmentEligibility<'a>(pub &'a [u8]);
/// The equipment tables of a race record.
pub struct RaceEquipment<'a>(pub &'a [u8]);
/// The equipment tables of an MOS record.
pub struct MosEquipment<'a>(pub &'a [u8]);

/// Whether bit `index` of the table at `offset` is set, counting from the high bit.
fn bit(bytes: &[u8], offset: usize, index: usize) -> Result<bool, ContentError> {
    Ok(span(bytes, offset + index / 8, 1)?[0] & (0x80 >> (index % 8)) != 0)
}

impl EquipmentEligibility<'_> {
    /// The race that alone may use the item; zero allows every race.
    pub fn required_race(&self) -> Result<i16, ContentError> {
        word(self.0, 108)
    }
    /// The MOS that alone may use the item; zero allows every MOS.
    pub fn required_mos(&self) -> Result<i16, ContentError> {
        word(self.0, 110)
    }
    /// Whether the item's ability table sets `index`. The item's ability is the first one set;
    /// the race and the MOS must both permit it.
    pub fn ability(&self, index: usize) -> Result<bool, ContentError> {
        bit(self.0, 128, index)
    }
    /// Whether the item cannot be used by a race with trait `index`.
    pub fn forbids_trait(&self, index: usize) -> Result<bool, ContentError> {
        bit(self.0, 104, index)
    }
    /// Whether the item can be used only by a race with trait `index`.
    pub fn requires_trait(&self, index: usize) -> Result<bool, ContentError> {
        bit(self.0, 112, index)
    }
    /// Whether the item cannot be used by a warrior of Psi school `index`.
    pub fn forbids_school(&self, index: usize) -> Result<bool, ContentError> {
        bit(self.0, 106, index)
    }
    /// Whether the item names Psi schools that alone may use it.
    pub fn has_required_schools(&self) -> Result<bool, ContentError> {
        Ok(word(self.0, 114)? != 0)
    }
    /// Whether the item may be used by a warrior of Psi school `index`.
    pub fn permits_required_school(&self, index: usize) -> Result<bool, ContentError> {
        bit(self.0, 114, index)
    }
    /// The index is signed, as BitTst takes it: -1 reads bit 7 of byte 113.
    pub fn permits_required_school_bit(&self, index: i16) -> Result<bool, ContentError> {
        let byte = 114 + isize::from(index >> 3);
        let byte = usize::try_from(byte)
            .map_err(|_| ContentError::Invalid("required school bit before record".into()))?;
        Ok(span(self.0, byte, 1)?[0] & (0x80 >> (index & 7)) != 0)
    }
}

impl RaceEquipment<'_> {
    /// Whether the race permits items of ability `index`.
    pub fn permits_ability(&self, index: usize) -> Result<bool, ContentError> {
        bit(self.0, 336, index)
    }
    /// Whether the race has trait `index`.
    pub fn has_trait(&self, index: usize) -> Result<bool, ContentError> {
        bit(self.0, 344, index)
    }
}

impl MosEquipment<'_> {
    /// Whether the MOS permits items of ability `index`.
    pub fn permits_ability(&self, index: usize) -> Result<bool, ContentError> {
        bit(self.0, 442, index)
    }
    /// The Psi school the MOS belongs to.
    pub fn psi_school(&self) -> Result<i16, ContentError> {
        word(self.0, 254)
    }
}
