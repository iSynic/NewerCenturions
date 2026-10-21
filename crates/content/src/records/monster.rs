use super::put_word;
use crate::{ContentError, combat::MonsterRecord, word};

impl MonsterRecord {
    pub fn record(&self) -> EnemyRecord<'_> {
        EnemyRecord(&self.bytes)
    }
    pub fn record_mut(&mut self) -> EnemyRecordMut<'_> {
        EnemyRecordMut(&mut self.bytes)
    }
}

/// Original enemy fields consumed by combatchoice and killbody callers.
pub struct EnemyRecord<'a>(&'a [u8]);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FootprintCell {
    Anchor,
    North,
    West,
    Northwest,
}
impl FootprintCell {
    fn offset(self) -> usize {
        match self {
            Self::Anchor => 0x11e,
            Self::North => 0x11c,
            Self::West => 0x11a,
            Self::Northwest => 0x118,
        }
    }
}

pub struct NaturalAttack {
    pub minimum: i16,
    pub maximum: i16,
    pub impact_sound: i16,
    pub condition: u8,
    /// Elemental effects read the selected bound even when physical damage falls back.
    pub effect_maximum: i16,
}

impl<'a> EnemyRecord<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }
    pub fn base_stamina(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    pub fn death_script(&self) -> Result<i16, ContentError> {
        word(self.0, 0x60)
    }
    pub fn definition_id(&self) -> i8 {
        self.0[0x103] as i8
    }
    pub fn level(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn credit_drop_bound(&self) -> Result<i16, ContentError> {
        word(self.0, 0x3a)
    }
    pub fn experience_bonus(&self) -> Result<i16, ContentError> {
        word(self.0, 0x54)
    }
    pub fn flying_flag(&self) -> u8 {
        self.0[0x7f]
    }
    pub fn loot_item(&self, slot: usize) -> Result<i16, ContentError> {
        word(self.0, 0x26 + slot * 2)
    }
    pub fn speed(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    pub fn movement_per_round(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    pub fn turn_script_chance(&self) -> Result<i16, ContentError> {
        word(self.0, 0x5c)
    }
    pub fn missile_accuracy_bonus(&self) -> Result<i16, ContentError> {
        word(self.0, 0x66)
    }
    pub fn melee_bonus(&self) -> Result<i16, ContentError> {
        word(self.0, 0x40)
    }
    pub fn natural_attack(&self, index: usize) -> NaturalAttack {
        let selected = 0x93 + index * 4;
        let damage = if self.0[selected] == 0 {
            0x93
        } else {
            selected
        };
        NaturalAttack {
            minimum: i16::from(self.0[damage] as i8),
            maximum: i16::from(self.0[damage + 1] as i8),
            impact_sound: i16::from(self.0[damage + 2] as i8),
            // The fallback applies only to damage/sound, never the selected effect.
            condition: self.0[selected + 3],
            effect_maximum: i16::from(self.0[selected + 1] as i8),
        }
    }
    pub fn missile_dodge_bonus(&self) -> Result<i16, ContentError> {
        word(self.0, 0x62)
    }
    /// Reactions clear this byte; attack retains its signed multiplier value.
    pub fn guarding(&self) -> i8 {
        self.0[0x104] as i8
    }
    pub fn condition(&self, condition: super::Condition) -> i16 {
        self.condition_duration(condition as usize)
    }
    pub fn condition_duration(&self, index: usize) -> i16 {
        i16::from(self.0[0xb2 + index] as i8)
    }
    pub fn magic_immunity(&self) -> Result<i16, ContentError> {
        word(self.0, 0xa)
    }
    pub fn saving_throw(&self, category: usize) -> i16 {
        i16::from(self.0[0x8b + category] as i8)
    }
    pub fn level_protection(&self, level: usize) -> u8 {
        self.0[0xc2 + level]
    }
    pub fn has_creature_category(&self, category: usize) -> bool {
        self.0[0xa8 + category] != 0
    }
    pub fn name(&self) -> Result<String, ContentError> {
        crate::combat::monster_name(self.0)
    }
    pub fn faction(&self) -> Result<i16, ContentError> {
        word(self.0, 0xe)
    }
    pub fn stamina(&self) -> Result<i16, ContentError> {
        word(self.0, 0x110)
    }
    pub fn maximum_stamina(&self) -> Result<i16, ContentError> {
        word(self.0, 0x112)
    }
    pub fn psi_points(&self) -> Result<i16, ContentError> {
        word(self.0, 0x10e)
    }
    pub fn maximum_psi(&self) -> Result<i16, ContentError> {
        word(self.0, 0x52)
    }
    pub fn shield_chance(&self) -> Result<i16, ContentError> {
        word(self.0, 0x56)
    }
    pub fn facing_flag(&self) -> u8 {
        self.0[0x107]
    }
    pub fn shield_points(&self) -> Result<i16, ContentError> {
        word(self.0, 0x58)
    }
    pub fn weapon_immunity(&self) -> Result<i16, ContentError> {
        word(self.0, 12)
    }
    pub fn armor(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
    pub fn tactical_icon(&self) -> Result<i16, ContentError> {
        word(self.0, 0x50)
    }
    pub fn survivor_marker(&self) -> Result<i16, ContentError> {
        word(self.0, 0x5a)
    }
    pub fn removed(&self) -> bool {
        self.0[0xb0] == 255
    }
    /// 0 is an ordinary body, 1 a mechanical body, 255 a disabled body.
    /// Capture and target eligibility intentionally test different values.
    pub fn mechanical_state(&self) -> u8 {
        self.0[0xb0]
    }
    pub fn capture_flag(&self) -> u8 {
        self.0[0xb1]
    }
    pub fn size(&self) -> Result<i16, ContentError> {
        word(self.0, 0x10)
    }
    pub fn underlying_tile(&self) -> Result<i16, ContentError> {
        word(self.0, 0x11e)
    }
    pub fn footprint_tile(&self, cell: FootprintCell) -> Result<i16, ContentError> {
        word(self.0, cell.offset())
    }
    pub fn movement_sound(&self) -> Result<i16, ContentError> {
        word(self.0, 0x6c)
    }
    pub fn death_sound(&self) -> Result<i16, ContentError> {
        word(self.0, 0x6a)
    }
    pub fn target_body(&self) -> Result<i16, ContentError> {
        word(self.0, 0x10c)
    }
    /// combat1153c0 excludes this byte only while scanning a new opponent.
    pub fn excluded_from_target_scan(&self) -> bool {
        self.0[0x82] != 0
    }
    pub fn primary_weapon(&self) -> Result<i16, ContentError> {
        word(self.0, 0x26)
    }
    pub fn melee_weapon(&self) -> Result<i16, ContentError> {
        word(self.0, 0x28)
    }
    pub fn selected_item(&self) -> Result<i16, ContentError> {
        word(self.0, 0x4e)
    }
    pub fn ammunition(&self) -> Result<i16, ContentError> {
        word(self.0, 0x4c)
    }
    pub fn firearm_chance(&self) -> Result<i16, ContentError> {
        word(self.0, 0x48)
    }
    pub fn spell_chance(&self) -> Result<i16, ContentError> {
        word(self.0, 0x42)
    }
    pub fn final_turn_mode(&self) -> Result<i16, ContentError> {
        word(self.0, 0x5e)
    }
    pub fn action_points(&self) -> i8 {
        self.0[0x109] as i8
    }
    pub fn actions_per_round(&self) -> i8 {
        self.0[0x8a] as i8
    }
    pub fn spell_attempts(&self) -> i8 {
        self.0[0x7e] as i8
    }
    pub fn turn_skip_duration(&self) -> u8 {
        self.0[0xb3]
    }
    pub fn fleeing_duration(&self) -> u8 {
        self.0[0xb2]
    }
    pub fn reaction_suppression(&self) -> u8 {
        self.0[0xca]
    }
    pub fn movement_points(&self) -> i8 {
        self.0[0x106] as i8
    }
    pub fn spell_blocked_before_movement(&self) -> bool {
        [0xb7, 0xcf, 0xd9, 0xb3, 0x105]
            .iter()
            .any(|offset| self.0[*offset] != 0)
    }
    pub fn spell_blocked_after_movement(&self) -> bool {
        [0xb3, 0xb7, 0xcf, 0xd9, 0x105]
            .iter()
            .any(|offset| self.0[*offset] != 0)
    }
}

pub struct EnemyRecordMut<'a>(&'a mut [u8]);

impl EnemyRecordMut<'_> {
    pub fn clear_definition_id(&mut self) {
        self.0[0x103] = 0;
    }

    /// Menu transfer consumes the battle copy's level, health and rewards.
    pub fn clear_survivor_rewards(&mut self) {
        put_word(self.0, 0, 0);
        put_word(self.0, 0x110, 0);
        for slot in 0..6 {
            put_word(self.0, 0x26 + slot * 2, 0);
        }
        put_word(self.0, 0x3a, 0);
    }

    pub fn set_target_scan_excluded(&mut self, excluded: bool) {
        self.0[0x82] = u8::from(excluded);
    }
    pub fn set_death_script(&mut self, script: i16) {
        put_word(self.0, 0x60, script);
    }
    pub fn set_horizontal_direction(&mut self, direction: i8) {
        self.0[0x107] = direction as u8;
    }

    pub fn set_magic_immunity(&mut self, value: i16) {
        put_word(self.0, 0xa, value);
    }
    pub fn adjust_elemental_saves(&mut self, amount: i16) {
        for value in &mut self.0[0x8b..0x91] {
            *value = value.wrapping_add(amount as u8);
        }
    }
    pub fn set_armor(&mut self, value: i16) {
        put_word(self.0, 8, value);
    }
    pub fn set_speed(&mut self, value: i16) {
        put_word(self.0, 4, value);
    }
    pub fn set_maximum_stamina(&mut self, value: i16) {
        put_word(self.0, 0x112, value);
    }
    pub fn set_maximum_psi(&mut self, value: i16) {
        put_word(self.0, 0x52, value);
    }

    pub fn set_footprint_tile(&mut self, cell: FootprintCell, tile: i16) {
        put_word(self.0, cell.offset(), tile);
    }
    pub fn set_movement_direction(&mut self, dx: i8, dy: i8) {
        self.0[0x107] = dx as u8;
        self.0[0x108] = dy as u8;
    }
    pub fn set_fleeing_chance(&mut self, chance: i16) {
        put_word(self.0, 0x46, chance);
    }

    pub fn set_mechanical_state(&mut self, value: u8) {
        self.0[0xb0] = value;
    }
    pub fn clear_conditions(&mut self) {
        self.0[0xb2..0xda].fill(0);
    }
    pub fn set_turn_script_chance(&mut self, value: i16) {
        put_word(self.0, 0x5c, value);
    }
    pub fn set_faction(&mut self, value: i16) {
        put_word(self.0, 0xe, value);
    }
    pub fn set_condition(&mut self, condition: super::Condition, duration: i16) {
        self.set_condition_duration(condition as usize, duration);
    }
    pub fn set_condition_duration(&mut self, index: usize, duration: i16) {
        self.0[0xb2 + index] = duration as u8;
    }
    /// getup's haste branch copies only an absent slot, not an effect whose
    /// damage subsequently falls back to slot zero during attack resolution.
    pub fn fill_missing_natural_attack(&mut self, slot: usize) {
        let offset = 0x93 + slot * 4;
        if self.0[offset] == 0 {
            self.0.copy_within(0x93..0x97, offset);
        }
    }
    pub fn clear_damage_marker(&mut self) {
        self.0[0x105] = 0;
    }
    pub fn decrement_temporary_conditions(&mut self) {
        for duration in &mut self.0[0xb2..0xda] {
            if (*duration as i8) > 0 {
                *duration -= 1;
            }
        }
    }
    pub fn from_bytes(bytes: &mut [u8]) -> EnemyRecordMut<'_> {
        EnemyRecordMut(bytes)
    }
    pub fn mark_damaged(&mut self) {
        self.0[0x105] = 1;
    }
    pub fn set_stamina(&mut self, value: i16) {
        put_word(self.0, 0x110, value);
    }
    pub fn set_psi_points(&mut self, value: i16) {
        put_word(self.0, 0x10e, value);
    }
    pub fn set_action_points(&mut self, value: i8) {
        self.0[0x109] = value as u8;
    }
    pub fn set_underlying_tile(&mut self, value: i16) {
        put_word(self.0, 0x11e, value);
    }
    pub fn set_target_body(&mut self, value: i16) {
        put_word(self.0, 0x10c, value);
    }
    pub fn set_selected_item(&mut self, value: i16) {
        put_word(self.0, 0x4e, value);
    }
    pub fn face_towards(&mut self, origin_x: i16, target_x: i16) {
        if origin_x < target_x && self.0[0x107] == 0 {
            self.0[0x107] = 1;
        } else if target_x < origin_x && self.0[0x107] == 1 {
            self.0[0x107] = 0;
        }
    }
    pub fn set_reaction_ready(&mut self, ready: bool) {
        self.0[0x104] = u8::from(ready);
    }
    pub fn set_movement_points(&mut self, points: i8) {
        self.0[0x106] = points as u8;
    }
}
