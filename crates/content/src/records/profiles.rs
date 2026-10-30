//! Race (408-byte) and M.O.S. (576-byte) profiles that shape a warrior at
//! creation, on promotion and when upgrading.
use super::Attribute;
use crate::{ContentError, span, word};

/// Bytes in one Data race record.
pub const RACE_PROFILE_BYTES: usize = 408;
/// Bytes in one Data MOS record.
pub const PROFESSION_PROFILE_BYTES: usize = 576;

/// A Data race or MOS table. Ids are one-based; each caller checks the id
/// itself and passes the zero-based row.
#[derive(Clone, Copy)]
pub struct ProfileTable<'a> {
    bytes: &'a [u8],
    stride: usize,
}

impl<'a> ProfileTable<'a> {
    /// The table of race records.
    pub fn races(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            stride: RACE_PROFILE_BYTES,
        }
    }
    /// The table of profession records.
    pub fn professions(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            stride: PROFESSION_PROFILE_BYTES,
        }
    }
    /// The zero-based record; a row past the table is an error.
    pub fn row(&self, row: usize) -> Result<&'a [u8], ContentError> {
        span(self.bytes, row * self.stride, self.stride)
    }
}

/// A race's record: what a warrior of the race starts with and may become.
pub struct RaceProfile<'a>(pub &'a [u8]);
/// A profession's record, called M.O.S. in the files: what a warrior of the
/// profession starts with and gains.
pub struct ProfessionProfile<'a>(pub &'a [u8]);

impl RaceProfile<'_> {
    /// Whether the race may take the one-based `profession`.
    pub fn accepts_profession(&self, profession: i16) -> Result<bool, ContentError> {
        Ok(span(self.0, 208 + profession as usize - 1, 1)?[0] != 0)
    }
    /// The race's bonus against creature category `category`.
    pub fn creature_bonus(&self, category: usize) -> Result<i16, ContentError> {
        word(self.0, category * 2)
    }
    /// The race's rating in special skill `skill`.
    pub fn special_skill(&self, skill: usize) -> Result<i16, ContentError> {
        word(self.0, 16 + skill * 2)
    }
    /// The race's saving throw in category `category`.
    pub fn resistance(&self, category: usize) -> Result<i16, ContentError> {
        word(self.0, 44 + category * 2)
    }
    /// What the race adds to `attribute` at creation.
    pub fn attribute_bonus(&self, attribute: Attribute) -> Result<i16, ContentError> {
        word(self.0, 60 + creation_attribute_row(attribute) * 2)
    }
    /// The lowest and highest `attribute` a new warrior of the race can have.
    pub fn attribute_bounds(&self, attribute: Attribute) -> Result<(i16, i16), ContentError> {
        let start = 74 + creation_attribute_row(attribute) * 4;
        Ok((word(self.0, start)?, word(self.0, start + 2)?))
    }
    /// The duration of condition `condition` that a new warrior of the race starts with.
    pub fn condition(&self, condition: usize) -> Result<i16, ContentError> {
        word(self.0, 112 + condition * 2)
    }
    /// The race's movement rating, which scales the movement a warrior's load leaves.
    pub fn movement_adjustment(&self) -> i8 {
        self.0[197] as i8
    }
    /// The race's resistance to scans.
    pub fn psi_resistance(&self) -> Result<i16, ContentError> {
        word(self.0, 198)
    }
    /// The race's melee accuracy.
    pub fn melee_accuracy(&self) -> Result<i16, ContentError> {
        word(self.0, 200)
    }
    /// The race's missile accuracy.
    pub fn missile_accuracy(&self) -> Result<i16, ContentError> {
        word(self.0, 202)
    }
    /// Actions per round the race gives.
    pub fn action_points(&self) -> Result<i16, ContentError> {
        word(self.0, 204)
    }
    /// Half the most actions per round a warrior of the race can have.
    pub fn action_limit(&self) -> Result<i16, ContentError> {
        word(self.0, 206)
    }
    /// The youngest and oldest age of the race in age category `category`, in years.
    pub fn age_bounds(&self, category: usize) -> Result<(i16, i16), ContentError> {
        Ok((
            word(self.0, 238 + category * 4)?,
            word(self.0, 240 + category * 4)?,
        ))
    }
}

