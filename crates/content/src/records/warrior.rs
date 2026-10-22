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
    pub fn initiative_index(&self) -> Result<i16, ContentError> {
        word(self.0, 0x20)
    }
    pub fn healing_available(&self) -> bool {
        self.0[0x9dc] != 0
    }

    pub fn from_bytes(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }
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
    pub fn base_psionic_resistance(&self) -> Result<i16, ContentError> {
        word(self.0, 0x22)
    }
    pub fn level_protection(&self, level: usize) -> Result<i16, ContentError> {
        self.condition_duration(level + 16)
    }
    pub fn shield_chance(&self) -> Result<i16, ContentError> {
        word(self.0, 0x24)
    }
    pub fn shield_points(&self) -> Result<i16, ContentError> {
        word(self.0, 0x26)
    }
    /// The `ShieldCapacity` equipment statistic.
    pub fn shield_capacity(&self) -> Result<i16, ContentError> {
        word(self.0, 0x28)
    }
    /// Item ID of the worn battery, cleared by `clear_equipped_battery`.
    pub fn equipped_battery(&self) -> Result<i16, ContentError> {
        word(self.0, 0xee)
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
    /// A condition slot by raw index, for story data that names the slot.
    pub fn condition_slot(&self, slot: usize) -> Result<i16, ContentError> {
        word(self.0, 0x5e + slot * 2)
    }
    pub fn condition(&self, condition: super::Condition) -> Result<i16, ContentError> {
        self.condition_duration(condition as usize)
    }
    pub fn melee_accuracy(&self) -> Result<i16, ContentError> {
        Ok(word(self.0, 6)?.wrapping_add(word(self.0, 0xb5a)?))
    }
    pub fn melee_dodge(&self) -> Result<i16, ContentError> {
        Ok(word(self.0, 0xa)?.wrapping_add(word(self.0, 0xb5e)?))
    }
    /// attack2 does not include the temporary upgrade bonus used by player attacks.
    pub fn base_melee_dodge(&self) -> Result<i16, ContentError> {
        word(self.0, 0xa)
    }
    pub fn bare_hand_damage(&self) -> Result<i16, ContentError> {
        word(self.0, 12)
    }
    pub fn major_wound_chance(&self) -> Result<i16, ContentError> {
        Ok(word(self.0, 0xfa)?.wrapping_add(word(self.0, 0xb68)?))
    }
    pub fn creature_category_bonus(&self, category: usize) -> Result<i16, ContentError> {
        word(self.0, 0xae + category * 2)
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
    pub fn firing_mode_flag(&self) -> u8 {
        self.0[0x9dd]
    }
    pub fn missile_accuracy(&self) -> Result<i16, ContentError> {
        Ok(word(self.0, 4)?.wrapping_add(word(self.0, 0xb58)?))
    }
    pub fn missile_dodge(&self) -> Result<i16, ContentError> {
        Ok(word(self.0, 8)?.wrapping_add(word(self.0, 0xb5c)?))
    }
    pub fn quickness(&self) -> Result<i16, ContentError> {
        word(self.0, 0x9e8)
    }
    pub fn guarding(&self) -> Result<i16, ContentError> {
        word(self.0, 0x16)
    }
    pub fn action_points(&self) -> Result<i16, ContentError> {
        word(self.0, 0x40)
    }
    pub fn movement_points(&self) -> Result<i16, ContentError> {
        word(self.0, 0x3c)
    }
    pub fn movement_per_round(&self) -> Result<i16, ContentError> {
        word(self.0, 0x3e)
    }
    pub fn movement_bonus(&self) -> Result<i16, ContentError> {
        word(self.0, 0x30)
    }
    pub fn action_point_bonus(&self) -> Result<i16, ContentError> {
        word(self.0, 0x1e)
    }
    pub fn brawn(&self) -> Result<i16, ContentError> {
        word(self.0, 0x9e2)
    }
    pub fn brawn_upgrades(&self) -> Result<i16, ContentError> {
        word(self.0, 0xace)
    }
    /// calcdamage reads the displayed equipment separately from the worn slots.
    pub fn displayed_weapon(&self, firing: bool) -> Result<i16, ContentError> {
        word(self.0, if firing { 0x12c } else { 0x12e })
    }
    pub fn loaded_ammunition(&self) -> Result<i16, ContentError> {
        word(self.0, 0x126)
    }
    pub fn bare_hand_bonus(&self) -> Result<i16, ContentError> {
        word(self.0, 0xb7a)
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
    pub fn damage_marker(&self) -> Result<i16, ContentError> {
        word(self.0, 0x14)
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
    pub fn carried_credits(&self) -> Result<i16, ContentError> {
        word(self.0, 0x9da)
    }
    pub fn weight_capacity(&self) -> Result<i16, ContentError> {
        word(self.0, 0x9d8)
    }
    pub fn race(&self) -> Result<i16, ContentError> {
        word(self.0, 0x32)
    }
    pub fn saving_throw(&self, save: super::SavingThrow) -> Result<i16, ContentError> {
        word(self.0, 0x112 + save as usize * 2)
    }
    pub fn profession(&self) -> Result<i16, ContentError> {
        word(self.0, 0x34)
    }
    pub fn gender(&self) -> Result<i16, ContentError> {
        word(self.0, 0x38)
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
    pub fn scans_used(&self) -> Result<i16, ContentError> {
        word(self.0, 0xac8)
    }
    pub fn upgrade_points(&self) -> i32 {
        i32::from_be_bytes(self.0[0x9d2..0x9d6].try_into().unwrap())
    }
    pub fn remaining_experience(&self) -> Result<i32, ContentError> {
        crate::mission::long(self.0, 0x9ce)
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
    pub fn set_initiative_index(&mut self, index: i16) {
        put_word(self.0, 0x20, index);
    }

    /// Placement initializes this byte independently of active, firing and bleed
    /// state. Its later meaning remains unconfirmed; do not infer a flag role.
    pub fn initialize_placement_marker(&mut self) {
        self.0[0x9e0] = 1;
    }

    pub fn set_damage_marker(&mut self, value: i16) {
        super::put_word(self.0, 0x14, value);
    }
    pub fn set_active_flag(&mut self, value: u8) {
        self.0[0x9df] = value;
    }
    pub fn set_defeated_marker(&mut self, value: i16) {
        put_word(self.0, 0x20, value);
    }
    pub fn clear_temporary_conditions(&mut self) -> Result<(), ContentError> {
        for offset in (0x5e..0xae).step_by(2) {
            if word(self.0, offset)? > 0 {
                put_word(self.0, offset, 0);
            }
        }
        Ok(())
    }
    pub fn set_special_skill(&mut self, index: usize, value: i16) {
        put_word(self.0, 0xf4 + index * 2, value);
    }
    pub fn add_remaining_experience(&mut self, value: i32) {
        self.increment_statistic(0x9ce, value);
    }
    pub fn clear_damage_marker(&mut self) {
        put_word(self.0, 0x14, 0);
    }
    pub fn set_faction(&mut self, faction: i16) {
        put_word(self.0, 0x10, faction);
    }
    pub fn set_condition(&mut self, condition: super::Condition, duration: i16) {
        self.set_condition_duration(condition as usize, duration);
    }
    pub fn set_condition_duration(&mut self, index: usize, duration: i16) {
        put_word(self.0, 0x5e + index * 2, duration);
    }
    pub fn count_scan_cast(&mut self) {
        self.increment_statistic(0xa66, 1);
    }
    /// reduce visits all forty durations, including unnamed slots; permanent
    /// negative durations do not expire. Enemy getup visits signed bytes instead.
    pub fn decrement_temporary_conditions(&mut self) -> Result<(), ContentError> {
        for offset in (0x5e..0xae).step_by(2) {
            let duration = word(self.0, offset)?;
            if duration > 0 {
                put_word(self.0, offset, duration - 1);
            }
        }
        Ok(())
    }
    pub fn record_bleeding_death(&mut self) {
        self.increment_statistic(0xa62, -1);
        self.increment_statistic(0xa5e, 1);
    }
    pub fn from_bytes(bytes: &mut [u8]) -> WarriorRecordMut<'_> {
        WarriorRecordMut(bytes)
    }
    pub fn set_attacks_used(&mut self, count: i16) {
        put_word(self.0, 0xacc, count);
    }
    pub fn count_miss(&mut self) {
        self.increment_statistic(0xa52, 1);
    }
    pub fn add_damage_received(&mut self, amount: i16) {
        self.increment_statistic(0xa42, i32::from(amount));
    }
    pub fn mark_damaged(&mut self) {
        put_word(self.0, 0x14, 1);
    }
    pub fn add_damage_dealt(&mut self, amount: i16) {
        self.increment_statistic(0xa46, i32::from(amount));
    }
    pub fn count_hit(&mut self) {
        self.increment_statistic(0xa4a, 1);
    }
    pub fn count_kill(&mut self) {
        self.increment_statistic(0xa5a, 1);
    }
    pub fn count_avoided_attack(&mut self) {
        self.increment_statistic(0xa56, 1);
    }
    pub fn count_hit_received(&mut self) {
        self.increment_statistic(0xa4e, 1);
    }
    fn increment_statistic(&mut self, offset: usize, amount: i32) {
        let previous = i32::from_be_bytes(self.0[offset..offset + 4].try_into().unwrap());
        self.0[offset..offset + 4].copy_from_slice(&previous.wrapping_add(amount).to_be_bytes());
    }
    pub fn set_shield_points(&mut self, value: i16) {
        put_word(self.0, 0x26, value);
    }
    pub fn clear_equipped_battery(&mut self) {
        put_word(self.0, 0xee, 0);
    }
    pub fn set_firing_mode(&mut self, firing: bool) {
        self.0[0x9dd] = u8::from(firing);
    }
    pub fn set_loaded_ammunition(&mut self, id: i16) {
        put_word(self.0, 0x126, id);
    }
    pub fn set_inventory_ammunition(&mut self, slot: usize, id: i16) {
        put_word(self.0, INVENTORY + slot * INVENTORY_BYTES + 4, id);
    }
    pub fn set_inventory_equipment_flag(&mut self, slot: usize, flag: u8) {
        self.0[INVENTORY + slot * INVENTORY_BYTES + 9] = flag;
    }
    pub fn set_inventory_slot(&mut self, slot: usize, item: &crate::InventoryRecord) {
        let start = INVENTORY + slot * INVENTORY_BYTES;
        self.0[start..start + INVENTORY_BYTES].copy_from_slice(&item.0);
    }
    pub fn set_stamina(&mut self, value: i16) {
        put_word(self.0, 0x50, value);
    }
    pub fn set_psi_points(&mut self, value: i16) {
        put_word(self.0, 0x58, value);
    }
    pub fn set_maximum_stamina(&mut self, value: i16) {
        put_word(self.0, 0x52, value);
    }
    pub fn set_maximum_psi(&mut self, value: i16) {
        put_word(self.0, 0x5a, value);
    }
    pub fn set_movement_points(&mut self, value: i16) {
        put_word(self.0, 0x3c, value);
    }
    pub fn set_movement_per_round(&mut self, value: i16) {
        put_word(self.0, 0x3e, value);
    }
    pub fn set_weight_capacity(&mut self, value: i16) {
        put_word(self.0, 0x9d8, value);
    }
    pub fn set_scans_used(&mut self, value: i16) {
        put_word(self.0, 0xac8, value);
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
    pub fn set_carried_credits(&mut self, value: i16) {
        put_word(self.0, 0x9da, value);
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
    pub fn set_remaining_experience(&mut self, points: i32) {
        self.0[0x9ce..0x9d2].copy_from_slice(&points.to_be_bytes());
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
