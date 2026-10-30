use super::put_word;
use crate::{ContentError, word};

/// Item slots in a shop's stock.
pub const SHOP_SLOTS: usize = 1000;
/// Size of a shop record.
pub const SHOP_BYTES: usize = 3002;
/// Where the quantities begin, after the item ids.
const QUANTITIES: usize = SHOP_SLOTS * 2;

/// A shop's stock: signed item ids, byte quantities and a signed price percent.
pub struct ShopRecord<'a>(pub &'a [u8]);

impl ShopRecord<'_> {
    /// The item id in slot `slot`; zero is empty.
    pub fn item(&self, slot: usize) -> Result<i16, ContentError> {
        word(self.0, slot * 2)
    }
    /// How many of the item the slot holds.
    pub fn quantity(&self, slot: usize) -> u8 {
        self.0[QUANTITIES + slot]
    }
    /// The shop's prices as a percentage of the standard prices.
    pub fn price_percent(&self) -> Result<i16, ContentError> {
        word(self.0, QUANTITIES + SHOP_SLOTS)
    }
}

/// A shop's stock as a view that can change it.
pub struct ShopRecordMut<'a>(pub &'a mut [u8]);

impl ShopRecordMut<'_> {
    /// Puts `quantity` of item `id` in slot `slot`.
    pub fn set_stock(&mut self, slot: usize, id: i16, quantity: u8) {
        put_word(self.0, slot * 2, id);
        self.0[QUANTITIES + slot] = quantity;
    }

    /// Sets how many the slot holds.
    pub fn set_quantity(&mut self, slot: usize, quantity: u8) {
        self.0[QUANTITIES + slot] = quantity;
    }
    /// Sets the shop's prices as a percentage of the standard prices.
    pub fn set_price_percent(&mut self, percent: i16) {
        put_word(self.0, QUANTITIES + SHOP_SLOTS, percent);
    }

    /// Sells one of the slot's items; the slot empties when the last one goes.
    pub fn buy_one(&mut self, slot: usize) {
        let quantity = &mut self.0[QUANTITIES + slot];
        *quantity = quantity.wrapping_sub(1);
        if *quantity == 0 {
            self.0[slot * 2..slot * 2 + 2].fill(0);
        }
    }
    /// Takes a sold item into its family of 200 slots: onto an existing stack
    /// of the same id, else into an empty slot. A full family discards the
    /// item; the caller still pays for it.
    pub fn accept_sale(&mut self, id: i16) {
        let family = usize::from(id.unsigned_abs()) / 200 * 200;
        let current = ShopRecord(self.0);
        let existing = (family..family + 200).find(|i| current.item(*i).ok() == Some(id));
        if let Some(slot) =
            existing.or_else(|| (family..family + 200).find(|i| current.quantity(*i) == 0))
        {
            put_word(self.0, slot * 2, id);
            let quantity = &mut self.0[QUANTITIES + slot];
            *quantity = quantity.wrapping_add(1);
        }
    }
}
