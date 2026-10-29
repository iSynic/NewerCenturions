//! Retained loadshop/moveicon work at 0x478288, separate from persistent shop files.
use crate::RuleError;
use centurions_content::{
    mission::ScenarioContent,
    records::{SHOP_BYTES, SHOP_SLOTS, ShopRecord},
};
use serde::{Deserialize, Serialize};

/// Original shop IDs, quantities and price.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShopWork {
    bytes: Vec<u8>,
}

impl Default for ShopWork {
    fn default() -> Self {
        Self::startup()
    }
}

impl ShopWork {
    /// All 3002 bytes are zero in initialized DATA, before any file or shop load.
    pub(crate) fn startup() -> Self {
        Self {
            bytes: vec![0; SHOP_BYTES],
        }
    }

    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        if self.bytes.len() != SHOP_BYTES {
            return Err("invalid retained shop work length".into());
        }
        Ok(())
    }

    /// Native aliases may read a WORD independently of quantity or an open shop.
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

    /// A warrior-indexed NPC write may land on any WORD of the record.
    pub(crate) fn write_word(&mut self, offset: usize, value: i16) -> Result<(), RuleError> {
        self.read_word(offset)?;
        self.set_word(offset, value);
        Ok(())
    }

    fn set_word(&mut self, offset: usize, value: i16) {
        self.bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }

    pub(crate) fn item(&self, slot: usize) -> Result<i16, RuleError> {
        if slot >= SHOP_SLOTS {
            return Err("shop work slot outside the record".into());
        }
        self.read_word(slot * 2)
    }

    pub(crate) fn quantity(&self, slot: usize) -> Result<u8, RuleError> {
        if slot >= SHOP_SLOTS {
            return Err("shop work slot outside the record".into());
        }
        Ok(self.bytes[SHOP_SLOTS * 2 + slot])
    }

    pub(crate) fn record(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    fn set_item(&mut self, slot: usize, item: i16) {
        self.set_word(slot * 2, item);
    }

    /// fread loops in resetgamefiles/load replace this buffer for every record;
    /// a final short read replaces only its available prefix.
    pub(crate) fn copy_file(&mut self, table: &[u8]) {
        for record in table.chunks(SHOP_BYTES) {
            for (destination, byte) in self.bytes.iter_mut().zip(record) {
                *destination = *byte;
            }
        }
    }

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

    /// loadshop compacts each family without clearing its unwritten trailing rows.
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

    pub(crate) fn buy_one(&mut self, slot: usize) -> Result<(), RuleError> {
        let previous = self.quantity(slot)?;
        self.bytes[SHOP_SLOTS * 2 + slot] = previous.wrapping_sub(1);
        if previous == 1 {
            self.set_item(slot, 0);
        }
        Ok(())
    }

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
