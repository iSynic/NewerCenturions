//! Race (408 bytes) and MOS (576 bytes) fields used by NewCharacter,
//! levelup and upgradewarrior. These borrowed views preserve native records.
use super::Attribute;
use crate::{ContentError, span, word};

pub struct RaceProfile<'a>(pub &'a [u8]);
pub struct ProfessionProfile<'a>(pub &'a [u8]);

impl RaceProfile<'_> {
    pub fn accepts_profession(&self, profession: i16) -> Result<bool, ContentError> {
        Ok(span(self.0, 208 + profession as usize - 1, 1)?[0] != 0)
    }
    pub fn creature_bonus(&self, category: usize) -> Result<i16, ContentError> {
        word(self.0, category * 2)
    }
    pub fn special_skill(&self, skill: usize) -> Result<i16, ContentError> {
        word(self.0, 16 + skill * 2)
    }
    pub fn resistance(&self, category: usize) -> Result<i16, ContentError> {
        word(self.0, 44 + category * 2)
    }
    pub fn attribute_bonus(&self, attribute: Attribute) -> Result<i16, ContentError> {
        word(self.0, 60 + creation_attribute_row(attribute) * 2)
    }
    pub fn attribute_bounds(&self, attribute: Attribute) -> Result<(i16, i16), ContentError> {
        let start = 74 + creation_attribute_row(attribute) * 4;
        Ok((word(self.0, start)?, word(self.0, start + 2)?))
    }
    pub fn condition(&self, condition: usize) -> Result<i16, ContentError> {
        word(self.0, 112 + condition * 2)
    }
    pub fn movement_adjustment(&self) -> i8 {
        self.0[197] as i8
    }
    pub fn psi_resistance(&self) -> Result<i16, ContentError> {
        word(self.0, 198)
    }
    pub fn melee_accuracy(&self) -> Result<i16, ContentError> {
        word(self.0, 200)
    }
    pub fn missile_accuracy(&self) -> Result<i16, ContentError> {
        word(self.0, 202)
    }
    pub fn action_points(&self) -> Result<i16, ContentError> {
        word(self.0, 204)
    }
    pub fn action_limit(&self) -> Result<i16, ContentError> {
        word(self.0, 206)
    }
    pub fn age_bounds(&self, category: usize) -> Result<(i16, i16), ContentError> {
        Ok((
            word(self.0, 238 + category * 4)?,
            word(self.0, 240 + category * 4)?,
        ))
    }
}

// NewCharacter deliberately uses Vitality's profile row for Psionics too.
fn creation_attribute_row(attribute: Attribute) -> usize {
    (attribute as usize).min(5)
}

impl ProfessionProfile<'_> {
    pub fn special_skill(&self, skill: usize) -> Result<i16, ContentError> {
        word(self.0, skill * 2)
    }
    pub fn resistance(&self, category: usize) -> Result<i16, ContentError> {
        word(self.0, 56 + category * 2)
    }
    pub fn attribute_bonus(&self, attribute: Attribute) -> Result<i16, ContentError> {
        word(self.0, 72 + creation_attribute_row(attribute) * 2)
    }
    pub fn attribute_bounds(&self, attribute: Attribute) -> Result<(i16, i16), ContentError> {
        let start = 110 + creation_attribute_row(attribute) * 4;
        Ok((word(self.0, start)?, word(self.0, start + 2)?))
    }
    pub fn psi_school_enabled(&self, school: usize) -> Result<bool, ContentError> {
        Ok(word(self.0, 88 + school * 6)? != 0)
    }
    pub fn highest_scan_tier(&self) -> Result<i16, ContentError> {
        Ok(word(self.0, 90)?
            .wrapping_add(word(self.0, 96)?)
            .wrapping_add(word(self.0, 102)?))
    }
    pub fn condition_level(&self, condition: usize) -> Result<i16, ContentError> {
        word(self.0, 138 + condition * 2)
    }
    pub fn initial_stamina_bound(&self) -> Result<i16, ContentError> {
        word(self.0, 222)
    }
    pub fn melee_accuracy(&self) -> Result<i16, ContentError> {
        word(self.0, 234)
    }
    pub fn missile_accuracy(&self) -> Result<i16, ContentError> {
        word(self.0, 238)
    }
    pub fn bare_hand_maximum(&self) -> Result<i16, ContentError> {
        word(self.0, 242)
    }
    pub fn missile_dodge(&self) -> Result<i16, ContentError> {
        word(self.0, 246)
    }
    pub fn melee_dodge(&self) -> Result<i16, ContentError> {
        word(self.0, 250)
    }
    pub fn age_category(&self) -> Result<i16, ContentError> {
        word(self.0, 256)
    }
    pub fn movement_bonus(&self) -> Result<i16, ContentError> {
        word(self.0, 258)
    }
    pub fn psi_resistance(&self) -> Result<i16, ContentError> {
        word(self.0, 260)
    }
    pub fn action_points(&self) -> Result<i16, ContentError> {
        word(self.0, 266)
    }
    pub fn attack_limit(&self) -> Result<i16, ContentError> {
        word(self.0, 268)
    }
    pub fn starting_credits(&self) -> Result<i16, ContentError> {
        word(self.0, 390)
    }
    pub fn starting_item(&self, slot: usize) -> Result<i16, ContentError> {
        word(self.0, 392 + slot * 2)
    }
    pub fn action_unlock_levels(&self) -> &[u8] {
        &self.0[432..442]
    }
    pub fn scan_capacity(&self) -> Result<i16, ContentError> {
        word(self.0, 452)
    }
    pub fn upgrade_percent(&self, selector: u16) -> Result<i16, ContentError> {
        let offset = match selector {
            7 => 0xe4,
            16 => 0xf0,
            17 => 0xec,
            18 => 0xf8,
            19 => 0xfc,
            20 => 0xe0,
            21 => 0x108,
            22 => 0x106,
            23..=31 => 0x1c + usize::from(selector - 23) * 2,
            32 => 0xf4,
            _ => return Ok(10),
        };
        word(self.0, offset)
    }
}
