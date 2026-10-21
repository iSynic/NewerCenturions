//! Item fields used by showitemstats and quickinfo. Display composition differs
//! from equipped gameplay stats, so the callers retain their own merge rules.
use crate::{ContentError, word};

pub struct ItemPanelRecord<'a>(pub &'a [u8]);

impl ItemPanelRecord<'_> {
    pub fn durability(&self) -> i16 {
        i16::from(self.0[6] as i8)
    }
    pub fn required_level(&self) -> Result<i16, ContentError> {
        word(self.0, 138)
    }
    pub fn armor(&self) -> Result<i16, ContentError> {
        word(self.0, 94)
    }
    pub fn shield_chance(&self) -> Result<i16, ContentError> {
        word(self.0, 88)
    }
    pub fn shield_cost(&self) -> Result<i16, ContentError> {
        word(self.0, 92)
    }
    pub fn capacity_modifier(&self) -> Result<i16, ContentError> {
        word(self.0, 148)
    }
    pub fn range(&self) -> Result<i16, ContentError> {
        word(self.0, 60)
    }
    pub fn missile_dodge(&self) -> i8 {
        self.0[3] as i8
    }
    pub fn melee_dodge(&self) -> i8 {
        self.0[4] as i8
    }
    pub fn equipment_type(&self) -> i8 {
        self.0[87] as i8
    }
    pub fn range_modifier(&self) -> i8 {
        self.0[79] as i8
    }
    pub fn cost_modifier(&self) -> i8 {
        self.0[80] as i8
    }
    pub fn attribute_changes(&self) -> Result<([i16; 2], i16), ContentError> {
        Ok(([word(self.0, 46)?, word(self.0, 48)?], word(self.0, 50)?))
    }
    pub fn target_mode(&self) -> Result<i16, ContentError> {
        word(self.0, 56)
    }
    pub fn animation(&self) -> Result<i16, ContentError> {
        word(self.0, 58)
    }
}
