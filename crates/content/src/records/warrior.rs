use super::{UpgradeRecord, put_word};
use crate::{ContentError, INVENTORY_BYTES, Warrior, word};

const INVENTORY: usize = 0x132;

impl Warrior {
    pub fn record(&self) -> WarriorRecord<'_> {
        WarriorRecord(&self.bytes)
    }

    pub fn record_mut(&mut self) -> WarriorRecordMut<'_> {
        WarriorRecordMut(&mut self.bytes)
    }
}

/// Native warrior layout used by combatchoice (0x11c564..0x11c854),
/// firearm callers and moveicon. Predicates remain at the caller: active == 1
/// and active != 0 are intentionally different original rules.
pub struct WarriorRecord<'a>(&'a [u8]);

impl<'a> WarriorRecord<'a> {
    pub fn upgrade(&self, selector: u16) -> UpgradeRecord<'a> {
        UpgradeRecord::new(self.0, selector)
    }
    pub fn faction(&self) -> Result<i16, ContentError> {
        word(self.0, 0x10)
    }
    pub fn active_flag(&self) -> u8 {
        self.0[0x9df]
    }
    pub fn stamina(&self) -> Result<i16, ContentError> {
        word(self.0, 0x50)
    }
    pub fn maximum_stamina(&self) -> Result<i16, ContentError> {
        word(self.0, 0x52)
    }
    pub fn maximum_psi(&self) -> Result<i16, ContentError> {
        word(self.0, 0x5a)
    }
    pub fn psi_points(&self) -> Result<i16, ContentError> {
        word(self.0, 0x58)
    }
    pub fn shield_chance(&self) -> Result<i16, ContentError> {
        word(self.0, 0x24)
    }
    pub fn shield_points(&self) -> Result<i16, ContentError> {
        word(self.0, 0x26)
    }
    pub fn armor(&self) -> Result<i16, ContentError> {
        word(self.0, 0x2c)
    }
    pub fn actions_per_round(&self) -> Result<i16, ContentError> {
        word(self.0, 0x12)
    }
    pub fn tactical_icon(&self) -> Result<i16, ContentError> {
        word(self.0, 0x56)
    }
    pub fn condition_duration(&self, index: usize) -> Result<i16, ContentError> {
        word(self.0, 0x5e + index * 2)
    }
    pub fn first_inventory_work(&self) -> &[u8] {
        &self.0[0x132..0x148]
    }
    /// A caller-validated slot. Deliberately does not add an inventory-count
    /// admission check to native display/work-buffer callers.
    pub fn inventory_slot(&self, slot: usize) -> crate::InventoryRecord {
        let start = INVENTORY + slot * INVENTORY_BYTES;
        crate::InventoryRecord(self.0[start..start + INVENTORY_BYTES].try_into().unwrap())
    }
    pub fn firing_mode(&self) -> bool {
        self.0[0x9dd] != 0
    }
    pub fn action_points(&self) -> Result<i16, ContentError> {
        word(self.0, 0x40)
    }
    pub fn movement_points(&self) -> Result<i16, ContentError> {
        word(self.0, 0x3c)
    }
    pub fn equipped_weapon(&self) -> Result<i16, ContentError> {
        word(self.0, 0xe4)
    }
    pub fn melee_weapon(&self) -> Result<i16, ContentError> {
        word(self.0, 0xca)
    }
    pub fn attacks_used(&self) -> Result<i16, ContentError> {
        word(self.0, 0xacc)
    }
    pub fn attack_limit(&self) -> Result<i16, ContentError> {
        word(self.0, 0xaca)
    }
    pub fn melee_hit_sound(&self) -> Result<i16, ContentError> {
        word(self.0, 0x128)
    }
    pub fn turn_skip_duration(&self) -> Result<i16, ContentError> {
        word(self.0, 0x60)
    }
    pub fn reaction_suppression(&self) -> Result<i16, ContentError> {
        word(self.0, 0x8e)
    }
    pub fn defeated_marker(&self) -> Result<i16, ContentError> {
        word(self.0, 0x20)
    }
    pub fn target_body(&self) -> Result<i16, ContentError> {
        word(self.0, 0x18)
    }
    pub fn carried_weight(&self) -> Result<i16, ContentError> {
        word(self.0, 0x9d6)
    }
    pub fn weight_capacity(&self) -> Result<i16, ContentError> {
        word(self.0, 0x9d8)
    }
    pub fn race(&self) -> Result<i16, ContentError> {
        word(self.0, 0x32)
    }
    pub fn profession(&self) -> Result<i16, ContentError> {
        word(self.0, 0x34)
    }
    pub fn level(&self) -> Result<i16, ContentError> {
        word(self.0, 0x3a)
    }
    pub fn portrait(&self) -> Result<i16, ContentError> {
        word(self.0, 0x54)
    }
    pub fn automatic_mode(&self) -> Result<i16, ContentError> {
        word(self.0, 0x90)
    }
    /// Fleeing (condition 1 in attack2's duration table). Keep its signed duration;
    /// callers distinguish positive, negative (permanent), and zero values.
    pub fn fleeing_duration(&self) -> Result<i16, ContentError> {
        word(self.0, 0x5e)
    }
    pub fn confusion_duration(&self) -> Result<i16, ContentError> {
        word(self.0, 0x98)
    }
    pub fn bleeding_flag(&self) -> u8 {
        self.0[0x9de]
    }
    pub fn psi_school(&self) -> Result<i16, ContentError> {
        word(self.0, 0x36)
    }
    pub fn scan_capacity(&self) -> Result<i16, ContentError> {
        word(self.0, 0xac6)
    }
    pub fn upgrade_points(&self) -> i32 {
        i32::from_be_bytes(self.0[0x9d2..0x9d6].try_into().unwrap())
    }
    pub fn learned_scan(&self, tier: usize, column: usize) -> u8 {
        self.0[0x9f0 + tier * 7 + column]
    }
    pub fn special_skill(&self, index: usize) -> Result<i16, ContentError> {
        word(self.0, 0xf4 + index * 2)
    }
    pub fn special_skill_bonus(&self, index: usize) -> Result<i16, ContentError> {
        word(self.0, 0xb62 + index * 2)
    }
    pub fn upgrade_count(&self, selector: u16) -> Result<i16, ContentError> {
        self.upgrade(selector).purchases()
    }
    pub fn inventory_quantity(&self, slot: usize) -> Result<i16, ContentError> {
        word(self.0, INVENTORY + slot * INVENTORY_BYTES + 2)
    }
    pub fn inventory_ammunition(&self, slot: usize) -> Result<i16, ContentError> {
        word(self.0, INVENTORY + slot * INVENTORY_BYTES + 4)
    }
}

