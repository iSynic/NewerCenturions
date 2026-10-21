//! Fields used by wear (0x164a0e) and removeitem (0x14ef76).
//! Indexed effects retain the original signed selector, including native aliases.
use super::put_word;
use crate::{ContentError, InventoryRecord, Warrior, word};

#[derive(Clone, Copy)]
pub enum EquipmentStat {
    MissileAccuracy,
    MeleeAccuracy,
    MissileDodge,
    MeleeDodge,
    Hands,
    Strength,
    Quickness,
    MovementBonus,
    PsiResistance,
    MaximumPsi,
    WeaponRange,
    Armor,
    ShieldChance,
    ShieldCharge,
    ShieldCapacity,
    ShieldCost,
    Equipment(i16),
    Condition(i16),
    CreatureBonus(i16),
    Skill(i16),
    Resistance(usize),
    SpecialEffect122,
    MeleeSound,
    DisplayedMelee,
    DisplayedMissile,
    LoadedAmmunition,
}

impl EquipmentStat {
    fn offset(self) -> usize {
        match self {
            Self::MissileAccuracy => 4,
            Self::MeleeAccuracy => 6,
            Self::MissileDodge => 8,
            Self::MeleeDodge => 10,
            Self::Hands => 0x5c,
            Self::Strength => 0x9e2,
            Self::Quickness => 0x9e8,
            Self::MovementBonus => 0x30,
            Self::PsiResistance => 0x22,
            Self::MaximumPsi => 0x5a,
            Self::WeaponRange => 0x130,
            Self::Armor => 0x2c,
            Self::ShieldChance => 0x24,
            Self::ShieldCharge => 0x26,
            Self::ShieldCapacity => 0x28,
            Self::ShieldCost => 0x2a,
            Self::Equipment(index) => (0xc6_i32 + i32::from(index) * 2) as usize,
            Self::Condition(index) => (0x5e_i32 + i32::from(index) * 2) as usize,
            Self::CreatureBonus(index) => (0xae_i32 + i32::from(index) * 2) as usize,
            Self::Skill(index) => (0xf4_i32 + i32::from(index) * 2) as usize,
            Self::Resistance(index) => 0x114 + index * 2,
            Self::SpecialEffect122 => 0x1e,
            Self::MeleeSound => 0x128,
            Self::DisplayedMelee => 0x12e,
            Self::DisplayedMissile => 0x12c,
            Self::LoadedAmmunition => 0x126,
        }
    }
    pub fn read(self, warrior: &Warrior) -> Result<i16, ContentError> {
        word(&warrior.bytes, self.offset())
    }
    pub fn write(self, warrior: &mut Warrior, value: i16) -> Result<(), ContentError> {
        self.read(warrior)?;
        put_word(&mut warrior.bytes, self.offset(), value);
        Ok(())
    }
}

/// Definition and retained-instance modifiers are deliberately separate. Wear
/// reads the caller's retained instance, which need not be the inventory slot.
pub struct EquipmentModifiers {
    pub kind: i16,
    pub id: i16,
    pub hands: i16,
    pub impact_sound: i16,
    pub energy_requirement: i16,
    pub adds_energy: bool,
    pub energy_capacity: i16,
    pub shield_chance: i16,
    pub shield_cost: i16,
    pub removal_shield_cost: i16,
    pub strength: i16,
    pub quickness: i16,
    pub missile_dodge: i16,
    pub melee_dodge: i16,
    pub movement: i16,
    pub psi_resistance: i16,
    pub maximum_psi: i16,
    pub range: i16,
    pub armor: i16,
    pub resistances: [i16; 7],
    pub effect: i16,
    pub effect_amount: i16,
    pub attributes: [i16; 2],
    pub attribute_amount: i16,
}

impl EquipmentModifiers {
    pub fn read(definition: &[u8]) -> Result<Self, ContentError> {
        crate::span(definition, 0, 150)?;
        let signed = |offset| i16::from(definition[offset] as i8);
        Ok(Self {
            kind: word(definition, 26)?,
            id: word(definition, 102)?,
            hands: signed(8),
            impact_sound: word(definition, 14)?,
            energy_requirement: word(definition, 146)?,
            adds_energy: word(definition, 90)? != 0,
            energy_capacity: word(definition, 148)?,
            shield_chance: word(definition, 88)?,
            shield_cost: word(definition, 92)?,
            removal_shield_cost: signed(80),
            strength: signed(0),
            quickness: signed(2),
            missile_dodge: signed(3),
            melee_dodge: signed(4),
            movement: signed(5),
            psi_resistance: signed(6),
            maximum_psi: signed(7),
            range: word(definition, 60)?,
            armor: word(definition, 94)?,
            resistances: [140, 64, 66, 68, 70, 72, 74]
                .map(|offset| word(definition, offset).unwrap()),
            effect: word(definition, 42)?,
            effect_amount: word(definition, 44)?,
            attributes: [word(definition, 46)?, word(definition, 48)?],
            attribute_amount: word(definition, 50)?,
        })
    }
}

pub struct RetainedEquipmentModifiers {
    pub shield_chance: i16,
    pub shield_cost: i16,
    pub accuracy: i16,
    pub wear_melee_accuracy: i16,
    pub missile_dodge: i16,
    pub melee_dodge: i16,
    pub armor: i16,
    pub range: i16,
}

impl RetainedEquipmentModifiers {
    pub fn read(record: &InventoryRecord) -> Self {
        let signed = |offset| i16::from(record.0[offset] as i8);
        Self {
            shield_chance: signed(14),
            shield_cost: signed(21),
            accuracy: signed(15),
            wear_melee_accuracy: signed(11),
            missile_dodge: signed(16),
            melee_dodge: signed(17),
            armor: signed(19),
            range: signed(20),
        }
    }
}

/// Instance BYTE modifiers shared by upgrade, random generation and editing.
#[derive(Clone, Copy)]
pub enum InventoryModifier {
    MeleeAccuracy = 11,
    RequiredLevel = 13,
    ShieldChance = 14,
    Accuracy = 15,
    MissileDodge = 16,
    MeleeDodge = 17,
    Damage = 18,
    Armor = 19,
    Range = 20,
    ShieldCost = 21,
}

impl InventoryModifier {
    pub fn read(self, record: &InventoryRecord) -> i8 {
        record.0[self as usize] as i8
    }
    pub fn write(self, record: &mut InventoryRecord, value: i8) {
        record.0[self as usize] = value as u8;
    }
    pub fn add(self, record: &mut InventoryRecord, amount: i8) {
        self.write(record, self.read(record).wrapping_add(amount));
    }
}

pub struct ItemUpgrade {
    pub capacity_percent: i16,
    pub shield_chance: i8,
    pub accuracy: i8,
    pub missile_dodge: i8,
    pub melee_dodge: i8,
    pub damage: i8,
    pub armor: i8,
    pub range: i8,
    pub shield_cost: i8,
    pub equipment_kind: i8,
}

impl ItemUpgrade {
    pub fn read(definition: &[u8]) -> Result<Self, ContentError> {
        crate::span(definition, 0, 150)?;
        Ok(Self {
            capacity_percent: word(definition, 148)?,
            shield_chance: definition[78] as i8,
            accuracy: definition[81] as i8,
            missile_dodge: definition[83] as i8,
            melee_dodge: definition[84] as i8,
            damage: definition[85] as i8,
            armor: definition[86] as i8,
            range: definition[79] as i8,
            shield_cost: definition[80] as i8,
            equipment_kind: definition[87] as i8,
        })
    }
}
