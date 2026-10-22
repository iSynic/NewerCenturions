use crate::{ContentError, InventoryRecord, mission::long, word};

// edititems 0x134034 display/control order, shared by drawing and mutation.
const EDITOR_MODIFIERS: [usize; 10] = [13, 15, 16, 11, 17, 18, 19, 14, 21, 20];

/// Item class codes: zero-based entries of Cargo Pod `STR#` -11000 "Item Classes".
pub mod item_class {
    /// "Melee Weapon".
    pub const MELEE_WEAPON: i16 = 2;
    /// "Missile Weapon".
    pub const MISSILE_WEAPON: i16 = 15;
    /// "Shield Battery".
    pub const SHIELD_BATTERY: i16 = 20;
    /// "Ammo Item".
    pub const AMMUNITION: i16 = 26;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeaponCondition {
    pub mode: i16,
    pub threshold: i16,
    pub slot: i16,
    pub amount: i16,
}

/// Original 150-byte definition, borrowed without eager validation. Callers
/// keep their existing field-read order and malformed-record diagnostics.
#[derive(Clone, Copy)]
pub struct ItemDefinition<'a>(pub &'a [u8]);

impl ItemDefinition<'_> {
    /// Class-23 items interpret this shared WORD as their continuation script.
    pub fn story_script(&self) -> Result<i16, ContentError> {
        word(self.0, 0x32)
    }

    pub fn icon(&self) -> Result<i16, ContentError> {
        word(self.0, 10)
    }
    pub fn description_id(&self) -> Result<i16, ContentError> {
        word(self.0, 102)
    }
    /// Matched against the enemy's special weapon-immunity classes by attack.
    pub fn immunity_class(&self) -> Result<i16, ContentError> {
        word(self.0, 100)
    }
    pub fn impact_sound(&self) -> Result<i16, ContentError> {
        word(self.0, 14)
    }
    pub fn pickup_sound(&self) -> Result<i16, ContentError> {
        word(self.0, 12)
    }
    /// fit/refit use this field as the weapon's ammunition compatibility key.
    /// Other item categories assign different meanings to the same stored word.
    pub fn ammunition_caliber(&self) -> Result<i16, ContentError> {
        word(self.0, 32)
    }
    pub fn supplies_shield_energy(&self) -> Result<bool, ContentError> {
        Ok(word(self.0, 18)? != 0)
    }
    pub fn base_weight(&self) -> Result<i16, ContentError> {
        word(self.0, 16)
    }
    pub fn supplied_quantity(&self) -> Result<i16, ContentError> {
        word(self.0, 20)
    }
    pub fn magazine_capacity(&self) -> Result<i16, ContentError> {
        word(self.0, 24)
    }
    pub fn battery_capacity(&self) -> Result<i16, ContentError> {
        word(self.0, 90)
    }
    pub fn capacity_bonus(&self) -> Result<i16, ContentError> {
        word(self.0, 148)
    }
    pub fn required_level(&self) -> Result<i16, ContentError> {
        word(self.0, 138)
    }
    pub fn base_melee_accuracy(&self) -> i8 {
        self.0[82] as i8
    }
    pub fn base_accuracy(&self) -> i8 {
        self.0[81] as i8
    }
    pub fn rounds_per_shot(&self) -> Result<i16, ContentError> {
        word(self.0, 22)
    }
    pub fn missile_damage(&self) -> Result<i16, ContentError> {
        word(self.0, 62)
    }
    /// prepattack treats the low byte as signed, while the estimate uses the word.
    pub fn selected_attack_count(&self) -> i8 {
        self.0[77] as i8
    }
    pub fn hit_roll_filter(&self) -> Result<i16, ContentError> {
        word(self.0, 136)
    }
    pub fn attack_count(&self) -> Result<i16, ContentError> {
        word(self.0, 76)
    }
    pub fn bypasses_hit_roll(&self) -> Result<bool, ContentError> {
        Ok(word(self.0, 136)? != 0)
    }
    pub fn damage_enhancement(&self) -> i8 {
        self.0[85] as i8
    }
    pub fn melee_damage(&self) -> Result<i16, ContentError> {
        word(self.0, 40)
    }
    pub fn elemental_damage(&self, element: usize) -> Result<i16, ContentError> {
        word(self.0, [140, 64, 66, 68, 70, 72, 74][element])
    }
    pub fn category(&self) -> Result<i16, ContentError> {
        word(self.0, 26)
    }
    /// Retained by loaditem and saved unchanged; callers must not infer a
    /// single universal meaning for this category-dependent native field.
    pub fn retained_field32(&self) -> Result<i16, ContentError> {
        word(self.0, 32)
    }
    pub fn reserve_ammunition_kind(&self) -> Result<i16, ContentError> {
        word(self.0, 34)
    }
    pub fn unit_weight(&self) -> Result<i16, ContentError> {
        word(self.0, 36)
    }
    pub fn discarded_when_empty(&self) -> Result<i16, ContentError> {
        word(self.0, 38)
    }
    pub fn effect_selector(&self) -> Result<i16, ContentError> {
        word(self.0, 42)
    }
    pub fn spell(&self) -> Result<i16, ContentError> {
        word(self.0, 44)
    }
    /// Effect selector -10 on a melee weapon: a condition admitted by mode word 44
    /// (1 a failed saving throw indexed by word 48, 2 Rand(100) at most word 48,
    /// otherwise always) that adds word 50 to condition slot word 46 minus 20.
    pub fn weapon_condition(&self) -> Result<Option<WeaponCondition>, ContentError> {
        if self.effect_selector()? != -10 {
            return Ok(None);
        }
        Ok(Some(WeaponCondition {
            mode: word(self.0, 44)?,
            threshold: word(self.0, 48)?,
            slot: word(self.0, 46)?.wrapping_sub(20),
            amount: word(self.0, 50)?,
        }))
    }
    pub fn price(&self) -> Result<i32, ContentError> {
        long(self.0, 142)
    }
    /// randomitem copies these initial modifiers before its random upgrades.
    pub fn install_generated_modifiers(&self, record: &mut InventoryRecord) {
        for (target, source) in [
            (14, 78),
            (15, 81),
            (11, 82),
            (16, 83),
            (17, 84),
            (18, 85),
            (19, 86),
            (21, 80),
            (20, 79),
            (13, 139),
        ] {
            record.0[target] = self.0[source];
        }
    }
    pub fn shield_chance(&self) -> Result<i16, ContentError> {
        word(self.0, 88)
    }
    pub fn armor(&self) -> Result<i16, ContentError> {
        word(self.0, 94)
    }
    pub fn missile_dodge_limit(&self) -> i8 {
        self.0[3] as i8
    }
    pub fn melee_dodge_limit(&self) -> i8 {
        self.0[4] as i8
    }
    pub fn range(&self) -> Result<i16, ContentError> {
        word(self.0, 60)
    }
    pub fn base_range_modifier(&self) -> i8 {
        self.0[79] as i8
    }
    pub fn generated_required_level(&self) -> i8 {
        self.0[139] as i8
    }
    /// installitem 0x12c874 fresh-stock branch. Capacity selection and copied
    /// modifiers stay distinct from updateshop's retained display record.
    pub fn fresh_stock(&self) -> Result<InventoryRecord, ContentError> {
        let mut record = InventoryRecord([0; 22]);
        let quantity = word(self.0, 148)?.wrapping_add(word(self.0, 20)?);
        let capacity = match word(self.0, 90)? {
            0 => match word(self.0, 24)? {
                0 => quantity,
                n => n,
            },
            n => n,
        };
        for (offset, value) in [(0, word(self.0, 102)?), (2, quantity), (6, capacity)] {
            record.0[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        }
        for (destination, source) in [
            (8, 35),
            (10, 27),
            (13, 139),
            (14, 78),
            (15, 81),
            (16, 83),
            (17, 84),
            (18, 85),
            (19, 86),
            (21, 80),
            (20, 79),
        ] {
            record.0[destination] = self.0[source];
        }
        Ok(record)
    }

    /// install_fresh leaves the two private instance bytes 11/12 untouched.
    pub fn install_fresh_into(&self, record: &mut InventoryRecord) -> Result<(), ContentError> {
        let fresh = self.fresh_stock()?;
        record.0[..11].copy_from_slice(&fresh.0[..11]);
        record.0[13..].copy_from_slice(&fresh.0[13..]);
        record.0[10] = self.category()?.wrapping_abs() as u8;
        Ok(())
    }

    /// updateshop 0x139244 deliberately leaves instance bytes 11 and 12 intact.
    pub fn copy_display_stock_into(
        &self,
        record: &mut InventoryRecord,
    ) -> Result<(), ContentError> {
        for (offset, value) in [
            (0, word(self.0, 102)?),
            (2, word(self.0, 20)?),
            (4, 0),
            (6, word(self.0, 20)?),
        ] {
            record.0[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        }
        record.0[9] = 0;
        for (destination, source) in [
            (13, 139),
            (8, 35),
            (10, 27),
            (14, 78),
            (15, 81),
            (16, 83),
            (17, 84),
            (18, 85),
            (19, 86),
            (21, 80),
            (20, 79),
        ] {
            record.0[destination] = self.0[source];
        }
        Ok(())
    }
}

impl InventoryRecord {
    pub fn required_level(&self) -> i8 {
        self.0[13] as i8
    }
    pub fn clear_id(&mut self) {
        self.0[..2].fill(0);
    }
    pub fn accuracy_modifier(&self) -> i8 {
        self.0[15] as i8
    }
    pub fn damage_enhancement(&self) -> i8 {
        self.0[18] as i8
    }
    pub fn range_modifier(&self) -> u8 {
        self.0[20]
    }
    pub fn editor_modifier(&self, row: usize) -> i8 {
        self.0[EDITOR_MODIFIERS[row]] as i8
    }
    pub fn set_editor_modifier(&mut self, row: usize, value: u8) {
        self.0[EDITOR_MODIFIERS[row]] = value;
    }
    pub fn adjust_editor_modifier(&mut self, row: usize, delta: i8) {
        let value = &mut self.0[EDITOR_MODIFIERS[row]];
        *value = value.wrapping_add(delta as u8);
    }
    pub fn set_id(&mut self, id: i16) {
        super::put_word(&mut self.0, 0, id);
    }
    pub fn set_category(&mut self, category: u8) {
        self.0[10] = category;
    }
    pub fn set_ammunition(&mut self, id: i16) {
        super::put_word(&mut self.0, 4, id);
    }
    pub fn set_equipment_flag(&mut self, value: u8) {
        self.0[9] = value;
    }
    pub fn set_quantity(&mut self, quantity: i16) {
        super::put_word(&mut self.0, 2, quantity);
    }
    pub fn capacity(&self) -> Result<i16, ContentError> {
        word(&self.0, 6)
    }
    pub fn set_capacity(&mut self, capacity: i16) {
        super::put_word(&mut self.0, 6, capacity);
    }
    /// Stack count or loaded rounds, according to the original item category.
    pub fn quantity(&self) -> Result<i16, ContentError> {
        word(&self.0, 2)
    }
    pub fn ammunition_id(&self) -> Result<i16, ContentError> {
        word(&self.0, 4)
    }
    pub fn equipment_flag(&self) -> u8 {
        self.0[9]
    }
    pub fn category_code(&self) -> u8 {
        self.0[10]
    }
}
