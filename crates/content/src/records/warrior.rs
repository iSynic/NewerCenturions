use super::warrior_layout::{
    BLEEDING, COMPUTER_CONTROL, CONDITIONS, CONDITIONS_END, DEFINE_SPELLS, ITEM_COUNT, ITEMS,
    KNOWN_SCANS, LEARNED_SCAN_BYTES, NAME, PSI_MEMORY_END, PSI_SCHOOL, REMAINING_VICTORY_POINTS,
    STAMINA, STAMINA_MAX, TOGGLE, UPGRADE_POINTS,
};
use super::{UpgradeRecord, put_word};
use crate::bytes::{add_long_wrapping, long, long_at, put_long, word_at};
use crate::{ContentError, INVENTORY_BYTES, Warrior, span, word};

impl Warrior {
    /// The warrior's record as a read-only view.
    pub fn record(&self) -> WarriorRecord<'_> {
        WarriorRecord(&self.bytes)
    }

    /// The warrior's record as a view that can change it.
    pub fn record_mut(&mut self) -> WarriorRecordMut<'_> {
        WarriorRecordMut(&mut self.bytes)
    }
}

/// A warrior's record (`c[n]`), as saved in the warrior file and used in
/// battle. Callers make their own tests: `active == 1` and `active != 0` are
/// deliberately different rules.
pub struct WarriorRecord<'a>(&'a [u8]);

impl<'a> WarriorRecord<'a> {
    /// The warrior's slot in the battle order; -1 once defeated.
    pub fn position(&self) -> Result<i16, ContentError> {
        word(self.0, 0x20)
    }
    /// Whether the warrior spends Psi points to heal the squad while it rests.
    pub fn can_heal(&self) -> bool {
        self.0[0x9dc] != 0
    }

