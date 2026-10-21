//! Original warrior inventory storage. Slot moves do not normalize records or
//! overwrite the private instance bytes that installitem leaves untouched.
use super::{ItemDefinition, put_word};
use crate::{ContentError, INVENTORY_BYTES, InventoryRecord, span, word};

const START: usize = 0x132;
pub struct InventorySlots<'a>(pub &'a [u8]);
pub struct InventorySlotsMut<'a>(pub &'a mut [u8]);

impl InventorySlots<'_> {
    /// Story selection also inspects the one-past-count slot. Keep its narrow
    /// ID read: a complete item decode would impose a different bounds check.
    pub fn item_id(&self, index: usize) -> Result<i16, ContentError> {
        word(self.0, START + index * INVENTORY_BYTES)
    }

    pub fn count(&self) -> Result<i16, ContentError> {
        word(self.0, 0x1a)
    }
    pub fn slot(&self, index: usize) -> Result<InventoryRecord, ContentError> {
        Ok(InventoryRecord(
            span(self.0, START + index * INVENTORY_BYTES, INVENTORY_BYTES)?
                .try_into()
                .unwrap(),
        ))
    }
    pub fn equipment_flag(&self, index: usize) -> u8 {
        self.0[START + index * INVENTORY_BYTES + 9]
    }
    pub fn battery_has_dependents(&self) -> Result<bool, ContentError> {
        Ok(word(self.0, 0xe0)? != 0 || word(self.0, 0xe2)? != 0)
    }
}

impl InventorySlotsMut<'_> {
    /// Ordinary remove differs from dropitem: shift only occupied slots and
    /// clear the vacated last slot, leaving all later work records untouched.
    pub fn remove_occupied_slot(&mut self, slot: usize, count: usize) {
        self.0.copy_within(
            START + (slot + 1) * INVENTORY_BYTES..START + count * INVENTORY_BYTES,
            START + slot * INVENTORY_BYTES,
        );
        self.0[START + (count - 1) * INVENTORY_BYTES..START + count * INVENTORY_BYTES].fill(0);
        self.set_count((count - 1) as i16);
    }

    pub fn set_count(&mut self, count: i16) {
        put_word(self.0, 0x1a, count);
    }
    pub fn append_collected(
        &mut self,
        count: usize,
        item: &InventoryRecord,
        definition: ItemDefinition<'_>,
    ) -> Result<(), ContentError> {
        let destination =
            &mut self.0[START + count * INVENTORY_BYTES..START + (count + 1) * INVENTORY_BYTES];
        destination[..8].copy_from_slice(&item.0[..8]);
        destination[..2].copy_from_slice(&definition.description_id()?.to_be_bytes());
        destination[8] = definition.reserve_ammunition_kind()? as u8;
        destination[9] = 0;
        destination[10] = definition.category()? as u8;
        destination[13..].copy_from_slice(&item.0[13..]);
        put_word(self.0, 0x1a, (count + 1) as i16);
        Ok(())
    }
    /// dropitem 0x121152 shifts only through slot 29, then clears from the new
    /// count through slot 99. This is deliberately not ordinary Vec removal.
    pub fn remove_dropped_slot(&mut self, slot: usize, remaining: i16) {
        for index in slot..29 {
            self.0.copy_within(
                START + (index + 1) * INVENTORY_BYTES..START + (index + 2) * INVENTORY_BYTES,
                START + index * INVENTORY_BYTES,
            );
        }
        for index in remaining as usize..100 {
            self.0[START + index * INVENTORY_BYTES..START + (index + 1) * INVENTORY_BYTES].fill(0);
        }
        put_word(self.0, 0x1a, remaining);
    }
}
