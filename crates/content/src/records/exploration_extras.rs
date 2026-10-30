use crate::{ContentError, span, word};

/// A map's EXTRA table (`CombatContent::encounters`): 10-byte records that
/// story parameters select. The caller decides how a negative index is read.
pub struct ExtraTable<'a>(pub &'a [u8]);

impl<'a> ExtraTable<'a> {
    /// Size of one EXTRA record.
    pub const RECORD_BYTES: usize = 10;

    /// EXTRA record `index`.
    pub fn record(&self, index: usize) -> Result<&'a [u8], ContentError> {
        span(self.0, index * Self::RECORD_BYTES, Self::RECORD_BYTES)
    }
    /// Combat story effects select their EXTRA by the parameter's magnitude.
    pub fn by_magnitude(&self, id: i16) -> Result<&'a [u8], ContentError> {
        self.record(usize::from(id.unsigned_abs()))
    }
}

/// EXTRA record for the map changes of opcodes 7, 12 and 13. Only the target
/// map is read here; the travel rules read the rest.
pub struct MapMutationExtra<'a>(pub &'a [u8]);

impl MapMutationExtra<'_> {
    /// The map the change applies to.
    pub fn map(&self) -> Result<i32, ContentError> {
        Ok(i32::from(word(self.0, 0)?))
    }
}

/// EXTRA record for opcode 23 (regional encounter change).
pub struct RegionChangeExtra<'a>(pub &'a [u8]);

