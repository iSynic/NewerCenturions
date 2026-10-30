//! Opening the Items screen, changing its owner, scrolling and redrawing rows.
use crate::{Content, Event, Phase, Presentation, RuleError, Session, combat, items};
use centurions_content::records::{Condition, InventorySlots, ItemDefinition};

/// Opens the Items screen for a warrior, tidying the pack, drawing the rows from the top and
/// showing the detail panel when the preference is on.
pub(crate) fn open(
    state: &mut Session,
    content: &Content,
    owner: usize,
    quick_info: bool,
) -> Result<Vec<Event>, RuleError> {
    admit(state, owner)?;
    state.globals.health_context.marks_affected = true;
    state.globals.party_track[1..].fill(0);
    let tail = &mut state.globals.track_tail;
    // Only targets 1..7 clear; target 0 and targets 8..109 keep their marks.
    tail[..2].fill(0);
    state.globals.item_used = 0;
    let events = vec![Event::Sound { resource_id: 20001 }];
    // The screen opens at the top, so the detail shows the first compacted slot.
    initialize_owner(state, content, owner, 0, quick_info.then_some(0))?;
    Ok(events)
}

/// Changes whose pack the Items screen shows, keeping the scroll position. Nothing happens when
/// the owner is already the one shown.
pub(crate) fn switch(
    state: &mut Session,
    content: &Content,
    owner: usize,
    scroll: usize,
    quick_info: Option<i16>,
) -> Result<Vec<Event>, RuleError> {
    admit(state, owner)?;
    let selected = state.globals.health_context.char_select_new;
    if selected == owner {
        return Ok(vec![]);
    }
    let row = quick_info;
    check_scroll(scroll)?;
    // The highlighted row is kept but clamped to the new owner's rows; the
    // detail then shows row + scroll.
    let count = InventorySlots(&state.warriors[owner].bytes).count()?;
    let slot = row.map(|row| {
        let row = if row >= count { count - 1 } else { row };
        row.max(0).wrapping_add(scroll as i16)
    });
    let slot = slot.and_then(|slot| usize::try_from(slot).ok());
    initialize_owner(state, content, owner, scroll, slot)?;
    Ok(vec![])
}

/// Sets the screen up for an owner: the pack is tidied and the rows drawn from the scroll position,
/// with the detail panel for the selected slot if asked.
fn initialize_owner(
    state: &mut Session,
    content: &Content,
    owner: usize,
    scroll: usize,
    quick_info_slot: Option<usize>,
) -> Result<(), RuleError> {
    // The owner changes without changing the warrior whose turn it is. Opening
    // the screen resets the scroll; an owner change keeps it.
    state.set_warrior_selector(owner);
    state.globals.inventory_scroll = scroll as u16;
    items::compact(&mut state.warriors[owner])?;
    redraw(state, content, owner, scroll, 0, 10)?;
    // The detail panel runs between the redraw and clearing Helpless.
    if let Some(slot) = quick_info_slot {
        super::quick_info(state, content, owner, slot)?;
    }
    resume(state, content, owner)?;
    Ok(())
}

/// Scrolls the Items rows by a number of rows. Each step plays the scroll sound, and a step that
/// cannot move also plays the failure sound; scrolling down from scroll position 30 does nothing.
/// The scroll arrows redraw without the detail panel.
pub(crate) fn scroll(
    state: &mut Session,
    content: &Content,
    owner: usize,
    mut position: usize,
    delta: i16,
    quick_info: bool,
) -> Result<Vec<Event>, RuleError> {
    admit(state, owner)?;
    // The scroll arrows redraw rows without the detail panel.
    let _ = quick_info;
    check_scroll(position)?;
    let selected = state.globals.health_context.char_select_new;
    if selected != owner {
        return Err("inventory scroll owner is not selected".into());
    }
    let count = state.warriors[owner].inventory()?.len();
    let mut events = vec![];
    for _ in 0..delta.unsigned_abs() {
        if delta > 0 && position == 30 {
            continue;
        }
        events.push(Event::Sound { resource_id: 141 });
        if delta > 0 && position + 11 < count {
            position += 1;
            redraw(state, content, owner, position, 10, 10)?;
        } else if delta < 0 && position > 0 {
            position -= 1;
            redraw(state, content, owner, position, 0, 0)?;
        } else {
            events.push(Event::Sound { resource_id: 6000 });
        }
    }
    state.globals.inventory_scroll = position as u16;
    Ok(events)
}

