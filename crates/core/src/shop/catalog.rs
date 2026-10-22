//! Catalog for shop.
use super::records;
use crate::{RuleError, Session};
use centurions_content::records::{ItemDefinition, SHOP_SLOTS, ShopRecord};
use centurions_content::{Content, InventoryRecord};

pub fn stock(state: &Session) -> Result<Vec<(usize, i16, u8)>, RuleError> {
    let records = ShopRecord(records(state)?);
    (0..SHOP_SLOTS)
        .filter(|i| records.quantity(*i) != 0)
        .map(|i| Ok((i, records.item(i)?, records.quantity(i))))
        .collect()
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
    super::pricing::percentage_price(cost, ShopRecord(records(state)?).price_percent()?, selling)
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
    let Some(record) = state.native_work.inventory_record.as_mut() else {
        return Ok(());
    };
    Ok(ItemDefinition(&item).copy_display_stock_into(record)?)
}

pub(super) fn enhanced_stock(
    state: &mut Session,
    content: &Content,
    id: i16,
) -> Result<InventoryRecord, RuleError> {
    super::generated_stock::generate(
        &content.items,
        id,
        state.shop_access.as_ref().unwrap().enhancement,
        state.difficulty,
        &mut state.random,
    )
}