impl RegionChangeExtra<'_> {
    /// The map holding the region.
    pub fn map(&self) -> Result<i32, ContentError> {
        Ok(i32::from(word(self.0, 0)?))
    }
    /// The region (random-encounter rectangle) to change.
    pub fn region(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// The region's new encounter chance; unlike the battle bounds it is always copied.
    pub fn encounter_chance(&self) -> i16 {
        i16::from_be_bytes([self.0[4], self.0[5]])
    }
    /// The region's new lowest battle number; negative keeps its current bound.
    pub fn battle_minimum(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    /// The region's new highest battle number; negative keeps its current bound.
    pub fn battle_maximum(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}

/// EXTRA record for opcode 128 (outdoor map effect).
pub struct MapEffectExtra<'a>(pub &'a [u8]);

impl MapEffectExtra<'_> {
    /// The effect shown on the outdoor map.
    pub fn effect(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// Horizontal position of the effect.
    pub fn x(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// Vertical position of the effect.
    pub fn y(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// Size of the effect.
    pub fn size(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    /// Delay before the effect plays.
    pub fn delay(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}

/// EXTRA record for opcode 85 (random branch).
pub struct RandomBranchExtra<'a>(pub &'a [u8]);

impl RandomBranchExtra<'_> {
    /// Where the branch goes: 0 another script, 1 an encounter, 2 a typed encounter;
    /// any other value carries on.
    pub fn mode(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// Lowest script or encounter number the draw can pick.
    pub fn minimum(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// Highest script or encounter number the draw can pick.
    pub fn maximum(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// Sound played as the branch is taken.
    pub fn sound(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    /// Message shown as the branch is taken.
    pub fn message(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}

/// EXTRA record for opcode 87 (ally class branch).
pub struct AllyClassBranchExtra<'a>(pub &'a [u8]);

impl AllyClassBranchExtra<'_> {
    /// The monster number looked for among the allies held over.
    pub fn class(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// Where the branch goes: 0 another script, 1 an encounter, 2 a typed encounter;
    /// any other value carries on.
    pub fn mode(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// Zero lets an unmatched party take `unmatched_target`; other values continue.
    pub fn unmatched_gate(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// Script or encounter number taken when an ally of the class is present.
    pub fn matched_target(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    /// Script or encounter number taken when no ally of the class is present.
    pub fn unmatched_target(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
}

/// EXTRA record for opcode 33 (payment branch). The jump fields are read by
/// the shared conditional-jump adapter.
pub struct PaymentBranchExtra<'a>(pub &'a [u8]);

impl PaymentBranchExtra<'_> {
    /// What the party is asked to pay.
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
    /// Percent chance the branch is taken; otherwise the script goes on.
    pub fn percent(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// What happens when the chance passes: -2 stops the script's actions, 1 takes the
    /// conditional jump, 2 ends the menu; other values carry on.
    pub fn action(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
}

/// EXTRA record for opcode -32 (clinic).
pub struct ClinicExtra<'a>(pub &'a [u8]);

impl ClinicExtra<'_> {
    /// The clinic's prices as a percentage of its standard rate.
    pub fn percent(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// Sound played as the party enters the clinic.
    pub fn sound(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// Message shown as the party enters the clinic.
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
    /// The limit placed on how much the party may take.
    pub fn limit(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// -1 leaves the loot unlimited.
    pub fn limit_mode(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
}

/// EXTRA record for opcode 43 (give condition).
pub struct GiveConditionExtra<'a>(pub &'a [u8]);

impl GiveConditionExtra<'_> {
    /// 0 every member, 1 picked members, 2 members with stamina above -10.
    pub fn recipients(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// Condition slot, cleared when positive before the duration is added.
    pub fn condition(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// How long the condition lasts; negative is permanent.
    pub fn duration(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// Sound played as the condition is given.
    pub fn sound(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
}

/// EXTRA record for opcode 50 (pick members by trait).
pub struct PickByTraitExtra<'a>(pub &'a [u8]);

impl PickByTraitExtra<'_> {
    /// 0 race, 1 gender, 2 caste, 3 race class, 4 caste class.
    pub fn mode(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// The gender to pick when the mode is gender.
    pub fn gender(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// The race, caste or class compared by every mode except gender.
    pub fn code(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// Whether only members with stamina left are considered.
    pub fn living_only(&self) -> Result<bool, ContentError> {
        Ok(word(self.0, 8)? != 0)
    }
}

/// EXTRA record for opcode 53 (pick members by caste).
pub struct PickByCasteExtra<'a>(pub &'a [u8]);

impl PickByCasteExtra<'_> {
    /// The caste to pick.
    pub fn caste(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// 1..=3 add a caste family to the picked castes.
    pub fn family(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// 0 every member, 1 living members, 2 already picked members.
    pub fn admission(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
}

/// EXTRA record for opcode 60 (drop member credits); the wealth type is unused.
pub struct DropCreditsExtra<'a>(pub &'a [u8]);

impl DropCreditsExtra<'_> {
    /// Whether only the picked members lose credits; otherwise every member does.
    pub fn picked_only(&self) -> Result<bool, ContentError> {
        Ok(word(self.0, 2)? != 0)
    }
}

/// EXTRA record for opcode 90 (take victory points).
pub struct VictoryPointsExtra<'a>(pub &'a [u8]);

impl VictoryPointsExtra<'_> {
    /// Victory points to take, moved from upgrade points into remaining victory points.
    pub fn amount(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// 1 tests the pick word, 2 divides the amount over the party.
    pub fn mode(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
}

/// EXTRA record for opcode 65 (random treasure).
pub struct RandomTreasureExtra<'a>(pub &'a [u8]);

impl RandomTreasureExtra<'_> {
    /// Item count; a negative count draws Rand(|count|) items.
    pub fn count(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// Lowest item id the draw can pick.
    pub fn minimum_item(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// Highest item id the draw can pick.
    pub fn maximum_item(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
}

/// EXTRA record for the shift-party opcode.
pub struct ShiftPartyExtra<'a>(pub &'a [u8]);

impl ShiftPartyExtra<'_> {
    /// How far to shift the party horizontally.
    pub fn dx(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// How far to shift the party vertically.
    pub fn dy(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// Nonzero draws a random signed shift up to each amount.
    pub fn random(&self) -> Result<bool, ContentError> {
        Ok(word(self.0, 6)? != 0)
    }
}

/// EXTRA record for opcode 103 (boat and camping branch).
pub struct BoatCampingExtra<'a>(pub &'a [u8]);

impl BoatCampingExtra<'_> {
    /// 1 requires the boat, 2 requires no boat.
    pub fn boat_condition(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// 1 requires camping, 2 requires not camping.
    pub fn camping_condition(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// 1 boards the boat, 2 leaves it.
    pub fn boat_change(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
}

/// EXTRA record for the set-land-light opcode.
pub struct LandLightExtra<'a>(pub &'a [u8]);

impl LandLightExtra<'_> {
    /// One-based light level.
    pub fn level(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// Nonzero abandons the script when the level is already set.
    pub fn abandon_if_unchanged(&self) -> Result<bool, ContentError> {
        Ok(word(self.0, 2)? != 0)
    }
}

/// EXTRA record for opcode 41 (disable encounter choice).
pub struct DisableChoiceExtra<'a>(pub &'a [u8]);

impl DisableChoiceExtra<'_> {
    /// The encounter whose choice is disabled.
    pub fn encounter(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// One-based choice.
    pub fn choice(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
}
