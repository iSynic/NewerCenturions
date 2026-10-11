use crate::RuleError;
use centurions_content::word;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponWork {
    pub sound: Option<i16>,
    pub weapon_armor_filter: Option<i16>,
    pub ammunition_armor_filter: Option<i16>,
}

impl WeaponWork {
    pub const fn zero() -> Self {
        Self {
            sound: Some(0),
            weapon_armor_filter: Some(0),
            ammunition_armor_filter: Some(0),
        }
    }

    pub fn load_weapon(&mut self, weapon: &[u8]) -> Result<(), RuleError> {
        validate_record(weapon)?;
        let sound = word(weapon, 0x0e)?;
        let armor_filter = word(weapon, 0x88)?;
        self.sound = Some(sound);
        self.weapon_armor_filter = Some(armor_filter);
        Ok(())
    }

    pub fn load_ammunition(&mut self, ammunition: &[u8]) -> Result<(), RuleError> {
        validate_record(ammunition)?;
        self.ammunition_armor_filter = Some(word(ammunition, 0x88)?);
        Ok(())
    }

    pub fn load_pair(&mut self, weapon: &[u8], ammunition: &[u8]) -> Result<(), RuleError> {
        let mut next = *self;
        next.load_weapon(weapon)?;
        next.load_ammunition(ammunition)?;
        *self = next;
        Ok(())
    }

    pub fn prepare(&mut self, weapon: &[u8], ammunition: &[u8]) -> Result<(), RuleError> {
        let mut next = *self;
        next.load_pair(weapon, ammunition)?;
        let sound = word(ammunition, 0x0e)?;
        if sound != 0 {
            next.sound = Some(sound);
        }
        *self = next;
        Ok(())
    }

    pub fn sound(&self) -> Result<i16, RuleError> {
        self.sound.ok_or(RuleError::Unsupported {
            action: "combat.weapon_work",
            dependency: "unreconstructable retained weapon sound",
        })
    }

    pub fn armor_filter(&self) -> Result<bool, RuleError> {
        let weapon = self.weapon_armor_filter.ok_or(RuleError::Unsupported {
            action: "combat.weapon_work",
            dependency: "unreconstructable retained weapon armor filter",
        })?;
        let ammunition = self.ammunition_armor_filter.ok_or(RuleError::Unsupported {
            action: "combat.weapon_work",
            dependency: "unreconstructable retained ammunition armor filter",
        })?;
        Ok(weapon.wrapping_add(ammunition) != 0)
    }
}

fn validate_record(bytes: &[u8]) -> Result<(), RuleError> {
    if bytes.len() != 150 {
        return Err(RuleError::Invalid("weapon work definition size".into()));
    }
    Ok(())
}
