//! Warrior development fields set at creation, raised by upgrades and
//! recalculated with the warrior's statistics. Base values and calculated
//! bonuses are separate saved words.
use super::put_word;
use super::warrior_layout::{PSI_SCHOOL, STAMINA, STAMINA_MAX};
use crate::{ContentError, word};

/// The seven attributes, in record order.
#[derive(Clone, Copy)]
#[repr(usize)]
pub enum Attribute {
    Brawn,
    Knowledge,
    Judgement,
    Quickness,
    Coordination,
    Vitality,
    Psionics,
}

/// Every attribute, in record order.
pub const ATTRIBUTES: [Attribute; 7] = [
    Attribute::Brawn,
    Attribute::Knowledge,
    Attribute::Judgement,
    Attribute::Quickness,
    Attribute::Coordination,
    Attribute::Vitality,
    Attribute::Psionics,
];

/// A warrior field that creation sets, upgrades raise and recalculation updates.
#[derive(Clone, Copy)]
pub enum DevelopmentField {
    Attribute(Attribute),
    CarryCapacity,
    Stamina,
    MaximumStamina,
    Psi,
    MaximumPsi,
    MovementPoints,
    MovementPerRound,
    ActionsPerRound,
    MovementBonus,
    ScanCapacity,
    AttackLimit,
    MeleeSound,
    Armor,
    PsiSchool,
    Level,

    Resistance(usize),
    ResistanceBonus(usize),
    MissileAccuracy,
    MeleeAccuracy,
    MissileDodge,
    MeleeDodge,
    MissileAccuracyBonus,
    MeleeAccuracyBonus,
    MissileDodgeBonus,
    MeleeDodgeBonus,
    BaseStamina,
    BasePsi,
    StaminaBonus,
    PsiBonus,
    PsiResistance,
    PsiResistanceBonus,
    SpecialSkill(usize),
    SpecialSkillBonus(usize),
    BareHandMaximum,
    BareHandMinimum,
    BareHandBonus,
    CreatureBonus(usize),
}

impl DevelopmentField {
    /// Locates the field in the warrior record.
    fn offset(self) -> usize {
        match self {
            Self::Attribute(attribute) => 0x9e2 + attribute as usize * 2,
            Self::Stamina => STAMINA,
            Self::MaximumStamina => STAMINA_MAX,
            Self::Psi => 0x58,
            Self::MaximumPsi => 0x5a,
            Self::MovementPoints => 0x3c,
            Self::MovementPerRound => 0x3e,
            Self::ActionsPerRound => 0x12,
            Self::MovementBonus => 0x30,
            Self::ScanCapacity => 0xac6,
            Self::AttackLimit => 0xaca,
            Self::MeleeSound => 0x128,
            Self::Armor => 0x2c,
            Self::PsiSchool => PSI_SCHOOL,
            Self::Level => 0x3a,
            Self::CarryCapacity => 0x9d8,
            Self::Resistance(index) => 0x112 + index * 2,
            Self::ResistanceBonus(index) => 0xb48 + index * 2,
            Self::MissileAccuracy => 4,
            Self::MeleeAccuracy => 6,
            Self::MissileDodge => 8,
            Self::MeleeDodge => 10,
            Self::MissileAccuracyBonus => 0xb58,
            Self::MeleeAccuracyBonus => 0xb5a,
            Self::MissileDodgeBonus => 0xb5c,
            Self::MeleeDodgeBonus => 0xb5e,
            Self::BaseStamina => 0xb42,
            Self::BasePsi => 0xb40,
            Self::StaminaBonus => 0xb46,
            Self::PsiBonus => 0xb44,
            Self::PsiResistance => 0x22,
            Self::PsiResistanceBonus => 0xb60,
            Self::SpecialSkill(index) => 0xf4 + index * 2,
            Self::SpecialSkillBonus(index) => 0xb62 + index * 2,
            Self::BareHandMaximum => 12,
            Self::BareHandMinimum => 14,
            Self::BareHandBonus => 0xb7a,
            Self::CreatureBonus(index) => 0xae + index * 2,
        }
    }
    /// Reads the field from a warrior record.
    pub fn read(self, bytes: &[u8]) -> Result<i16, ContentError> {
        word(bytes, self.offset())
    }
    /// Sets the field in a warrior record.
    pub fn write(self, bytes: &mut [u8], value: i16) {
        put_word(bytes, self.offset(), value);
    }
    /// Adds `value` to the field, wrapping.
    pub fn add(self, bytes: &mut [u8], value: i16) -> Result<(), ContentError> {
        self.write(bytes, self.read(bytes)?.wrapping_add(value));
        Ok(())
    }
    /// The field that row `selector` of the Upgrade Warrior dialog raises; none for a
    /// selector past the last row.
    pub fn upgrade(selector: u16) -> Option<Self> {
        Some(match selector {
            0..=6 => Self::Attribute(ATTRIBUTES[selector as usize]),
            7 => Self::CarryCapacity,
            8..=15 => Self::Resistance(selector as usize - 8),
            16 => Self::MissileAccuracy,
            17 => Self::MeleeAccuracy,
            18 => Self::MissileDodge,
            19 => Self::MeleeDodge,
            20 => Self::BaseStamina,
            21 => Self::BasePsi,
            22 => Self::PsiResistance,
            23..=31 => Self::SpecialSkill(selector as usize - 23),
            32 => Self::BareHandMaximum,
            _ => return None,
        })
    }
}
