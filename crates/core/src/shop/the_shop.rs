//! The shop being visited, kept apart from the shop files.
use crate::RuleError;
use centurions_content::{
    mission::ScenarioContent,
    records::{SHOP_BYTES, SHOP_SLOTS, ShopRecord},
};
use serde::{Deserialize, Serialize};

/// Original shop IDs, quantities and price.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TheShop {
    /// The shop's items, quantities and price percentage.
    bytes: Vec<u8>,
}

impl Default for TheShop {
    /// A new game's shop is empty.
    fn default() -> Self {
        Self::startup()
    }
}

impl TheShop {
    /// All 3002 bytes are zero before any file or shop is loaded.
    pub(crate) fn startup() -> Self {
        Self {
            bytes: vec![0; SHOP_BYTES],
        }
    }

    /// Checks that the shop record has its full length.
    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        if self.bytes.len() != SHOP_BYTES {
            return Err("invalid retained shop work length".into());
        }
        Ok(())
    }

    /// Other routines may read any value, whatever the quantity or whether a shop
    /// is open.
    pub(crate) fn read_word(&self, offset: usize) -> Result<i16, RuleError> {
        let bytes =
            self.bytes
                .get(offset..offset.saturating_add(2))
                .ok_or(RuleError::Unsupported {
                    action: "shop.work",
                    dependency: "shop work alias outside the retained record",
                })?;
        Ok(i16::from_be_bytes([bytes[0], bytes[1]]))
    }

    /// A monster write indexed by warrior may land on any value of the record.
    pub(crate) fn write_word(&mut self, offset: usize, value: i16) -> Result<(), RuleError> {
        self.read_word(offset)?;
        self.set_word(offset, value);
        Ok(())
    }

    /// Sets a two-byte value of the shop record.
    fn set_word(&mut self, offset: usize, value: i16) {
        self.bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }

    /// The item in a stock slot.
    pub(crate) fn item(&self, slot: usize) -> Result<i16, RuleError> {
        if slot >= SHOP_SLOTS {
            return Err("shop work slot outside the record".into());
        }
        self.read_word(slot * 2)
    }

    /// How many of the slot's item the shop has.
    pub(crate) fn quantity(&self, slot: usize) -> Result<u8, RuleError> {
        if slot >= SHOP_SLOTS {
            return Err("shop work slot outside the record".into());
        }
        Ok(self.bytes[SHOP_SLOTS * 2 + slot])
    }

    /// The shop's whole record.
    pub(crate) fn record(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    /// Sets the item in a stock slot.
    fn set_item(&mut self, slot: usize, item: i16) {
        self.set_word(slot * 2, item);
    }

    /// Reading the shop file replaces the record for every shop; a final short
    /// read replaces only what it has.
    ///
    /// Original: resetgamefiles
    pub(crate) fn copy_file(&mut self, table: &[u8]) {
        for record in table.chunks(SHOP_BYTES) {
            for (destination, byte) in self.bytes.iter_mut().zip(record) {
                *destination = *byte;
            }
        }
    }

    /// Copies the first map's shop table of each of the scenario's three levels into the record, as
    /// starting a scenario does.
    pub(crate) fn reset_scenario(&mut self, scenario: &ScenarioContent) {
        for level in 0..3 {
            let records = if scenario.level == level {
                Some(scenario)
            } else {
                scenario.catalog.get(&(scenario.name.clone(), level))
            };
            if let Some(map) = records.and_then(|records| records.maps.first()) {
                self.copy_file(&map.shops);
            }
        }
    }

    /// Compacts each family without clearing the rows after it.
    ///
    /// Original: loadshop
    pub(crate) fn load(&mut self, record: &[u8]) -> Result<(), RuleError> {
        if record.len() != SHOP_BYTES {
            return Err("invalid shop work input length".into());
        }
        let record = ShopRecord(record);
        self.bytes[3000..].copy_from_slice(&record.price_percent()?.to_be_bytes());
        for family in 0..5 {
            let mut next = family * 200;
            for slot in family * 200..(family + 1) * 200 {
                let quantity = record.quantity(slot);
                if quantity != 0 {
                    self.bytes[SHOP_SLOTS * 2 + next] = quantity;
                    self.set_item(next, record.item(slot)?);
                    next += 1;
                }
            }
        }
        Ok(())
    }

    /// The party buys one of a slot's items; the slot's item is cleared when the last is bought.
    pub(crate) fn buy_one(&mut self, slot: usize) -> Result<(), RuleError> {
        let previous = self.quantity(slot)?;
        self.bytes[SHOP_SLOTS * 2 + slot] = previous.wrapping_sub(1);
        if previous == 1 {
            self.set_item(slot, 0);
        }
        Ok(())
    }

    /// The shop takes a sold item: one more goes into the slot of the item's family of 200 that
    /// already holds that item, or else into the first empty slot of the family.
    pub(crate) fn accept_sale(&mut self, item: i16) -> Result<(), RuleError> {
        let family = usize::from(item.unsigned_abs()) / 200 * 200;
        if family >= SHOP_SLOTS {
            return Ok(());
        }
        for slot in family..family + 200 {
            if self.item(slot)? == item {
                self.bytes[SHOP_SLOTS * 2 + slot] = self.quantity(slot)?.wrapping_add(1);
                return Ok(());
            }
        }
        for slot in family..family + 200 {
            if self.quantity(slot)? == 0 {
                self.bytes[SHOP_SLOTS * 2 + slot] = 1;
                self.set_item(slot, item);
                break;
            }
        }
        Ok(())
    }

    /// Adds to the shop's price percentage and changes the quantity (not below zero) of every slot
    /// that holds the given item.
    pub(crate) fn alter(&mut self, percent: i16, item: i16, change: i8) -> Result<(), RuleError> {
        let price = self.read_word(3000)?.wrapping_add(percent);
        self.set_word(3000, price);
        for slot in 0..SHOP_SLOTS {
            if self.item(slot)? == item {
                let quantity = (self.quantity(slot)? as i8).wrapping_add(change).max(0) as u8;
                self.bytes[SHOP_SLOTS * 2 + slot] = quantity;
            }
        }
        Ok(())
    }
}
