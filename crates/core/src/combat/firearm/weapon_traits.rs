use crate::RuleError;
use centurions_content::ITEM_BYTES;
use centurions_content::records::ItemDefinition;
use serde::{Deserialize, Serialize};

/// What the weapon and ammunition in hand contribute to a shot: the impact sound, whether either
/// skips the hit roll and has its damage filtered by armor, and the natural attack in progress.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponTraits {
    /// Kept through neighbour reactions; it picks the sound for an unarmed caster's
    /// result.
    #[serde(default)]
    pub natural_attack_index: i16,
    pub sound: i16,
    pub weapon_armor_filter: i16,
    pub ammunition_armor_filter: i16,
}

impl WeaponTraits {
    /// No weapon in hand.
    pub const fn zero() -> Self {
        Self {
            natural_attack_index: 0,
            sound: 0,
            weapon_armor_filter: 0,
            ammunition_armor_filter: 0,
        }
    }

    /// Takes the impact sound and the armor filter from the weapon.
    pub fn load_weapon(&mut self, weapon: &[u8]) -> Result<(), RuleError> {
        validate_record(weapon)?;
        let sound = ItemDefinition(weapon).impact_sound()?;
        let armor_filter = ItemDefinition(weapon).hit_roll_filter()?;
        self.sound = sound;
        self.weapon_armor_filter = armor_filter;
        Ok(())
    }

    /// Takes the armor filter from the ammunition.
    pub fn load_ammunition(&mut self, ammunition: &[u8]) -> Result<(), RuleError> {
        validate_record(ammunition)?;
        self.ammunition_armor_filter = ItemDefinition(ammunition).hit_roll_filter()?;
        Ok(())
    }

    /// Loads a weapon and its ammunition together; neither is taken if either is bad.
    pub fn load_pair(&mut self, weapon: &[u8], ammunition: &[u8]) -> Result<(), RuleError> {
        let mut next = *self;
        next.load_weapon(weapon)?;
        next.load_ammunition(ammunition)?;
        *self = next;
        Ok(())
    }

    /// Loads a weapon and its ammunition, then lets the ammunition's own impact sound, when it has
    /// one, replace the weapon's.
    pub fn prepare(&mut self, weapon: &[u8], ammunition: &[u8]) -> Result<(), RuleError> {
        let mut next = *self;
        next.load_pair(weapon, ammunition)?;
        let sound = ItemDefinition(ammunition).impact_sound()?;
        if sound != 0 {
            next.sound = sound;
        }
        *self = next;
        Ok(())
    }

    /// Checks that the natural attack in progress is one the monster record holds.
    pub fn validate(&self) -> Result<(), RuleError> {
        if !(0..=34).contains(&self.natural_attack_index) {
            return Err("natural-attack index outside represented NPC record".into());
        }
        Ok(())
    }

    /// The sound a monster's natural attack result plays: the attack's own sound byte, or the first
    /// attack's when this attack is unused.
    pub fn natural_result_sound(&self, monster: &[u8]) -> Result<i16, RuleError> {
        let index = usize::try_from(self.natural_attack_index)
            .map_err(|_| RuleError::Invalid("negative natural-attack index".into()))?;
        let flag = *monster
            .get(0x93 + index * 4)
            .ok_or("natural-attack index outside NPC record")?;
        let offset = if flag == 0 { 0x95 } else { 0x95 + index * 4 };
        Ok(i16::from(
            *monster
                .get(offset)
                .ok_or("natural-attack sound outside NPC record")? as i8,
        ))
    }

    /// The impact sound for the weapon in hand.
    pub fn sound(&self) -> i16 {
        self.sound
    }

    /// Whether the shot skips the hit roll and has its damage filtered by armor: set by the weapon
    /// or its ammunition.
    pub fn armor_filter(&self) -> bool {
        self.weapon_armor_filter
            .wrapping_add(self.ammunition_armor_filter)
            != 0
    }
}

/// Checks that a record is a whole item definition.
fn validate_record(bytes: &[u8]) -> Result<(), RuleError> {
    if bytes.len() != ITEM_BYTES {
        return Err(RuleError::Invalid("weapon work definition size".into()));
    }
    Ok(())
}
