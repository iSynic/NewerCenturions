use super::put_word;
use crate::{ContentError, word};

pub const SHOP_SLOTS: usize = 1000;
pub const SHOP_BYTES: usize = 3002;
const QUANTITIES: usize = SHOP_SLOTS * 2;

/// Original shop record: signed item IDs, byte quantities, signed price percent.
pub struct ShopRecord<'a>(pub &'a [u8]);

impl ShopRecord<'_> {
    pub fn item(&self, slot: usize) -> Result<i16, ContentError> {
        word(self.0, slot * 2)
    }
    pub fn quantity(&self, slot: usize) -> u8 {
        self.0[QUANTITIES + slot]
    }
    pub fn price_percent(&self) -> Result<i16, ContentError> {
        word(self.0, QUANTITIES + SHOP_SLOTS)
    }
}

pub struct ShopRecordMut<'a>(pub &'a mut [u8]);

impl ShopRecordMut<'_> {
    pub fn set_stock(&mut self, slot: usize, id: i16, quantity: u8) {
        put_word(self.0, slot * 2, id);
        self.0[QUANTITIES + slot] = quantity;
    }

    pub fn buy_one(&mut self, slot: usize) {
        let quantity = &mut self.0[QUANTITIES + slot];
        *quantity = quantity.wrapping_sub(1);
        if *quantity == 0 {
            self.0[slot * 2..slot * 2 + 2].fill(0);
        }
    }
    /// The caller searches an existing ID before an empty row in that family.
    /// A full family discards the sold item; payment is still owned by the caller.
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
