//! Warrior record fields shown on the character screens, and the fixed fields
//! a new warrior receives.
use super::warrior_layout::{NAME, REMAINING_EXPERIENCE, UPGRADE_POINTS};
use crate::bytes::{long, put_long};
use crate::{ContentError, span, word};

/// The warrior fields the character screens show.
pub struct CharacterInformation<'a>(pub &'a [u8]);

impl CharacterInformation<'_> {
    /// Attribute `index`: Brawn, Knowledge, Judgement, Quickness, Coordination, Vitality, Psionics.
    pub fn attribute(&self, index: usize) -> Result<i16, ContentError> {
        word(self.0, 0x9e2 + index * 2)
    }
    /// The warrior's age in days.
    pub fn age_days(&self) -> Result<i32, ContentError> {
        long(self.0, 0x9ca)
    }
    /// Experience still to earn for the next level.
    pub fn experience(&self) -> Result<i32, ContentError> {
        long(self.0, REMAINING_EXPERIENCE)
    }
    /// Experience the warrior has earned and can spend on upgrades.
    pub fn upgrade_points(&self) -> Result<i32, ContentError> {
        long(self.0, UPGRADE_POINTS)
    }
    /// Credits the warrior carries.
    pub fn carried_credits(&self) -> Result<u16, ContentError> {
        Ok(word(self.0, 0x9da)? as u16)
    }
    /// The warrior's gender.
    pub fn gender(&self) -> Result<i16, ContentError> {
        word(self.0, 0x38)
    }
    /// Most charge the warrior's power shield holds.
    pub fn shield_capacity(&self) -> Result<i16, ContentError> {
        word(self.0, 0x28)
    }
    /// Charge the power shield spends for each point of damage it stops.
    pub fn shield_cost(&self) -> Result<i16, ContentError> {
        word(self.0, 0x2a)
    }
    /// Movement the warrior gets each round.
    pub fn movement_rate(&self) -> Result<i16, ContentError> {
        word(self.0, 0x3e)
    }
    /// The warrior's bare-hand damage minimum, as the character screen shows it.
    pub fn displayed_damage_minimum(&self) -> Result<i16, ContentError> {
        word(self.0, 0xe)
    }
    /// Resistance to scans: the base word plus the bonus the warrior's attributes give.
    pub fn psionic_resistance(&self) -> Result<i16, ContentError> {
        Ok(word(self.0, 0x22)?.wrapping_add(word(self.0, 0xb60)?))
    }
    /// Saving throw against damage kind `index`: the base word plus the bonus the warrior's
    /// attributes give.
    pub fn damage_resistance(&self, index: usize) -> Result<i16, ContentError> {
        Ok(word(self.0, 0x112 + index * 2)?.wrapping_add(word(self.0, 0xb48 + index * 2)?))
    }
    /// The name as stored, without its terminator.
    pub fn name_bytes(&self) -> Result<&[u8], ContentError> {
        let tail = span(self.0, NAME, self.0.len().saturating_sub(NAME))?;
        let length =
            tail.iter().take(255).position(|b| *b == 0).ok_or_else(|| {
                ContentError::Invalid("unterminated character dialog text".into())
            })?;
        Ok(&tail[..length])
    }
}

/// The identity a new warrior receives. The caller encodes and validates the
/// name.
pub struct CharacterIdentityMut<'a>(pub &'a mut [u8]);
impl CharacterIdentityMut<'_> {
    /// Stamps the three wrapping checksums a saved warrior carries. The first
    /// sums a field whose meaning is unknown; none of them is a statistic.
    ///
    /// Original: savecharacter
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

    /// Gives a new warrior its race, profession, gender, portrait, map icon and name,
    /// at level 1.
    ///
    /// Original: NewCharacter
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
        self.0[NAME..NAME + name.len()].copy_from_slice(name);
    }
    /// Sets the warrior's age in days.
    pub fn set_age_days(&mut self, days: i32) {
        put_long(self.0, 0x9ca, days);
    }
    /// A new warrior's creation marker is one; its wider meaning is unknown.
    ///
    /// Original: NewCharacter
    pub fn set_creation_marker(&mut self) {
        super::put_word(self.0, 0, 1);
    }
}
