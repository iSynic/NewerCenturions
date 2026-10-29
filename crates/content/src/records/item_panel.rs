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

/// The item work record at 0x45363c as quickinfo 0x132b0c rewrites it.
pub struct ItemPanelWorkMut<'a>(pub &'a mut [u8]);

impl ItemPanelWorkMut<'_> {
    /// Merges the selected instance's signed modifiers and its loaded ammunition,
    /// wrapping like the original. Other loaded definition bytes are retained.
    pub fn merge_quick_info(&mut self, record: &[u8], ammo: &[u8]) -> Result<(), ContentError> {
        self.merge_item_information(record, ammo)?;
        super::put_word(self.0, 138, i16::from(record[13] as i8));
        self.0[82] = record[11];
        Ok(())
    }
    /// showiteminfo 0x1555f2 merges the same modifiers as quickinfo except
    /// the required level (WORD 138) and melee accuracy (BYTE 82).
    pub fn merge_item_information(
        &mut self,
        record: &[u8],
        ammo: &[u8],
    ) -> Result<(), ContentError> {
        crate::span(record, 0, 22)?;
        crate::span(ammo, 0, 150)?;
        crate::span(self.0, 0, 150)?;
        let signed = |index: usize| i16::from(record[index] as i8);
        let add = |bytes: &mut [u8], offset: usize, delta: i16| -> Result<(), ContentError> {
            let value = word(bytes, offset)?.wrapping_add(delta);
            super::put_word(bytes, offset, value);
            Ok(())
        };
        add(self.0, 88, signed(14))?;
        self.0[81] = record[15].wrapping_add(ammo[81]);
        self.0[3] = self.0[3].wrapping_add(record[16]);
        self.0[4] = self.0[4].wrapping_add(record[17]);
        self.0[85] = record[18].wrapping_add(ammo[85]);
        add(self.0, 94, signed(19))?;
        let range = word(ammo, 60)?
            .wrapping_add(signed(20))
            .wrapping_add(i16::from(ammo[79] as i8));
        add(self.0, 60, range)
    }
    /// showiteminfo stores one attack (WORD 76) when the shown damage is nonzero.
    pub fn set_attack_count(&mut self, count: i16) -> Result<(), ContentError> {
        crate::span(self.0, 76, 2)?;
        super::put_word(self.0, 76, count);
        Ok(())
    }
}
