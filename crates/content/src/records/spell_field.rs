//! Saved 20-byte spell fields from spelltargets and collision_field. Coordinates,
//! power and level narrow to signed bytes before their later WORD use.
use super::put_word;
use crate::{ContentError, word};

pub const SPELL_FIELD_BYTES: usize = 20;
pub struct SpellField<'a>(pub &'a [u8]);
pub struct SpellFieldMut<'a>(pub &'a mut [u8]);
impl SpellField<'_> {
    pub fn duration(&self) -> u8 {
        self.0[15]
    }
    pub fn initiative(&self) -> i16 {
        i16::from(self.0[18] as i8)
    }
    pub fn spell(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
    pub fn rotation(&self) -> u8 {
        self.0[17]
    }
    pub fn power(&self) -> i16 {
        i16::from(self.0[16] as i8)
    }
    pub fn level(&self) -> i16 {
        i16::from(self.0[19] as i8)
    }
    pub fn center(&self) -> (i16, i16) {
        (
            i16::from(self.0[12] as i8).wrapping_add(3),
            i16::from(self.0[13] as i8).wrapping_add(3),
        )
    }
    pub fn contains(&self, point: (i16, i16)) -> Result<bool, ContentError> {
        Ok(point.1 >= word(self.0, 0)?
            && point.1 < word(self.0, 4)?
            && point.0 >= word(self.0, 2)?
            && point.0 < word(self.0, 6)?)
    }
}
impl SpellFieldMut<'_> {
    pub fn set_duration(&mut self, duration: u8) {
        self.0[15] = duration;
    }
    pub fn set_rotation(&mut self, rotation: u8) {
        self.0[17] = rotation;
    }
    pub fn set_initiative(&mut self, initiative: i16) {
        self.0[18] = initiative as u8;
    }
    pub fn set_power(&mut self, power: i16) {
        self.0[16] = power as u8;
    }
    pub fn set_level(&mut self, level: i16) {
        self.0[19] = level as u8;
    }
    pub fn set_shape(&mut self, shape: usize) {
        self.0[14] = shape as u8;
    }
    pub fn set_spell(&mut self, spell: i16) {
        put_word(self.0, 8, spell);
    }
    pub fn set_kind(&mut self, kind: i16) {
        put_word(self.0, 10, kind);
    }
    pub fn set_center(&mut self, center: (i16, i16)) {
        self.0[12] = center.0.wrapping_sub(3) as u8;
        self.0[13] = center.1.wrapping_sub(3) as u8;
        let left = i16::from(self.0[12] as i8);
        let top = i16::from(self.0[13] as i8);
        put_word(self.0, 0, top);
        put_word(self.0, 2, left);
        put_word(self.0, 4, top.wrapping_add(7));
        put_word(self.0, 6, left.wrapping_add(7));
    }
}