    /// Views a warrior record held in `bytes`.
    pub fn from_bytes(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }
    /// The warrior's record of the upgrade `selector` names.
    pub fn upgrade(&self, selector: u16) -> UpgradeRecord<'a> {
        UpgradeRecord::new(self.0, selector)
    }
    /// Which side the warrior fights on: zero is the party's, nonzero the monsters'.
    /// Confusion flips it.
    pub fn faction(&self) -> Result<i16, ContentError> {
        word(self.0, 0x10)
    }
    /// Nonzero while the warrior takes part in the battle; a defeated warrior's is
    /// cleared.
    pub fn in_battle(&self) -> u8 {
        self.0[0x9df]
    }
    /// Current stamina; at zero or below the warrior is out of the fight, and below -9
    /// it is killed.
    pub fn stamina(&self) -> Result<i16, ContentError> {
        word(self.0, STAMINA)
    }
    /// Maximum stamina.
    pub fn stamina_max(&self) -> Result<i16, ContentError> {
        word(self.0, STAMINA_MAX)
    }
    /// Maximum Psi points.
    pub fn psi_points_max(&self) -> Result<i16, ContentError> {
        word(self.0, 0x5a)
    }
    /// Current Psi points.
    pub fn psi_points(&self) -> Result<i16, ContentError> {
        word(self.0, 0x58)
    }
    /// The warrior's own resistance to scans, before the bonus the warrior's attributes give;
    /// a scan's power is compared with it.
    pub fn base_psionic_resistance(&self) -> Result<i16, ContentError> {
        word(self.0, 0x22)
    }
    /// Remaining duration of the protection against scans of `level`; a negative
    /// duration is permanent.
    pub fn level_protection(&self, level: usize) -> Result<i16, ContentError> {
        self.condition_duration(level + 16)
    }
    /// `level_protection` for a signed level, as resistance checks index it.
    pub fn signed_level_protection(&self, level: i16) -> Result<i16, ContentError> {
        let offset = usize::try_from(0x7e + i32::from(level) * 2).unwrap_or(usize::MAX);
        word(self.0, offset)
    }
    /// Percent chance that the warrior's power shield takes a hit.
    pub fn shield_chance(&self) -> Result<i16, ContentError> {
        word(self.0, 0x24)
    }
    /// Charge left in the power shield.
    pub fn shield_points(&self) -> Result<i16, ContentError> {
        word(self.0, 0x26)
    }
    /// Most charge the power shield holds; the worn equipment sets it.
    pub fn shield_capacity(&self) -> Result<i16, ContentError> {
        word(self.0, 0x28)
    }
    /// Item id of the worn battery, cleared by `clear_equipped_battery`.
    pub fn equipped_battery(&self) -> Result<i16, ContentError> {
        word(self.0, 0xee)
    }
    /// The warrior's armor rating.
    pub fn armor_rating(&self) -> Result<i16, ContentError> {
        word(self.0, 0x2c)
    }
    /// Actions per round the warrior has before bonuses.
    pub fn actions_per_round(&self) -> Result<i16, ContentError> {
        word(self.0, 0x12)
    }
    /// The icon that shows the warrior on the battle map.
    pub fn icon(&self) -> Result<i16, ContentError> {
        word(self.0, 0x56)
    }
    /// Set by a step to the left and cleared by a step to the right.
    pub fn horizontal_facing(&self) -> u8 {
        self.0.get(0x9e0).copied().unwrap_or(0)
    }
    /// Remaining duration of condition `index`; a negative duration is permanent.
    pub fn condition_duration(&self, index: usize) -> Result<i16, ContentError> {
        word(self.0, CONDITIONS + index * 2)
    }
    /// A condition slot by index, for story data that names the slot.
    pub fn condition_slot(&self, slot: usize) -> Result<i16, ContentError> {
        word(self.0, CONDITIONS + slot * 2)
    }
    /// Remaining duration of `condition`.
    pub fn condition(&self, condition: super::Condition) -> Result<i16, ContentError> {
        self.condition_duration(condition as usize)
    }
    /// The condition cicns drawn on the portrait, in drawing order. `automatic`
    /// is the member's `doauto` flag.
    ///
    /// Original: plotportrait
    pub fn portrait_condition_icons(&self, automatic: bool) -> Result<Vec<i16>, ContentError> {
        let any = |slots: &[usize]| -> Result<bool, ContentError> {
            for slot in slots {
                if self.condition_slot(*slot)? != 0 {
                    return Ok(true);
                }
            }
            Ok(false)
        };
        let controlled = self.condition_slot(25)? < 0 || automatic || self.faction()? != 0;
        Ok([
            (any(&[0, 2, 6])?, 0xb4),
            (any(&[1, 26, 27])?, 0xb7),
            (any(&[3, 28, 2])?, 0xb5),
            (controlled, 0xb6),
            (any(&[36, 37])?, 0xb9),
            (any(&[5, 29, 39])?, 0xb8),
        ]
        .into_iter()
        .filter(|(shown, _)| *shown)
        .map(|(_, icon)| icon)
        .collect())
    }
    /// Accuracy of melee attacks: the base word plus the bonus the warrior's attributes give.
    pub fn melee_accuracy(&self) -> Result<i16, ContentError> {
        Ok(word(self.0, 6)?.wrapping_add(word(self.0, 0xb5a)?))
    }
    /// Chance to avoid melee attacks: the base word plus the bonus the warrior's attributes give.
    pub fn melee_dodge(&self) -> Result<i16, ContentError> {
        Ok(word(self.0, 0xa)?.wrapping_add(word(self.0, 0xb5e)?))
    }
    /// Chance to avoid melee attacks without the attribute bonus; a monster's attack against the
    /// warrior subtracts only this.
    pub fn base_melee_dodge(&self) -> Result<i16, ContentError> {
        word(self.0, 0xa)
    }
    /// Damage of an attack without a weapon, before the bare-hand bonus.
    pub fn bare_hand_damage(&self) -> Result<i16, ContentError> {
        word(self.0, 12)
    }
    /// Percent chance that a hit is a major wound: the special skill plus the bonus the warrior's
    /// attributes give it.
    pub fn major_wound_chance(&self) -> Result<i16, ContentError> {
        Ok(word(self.0, 0xfa)?.wrapping_add(word(self.0, 0xb68)?))
    }
    /// The warrior's bonus against one category of creature.
    pub fn special(&self, category: usize) -> Result<i16, ContentError> {
        word(self.0, 0xae + category * 2)
    }
    /// The first inventory record, as stored.
    pub fn inventory_bytes(&self) -> &[u8] {
        &self.0[ITEMS..ITEMS + INVENTORY_BYTES]
    }
    /// A slot the caller has already checked; deliberately not limited to the
    /// inventory count.
    pub fn inventory_slot(&self, slot: usize) -> crate::InventoryRecord {
        let start = ITEMS + slot * INVENTORY_BYTES;
        crate::InventoryRecord(self.0[start..start + INVENTORY_BYTES].try_into().unwrap())
    }
    /// Whether the firing toggle is set: the warrior fights with the missile
    /// weapon instead of the melee weapon.
    pub fn toggle(&self) -> bool {
        self.0[TOGGLE] != 0
    }
    /// The firing toggle byte as stored.
    pub fn toggle_flag(&self) -> u8 {
        self.0[TOGGLE]
    }
    /// Accuracy of missile attacks: the base word plus the bonus the warrior's attributes give.
    pub fn missile_accuracy(&self) -> Result<i16, ContentError> {
        Ok(word(self.0, 4)?.wrapping_add(word(self.0, 0xb58)?))
    }
    /// Chance to avoid missile attacks: the base word plus the bonus the warrior's attributes give.
    pub fn missile_dodge(&self) -> Result<i16, ContentError> {
        Ok(word(self.0, 8)?.wrapping_add(word(self.0, 0xb5c)?))
    }
    /// Agility, which orders the warrior in the battle.
    pub fn quickness(&self) -> Result<i16, ContentError> {
        word(self.0, 0x9e8)
    }
    /// Nonzero while the warrior is guarding; it is cleared when the turn ends.
    pub fn guarding(&self) -> Result<i16, ContentError> {
        word(self.0, 0x16)
    }
    /// Action points left this round.
    pub fn action_points(&self) -> Result<i16, ContentError> {
        word(self.0, 0x40)
    }
    /// Movement left this round.
    pub fn movement(&self) -> Result<i16, ContentError> {
        word(self.0, 0x3c)
    }
    /// Movement the warrior gets each round.
    pub fn movement_max(&self) -> Result<i16, ContentError> {
        word(self.0, 0x3e)
    }
    /// Movement added to what the warrior's load and race allow.
    pub fn movement_bonus(&self) -> Result<i16, ContentError> {
        word(self.0, 0x30)
    }
    /// Action points added to the actions per round each round.
    pub fn action_point_bonus(&self) -> Result<i16, ContentError> {
        word(self.0, 0x1e)
    }
    /// Brawn, the strength attribute.
    pub fn brawn(&self) -> Result<i16, ContentError> {
        word(self.0, 0x9e2)
    }
    /// Brawn upgrades bought; each adds 10 to the weight the warrior can carry.
    pub fn brawn_upgrades(&self) -> Result<i16, ContentError> {
        word(self.0, 0xace)
    }
    /// The weapon shown with the warrior's damage, kept apart from the worn slots.
    pub fn displayed_weapon(&self, firing: bool) -> Result<i16, ContentError> {
        word(self.0, if firing { 0x12c } else { 0x12e })
    }
    /// Item id of the ammunition loaded in the missile weapon; zero is none.
    pub fn loaded_ammunition(&self) -> Result<i16, ContentError> {
        word(self.0, 0x126)
    }
    /// Damage added to a bare-hand attack.
    pub fn bare_hand_bonus(&self) -> Result<i16, ContentError> {
        word(self.0, 0xb7a)
    }
    /// Item id of the missile weapon in hand.
    pub fn equipped_weapon(&self) -> Result<i16, ContentError> {
        word(self.0, 0xe4)
    }
    /// Item id of the melee weapon in hand.
    pub fn melee_weapon(&self) -> Result<i16, ContentError> {
        word(self.0, 0xca)
    }
    /// Attacks the warrior has used this combat.
    pub fn attacks_used(&self) -> Result<i16, ContentError> {
        word(self.0, 0xacc)
    }
    /// Most attacks the warrior may use in a combat.
    pub fn attack_limit(&self) -> Result<i16, ContentError> {
        word(self.0, 0xaca)
    }
    /// The sound the warrior's attack makes.
    pub fn weapon_sound(&self) -> Result<i16, ContentError> {
        word(self.0, 0x128)
    }
    /// Rounds the warrior stays helpless and skips its turns; while it lasts the
    /// warrior cannot dodge.
    pub fn turn_skip_duration(&self) -> Result<i16, ContentError> {
        word(self.0, 0x60)
    }
    /// Nonzero while the warrior's movement draws no reaction attacks from its
    /// neighbours.
    pub fn reaction_suppression(&self) -> Result<i16, ContentError> {
        word(self.0, 0x8e)
    }
    /// Nonzero once the warrior has been attacked this round.
    pub fn been_attacked(&self) -> Result<i16, ContentError> {
        word(self.0, 0x14)
    }
    /// The body the warrior is aiming at; -1 is none.
    pub fn target(&self) -> Result<i16, ContentError> {
        word(self.0, 0x18)
    }
    /// Weight the warrior carries.
    pub fn load(&self) -> Result<i16, ContentError> {
        word(self.0, 0x9d6)
    }
    /// Credits the warrior carries.
    pub fn carried_credits(&self) -> Result<i16, ContentError> {
        word(self.0, 0x9da)
    }
    /// The most weight the warrior can carry.
    pub fn load_max(&self) -> Result<i16, ContentError> {
        word(self.0, 0x9d8)
    }
    /// The warrior's race, counted from one.
    pub fn race(&self) -> Result<i16, ContentError> {
        word(self.0, 0x32)
    }
    /// The warrior's saving throw against one kind of attack.
    pub fn save(&self, save: super::SavingThrow) -> Result<i16, ContentError> {
        word(self.0, 0x112 + save as usize * 2)
    }
    /// The warrior's MOS (`caste`).
    pub fn mos(&self) -> Result<i16, ContentError> {
        word(self.0, 0x34)
    }
    /// The warrior's gender.
    pub fn gender(&self) -> Result<i16, ContentError> {
        word(self.0, 0x38)
    }
    /// The warrior's level.
    pub fn level(&self) -> Result<i16, ContentError> {
        word(self.0, 0x3a)
    }
    /// The portrait shown for the warrior on the character screens.
    pub fn portrait(&self) -> Result<i16, ContentError> {
        word(self.0, 0x54)
    }
    /// Duration of condition 25, Computer Control, under which the warrior fights on
    /// its own; negative is permanent.
    pub fn computer_control(&self) -> Result<i16, ContentError> {
        word(self.0, COMPUTER_CONTROL)
    }
    /// Remaining duration of the Retreating condition, the first condition slot. Callers
    /// distinguish positive, negative (permanent) and zero values.
    pub fn fleeing_duration(&self) -> Result<i16, ContentError> {
        word(self.0, CONDITIONS)
    }
    /// Remaining duration of the Confused condition.
    pub fn confusion_duration(&self) -> Result<i16, ContentError> {
        word(self.0, 0x98)
    }
    /// Nonzero while the warrior is bleeding from a knockout; it loses stamina each
    /// round until it recovers or dies.
    pub fn bleeding(&self) -> u8 {
        self.0[BLEEDING]
    }
    /// The warrior's Psi school; zero is none.
    pub fn psi_school(&self) -> Result<i16, ContentError> {
        word(self.0, PSI_SCHOOL)
    }
    /// The 31-byte name field, NUL-terminated Mac Roman.
    pub fn name_field(&self) -> Result<&'a [u8], ContentError> {
        span(self.0, NAME, 31)
    }
    /// The known scans, row * 7 + column.
    pub fn known_scans(&self) -> Result<&'a [u8], ContentError> {
        span(self.0, KNOWN_SCANS, LEARNED_SCAN_BYTES)
    }
    /// The ten scan shortcuts copied into the party Psi selection memory.
    pub fn psi_selection_memory(&self) -> Result<&'a [u8], ContentError> {
        span(self.0, DEFINE_SPELLS, PSI_MEMORY_END - DEFINE_SPELLS)
    }
    /// How many items the warrior carries.
    pub fn item_count(&self) -> Result<i16, ContentError> {
        word(self.0, ITEM_COUNT)
    }
    /// The most scans the warrior casts in one combat round.
    pub fn max_spells_attacks(&self) -> Result<i16, ContentError> {
        word(self.0, 0xac6)
    }
    /// Scans cast so far this round.
    pub fn spells_so_far(&self) -> Result<i16, ContentError> {
        word(self.0, 0xac8)
    }
    /// Victory points the warrior has earned and can spend on upgrades.
    pub fn upgrade_points(&self) -> i32 {
        long_at(self.0, UPGRADE_POINTS)
    }
    /// Victory points the warrior must still earn to reach the next level.
    pub fn remaining_victory_points(&self) -> Result<i32, ContentError> {
        long(self.0, REMAINING_VICTORY_POINTS)
    }
    /// Whether the warrior knows the scan at this tier and column.
    pub fn known_scan(&self, tier: usize, column: usize) -> u8 {
        self.0[KNOWN_SCANS + tier * 7 + column]
    }
    /// The warrior's rating in one special skill.
    pub fn spec(&self, index: usize) -> Result<i16, ContentError> {
        word(self.0, 0xf4 + index * 2)
    }
    /// The bonus the warrior's attributes give special skill `index`.
    pub fn special_skill_bonus(&self, index: usize) -> Result<i16, ContentError> {
        word(self.0, 0xb62 + index * 2)
    }
    /// How many times the upgrade `selector` names has been bought.
    pub fn upgrade_count(&self, selector: u16) -> Result<i16, ContentError> {
        self.upgrade(selector).purchases()
    }
    /// The charge of the item in `slot`.
    pub fn inventory_quantity(&self, slot: usize) -> Result<i16, ContentError> {
        word(self.0, ITEMS + slot * INVENTORY_BYTES + 2)
    }
    /// The ammunition item loaded in the item in `slot`.
    pub fn inventory_ammunition(&self, slot: usize) -> Result<i16, ContentError> {
        word(self.0, ITEMS + slot * INVENTORY_BYTES + 4)
    }
}

