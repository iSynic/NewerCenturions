use crate::{ContentError, word};

/// Original attack packet shared by firearms and Psi. This is a borrowed view,
/// not a decoded replacement: unknown mode values keep their native comparisons.
pub struct AttackPacket<'a>(pub &'a [u8]);

impl AttackPacket<'_> {
    pub fn divides_shots_between_targets(&self) -> bool {
        self.0[1] == 0
    }
    pub fn requires_area_mask(&self) -> bool {
        self.0[1] >= 2
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
    pub fn reset_turn_attack_kind(&mut self) {
        self.0[19] = 0;
    }
}
