//! Items entry, owner change and exposed-row redraws, with quickinfo on entry.
use crate::{Content, Event, Phase, Presentation, RuleError, Session, combat, items};
use centurions_content::records::{Condition, InventorySlots, ItemDefinition};

pub(crate) fn open(
    state: &mut Session,
    content: &Content,
    owner: usize,
    quick_info: bool,
) -> Result<Vec<Event>, RuleError> {
    admit(state, owner)?;
    state.native_work.health_context.marks_affected = true;
    state.native_work.party_mask[1..].fill(0);
    let tail = &mut state.native_work.spell_target_tail;
    // 12959c clears WORDs1..7 only. Target0 and words8..109 survive.
    tail[..2].fill(0);
    // 1299be clears WORD44719a.
    state.native_work.fire_item = 0;
    // 1299e8 precedes the initial compaction and redraw.
    let events = vec![Event::Sound { resource_id: 20001 }];
    // Entry shows display row0 at scroll0, so quickinfo reads the compacted slot0.
    initialize_owner(state, content, owner, 0, quick_info.then_some(0))?;
    Ok(events)
}

pub(crate) fn switch(
    state: &mut Session,
    content: &Content,
    owner: usize,
    scroll: usize,
    quick_info: Option<i16>,
) -> Result<Vec<Event>, RuleError> {
    admit(state, owner)?;
    let selected = state.native_work.health_context.selected_warrior;
    // 12bda8/12bdae skips the whole owner-change body for the current owner.
    if selected == owner {
        return Ok(vec![]);
    }
    let row = quick_info;
    check_scroll(scroll)?;
    // updateitems160d08 clamps the kept display row to the new owner's rows
    // (count-1, and -1 to 0); quickinfo then reads the WORD row+scroll slot.
    let count = InventorySlots(&state.warriors[owner].bytes).count()?;
    let slot = row.map(|row| {
        let row = if row >= count { count - 1 } else { row };
        row.max(0).wrapping_add(scroll as i16)
    });
    let slot = slot.and_then(|slot| usize::try_from(slot).ok());
    initialize_owner(state, content, owner, scroll, slot)?;
    Ok(vec![])
}

fn initialize_owner(
    state: &mut Session,
    content: &Content,
    owner: usize,
    scroll: usize,
    quick_info_slot: Option<usize>,
) -> Result<(), RuleError> {
    // 12bdb6 changes the BYTE owner, preserving the selected-warrior WORD.
    // Entry clears the scroll WORD44719c (1299c4); an owner change keeps the shown scroll.
    state.set_warrior_selector(owner);
    state.native_work.inventory_scroll = scroll as u16;
    items::compact(&mut state.warriors[owner])?;
    redraw(state, content, owner, scroll, 0, 10)?;
    // 129b62 runs quickinfo132b0c between the redraw and the Helpless suffix.
    if let Some(slot) = quick_info_slot {
        super::quick_info(state, content, owner, slot)?;
    }
    resume(state, content, owner)?;
    Ok(())
}

pub(crate) fn scroll(
    state: &mut Session,
    content: &Content,
    owner: usize,
    mut position: usize,
    delta: i16,
    quick_info: bool,
) -> Result<Vec<Event>, RuleError> {
    admit(state, owner)?;
    // The scroll arrows redraw rows without calling quickinfo132b0c.
    let _ = quick_info;
    check_scroll(position)?;
    let selected = state.native_work.health_context.selected_warrior;
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
    // The arrows step WORD44719c by one (12b30a, 12b45e).
    state.native_work.inventory_scroll = position as u16;
    Ok(events)
}

/// items12c18a: a row click stores the display row and, with the quick-info
/// preference, runs quickinfo132b0c on the row+scroll slot (12c26e).
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
    let selected = state.native_work.health_context.selected_warrior;
    if selected != owner {
        return Err("inventory row owner is not selected".into());
    }
    state.native_work.inventory_scroll = scroll as u16;
    if quick_info {
        super::quick_info(state, content, owner, scroll + row)?;
    }
    Ok(vec![])
}

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

pub(super) fn check_scroll(scroll: usize) -> Result<(), RuleError> {
    if scroll > 30 {
        return Err("inventory scroll outside native rows".into());
    }
    Ok(())
}

/// updateitems160d08 copies the owner, then loads and caches nonempty rows in order.
pub(super) fn redraw(
    state: &mut Session,
    content: &Content,
    owner: usize,
    scroll: usize,
    first: usize,
    last: usize,
) -> Result<(), RuleError> {
    state.native_work.character_bare_hand_bonus =
        state.warriors[owner].record().bare_hand_bonus()?;
    for row in first..=last {
        let record = state.warriors[owner].record().inventory_slot(scroll + row);
        if record.id() != 0 {
            state.load_item_work(content, record.id())?;
            state.native_work.inventory_record = record;
        }
    }
    Ok(())
}

/// 129aca..129e0a clears every member's Helpless word, then calcw and owner info.
fn resume(state: &mut Session, content: &Content, owner: usize) -> Result<(), RuleError> {
    for warrior in &mut state.warriors {
        warrior.record_mut().set_condition(Condition::Helpless, 0);
    }
    calcw(state, content, owner)?;
    combat::item_activation::refresh_inventory_info_work(state, content, owner)
}

/// calcw 0x139192 clears the carried weight, then loads each carried item through ITEM
/// work and adds its weight.
pub(crate) fn calcw(state: &mut Session, content: &Content, owner: usize) -> Result<(), RuleError> {
    state.warriors[owner].record_mut().set_carried_weight(0);
    let count = state.warriors[owner].inventory()?.len();
    for slot in 0..count {
        let record = state.warriors[owner].record().inventory_slot(slot);
        state.load_item_work(content, record.id())?;
        let definition = ItemDefinition(state.native_work.ammunition_work.item.as_ref().unwrap());
        let weight = state.warriors[owner]
            .record()
            .carried_weight()?
            .wrapping_add(definition.unit_weight()?.wrapping_mul(record.quantity()?))
            .wrapping_add(definition.base_weight()?);
        state.warriors[owner]
            .record_mut()
            .set_carried_weight(weight);
    }
    Ok(())
}
