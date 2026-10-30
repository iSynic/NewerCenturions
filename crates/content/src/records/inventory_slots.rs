//! A warrior's hundred inventory slots (`items`) and their count (`numitems`).
//! Moving a slot copies its record as it stands, including the two instance
//! bytes that installing an item leaves alone.
use super::warrior_layout::{ITEM_COUNT, ITEMS as START};
use super::{ItemDefinition, put_word};
use crate::{ContentError, INVENTORY_BYTES, InventoryRecord, span, word};

/// A warrior's inventory as a read-only view.
pub struct InventorySlots<'a>(pub &'a [u8]);
/// A warrior's inventory as a view that can change it.
pub struct InventorySlotsMut<'a>(pub &'a mut [u8]);

impl InventorySlots<'_> {
    /// The item id in a slot. Story selection also reads the slot just past the
    /// count, so only the id is read here, not the whole record.
    pub fn item_id(&self, index: usize) -> Result<i16, ContentError> {
        word(self.0, START + index * INVENTORY_BYTES)
    }

    /// How many items the warrior carries.
    pub fn count(&self) -> Result<i16, ContentError> {
        word(self.0, ITEM_COUNT)
    }
    /// The item in slot `index`.
    pub fn slot(&self, index: usize) -> Result<InventoryRecord, ContentError> {
        Ok(InventoryRecord(
            span(self.0, START + index * INVENTORY_BYTES, INVENTORY_BYTES)?
                .try_into()
                .unwrap(),
        ))
    }
    /// Whether the item in slot `index` is worn or wielded.
    pub fn equipped(&self, index: usize) -> u8 {
        self.0[START + index * INVENTORY_BYTES + 9]
    }
    /// Whether either of the two equipment slots that depend on a worn battery holds an item.
    pub fn battery_has_dependents(&self) -> Result<bool, ContentError> {
        Ok(word(self.0, 0xe0)? != 0 || word(self.0, 0xe2)? != 0)
    }
}

impl InventorySlotsMut<'_> {
    /// Removes a slot by shifting only the occupied slots and clearing the one
    /// left vacant; the slots beyond the count are untouched. Dropping an item
    /// shifts differently.
    pub fn remove_occupied_slot(&mut self, slot: usize, count: usize) {
        self.0.copy_within(
            START + (slot + 1) * INVENTORY_BYTES..START + count * INVENTORY_BYTES,
            START + slot * INVENTORY_BYTES,
        );
        self.0[START + (count - 1) * INVENTORY_BYTES..START + count * INVENTORY_BYTES].fill(0);
        self.set_count((count - 1) as i16);
    }

    /// Sets how many items the warrior carries.
    pub fn set_count(&mut self, count: i16) {
        put_word(self.0, ITEM_COUNT, count);
    }
    /// Adds a collected item after the first `count` slots, as the definition makes it
    /// and unworn.
    pub fn append_collected(
        &mut self,
        count: usize,
        item: &InventoryRecord,
        definition: ItemDefinition<'_>,
    ) -> Result<(), ContentError> {
        let destination =
            &mut self.0[START + count * INVENTORY_BYTES..START + (count + 1) * INVENTORY_BYTES];
        destination[..8].copy_from_slice(&item.0[..8]);
        put_word(destination, 0, definition.description_id()?);
        destination[8] = definition.reserve_ammunition_kind()? as u8;
        destination[9] = 0;
        destination[10] = definition.item_type()? as u8;
        destination[13..].copy_from_slice(&item.0[13..]);
        put_word(self.0, ITEM_COUNT, (count + 1) as i16);
        Ok(())
    }
    /// Drops an item: shifts only through slot 29, then clears every slot from
    /// the new count through slot 99, as the game does.
    ///
    /// Original: dropitem
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
        put_word(self.0, ITEM_COUNT, remaining);
    }
}
