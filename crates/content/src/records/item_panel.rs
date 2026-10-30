//! Item fields shown in the item statistics and quick info panels. A panel
//! combines them differently from the statistics of worn equipment, so each
//! panel keeps its own merge rules.
use crate::{ContentError, word};

/// The item definition fields the item panels read.
pub struct ItemPanelRecord<'a>(pub &'a [u8]);

impl ItemPanelRecord<'_> {
    /// Psi resistance the item adds when worn, shown with its sign.
    pub fn durability(&self) -> i16 {
        i16::from(self.0[6] as i8)
    }
    /// Level a warrior must have to use the item.
    pub fn required_level(&self) -> Result<i16, ContentError> {
        word(self.0, 138)
    }
    /// Armour rating the item adds.
    pub fn armor(&self) -> Result<i16, ContentError> {
        word(self.0, 94)
    }
    /// Percent chance the item's shield takes a hit.
    pub fn shield_chance(&self) -> Result<i16, ContentError> {
        word(self.0, 88)
    }
    /// Charge the item's shield spends for each point of damage it stops.
    pub fn shield_cost(&self) -> Result<i16, ContentError> {
        word(self.0, 92)
    }
    /// Capacity the item adds to a battery or magazine.
    pub fn capacity_modifier(&self) -> Result<i16, ContentError> {
        word(self.0, 148)
    }
    /// The item's range.
    pub fn range(&self) -> Result<i16, ContentError> {
        word(self.0, 60)
    }
    /// Missile dodge the item adds.
    pub fn missile_dodge(&self) -> i8 {
        self.0[3] as i8
    }
    /// Melee dodge the item adds.
    pub fn melee_dodge(&self) -> i8 {
        self.0[4] as i8
    }
    /// Which kind of equipment an upgrade for the item must match.
    pub fn equipment_type(&self) -> i8 {
        self.0[87] as i8
    }
    /// Range the item adds.
    pub fn range_modifier(&self) -> i8 {
        self.0[79] as i8
    }
    /// Shield cost the item adds.
    pub fn cost_modifier(&self) -> i8 {
        self.0[80] as i8
    }
    /// The two attributes the item changes and the amount.
    pub fn attribute_changes(&self) -> Result<([i16; 2], i16), ContentError> {
        Ok(([word(self.0, 46)?, word(self.0, 48)?], word(self.0, 50)?))
    }
    /// Nonzero when the item picks targets when used; with no animation the panel then shows the
    /// first one.
    pub fn target_mode(&self) -> Result<i16, ContentError> {
        word(self.0, 56)
    }
    /// The number of the animation the item panel shows; zero shows none.
    pub fn animation(&self) -> Result<i16, ContentError> {
        word(self.0, 58)
    }
}

/// The loaded item record (`item`) as the item panels rewrite it for display.
pub struct ItemPanelMut<'a>(pub &'a mut [u8]);

impl ItemPanelMut<'_> {
    /// Merges the selected instance's signed modifiers and its loaded
    /// ammunition, with wrapping sums. Other definition bytes are kept.
    ///
    /// Original: quickinfo
    pub fn merge_quick_info(&mut self, record: &[u8], ammo: &[u8]) -> Result<(), ContentError> {
        self.merge_item_information(record, ammo)?;
        super::put_word(self.0, 138, i16::from(record[13] as i8));
        self.0[82] = record[11];
        Ok(())
    }
    /// Merges the same modifiers as quick info except the required level
    /// (word 138) and melee accuracy (byte 82).
    ///
    /// Original: showiteminfo
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
    /// Item information shows one attack (word 76) when its damage is nonzero.
    pub fn set_attack_count(&mut self, count: i16) -> Result<(), ContentError> {
        crate::span(self.0, 76, 2)?;
        super::put_word(self.0, 76, count);
        Ok(())
    }
}
