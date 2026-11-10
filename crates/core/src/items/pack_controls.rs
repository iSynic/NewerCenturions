//! The Items window's Use, Drop, Join, Split and Done buttons.
//!
//! Drop puts a copy of the item in the crate before it checks whether the item may be
//! dropped, so a worn item, or another warrior's item in battle, leaves its copy in the crate.
//! Join gathers every stack of the same item into the first, and Split halves a stack into a new
//! slot at the end of the pack.
use super::inventory_opening::{admit, check_scroll, redraw};
use crate::combat::item_activation::refresh_inventory_info_items;
use crate::feedback::GameFeedback;
use crate::{Event, RuleError, Session, combat, equipment};
use centurions_content::Content;
use centurions_content::records::{InventorySlots, ItemDefinition};

/// The most slots a pack holds before Split refuses.
const SPLIT_LIMIT: i16 = 30;
/// The slots Join searches: every slot of the record, whatever the item count.
const JOIN_SLOTS: usize = 100;

/// A warning that leaves what the button has already done in place.
fn warning(mut events: Vec<Event>, index: i16) -> Result<Vec<Event>, RuleError> {
    events.push(Event::Feedback(GameFeedback::Warning { index }));
    Ok(events)
}

/// Checks the owner and row a button acts on and returns the slot.
fn slot(state: &Session, owner: usize, scroll: usize, row: usize) -> Result<usize, RuleError> {
    admit(state, owner)?;
    check_scroll(scroll)?;
    if row > 10 {
        return Err("inventory row outside the eleven display rows".into());
    }
    if state.globals.health_context.char_select_new != owner {
        return Err("inventory owner is not selected".into());
    }
    Ok(scroll + row)
}

/// The loaded item's definition.
fn loaded(state: &Session) -> Result<ItemDefinition<'_>, RuleError> {
    state
        .globals
        .loaded_items
        .item
        .as_deref()
        .map(ItemDefinition)
        .ok_or_else(|| "no item loaded".into())
}

/// Use: an empty pack does nothing. Otherwise the remembered Psi power is forgotten, and in a
/// battle the owner must be the warrior whose turn it is (warning 10) with an action point left
/// (warning 17); the forgetting stands when either refuses. The button then pauses and the item
/// is used.
///
/// Original: items
pub(crate) fn press_use(
    state: &mut Session,
    content: &Content,
    owner: usize,
    (scroll, row): (usize, usize),
) -> Result<Vec<Event>, RuleError> {
    let slot = slot(state, owner, scroll, row)?;
    if InventorySlots(&state.warriors[owner].bytes).count()? == 0 {
        return Ok(vec![]);
    }
    state.globals.memory_spell = 0;
    if let Some(combat) = state.combat.as_ref() {
        if combat.active_body() != Some(owner as i16) {
            return warning(vec![], 10);
        }
        if state.warriors[owner].record().action_points()? < 1 {
            return warning(vec![], 17);
        }
    }
    let mut events = vec![Event::Delay { ticks: 10 }];
    events.extend(crate::exploration::use_item(state, content, owner, slot)?);
    Ok(events)
}

/// Done: the owner's pack is tidied as the window closes.
///
/// Original: items, compactitems
pub(crate) fn close(state: &mut Session, owner: usize) -> Result<Vec<Event>, RuleError> {
    admit(state, owner)?;
    super::compact(&mut state.warriors[owner])?;
    Ok(vec![])
}

/// Drop: an item of classes 23 to 25 needs the Command key (warning 72), and with Drop Item
/// Protection on so does every item (warning 75). The item's copy then goes to the crate (a full
/// crate gives warning 153); another warrior's item in battle (warning 10) or a worn item
/// (warning 5) stays in the pack. Otherwise the item is dropped with its sound and the rows from
/// the selected one down are redrawn.
///
/// Original: items, addtocrate, dropitem
pub(crate) fn drop(
    state: &mut Session,
    content: &Content,
    owner: usize,
    (scroll, row): (usize, usize),
    command_key: bool,
    protected: bool,
) -> Result<Vec<Event>, RuleError> {
    let slot = slot(state, owner, scroll, row)?;
    let record = state.warriors[owner].record().inventory_slot(slot);
    state.load_item(content, record.id())?;
    if !command_key && matches!(loaded(state)?.item_type()?, 23..=25) {
        return warning(vec![], 72);
    }
    if protected && !command_key {
        return warning(vec![], 75);
    }
    let mut events = crate::commands::insert(record.clone(), &mut state.crate_items, true)?;
    if events.contains(&Event::StorageFull) {
        return warning(events, 153);
    }
    if state.combat.is_some()
        && state.combat.as_ref().and_then(combat::Combat::active_body) != Some(owner as i16)
    {
        return warning(events, 10);
    }
    if record.equipped() != 0 {
        return warning(events, 5);
    }
    if InventorySlots(&state.warriors[owner].bytes).count()? == 0 {
        return Ok(events);
    }
    if record.id() == 0 {
        state.globals.inventory_scroll = 0;
        redraw(state, content, owner, 0, 0, 10)?;
        return Ok(events);
    }
    let mission = state
        .exploration
        .as_ref()
        .and_then(|p| content.map(p.map))
        .ok_or("drop profiles missing")?;
    state.globals.inventory_record = record;
    let globals = equipment::drop(&mut state.warriors[owner], slot, content, mission)?;
    state.apply_equipment_globals(&globals)?;
    events.push(Event::Sound { resource_id: 655 });
    redraw(state, content, owner, scroll, row, 10)?;
    refresh_inventory_info_items(state, content, owner)?;
    Ok(events)
}

