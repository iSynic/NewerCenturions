use crate::RuleError;
use centurions_content::records::ItemDefinition;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponWork {
    /// WORD4435d4 survives checkforenemy and selects an unarmed caster's result cue.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub natural_attack_index: Option<i16>,
    pub sound: Option<i16>,
    pub weapon_armor_filter: Option<i16>,
    pub ammunition_armor_filter: Option<i16>,
}

impl WeaponWork {
    pub const fn zero() -> Self {
        Self {
            natural_attack_index: Some(0),
            sound: Some(0),
            weapon_armor_filter: Some(0),
            ammunition_armor_filter: Some(0),
        }
    }

    pub fn load_weapon(&mut self, weapon: &[u8]) -> Result<(), RuleError> {
        validate_record(weapon)?;
        let sound = ItemDefinition(weapon).impact_sound()?;
        let armor_filter = ItemDefinition(weapon).hit_roll_filter()?;
        self.sound = Some(sound);
        self.weapon_armor_filter = Some(armor_filter);
        Ok(())
    }

    pub fn load_ammunition(&mut self, ammunition: &[u8]) -> Result<(), RuleError> {
        validate_record(ammunition)?;
        self.ammunition_armor_filter = Some(ItemDefinition(ammunition).hit_roll_filter()?);
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
        let sound = ItemDefinition(ammunition).impact_sound()?;
        if sound != 0 {
            next.sound = Some(sound);
        }
        *self = next;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), RuleError> {
        if self
            .natural_attack_index
            .is_some_and(|index| !(0..=34).contains(&index))
        {
            return Err("natural-attack index outside represented NPC record".into());
        }
        Ok(())
    }

    pub fn natural_result_sound(&self, monster: &[u8]) -> Result<i16, RuleError> {
        let index = self.natural_attack_index.ok_or(RuleError::Unsupported {
            action: "combat.weapon_work",
            dependency: "unreconstructable retained natural-attack sound index",
        })?;
        let index = usize::try_from(index)
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
