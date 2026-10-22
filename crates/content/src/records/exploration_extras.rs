use crate::{ContentError, span, word};

/// Map EXTRA table (`CombatContent::encounters`) of 10-byte records addressed
/// by story parameters. The caller keeps its own index sign handling.
pub struct ExtraTable<'a>(pub &'a [u8]);

impl<'a> ExtraTable<'a> {
    pub const RECORD_BYTES: usize = 10;

    pub fn record(&self, index: usize) -> Result<&'a [u8], ContentError> {
        span(self.0, index * Self::RECORD_BYTES, Self::RECORD_BYTES)
    }
    /// Combat story effects select their EXTRA by the parameter's magnitude.
    pub fn by_magnitude(&self, id: i16) -> Result<&'a [u8], ContentError> {
        self.record(usize::from(id.unsigned_abs()))
    }
}

/// EXTRA record for the opcode 7/12/13 map mutations; only the target map is
/// read here, the remaining fields belong to the travel adapters.
pub struct MapMutationExtra<'a>(pub &'a [u8]);

impl MapMutationExtra<'_> {
    pub fn map(&self) -> Result<i32, ContentError> {
        Ok(i32::from(word(self.0, 0)?))
    }
}

/// EXTRA record for opcode 23 (regional encounter change).
pub struct RegionChangeExtra<'a>(pub &'a [u8]);

impl RegionChangeExtra<'_> {
    pub fn map(&self) -> Result<i32, ContentError> {
        Ok(i32::from(word(self.0, 0)?))
    }
    pub fn region(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// Copied unconditionally as raw bytes, so this read has no failure path.
    pub fn encounter_chance(&self) -> i16 {
        i16::from_be_bytes([self.0[4], self.0[5]])
    }
    /// Negative values keep the region's current bound.
    pub fn battle_minimum(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    pub fn battle_maximum(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}

/// EXTRA record for opcode 128 (outdoor map effect).
pub struct MapEffectExtra<'a>(pub &'a [u8]);

impl MapEffectExtra<'_> {
    pub fn effect(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn x(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    pub fn y(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    pub fn size(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    pub fn delay(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}

/// EXTRA record for opcode 85 (random branch).
pub struct RandomBranchExtra<'a>(pub &'a [u8]);

impl RandomBranchExtra<'_> {
    pub fn mode(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn minimum(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    pub fn maximum(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    pub fn sound(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    pub fn message(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}

/// EXTRA record for opcode 87 (ally class branch).
pub struct AllyClassBranchExtra<'a>(pub &'a [u8]);

impl AllyClassBranchExtra<'_> {
    /// Compared with each ally's signed definition byte.
    pub fn class(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn mode(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// Zero lets an unmatched party take `unmatched_target`; other values continue.
    pub fn unmatched_gate(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    pub fn matched_target(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    pub fn unmatched_target(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}

/// EXTRA record for opcode 33 (payment branch). The jump fields are read by
/// the shared conditional-jump adapter.
pub struct PaymentBranchExtra<'a>(pub &'a [u8]);

impl PaymentBranchExtra<'_> {
    pub fn cost(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// -1 skips to action 6 when unpaid; 0, 1 and 2 jump when unpaid, paid or always.
    pub fn condition(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
}

/// EXTRA record for opcode 42 (chance branch).
pub struct ChanceBranchExtra<'a>(pub &'a [u8]);

impl ChanceBranchExtra<'_> {
    pub fn percent(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn action(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
}

/// EXTRA record for opcode -32 (clinic).
pub struct ClinicExtra<'a>(pub &'a [u8]);

impl ClinicExtra<'_> {
    pub fn percent(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn sound(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    pub fn text(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
}

/// EXTRA record for opcode 111 (limited shop or treasure).
pub struct LimitedTreasureExtra<'a>(pub &'a [u8]);

impl LimitedTreasureExtra<'_> {
    /// Positive values select a treasure; others open the restricted shop.
    pub fn treasure(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn limit(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// -1 leaves the loot unlimited.
    pub fn limit_mode(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
}