/// Join: an item without a charge weight cannot be joined (warning 41). Every slot holding the
/// same item adds its charge to the first such slot, which is worn if any of them was, and the
/// others are dropped without changing the load. Each slot found plays the join sound; the pack is
/// then tidied and redrawn from the top row.
///
/// Original: items, dropitem, compactitems
pub(crate) fn join(
    state: &mut Session,
    content: &Content,
    owner: usize,
    (scroll, row): (usize, usize),
) -> Result<Vec<Event>, RuleError> {
    let slot = slot(state, owner, scroll, row)?;
    let id = state.warriors[owner].record().inventory_slot(slot).id();
    state.load_item(content, id)?;
    if loaded(state)?.charge_weight()? == 0 {
        return warning(vec![], 41);
    }
    let mission = state
        .exploration
        .as_ref()
        .and_then(|p| content.map(p.map))
        .ok_or("join profiles missing")?;
    let mut events = vec![];
    let mut first = None;
    let mut worn = 0;
    let mut total = 0_i16;
    let mut index = 0;
    while index < JOIN_SLOTS {
        let item = state.warriors[owner].record().inventory_slot(index);
        if item.id() == id {
            events.push(Event::Sound { resource_id: 663 });
            if item.equipped() != 0 {
                worn = 1;
            }
            total = total.wrapping_add(item.charge()?);
            if first.is_some() {
                let definition = loaded(state)?;
                let weight = definition
                    .charge_weight()?
                    .wrapping_mul(item.charge()?)
                    .wrapping_add(definition.weight()?);
                let load = state.warriors[owner].record().load()?.wrapping_add(weight);
                state.warriors[owner].record_mut().set_load(load);
                let globals = equipment::drop(&mut state.warriors[owner], index, content, mission)?;
                state.apply_equipment_globals(&globals)?;
                index = index.wrapping_sub(1);
            } else {
                first = Some(index);
            }
        }
        index = index.wrapping_add(1);
    }
    if let Some(first) = first {
        let mut item = state.warriors[owner].record().inventory_slot(first);
        item.set_equipped(worn);
        item.set_charge(total);
        state.warriors[owner]
            .record_mut()
            .set_inventory_slot(first, &item);
    }
    super::compact(&mut state.warriors[owner])?;
    redraw(state, content, owner, scroll, 0, 10)?;
    Ok(events)
}

/// Split: a full pack cannot split (warning 43), nor a stack of fewer than two or an item without
/// a charge weight (warning 42). Otherwise a copy of the stack, not worn, goes after the last
/// item, each half keeping half the charge and the original the odd one.
///
/// Original: items
pub(crate) fn split(
    state: &mut Session,
    content: &Content,
    owner: usize,
    (scroll, row): (usize, usize),
) -> Result<Vec<Event>, RuleError> {
    let slot = slot(state, owner, scroll, row)?;
    let events = vec![
        Event::Sound { resource_id: 678 },
        Event::Delay { ticks: 10 },
    ];
    let mut item = state.warriors[owner].record().inventory_slot(slot);
    let charge = item.charge()?;
    let count = InventorySlots(&state.warriors[owner].bytes).count()?;
    state.load_item(content, item.id())?;
    if count >= SPLIT_LIMIT {
        return warning(events, 43);
    }
    if charge < 2 || loaded(state)?.charge_weight()? == 0 {
        return warning(events, 42);
    }
    let half = charge / 2;
    let mut copy = item.clone();
    copy.set_equipped(0);
    copy.set_charge(half);
    item.set_charge(if half.wrapping_add(half) < charge {
        half.wrapping_add(1)
    } else {
        half
    });
    let mut record = state.warriors[owner].record_mut();
    record.set_inventory_slot(slot, &item);
    record.set_inventory_slot(count as usize, &copy);
    centurions_content::records::InventorySlotsMut(&mut state.warriors[owner].bytes)
        .set_count(count.wrapping_add(1));
    redraw(state, content, owner, scroll, row, 10)?;
    Ok(events)
}
