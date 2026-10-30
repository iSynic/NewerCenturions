//! Using an item from the Items screen, and the return to whoever opened it.
use crate::{Content, Event, RuleError, Session, feedback::GameFeedback};
pub(crate) mod ordinary;
mod selected;
pub(crate) use selected::prepare_selected;

/// Who opened the Items screen: an ordinary use from the combat menu or an encounter that asked for
/// an item.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Caller {
    Ordinary,
    Encounter,
}

/// What using an item produced: the item, the spell it casts (0 for none), and the events to show.
pub(crate) struct Activation {
    pub item: i16,
    pub spell: i16,
    pub events: Vec<Event>,
}

/// Refuses an Items screen row past 40, outside the inventory viewport.
pub(crate) fn check_selected_slot(slot: usize) -> Result<(), RuleError> {
    if slot > 40 {
        return Err("selected item outside native inventory viewport".into());
    }
    Ok(())
}

/// Uses an item from a chosen warrior's inventory. The combat menu or the Items
/// screen's character chooser names the owner.
///
/// Original: Items
pub(crate) fn activate(
    state: &mut Session,
    content: &Content,
    owner: usize,
    slot: usize,
    caller: Caller,
) -> Result<Activation, RuleError> {
    crate::exploration::check_item_actor(state, owner)?;
    check_selected_slot(slot)?;
    state.set_warrior_selector(owner);
    activate_selected(state, content, owner, slot, caller)
}

/// Uses the item already chosen on the Items screen.
///
/// Original: Items
pub(crate) fn activate_selected(
    state: &mut Session,
    content: &Content,
    owner: usize,
    slot: usize,
    caller: Caller,
) -> Result<Activation, RuleError> {
    let result = prepare_selected(state, content, owner, slot, caller)?;
    if result.spell != 0 {
        state.set_in_spell(1);
    }
    if result.spell != 0 || caller == Caller::Encounter {
        finish_return(state, owner, caller);
    }
    Ok(result)
}

/// Closes the Items screen and returns to its caller.
///
/// Original: Items
pub(crate) fn finish_return(state: &mut Session, owner: usize, caller: Caller) {
    if caller == Caller::Encounter {
        state.globals.party_track[owner] = 1;
    }
    let health = &mut state.globals.health_context;
    health.marks_affected = false;
}

/// Drops the used-up item and redraws the rows from the selected one down; the
/// selected slot is always on screen.
///
/// Original: Items, updateitems
fn redraw_after_drop(state: &mut Session, owner: usize, slot: usize) -> Result<(), RuleError> {
    let warrior = &state.warriors[owner];
    state.globals.character_bare_hand_bonus = warrior.record().bare_hand_bonus()?;
    let scroll = usize::from(state.globals.inventory_scroll);
    if !(scroll..=scroll + 10).contains(&slot) {
        return Err("selected item outside native inventory viewport".into());
    }
    for index in slot..=scroll + 10 {
        let record = warrior.record().inventory_slot(index);
        if record.id() != 0 {
            state.globals.inventory_record = record;
        }
    }
    Ok(())
}

/// Refreshes the warrior panel after a drop. Its damage estimate loads the shown
/// equipment, whatever the firing mode.
///
/// Original: updatecharinfo, calcdamage
pub(crate) fn refresh_inventory_info_items(
    state: &mut Session,
    content: &Content,
    owner: usize,
) -> Result<(), RuleError> {
    let record = state.warriors[owner].record();
    let firearm = record.displayed_weapon(true)?;
    let ammunition = record.loaded_ammunition()?;
    let melee = record.displayed_weapon(false)?;
    if firearm != 0 {
        state.load_item(content, firearm)?;
        // Ammunition loads only when a weapon is equipped.
        if ammunition != 0 {
            let definition = crate::ammunition::definition(content, ammunition)?;
            state.globals.weapon_traits.load_ammunition(&definition)?;
            if let Some(combat) = state.combat.as_mut() {
                combat.weapon_traits.load_ammunition(&definition)?;
            }
            state.globals.loaded_items.ammo = Some(definition);
        }
    }
    if melee != 0 {
        state.load_item(content, melee)?;
    }
    Ok(())
}

/// Refuses with a warning message after the item's effects have started.
fn refusal<T>(index: i16) -> Result<T, RuleError> {
    Err(RuleError::RefusedAfterEffects {
        feedback: vec![GameFeedback::Warning { index }],
    })
}

/// Returns from the Items screen to an encounter: forgets the affected warriors, remembers the item
/// used, and matches it against the encounter (no match is sought when no item was used).
///
/// Original: encounter2
pub(crate) fn encounter_return(
    state: &mut Session,
    content: &Content,
    item: i16,
) -> Result<Option<i8>, RuleError> {
    clear_active_affected(state);
    // The encounter remembers which item was used.
    state.globals.item_used = item;
    if item == 0 {
        return Ok(None);
    }
    crate::typed_encounter::select_item(&crate::exploration::typed_record(state, content)?, item)
        .map(Some)
}

/// Forgets which warriors the last item affected.
pub(crate) fn clear_active_affected(state: &mut Session) {
    // Redrawing the selection clears only the active party marks.
    let health = &mut state.globals.health_context;
    health.affected[..state.warriors.len()].fill(0);
}
