//! The shop's stock as the window lists it.
use super::access::current_shop;
use crate::{RuleError, Session};
use centurions_content::records::{ItemDefinition, SHOP_SLOTS};
use centurions_content::{Content, InventoryRecord};

/// The stock the shop window lists: slot, item and quantity for each slot with items.
pub fn stock(state: &Session) -> Result<Vec<(usize, i16, u8)>, RuleError> {
    stock_rows(state, 0..SHOP_SLOTS)
}

/// One 200-slot family of the shop's stock, independent of the others.
///
/// Original: updateshop
pub fn stock_family(state: &Session, family: usize) -> Result<Vec<(usize, i16, u8)>, RuleError> {
    if family >= 5 {
        return Err("shop family outside the native stock".into());
    }
    stock_rows(state, family * 200..(family + 1) * 200)
}

/// The stock rows of a range of slots, skipping empty ones.
fn stock_rows(
    state: &Session,
    rows: std::ops::Range<usize>,
) -> Result<Vec<(usize, i16, u8)>, RuleError> {
    let shop = current_shop(state)?;
    let mut stock = Vec::new();
    for slot in rows {
        let quantity = shop.quantity(slot)?;
        if quantity != 0 {
            stock.push((slot, shop.item(slot)?, quantity));
        }
    }
    Ok(stock)
}

/// The price of an item at the current shop: its listed price, divided by (10 plus twice the
/// difficulty) when the party sells, then scaled by the shop's percentage.
pub fn price(state: &Session, content: &Content, id: i16, selling: bool) -> Result<i32, RuleError> {
    let definition = content
        .items
        .get(id.unsigned_abs() as usize)
        .ok_or("shop item unavailable")?;
    let mut cost = ItemDefinition(definition).price()?.wrapping_abs();
    if selling {
        cost = cost
            .checked_div(i32::from(state.difficulty.wrapping_mul(2).wrapping_add(10)))
            .ok_or("native shop division traps")?;
    }
    super::pricing::percentage_price(cost, current_shop(state)?.read_word(3000)?, selling)
}

/// A fresh stock item, without randomized loot.
///
/// Original: installitem
pub fn preview(content: &Content, id: i16) -> Result<InventoryRecord, RuleError> {
    let item = crate::ammunition::definition(content, id)?;
    Ok(ItemDefinition(&item).fresh_stock()?)
}

/// A stock row as the shop shows it; the two instance bytes it never sets keep
/// their values.
///
/// Original: updateshop
pub(crate) fn display_record(
    state: &mut Session,
    content: &Content,
    id: i16,
) -> Result<(), RuleError> {
    state.load_item(content, id)?;
    let item = crate::ammunition::definition(content, id)?;
    let record = &mut state.globals.inventory_record;
    Ok(ItemDefinition(&item).copy_display_stock_into(record)?)
}

/// A fresh item for a shop that stocks enhanced items.
pub(super) fn enhanced_stock(
    state: &mut Session,
    content: &Content,
    id: i16,
) -> Result<InventoryRecord, RuleError> {
    let enhancement = state.shop_access.as_ref().unwrap().enhancement;
    let item = super::generated_stock::generate(
        &content.items,
        id,
        enhancement,
        state.difficulty,
        &mut state.random,
    )?;
    if enhancement != 0 {
        // installitem randomizes a scratch item before installing the stock row.
        state.globals.loot_quality = None;
    }
    Ok(item)
}