/// A warrior's record as a view that can change it.
pub struct WarriorRecordMut<'a>(&'a mut [u8]);

impl WarriorRecordMut<'_> {
    /// Puts the warrior in battle-order slot `index`.
    pub fn set_position(&mut self, index: i16) {
        put_word(self.0, 0x20, index);
    }

    /// Placing the warrior on the battle map sets the horizontal facing byte to 1 (left),
    /// apart from the in-battle, firing and bleeding state.
    pub fn initialize_placement_marker(&mut self) {
        self.0[0x9e0] = 1;
    }

    /// Sets the attacked-this-round flag.
    pub fn set_been_attacked(&mut self, value: i16) {
        super::put_word(self.0, 0x14, value);
    }
    /// Sets whether the warrior takes part in the battle.
    pub fn set_in_battle(&mut self, value: u8) {
        self.0[0x9df] = value;
    }
    /// Ends every condition that has a running duration; permanent ones stay.
    ///
    /// Original: killbody
    pub fn clear_temporary_conditions(&mut self) -> Result<(), ContentError> {
        for offset in (CONDITIONS..CONDITIONS_END).step_by(2) {
            if word(self.0, offset)? > 0 {
                put_word(self.0, offset, 0);
            }
        }
        Ok(())
    }
    /// Sets the warrior's rating in one special skill.
    pub fn set_spec(&mut self, index: usize, value: i16) {
        put_word(self.0, 0xf4 + index * 2, value);
    }
    /// Changes the victory points still to earn for the next level by `value`.
    pub fn add_remaining_victory_points(&mut self, value: i32) {
        self.increment_statistic(REMAINING_VICTORY_POINTS, value);
    }
    /// Clears the attacked-this-round flag.
    pub fn clear_been_attacked(&mut self) {
        put_word(self.0, 0x14, 0);
    }
    /// Moves the warrior to a side: zero is the party's, nonzero the monsters'.
    pub fn set_faction(&mut self, faction: i16) {
        put_word(self.0, 0x10, faction);
    }
    /// Sets how long `condition` lasts.
    pub fn set_condition(&mut self, condition: super::Condition, duration: i16) {
        self.set_condition_duration(condition as usize, duration);
    }
    /// Sets how long condition `index` lasts.
    pub fn set_condition_duration(&mut self, index: usize, duration: i16) {
        put_word(self.0, CONDITIONS + index * 2, duration);
    }
    /// Counts one more scan cast in the warrior's statistics.
    pub fn count_scan_cast(&mut self) {
        self.increment_statistic(0xa66, 1);
    }
    /// Counts down all forty durations, including unnamed slots; a negative,
    /// permanent duration never expires. Monsters count down signed bytes
    /// instead.
    ///
    /// Original: reduce
    pub fn decrement_temporary_conditions(&mut self) -> Result<(), ContentError> {
        for offset in (CONDITIONS..CONDITIONS_END).step_by(2) {
            let duration = word(self.0, offset)?;
            if duration > 0 {
                put_word(self.0, offset, duration - 1);
            }
        }
        Ok(())
    }
    /// Turns a knockout in the statistics into a death.
    pub fn record_bleeding_death(&mut self) {
        self.increment_statistic(0xa62, -1);
        self.increment_statistic(0xa5e, 1);
    }
    /// Views a warrior record held in `bytes` for changing.
    pub fn from_bytes(bytes: &mut [u8]) -> WarriorRecordMut<'_> {
        WarriorRecordMut(bytes)
    }
    /// Sets the attacks used this combat.
    pub fn set_attacks_used(&mut self, count: i16) {
        put_word(self.0, 0xacc, count);
    }
    /// Counts an attack by the warrior that missed.
    pub fn count_miss(&mut self) {
        self.increment_statistic(0xa52, 1);
    }
    /// Adds `amount` to the damage the warrior has taken.
    pub fn add_damage_received(&mut self, amount: i16) {
        self.increment_statistic(0xa42, i32::from(amount));
    }
    /// Marks the warrior as attacked this round.
    pub fn mark_been_attacked(&mut self) {
        put_word(self.0, 0x14, 1);
    }
    /// Adds `amount` to the damage the warrior has dealt.
    pub fn add_damage_dealt(&mut self, amount: i16) {
        self.increment_statistic(0xa46, i32::from(amount));
    }
    /// Counts an attack by the warrior that hit.
    pub fn count_hit(&mut self) {
        self.increment_statistic(0xa4a, 1);
    }
    /// Counts a kill by the warrior.
    pub fn count_kill(&mut self) {
        self.increment_statistic(0xa5a, 1);
    }
    /// Counts an attack on the warrior that missed.
    pub fn count_avoided_attack(&mut self) {
        self.increment_statistic(0xa56, 1);
    }
    /// Counts an attack on the warrior that hit.
    pub fn count_hit_received(&mut self) {
        self.increment_statistic(0xa4e, 1);
    }
    /// Adds `amount` to the statistic at `offset`, wrapping.
    fn increment_statistic(&mut self, offset: usize, amount: i32) {
        add_long_wrapping(self.0, offset, amount);
    }
    /// Sets the charge left in the power shield.
    pub fn set_shield_points(&mut self, value: i16) {
        put_word(self.0, 0x26, value);
    }
    /// Removes the worn battery.
    pub fn clear_equipped_battery(&mut self) {
        put_word(self.0, 0xee, 0);
    }
    /// Sets or clears the firing toggle.
    pub fn set_toggle(&mut self, firing: bool) {
        self.0[TOGGLE] = u8::from(firing);
    }
    /// Sets the ammunition loaded in the missile weapon; zero unloads it.
    pub fn set_loaded_ammunition(&mut self, id: i16) {
        put_word(self.0, 0x126, id);
    }
    /// Sets the ammunition loaded in the item in `slot`.
    pub fn set_inventory_ammunition(&mut self, slot: usize, id: i16) {
        put_word(self.0, ITEMS + slot * INVENTORY_BYTES + 4, id);
    }
    /// Sets whether the item in `slot` is worn or wielded.
    pub fn set_inventory_equipment_flag(&mut self, slot: usize, flag: u8) {
        self.0[ITEMS + slot * INVENTORY_BYTES + 9] = flag;
    }
    /// Replaces the item in `slot`.
    pub fn set_inventory_slot(&mut self, slot: usize, item: &crate::InventoryRecord) {
        let start = ITEMS + slot * INVENTORY_BYTES;
        self.0[start..start + INVENTORY_BYTES].copy_from_slice(&item.0);
    }
    /// Sets current stamina.
    pub fn set_stamina(&mut self, value: i16) {
        put_word(self.0, STAMINA, value);
    }
    /// Sets current Psi points.
    pub fn set_psi_points(&mut self, value: i16) {
        put_word(self.0, 0x58, value);
    }
    /// Raises the attribute named by a signed, one-based selector by one; a
    /// selector outside the attributes reaches a neighbouring field.
    ///
    /// Original: resolvespell
    pub fn train_attribute(&mut self, selector: i8) {
        let offset = (0x9e2 + 2 * (isize::from(selector) - 1)) as usize;
        let value = word_at(self.0, offset).wrapping_add(1);
        put_word(self.0, offset, value);
    }
    /// Sets maximum stamina.
    pub fn set_stamina_max(&mut self, value: i16) {
        put_word(self.0, STAMINA_MAX, value);
    }
    /// Sets maximum Psi points.
    pub fn set_psi_points_max(&mut self, value: i16) {
        put_word(self.0, 0x5a, value);
    }
    /// Sets the movement left this round.
    pub fn set_movement(&mut self, value: i16) {
        put_word(self.0, 0x3c, value);
    }
    /// Sets the movement the warrior gets each round.
    pub fn set_movement_max(&mut self, value: i16) {
        put_word(self.0, 0x3e, value);
    }
    /// Sets the most weight the warrior can carry.
    pub fn set_weight_capacity(&mut self, value: i16) {
        put_word(self.0, 0x9d8, value);
    }
    /// Sets the scans cast so far this round.
    pub fn set_spells_so_far(&mut self, value: i16) {
        put_word(self.0, 0xac8, value);
    }
    /// Sets how long the warrior stays helpless.
    pub fn set_turn_skip_duration(&mut self, value: i16) {
        put_word(self.0, 0x60, value);
    }
    /// Sets the action points left this round.
    pub fn set_action_points(&mut self, value: i16) {
        put_word(self.0, 0x40, value);
    }
    /// Sets the weight the warrior carries.
    pub fn set_load(&mut self, value: i16) {
        put_word(self.0, 0x9d6, value);
    }
    /// Sets the credits the warrior carries.
    pub fn set_carried_credits(&mut self, value: i16) {
        put_word(self.0, 0x9da, value);
    }
    /// Sets the charge of the item in `slot`.
    pub fn set_inventory_quantity(&mut self, slot: usize, value: i16) {
        put_word(self.0, ITEMS + slot * INVENTORY_BYTES + 2, value);
    }
    /// Sets the duration of condition 25, Computer Control; negative is permanent.
    pub fn set_computer_control(&mut self, mode: i16) {
        put_word(self.0, COMPUTER_CONTROL, mode);
    }
    /// Sets the bleeding flag.
    pub fn set_bleeding(&mut self, flag: u8) {
        self.0[BLEEDING] = flag;
    }
    /// Sets the guarding flag.
    pub fn set_guarding(&mut self, value: i16) {
        put_word(self.0, 0x16, value);
    }
    /// Aims the warrior at `body`; -1 clears it.
    pub fn set_target(&mut self, body: i16) {
        put_word(self.0, 0x18, body);
    }
    /// Ending the turn clears guarding, then the remaining movement; action points
    /// are not cleared here.
    pub fn finish_turn_movement(&mut self) {
        self.0[0x16..0x18].fill(0);
        self.0[0x3c..0x3e].fill(0);
    }
    /// Forgets every scan the warrior has learned.
    pub fn clear_learned_scans(&mut self) {
        self.0[KNOWN_SCANS..NAME].fill(0);
    }
    /// Puts back the first inventory record saved by `inventory_bytes`.
    pub fn restore_inventory_bytes(&mut self, bytes: &[u8]) {
        self.0[ITEMS..ITEMS + INVENTORY_BYTES].copy_from_slice(bytes);
    }
    /// Sets the warrior's portrait picture or, when `portrait` is false, its map icon.
    pub fn set_appearance(&mut self, portrait: bool, resource: i16) {
        put_word(self.0, if portrait { 0x54 } else { 0x56 }, resource);
    }
    /// Sets the victory points available for upgrades.
    pub fn set_upgrade_points(&mut self, points: i32) {
        put_long(self.0, UPGRADE_POINTS, points);
    }
    /// Sets the victory points still to earn for the next level.
    pub fn set_remaining_victory_points(&mut self, points: i32) {
        put_long(self.0, REMAINING_VICTORY_POINTS, points);
    }
    /// Sets how many times the upgrade `selector` names has been bought.
    pub fn set_upgrade_count(&mut self, selector: u16, count: i16) {
        put_word(self.0, 0xace + usize::from(selector) * 2, count);
    }
    /// Turns the warrior to face the way a horizontal step goes: left for a negative
    /// `dx`, right for a positive one, unchanged for none.
    pub fn face_horizontal_step(&mut self, dx: i8) {
        if dx < 0 {
            self.0[0x9e0] = 1;
        } else if dx > 0 {
            self.0[0x9e0] = 0;
        }
    }
    /// Appends a bought item without compacting; the caller has already
    /// checked capacity and updates the quantity and count itself.
    pub fn append_inventory(&mut self, count: usize, item: &crate::InventoryRecord) {
        let start = ITEMS + count * INVENTORY_BYTES;
        self.0[start..start + INVENTORY_BYTES].copy_from_slice(&item.0);
        put_word(self.0, ITEM_COUNT, (count + 1) as i16);
    }
}
