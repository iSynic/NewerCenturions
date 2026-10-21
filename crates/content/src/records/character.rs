//! Display fields read by ShowStats (0x159a96) and viewcharacter (0x164174).
//! This is a borrowed record codec, not a replacement save representation.
use crate::{ContentError, mission::long, span, word};

pub struct CharacterInformation<'a>(pub &'a [u8]);

impl CharacterInformation<'_> {
    pub fn attribute(&self, index: usize) -> Result<i16, ContentError> {
        word(self.0, 0x9e2 + index * 2)
    }
    pub fn age_days(&self) -> Result<i32, ContentError> {
        long(self.0, 0x9ca)
    }
    pub fn experience(&self) -> Result<i32, ContentError> {
        long(self.0, 0x9ce)
    }
    pub fn upgrade_points(&self) -> Result<i32, ContentError> {
        long(self.0, 0x9d2)
    }
    pub fn carried_credits(&self) -> Result<u16, ContentError> {
        Ok(word(self.0, 0x9da)? as u16)
    }
    pub fn gender(&self) -> Result<i16, ContentError> {
        word(self.0, 0x38)
    }
    pub fn shield_capacity(&self) -> Result<i16, ContentError> {
        word(self.0, 0x28)
    }
    pub fn shield_cost(&self) -> Result<i16, ContentError> {
        word(self.0, 0x2a)
    }
    pub fn movement_rate(&self) -> Result<i16, ContentError> {
        word(self.0, 0x3e)
    }
    /// The displayed lower bound is independent of attack's constant minimum.
    pub fn displayed_damage_minimum(&self) -> Result<i16, ContentError> {
        word(self.0, 0xe)
    }
    pub fn psionic_resistance(&self) -> Result<i16, ContentError> {
        Ok(word(self.0, 0x22)?.wrapping_add(word(self.0, 0xb60)?))
    }
    pub fn damage_resistance(&self, index: usize) -> Result<i16, ContentError> {
        Ok(word(self.0, 0x112 + index * 2)?.wrapping_add(word(self.0, 0xb48 + index * 2)?))
    }
    pub fn name_bytes(&self) -> Result<&[u8], ContentError> {
        let tail = span(self.0, 0xa21, self.0.len().saturating_sub(0xa21))?;
        let length =
            tail.iter().take(255).position(|b| *b == 0).ok_or_else(|| {
                ContentError::Invalid("unterminated character dialog text".into())
            })?;
        Ok(&tail[..length])
    }
}

/// NewCharacter's fixed metadata writes. Name encoding and eligibility belong
/// to the caller; this codec only installs already validated bytes.
pub struct CharacterIdentityMut<'a>(pub &'a mut [u8]);
impl CharacterIdentityMut<'_> {
    /// savecharacter 0x1374bc stamps three wrapping WORD checksums. The first
    /// includes an unidentified saved WORD; these are format receipts, not stats.
    pub fn refresh_save_checksums(&mut self) -> Result<(), ContentError> {
        for (destination, fields) in [
            (2, &[0x3a, 0x52, 0x1e][..]),
            (0x124, &[6, 0x5a, 0x9e4, 0x9e6][..]),
            (0xa40, &[0xa, 0x52, 0x9e8, 0x52, 0x22][..]),
        ] {
            let value = fields.iter().try_fold(0_i16, |sum, offset| {
                word(self.0, *offset).map(|n| sum.wrapping_add(n))
            })?;
            super::put_word(self.0, destination, value);
        }
        Ok(())
    }

    pub fn initialize(
        &mut self,
        race: i16,
        profession: i16,
        gender: i16,
        portrait: i16,
        icon: i16,
        name: &[u8],
    ) {
        for (offset, value) in [
            (0x32, race),
            (0x34, profession),
            (0x38, gender),
            (0x3a, 1),
            (0x54, portrait),
            (0x56, icon),
        ] {
            super::put_word(self.0, offset, value);
        }
        self.0[0xa21..0xa21 + name.len()].copy_from_slice(name);
    }
    pub fn set_age_days(&mut self, days: i32) {
        self.0[0x9ca..0x9ce].copy_from_slice(&days.to_be_bytes());
    }
    /// NewCharacter sets this WORD to one. Its wider meaning is unconfirmed.
    pub fn set_creation_marker(&mut self) {
        super::put_word(self.0, 0, 1);
    }
}