/// A row click highlights the row and, with the quick-info preference, shows
/// the detail for that slot.
///
/// Original: items, quickinfo
pub(crate) fn highlight(
    state: &mut Session,
    content: &Content,
    owner: usize,
    scroll: usize,
    row: usize,
    quick_info: bool,
) -> Result<Vec<Event>, RuleError> {
    admit(state, owner)?;
    check_scroll(scroll)?;
    if row > 10 {
        return Err("inventory row outside the eleven display rows".into());
    }
    let selected = state.globals.health_context.char_select_new;
    if selected != owner {
        return Err("inventory row owner is not selected".into());
    }
    state.globals.inventory_scroll = scroll as u16;
    if quick_info {
        super::quick_info(state, content, owner, scroll + row)?;
    }
    Ok(vec![])
}

/// Checks that the Items screen can be opened for an owner: in exploration or battle, with the
/// owner in the squad and nothing but a typed prompt on screen.
pub(super) fn admit(state: &Session, owner: usize) -> Result<(), RuleError> {
    if !matches!(state.phase, Phase::Exploration | Phase::Combat)
        || owner >= state.warriors.len()
        || state.exploration.as_ref().is_none_or(|position| {
            position
                .presentation
                .as_ref()
                .is_some_and(|page| !matches!(page, Presentation::Typed { .. }))
        })
    {
        return Err("inventory entry requires an available owner and caller".into());
    }
    Ok(())
}

/// Checks that the scroll position is one the screen can reach.
pub(super) fn check_scroll(scroll: usize) -> Result<(), RuleError> {
    if scroll > 30 {
        return Err("inventory scroll outside native rows".into());
    }
    Ok(())
}

/// Redraws the owner's rows, loading each nonempty row in order.
///
/// Original: updateitems
pub(super) fn redraw(
    state: &mut Session,
    content: &Content,
    owner: usize,
    scroll: usize,
    first: usize,
    last: usize,
) -> Result<(), RuleError> {
    state.globals.character_bare_hand_bonus = state.warriors[owner].record().bare_hand_bonus()?;
    for row in first..=last {
        let record = state.warriors[owner].record().inventory_slot(scroll + row);
        if record.id() != 0 {
            state.load_item(content, record.id())?;
            state.globals.inventory_record = record;
        }
    }
    Ok(())
}

/// Clears every member's Helpless condition, then weighs the owner's load and
/// refreshes the owner's information.
fn resume(state: &mut Session, content: &Content, owner: usize) -> Result<(), RuleError> {
    for warrior in &mut state.warriors {
        warrior.record_mut().set_condition(Condition::Helpless, 0);
    }
    calcw(state, content, owner)?;
    combat::item_activation::refresh_inventory_info_items(state, content, owner)
}

/// Weighs a warrior's load: each carried item's weight, loaded in turn.
///
/// Original: calcw
pub(crate) fn calcw(state: &mut Session, content: &Content, owner: usize) -> Result<(), RuleError> {
    state.warriors[owner].record_mut().set_load(0);
    let count = state.warriors[owner].inventory()?.len();
    for slot in 0..count {
        let record = state.warriors[owner].record().inventory_slot(slot);
        state.load_item(content, record.id())?;
        let definition = ItemDefinition(state.globals.loaded_items.item.as_ref().unwrap());
        let weight = state.warriors[owner]
            .record()
            .load()?
            .wrapping_add(definition.charge_weight()?.wrapping_mul(record.charge()?))
            .wrapping_add(definition.weight()?);
        state.warriors[owner].record_mut().set_load(weight);
    }
    Ok(())
}
