//! Catalog for shop.
use super::access::current_work;
use crate::{RuleError, Session};
use centurions_content::records::{ItemDefinition, SHOP_SLOTS};
use centurions_content::{Content, InventoryRecord};

pub fn stock(state: &Session) -> Result<Vec<(usize, i16, u8)>, RuleError> {
    stock_rows(state, 0..SHOP_SLOTS)
}

/// updateshop reads the selected 200-slot family, independently of the others.
pub fn stock_family(state: &Session, family: usize) -> Result<Vec<(usize, i16, u8)>, RuleError> {
    if family >= 5 {
        return Err("shop family outside the native stock".into());
    }
    stock_rows(state, family * 200..(family + 1) * 200)
}

fn stock_rows(
    state: &Session,
    rows: std::ops::Range<usize>,
) -> Result<Vec<(usize, i16, u8)>, RuleError> {
    let work = current_work(state)?;
    let mut stock = Vec::new();
    for slot in rows {
        let quantity = work.quantity(slot)?;
        if quantity != 0 {
            stock.push((slot, work.item(slot)?, quantity));
        }
    }
    Ok(stock)
}

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
    super::pricing::percentage_price(cost, current_work(state)?.read_word(3000)?, selling)
}

/// installitem 0x12c874 fresh-stock branch, without its randomized-loot branch.
pub fn preview(content: &Content, id: i16) -> Result<InventoryRecord, RuleError> {
    let item = crate::ammunition::definition(content, id)?;
    Ok(ItemDefinition(&item).fresh_stock()?)
}

/// updateshop139244 stock rows preserve the two unwritten instance bytes.
pub(crate) fn display_record(
    state: &mut Session,
    content: &Content,
    id: i16,
) -> Result<(), RuleError> {
    state.load_item_work(content, id)?;
    let item = crate::ammunition::definition(content, id)?;
    let record = &mut state.native_work.inventory_record;
    Ok(ItemDefinition(&item).copy_display_stock_into(record)?)
}

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
        // installitem randomizes scratch slot0 before installing the stock row.
        state.native_work.loot_quality = None;
    }
    Ok(item)
}