// A new warrior's Psionics deliberately uses Vitality's profile row.
/// The profile row that sets `attribute` at creation; Psionics uses Vitality's.
fn creation_attribute_row(attribute: Attribute) -> usize {
    (attribute as usize).min(5)
}

impl ProfessionProfile<'_> {
    /// The profession's rating in special skill `skill`.
    pub fn special_skill(&self, skill: usize) -> Result<i16, ContentError> {
        word(self.0, skill * 2)
    }
    /// The profession's saving throw in category `category`.
    pub fn resistance(&self, category: usize) -> Result<i16, ContentError> {
        word(self.0, 56 + category * 2)
    }
    /// What the profession adds to `attribute` at creation.
    pub fn attribute_bonus(&self, attribute: Attribute) -> Result<i16, ContentError> {
        word(self.0, 72 + creation_attribute_row(attribute) * 2)
    }
    /// The lowest and highest `attribute` a new warrior of the profession can have.
    pub fn attribute_bounds(&self, attribute: Attribute) -> Result<(i16, i16), ContentError> {
        let start = 110 + creation_attribute_row(attribute) * 4;
        Ok((word(self.0, start)?, word(self.0, start + 2)?))
    }
    /// Whether the profession can follow Psi school `school`.
    pub fn psi_school_enabled(&self, school: usize) -> Result<bool, ContentError> {
        Ok(word(self.0, 88 + school * 6)? != 0)
    }
    /// The highest tier of scans the profession can learn.
    pub fn highest_scan_tier(&self) -> Result<i16, ContentError> {
        Ok(word(self.0, 90)?
            .wrapping_add(word(self.0, 96)?)
            .wrapping_add(word(self.0, 102)?))
    }
    /// The level at which the profession gains condition `condition` for good; 1 is at creation.
    pub fn condition_level(&self, condition: usize) -> Result<i16, ContentError> {
        word(self.0, 138 + condition * 2)
    }
    /// The bound of the stamina a new warrior of the profession is dealt.
    pub fn initial_stamina_bound(&self) -> Result<i16, ContentError> {
        word(self.0, 222)
    }
    /// The profession's melee accuracy.
    pub fn melee_accuracy(&self) -> Result<i16, ContentError> {
        word(self.0, 234)
    }
    /// The profession's missile accuracy.
    pub fn missile_accuracy(&self) -> Result<i16, ContentError> {
        word(self.0, 238)
    }
    /// The profession's largest bare-hand damage.
    pub fn bare_hand_maximum(&self) -> Result<i16, ContentError> {
        word(self.0, 242)
    }
    /// The profession's missile dodge.
    pub fn missile_dodge(&self) -> Result<i16, ContentError> {
        word(self.0, 246)
    }
    /// The profession's melee dodge.
    pub fn melee_dodge(&self) -> Result<i16, ContentError> {
        word(self.0, 250)
    }
    /// The race age category the profession's warriors are drawn from, counted from one.
    pub fn age_category(&self) -> Result<i16, ContentError> {
        word(self.0, 256)
    }
    /// Movement the profession adds.
    pub fn movement_bonus(&self) -> Result<i16, ContentError> {
        word(self.0, 258)
    }
    /// The profession's resistance to scans.
    pub fn psi_resistance(&self) -> Result<i16, ContentError> {
        word(self.0, 260)
    }
    /// Actions per round the profession gives.
    pub fn action_points(&self) -> Result<i16, ContentError> {
        word(self.0, 266)
    }
    /// Attacks per combat the profession allows.
    pub fn attack_limit(&self) -> Result<i16, ContentError> {
        word(self.0, 268)
    }
    /// Credits a new warrior of the profession starts with.
    pub fn starting_credits(&self) -> Result<i16, ContentError> {
        word(self.0, 390)
    }
    /// Item id in starting slot `slot`.
    pub fn starting_item(&self, slot: usize) -> Result<i16, ContentError> {
        word(self.0, 392 + slot * 2)
    }
    /// The levels at which the profession gains an extra action each round; zero is unused.
    pub fn action_unlock_levels(&self) -> &[u8] {
        &self.0[432..442]
    }
    /// Scans the profession may cast in a round.
    pub fn scan_capacity(&self) -> Result<i16, ContentError> {
        word(self.0, 452)
    }
    /// The percent added to the price of upgrade `selector` for each purchase; 10 for
    /// an upgrade the profession has no entry for.
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
