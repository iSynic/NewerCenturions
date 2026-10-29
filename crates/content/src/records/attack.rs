use crate::{ContentError, word};

/// Original attack packet shared by firearms and Psi. This is a borrowed view,
/// not a decoded replacement: unknown mode values keep their native comparisons.
pub struct AttackPacket<'a>(pub &'a [u8]);

impl AttackPacket<'_> {
    pub fn field_kind(&self) -> i16 {
        i16::from(self.0[4] as i8)
    }
    pub fn effect_animation(&self) -> i16 {
        i16::from(self.0[20] as i8)
    }

    pub fn area_shape(&self) -> u8 {
        self.0[31]
    }
    pub fn mode_allowed(&self, combat: bool) -> bool {
        self.0[if combat { 21 } else { 22 }] != 0
    }

    pub fn range_per_power(&self) -> i16 {
        i16::from(self.0[3] as i8)
    }

    pub fn base_range(&self) -> i16 {
        i16::from(self.0[2] as i8)
    }

    pub fn psi_cost(&self) -> i16 {
        i16::from(self.0[30])
    }

    pub fn effect(&self) -> Result<i16, ContentError> {
        word(self.0, 40)
    }
    pub fn race_restriction(&self) -> Result<i16, ContentError> {
        word(self.0, 50)
    }
    pub fn defense_category(&self) -> u8 {
        self.0[19]
    }
    pub fn saving_throw_mode(&self) -> u8 {
        self.0[9]
    }
    pub fn save_bonus_per_attack(&self) -> i16 {
        i16::from(self.0[8] as i8)
    }
    pub fn resistance_bonus_per_attack(&self) -> i16 {
        i16::from(self.0[10] as i8)
    }
    pub fn additional_duration_bounds(&self) -> (i16, i16) {
        (i16::from(self.0[15] as i8), i16::from(self.0[16] as i8))
    }
    pub fn targeting_mode(&self) -> u8 {
        self.0[1]
    }
    pub fn attack_count(&self) -> i16 {
        i16::from(self.0[0] as i8)
    }
    pub fn range_modifier(&self) -> i16 {
        i16::from(self.0[2] as i8)
    }
    pub fn is_direct_weapon_damage(&self) -> Result<bool, ContentError> {
        Ok(self.0[19] == 9 && self.0[20] == 9 && word(self.0, 40)? == 0)
    }
    pub fn divides_shots_between_targets(&self) -> bool {
        self.0[1] == 0
    }
    /// combatchoice, the NPC decision, cast and spelltargets compare the mode as a
    /// signed BYTE, so modes 128..255 target bodies like 0 and 1.
    pub fn requires_area_mask(&self) -> bool {
        self.0[1] as i8 >= 2
    }
    /// castspell's animation setup consumes a draw even in headless callers.
    pub fn has_randomized_animation(&self) -> bool {
        self.0[7] != 0
    }
    pub fn duration_bounds(&self) -> (i16, i16) {
        (i16::from(self.0[13] as i8), i16::from(self.0[14] as i8))
    }
    pub fn launch_sound(&self) -> Result<i16, ContentError> {
        word(self.0, 36)
    }
    pub fn impact_sound(&self) -> Result<i16, ContentError> {
        word(self.0, 38)
    }
    pub fn plays_impact_when_wounded(&self) -> bool {
        self.0[19] == 9
    }
    pub fn health_effect(&self) -> i8 {
        self.0[18] as i8
    }
}

pub struct AttackPacketMut<'a>(pub &'a mut [u8]);

impl AttackPacketMut<'_> {
    pub fn set_area_shape(&mut self, shape: u8) {
        self.0[31] = shape;
    }
    pub fn rotate_area(&mut self, rotation: u8) -> Option<()> {
        self.0[31] = self.0[31].checked_add(rotation)?;
        Some(())
    }
    pub fn prepare_field_collision(&mut self) {
        if self.0[19] == 9 {
            self.0[19] = 1;
            self.0[20] = 1;
        }
        self.0[4] = 0;
    }
    pub fn add_save_bonus(&mut self, bonus: i16) {
        self.0[8] = self.0[8].wrapping_add(bonus as u8);
    }
    pub fn set_saving_throw_mode(&mut self, mode: u8) {
        self.0[9] = mode;
    }

    pub fn set_health_effect(&mut self, effect: u8) {
        self.0[18] = effect;
    }
    pub fn set_elemental_damage(&mut self, element: usize, amount: i16) {
        self.0[23 + element] = amount as u8;
    }
    pub fn set_damage_bounds(&mut self, minimum: i16, maximum: i16) {
        super::put_word(self.0, 32, minimum);
        super::put_word(self.0, 34, maximum);
    }
    pub fn set_impact_sound(&mut self, sound: i16) {
        super::put_word(self.0, 38, sound);
    }
    pub fn add_range_modifier(&mut self, modifier: u8) {
        self.0[2] = self.0[2].wrapping_add(modifier);
    }
    pub fn reset_turn_attack_kind(&mut self) {
        self.0[19] = 0;
    }
}
