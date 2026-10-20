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

impl EnemyRecord<'_> {
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
    pub fn shield_points(&self) -> Result<i16, ContentError> {
        word(self.0, 0x58)
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
    pub fn size(&self) -> Result<i16, ContentError> {
        word(self.0, 0x10)
    }
    pub fn underlying_tile(&self) -> Result<i16, ContentError> {
        word(self.0, 0x11e)
    }
    pub fn death_sound(&self) -> Result<i16, ContentError> {
        word(self.0, 0x6a)
    }
    pub fn target_body(&self) -> Result<i16, ContentError> {
        word(self.0, 0x10c)
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
    pub fn set_underlying_tile(&mut self, value: i16) {
        put_word(self.0, 0x11e, value);
    }
    pub fn set_target_body(&mut self, value: i16) {
        put_word(self.0, 0x10c, value);
    }
    pub fn set_selected_item(&mut self, value: i16) {
        put_word(self.0, 0x4e, value);
    }
    pub fn set_reaction_ready(&mut self, ready: bool) {
        self.0[0x104] = u8::from(ready);
    }
    pub fn set_movement_points(&mut self, points: i8) {
        self.0[0x106] = points as u8;
    }
}