pub struct WarriorRecordMut<'a>(&'a mut [u8]);

impl WarriorRecordMut<'_> {
    pub fn set_stamina(&mut self, value: i16) {
        put_word(self.0, 0x50, value);
    }
    pub fn set_psi_points(&mut self, value: i16) {
        put_word(self.0, 0x58, value);
    }
    pub fn set_movement_points(&mut self, value: i16) {
        put_word(self.0, 0x3c, value);
    }
    pub fn set_turn_skip_duration(&mut self, value: i16) {
        put_word(self.0, 0x60, value);
    }
    pub fn set_action_points(&mut self, value: i16) {
        put_word(self.0, 0x40, value);
    }
    pub fn set_carried_weight(&mut self, value: i16) {
        put_word(self.0, 0x9d6, value);
    }
    pub fn set_inventory_quantity(&mut self, slot: usize, value: i16) {
        put_word(self.0, INVENTORY + slot * INVENTORY_BYTES + 2, value);
    }
    pub fn set_automatic_mode(&mut self, mode: i16) {
        put_word(self.0, 0x90, mode);
    }
    pub fn set_bleeding_flag(&mut self, flag: u8) {
        self.0[0x9de] = flag;
    }
    pub fn set_guarding(&mut self, value: i16) {
        put_word(self.0, 0x16, value);
    }
    pub fn set_target_body(&mut self, body: i16) {
        put_word(self.0, 0x18, body);
    }
    /// Finish clears guard before remaining movement, without clearing AP here.
    pub fn finish_turn_movement(&mut self) {
        self.0[0x16..0x18].fill(0);
        self.0[0x3c..0x3e].fill(0);
    }
    pub fn clear_learned_scans(&mut self) {
        self.0[0x9f0..0xa21].fill(0);
    }
    pub fn restore_first_inventory_work(&mut self, bytes: &[u8]) {
        self.0[0x132..0x148].copy_from_slice(bytes);
    }
    pub fn set_appearance(&mut self, portrait: bool, resource: i16) {
        put_word(self.0, if portrait { 0x54 } else { 0x56 }, resource);
    }
    pub fn set_upgrade_points(&mut self, points: i32) {
        self.0[0x9d2..0x9d6].copy_from_slice(&points.to_be_bytes());
    }
    pub fn set_upgrade_count(&mut self, selector: u16, count: i16) {
        put_word(self.0, 0xace + usize::from(selector) * 2, count);
    }
    pub fn face_horizontal_step(&mut self, dx: i8) {
        if dx < 0 {
            self.0[0x9e0] = 1;
        } else if dx > 0 {
            self.0[0x9e0] = 0;
        }
    }
    /// Shop installitem appends without compaction; the caller has already
    /// checked capacity. Preserve quantity updates and count writes separately.
    pub fn append_inventory(&mut self, count: usize, item: &crate::InventoryRecord) {
        let start = INVENTORY + count * INVENTORY_BYTES;
        self.0[start..start + INVENTORY_BYTES].copy_from_slice(&item.0);
        put_word(self.0, 0x1a, (count + 1) as i16);
    }
}
