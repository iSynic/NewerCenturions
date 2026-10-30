use super::warrior_layout::PSI_SCHOOL;
use crate::{ContentError, word};

/// One row of the Upgrade Warrior dialog (selectors 0..32), read from the
/// warrior record. Base values, calculated bonuses and purchase counters are
/// separate saved fields.
pub struct UpgradeRecord<'a> {
    bytes: &'a [u8],
    selector: u16,
}

impl<'a> UpgradeRecord<'a> {
    /// Views the row `selector` of a warrior record.
    pub(super) fn new(bytes: &'a [u8], selector: u16) -> Self {
        Self { bytes, selector }
    }

    /// Whether the row can be raised: a special skill needs a rating of at least one
    /// and the Psi row needs a Psi school.
    pub fn can_increase(&self) -> Result<bool, ContentError> {
        if (23..=31).contains(&self.selector)
            && word(self.bytes, 0xf4 + usize::from(self.selector - 23) * 2)? < 1
            || self.selector == 21 && word(self.bytes, PSI_SCHOOL)? == 0
        {
            return Ok(false);
        }
        Ok(true)
    }

    /// How many times the row has been bought.
    pub fn purchases(&self) -> Result<i16, ContentError> {
        word(self.bytes, 0xace + usize::from(self.selector) * 2)
    }

    /// The value shown for the row. Base and bonus each fall back to zero on
    /// their own, so a missing base does not hide a calculated bonus.
    pub fn display_value(&self) -> i16 {
        let (base, bonus) = self.display_offsets();
        word(self.bytes, base).unwrap_or(0).wrapping_add(
            bonus
                .and_then(|offset| word(self.bytes, offset).ok())
                .unwrap_or(0),
        )
    }

    /// Whether the row is offered in the dialog.
    pub fn display_available(&self) -> bool {
        let (base, _) = self.display_offsets();
        !(self.selector == 21 && word(self.bytes, PSI_SCHOOL).unwrap_or(0) == 0
            || (23..=31).contains(&self.selector) && word(self.bytes, base).unwrap_or(0) < 1)
    }

    /// Where the row's base value and, if it has one, its calculated bonus lie.
    fn display_offsets(&self) -> (usize, Option<usize>) {
        let selector = usize::from(self.selector);
        match selector {
            // Attributes, load, resistances, hit/dodge, stamina, Psi, special skills, damage.
            0..=6 => (0x9e2 + selector * 2, None),
            7 => (0x9d8, None),
            8..=15 => (0x112 + (selector - 8) * 2, Some(0xb48 + (selector - 8) * 2)),
            16..=19 => (4 + (selector - 16) * 2, Some(0xb58 + (selector - 16) * 2)),
            20 => (0xb42, Some(0xb46)),
            21 => (0xb40, Some(0xb44)),
            22 => (0x22, Some(0xb60)),
            23..=31 => (
                0xf4 + (selector - 23) * 2,
                Some(0xb62 + (selector - 23) * 2),
            ),
            _ => (12, Some(0xb7a)),
        }
    }
}
