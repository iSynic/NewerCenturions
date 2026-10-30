//! Inspecting equipment in the shop window.
use crate::{Event, RuleError, Session};
use centurions_content::records::InventorySlots;
use centurions_content::{Content, InventoryRecord};

/// Shows all nine visible rows, then the selected owned row.
///
/// Original: updateshop
pub(crate) fn inspect(
    state: &mut Session,
    content: &Content,
    members: [usize; 2],
    scroll: [usize; 2],
    selected: Option<(usize, usize)>,
    shop_family: usize,
) -> Result<Vec<Event>, RuleError> {
    if state.exploration.as_ref().is_some_and(|position| {
        position.presentation.as_ref().is_some_and(|presentation| {
            !matches!(
                presentation,
                crate::Presentation::Shop | crate::Presentation::Typed { .. }
            )
        })
    }) {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let shop = state
        .exploration
        .as_ref()
        .is_some_and(|position| position.presentation == Some(crate::Presentation::Shop));
    for side in 0..2 {
        if shop && side == 1 {
            if shop_family >= 5 || scroll[side] >= 1000 {
                return Err("shop inventory view outside native rows".into());
            }
            let rows = crate::shop::stock_family(state, shop_family)?;
            for row in scroll[side]..scroll[side] + 9 {
                let id = rows.get(row).map_or(0, |(_, id, _)| *id);
                crate::shop::display_record(state, content, id)?;
            }
            continue;
        }
        let warrior = state
            .warriors
            .get(members[side])
            .ok_or("inventory member missing")?;
        let row = scroll[side]
            .checked_add(8)
            .filter(|row| *row < 100)
            .ok_or("inventory scroll outside native rows")?;
        let record = InventorySlots(&warrior.bytes).slot(row)?;
        display_owned_row(state, content, record)?;
    }
    if let Some((side, slot)) = selected {
        if side > 1 || slot >= if shop && side == 1 { 1000 } else { 100 } {
            return Err("inventory selection outside native rows".into());
        }
        if shop && side == 1 {
            let id = crate::shop::stock_family(state, shop_family)?
                .into_iter()
                .find(|(index, _, _)| *index == slot)
                .ok_or("selected shop stock missing")?
                .1;
            if usize::from(id.unsigned_abs()) / 200 != shop_family {
                return Err("selected shop stock outside displayed family".into());
            }
            crate::shop::display_record(state, content, id)?;
        } else {
            let record = state.warriors[members[side]]
                .inventory()?
                .get(slot)
                .cloned()
                .ok_or("inventory selection missing")?;
            display_owned_row(state, content, record)?;
        }
    }
    Ok(vec![])
}

/// Loads the item's definition as stored and clears the icon of an empty owned row.
///
/// Original: ploticon
fn display_owned_row(
    state: &mut Session,
    content: &Content,
    record: InventoryRecord,
) -> Result<(), RuleError> {
    let mut definition = content
        .items
        .get(usize::from(record.id().unsigned_abs()))
        .ok_or("inventory display definition missing")?
        .clone();
    if record.id() == 0 {
        definition[10..12].fill(0);
    }
    state.globals.loaded_item_field32 = centurions_content::word(&definition, 32)?;
    state.globals.loaded_items.item = Some(definition);
    state.globals.inventory_record = record;
    Ok(())
}
