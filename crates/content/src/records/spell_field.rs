//! Saved 20-byte spell fields (`que`), laid down when a spell picks its
//! targets. Coordinates, power and level are stored as signed bytes and widened
//! to words when read.
use super::put_word;
use crate::{ContentError, word};

/// Size of one saved spell field.
pub const SPELL_FIELD_BYTES: usize = 20;
/// A spell field as a read-only view.
pub struct SpellField<'a>(pub &'a [u8]);
/// A spell field as a view that can change it.
pub struct SpellFieldMut<'a>(pub &'a mut [u8]);
impl SpellField<'_> {
    /// Rounds the field lasts; zero is an unused slot.
    pub fn duration(&self) -> u8 {
        self.0[15]
    }
    /// The battle-order position at which the field's duration counts down each round.
    pub fn initiative(&self) -> i16 {
        i16::from(self.0[18] as i8)
    }
    /// The scan that laid the field.
    pub fn spell(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
    /// The turn added to the area pattern number when the attack can rotate; zero otherwise.
    pub fn rotation(&self) -> u8 {
        self.0[17]
    }
    /// The power level the field was laid at.
    pub fn power(&self) -> i16 {
        i16::from(self.0[16] as i8)
    }
    /// The level of the scan that laid the field.
    pub fn level(&self) -> i16 {
        i16::from(self.0[19] as i8)
    }
    /// The map cell the field is centred on.
    pub fn center(&self) -> (i16, i16) {
        (
            i16::from(self.0[12] as i8).wrapping_add(3),
            i16::from(self.0[13] as i8).wrapping_add(3),
        )
    }
    /// Whether the map cell `point` lies inside the field's seven by seven area.
    pub fn contains(&self, point: (i16, i16)) -> Result<bool, ContentError> {
        Ok(point.1 >= word(self.0, 0)?
            && point.1 < word(self.0, 4)?
            && point.0 >= word(self.0, 2)?
            && point.0 < word(self.0, 6)?)
    }
}
impl SpellFieldMut<'_> {
    /// Sets the rounds the field lasts.
    pub fn set_duration(&mut self, duration: u8) {
        self.0[15] = duration;
    }
    /// Sets the turn added to the area pattern number.
    pub fn set_rotation(&mut self, rotation: u8) {
        self.0[17] = rotation;
    }
    /// Sets the battle-order position at which the field's duration counts down.
    pub fn set_initiative(&mut self, initiative: i16) {
        self.0[18] = initiative as u8;
    }
    /// Sets the power level the field was laid at.
    pub fn set_power(&mut self, power: i16) {
        self.0[16] = power as u8;
    }
    /// Sets the level of the scan that laid the field.
    pub fn set_level(&mut self, level: i16) {
        self.0[19] = level as u8;
    }
    /// Sets the area pattern number.
    pub fn set_shape(&mut self, shape: usize) {
        self.0[14] = shape as u8;
    }
    /// Sets the scan that laid the field.
    pub fn set_spell(&mut self, spell: i16) {
        put_word(self.0, 8, spell);
    }
    /// Sets the kind of lingering field the attack lays down.
    pub fn set_kind(&mut self, kind: i16) {
        put_word(self.0, 10, kind);
    }
    /// Centres the field on a map cell and fixes its area around it.
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
