use super::put_word;
use crate::bytes::long;
use crate::{ContentError, InventoryRecord, word};

// The item editor's modifier order, used both to draw and to change them.
/// The inventory bytes that make up the item editor's ten modifier rows, in row order.
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

/// A condition a melee weapon gives its target; see `ItemDefinition::weapon_condition`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeaponCondition {
    /// When the condition lands: 1 on a failed saving throw, 2 by chance, otherwise always.
    pub mode: i16,
    /// The saving throw that is tried, or the percent chance, by mode.
    pub threshold: i16,
    /// The condition slot that is raised.
    pub slot: i16,
    /// How much the slot is raised.
    pub amount: i16,
}

/// A 150-byte item definition from the Data ID item table. Fields are read on
/// demand, so each caller reports a short record where it reads it.
#[derive(Clone, Copy)]
pub struct ItemDefinition<'a>(pub &'a [u8]);

impl ItemDefinition<'_> {
    /// Class-23 items keep their continuation script in this shared word.
    pub fn story_script(&self) -> Result<i16, ContentError> {
        word(self.0, 0x32)
    }

    /// The item's icon.
    pub fn icon(&self) -> Result<i16, ContentError> {
        word(self.0, 10)
    }
    /// The item's own id, as an inventory record stores it and the description tables number it.
    pub fn description_id(&self) -> Result<i16, ContentError> {
        word(self.0, 102)
    }
    /// Matched against a monster's weapon immunities when it is attacked.
    pub fn immunity_class(&self) -> Result<i16, ContentError> {
        word(self.0, 100)
    }
    /// The sound played where the item's attack lands.
    pub fn impact_sound(&self) -> Result<i16, ContentError> {
        word(self.0, 14)
    }
    /// The item's sound; a wielded weapon lends it to the warrior's attack.
    pub fn sound(&self) -> Result<i16, ContentError> {
        word(self.0, 12)
    }
    /// The ammunition a weapon takes; reloading matches it against the round.
    /// Other item classes give the same word other meanings.
    pub fn ammunition_caliber(&self) -> Result<i16, ContentError> {
        word(self.0, 32)
    }
    /// Whether the item recharges a power shield.
    pub fn supplies_shield_energy(&self) -> Result<bool, ContentError> {
        Ok(word(self.0, 18)? != 0)
    }
    /// The item's weight.
    pub fn weight(&self) -> Result<i16, ContentError> {
        word(self.0, 16)
    }
    /// Charges a fresh item starts with; a negative value marks an item that needs no charge to
    /// use.
    pub fn supplied_quantity(&self) -> Result<i16, ContentError> {
        word(self.0, 20)
    }
    /// Rounds the weapon's magazine holds.
    pub fn magazine_capacity(&self) -> Result<i16, ContentError> {
        word(self.0, 24)
    }
    /// Charge a battery holds.
    pub fn battery_capacity(&self) -> Result<i16, ContentError> {
        word(self.0, 90)
    }
    /// Added to a battery's capacity when a fresh item is made.
    pub fn capacity_bonus(&self) -> Result<i16, ContentError> {
        word(self.0, 148)
    }
    /// Level a warrior must have to use the item.
    pub fn required_level(&self) -> Result<i16, ContentError> {
        word(self.0, 138)
    }
    /// Melee accuracy the item gives.
    pub fn base_melee_accuracy(&self) -> i8 {
        self.0[82] as i8
    }
    /// Accuracy the item gives.
    pub fn base_accuracy(&self) -> i8 {
        self.0[81] as i8
    }
    /// Rounds one shot of the weapon uses.
    pub fn rounds_per_shot(&self) -> Result<i16, ContentError> {
        word(self.0, 22)
    }
    /// Damage of the weapon's missile attack.
    pub fn missile_damage(&self) -> Result<i16, ContentError> {
        word(self.0, 62)
    }
    /// Shots the weapon fires in one attack: the low byte of the attack count, read as signed.
    pub fn selected_attack_count(&self) -> i8 {
        self.0[77] as i8
    }
    /// Nonzero makes the attack skip the hit roll and lets armor reduce its damage; a weapon and
    /// its ammunition add their words.
    pub fn hit_roll_filter(&self) -> Result<i16, ContentError> {
        word(self.0, 136)
    }
    /// Attacks the weapon makes, as the item information shows them.
    pub fn attack_count(&self) -> Result<i16, ContentError> {
        word(self.0, 76)
    }
    /// Whether the weapon's attack skips the ordinary hit roll.
    pub fn bypasses_hit_roll(&self) -> Result<bool, ContentError> {
        Ok(word(self.0, 136)? != 0)
    }
    /// Damage the item adds.
    pub fn damage_enhancement(&self) -> i8 {
        self.0[85] as i8
    }
    /// Damage of the weapon's melee attack.
    pub fn melee_damage(&self) -> Result<i16, ContentError> {
        word(self.0, 40)
    }
    /// Damage of element `element`, in the order of the spell info packet.
    pub fn elemental_damage(&self, element: usize) -> Result<i16, ContentError> {
        word(self.0, [140, 64, 66, 68, 70, 72, 74][element])
    }
    /// The item's class, an entry of the item class names.
    pub fn item_type(&self) -> Result<i16, ContentError> {
        word(self.0, 26)
    }
    /// The kind of reserve ammunition the item takes or is.
    pub fn reserve_ammunition_kind(&self) -> Result<i16, ContentError> {
        word(self.0, 34)
    }
    /// Weight of each charge the item holds.
    pub fn charge_weight(&self) -> Result<i16, ContentError> {
        word(self.0, 36)
    }
    /// Nonzero when the item is thrown away once its charges are used up.
    pub fn discarded_when_empty(&self) -> Result<i16, ContentError> {
        word(self.0, 38)
    }
    /// Which effect the item has when used.
    pub fn effect_selector(&self) -> Result<i16, ContentError> {
        word(self.0, 42)
    }
    /// The spell descriptor the item carries; a value over 1100 names a scan.
    pub fn spell(&self) -> Result<i16, ContentError> {
        word(self.0, 44)
    }
    /// Effect selector -10 on a melee weapon gives a condition. Mode word 44
    /// decides whether it lands: 1 on a failed saving throw indexed by word 48,
    /// 2 when Rand(100) is at most word 48, otherwise always. It then adds word 50
    /// to condition slot word 46 minus 20.
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
    /// Word 46 minus 19: the one-based STR#134 message shown when the weapon's
    /// condition lands.
    pub fn condition_feedback_index(&self) -> Result<i16, ContentError> {
        Ok(word(self.0, 46)?.wrapping_sub(19))
    }
    /// What the item costs; the shops ignore its sign.
    pub fn price(&self) -> Result<i32, ContentError> {
        long(self.0, 142)
    }
    /// Copies the definition's starting modifiers into the record of a generated item.
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
    /// Percent chance the item's shield takes a hit.
    pub fn shield_chance(&self) -> Result<i16, ContentError> {
        word(self.0, 88)
    }
    /// Armour rating the item adds.
    pub fn armor(&self) -> Result<i16, ContentError> {
        word(self.0, 94)
    }
    /// Missile dodge the item adds.
    pub fn missile_dodge_limit(&self) -> i8 {
        self.0[3] as i8
    }
    /// Melee dodge the item adds.
    pub fn melee_dodge_limit(&self) -> i8 {
        self.0[4] as i8
    }
    /// The item's range.
    pub fn range(&self) -> Result<i16, ContentError> {
        word(self.0, 60)
    }
    /// Range the item adds.
    pub fn base_range_modifier(&self) -> i8 {
        self.0[79] as i8
    }
    /// Required level a generated copy of the item starts with.
    pub fn generated_required_level(&self) -> i8 {
        self.0[139] as i8
    }
    /// A fresh instance of this item, with its capacity and starting modifiers.
    /// Shop display stock is copied differently.
    ///
    /// Original: installitem
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
            put_word(&mut record.0, offset, value);
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

    /// Installs a fresh instance, leaving the record's bytes 11 and 12 as they were.
    pub fn install_fresh_into(&self, record: &mut InventoryRecord) -> Result<(), ContentError> {
        let fresh = self.fresh_stock()?;
        record.0[..11].copy_from_slice(&fresh.0[..11]);
        record.0[13..].copy_from_slice(&fresh.0[13..]);
        record.0[10] = self.item_type()?.wrapping_abs() as u8;
        Ok(())
    }

    /// Copies shop display stock, deliberately leaving bytes 11 and 12 intact.
    ///
    /// Original: updateshop
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
            put_word(&mut record.0, offset, value);
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
    /// The definition's reserve ammunition kind, copied on install.
    pub fn reserve_kind_code(&self) -> u8 {
        self.0[8]
    }
    /// Bytes 11 and 12: the first is the melee accuracy modifier; the second's meaning is unknown.
    pub fn unidentified_bytes(&self) -> &[u8] {
        &self.0[11..13]
    }
    /// The level a warrior must have to use the item, as stored.
    pub fn required_level_code(&self) -> u8 {
        self.0[13]
    }
    /// The modifier bytes that follow the required level.
    pub fn modifier_bytes(&self) -> &[u8] {
        &self.0[14..]
    }
    /// The level a warrior must have to use the item.
    pub fn required_level(&self) -> i8 {
        self.0[13] as i8
    }
    /// Empties the slot.
    pub fn clear_id(&mut self) {
        self.0[..2].fill(0);
    }
    /// Accuracy this item adds.
    pub fn accuracy_modifier(&self) -> i8 {
        self.0[15] as i8
    }
    /// Damage this item adds.
    pub fn damage_enhancement(&self) -> i8 {
        self.0[18] as i8
    }
    /// Range this item adds, as stored.
    pub fn range_modifier(&self) -> u8 {
        self.0[20]
    }
    /// The modifier on row `row` of the item editor.
    pub fn editor_modifier(&self, row: usize) -> i8 {
        self.0[EDITOR_MODIFIERS[row]] as i8
    }
    /// Sets the modifier on row `row` of the item editor.
    pub fn set_editor_modifier(&mut self, row: usize, value: u8) {
        self.0[EDITOR_MODIFIERS[row]] = value;
    }
    /// Changes the modifier on row `row` of the item editor by `delta`, wrapping.
    pub fn adjust_editor_modifier(&mut self, row: usize, delta: i8) {
        let value = &mut self.0[EDITOR_MODIFIERS[row]];
        *value = value.wrapping_add(delta as u8);
    }
    /// Sets the item's id.
    pub fn set_id(&mut self, id: i16) {
        super::put_word(&mut self.0, 0, id);
    }
    /// Sets the item's class.
    pub fn set_category(&mut self, category: u8) {
        self.0[10] = category;
    }
    /// Sets the ammunition loaded in the item.
    pub fn set_ammunition(&mut self, id: i16) {
        super::put_word(&mut self.0, 4, id);
    }
    /// Sets whether the item is worn or wielded.
    pub fn set_equipped(&mut self, value: u8) {
        self.0[9] = value;
    }
    /// Sets the item's charge.
    pub fn set_charge(&mut self, quantity: i16) {
        super::put_word(&mut self.0, 2, quantity);
    }
    /// How much the item can hold.
    pub fn capacity(&self) -> Result<i16, ContentError> {
        word(&self.0, 6)
    }
    /// Sets how much the item can hold.
    pub fn set_capacity(&mut self, capacity: i16) {
        super::put_word(&mut self.0, 6, capacity);
    }
    /// The instance's charge: stack count or loaded rounds, by item class.
    pub fn charge(&self) -> Result<i16, ContentError> {
        word(&self.0, 2)
    }
    /// The ammunition loaded in the item.
    pub fn ammunition_id(&self) -> Result<i16, ContentError> {
        word(&self.0, 4)
    }
    /// Nonzero when the item is worn or wielded.
    pub fn equipped(&self) -> u8 {
        self.0[9]
    }
    /// The item's class, as stored.
    pub fn category_code(&self) -> u8 {
        self.0[10]
    }
}
