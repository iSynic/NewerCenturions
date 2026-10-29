//! Items activation through its caller-dependent combat return.
use crate::{Content, Event, RuleError, Session, feedback::NativeFeedback};
pub(crate) mod ordinary;
mod selected;
pub(crate) use selected::prepare_selected;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Caller {
    Ordinary,
    Encounter,
}

pub(crate) struct Activation {
    pub item: i16,
    pub spell: i16,
    pub events: Vec<Event>,
}

/// Items scroll is0..30 and its selected display row is0..10.
pub(crate) fn check_selected_slot(slot: usize) -> Result<(), RuleError> {
    if slot > 40 {
        return Err("selected item outside native inventory viewport".into());
    }
    Ok(())
}

/// The explicit inventory-owner selection is BYTE44273f. Ordinary Combatchoice
/// supplies it at119000; Items' character chooser supplies it at12bdb6.
pub(crate) fn activate(
    state: &mut Session,
    content: &Content,
    owner: usize,
    slot: usize,
    caller: Caller,
) -> Result<Activation, RuleError> {
    crate::exploration::check_item_actor(state, owner)?;
    check_selected_slot(slot)?;
    // Commands name the selected inventory row; opening the pane is a separate
    // caller boundary from this explicit owner choice and activation.
    state.set_warrior_selector(owner);
    activate_selected(state, content, owner, slot, caller)
}

/// Post-selection Items activation. The caller supplies the already chosen
/// inventory owner/slot and literal work after compactitems/calcw and UI setup.
/// This boundary preserves that earlier setup and the selected row.
pub(crate) fn activate_selected(
    state: &mut Session,
    content: &Content,
    owner: usize,
    slot: usize,
    caller: Caller,
) -> Result<Activation, RuleError> {
    let result = prepare_selected(state, content, owner, slot, caller)?;
    if result.spell != 0 {
        state.set_ready(1);
    }
    if result.spell != 0 || caller == Caller::Encounter {
        finish_return(state, owner, caller);
    }
    Ok(result)
}

/// The common Items12c422/12c452 RTS tail, after any outdoor resolution.
pub(crate) fn finish_return(state: &mut Session, owner: usize, caller: Caller) {
    if caller == Caller::Encounter {
        state.native_work.party_mask[owner] = 1;
    }
    let health = &mut state.native_work.health_context;
    health.marks_affected = false;
}

/// Items12bbba drops slot WORD4471aa + WORD44719c and calls updateitems160d08
/// from row WORD4471aa through row10, so the selected slot always lies in the
/// retained scroll viewport.
fn retain_drop_redraw(state: &mut Session, owner: usize, slot: usize) -> Result<(), RuleError> {
    let warrior = &state.warriors[owner];
    state.native_work.character_bare_hand_bonus = warrior.record().bare_hand_bonus()?;
    let scroll = usize::from(state.native_work.inventory_scroll);
    if !(scroll..=scroll + 10).contains(&slot) {
        return Err("selected item outside native inventory viewport".into());
    }
    for index in slot..=scroll + 10 {
        let record = warrior.record().inventory_slot(index);
        if record.id() != 0 {
            state.native_work.inventory_record = record;
        }
    }
    Ok(())
}

/// Items12bc20 calls updatecharinfo after the drop and inventory redraw.
/// Its two calcdamage calls load displayed equipment independently of firing mode.
pub(crate) fn refresh_inventory_info_work(
    state: &mut Session,
    content: &Content,
    owner: usize,
) -> Result<(), RuleError> {
    let record = state.warriors[owner].record();
    let firearm = record.displayed_weapon(true)?;
    let ammunition = record.loaded_ammunition()?;
    let melee = record.displayed_weapon(false)?;
    if firearm != 0 {
        state.load_item_work(content, firearm)?;
        // calcdamage13706e loads AMMO only inside the nonzero equipment branch.
        if ammunition != 0 {
            let definition = crate::ammunition::definition(content, ammunition)?;
            state.native_work.weapon_work.load_ammunition(&definition)?;
            if let Some(combat) = state.combat.as_mut() {
                combat.weapon_work.load_ammunition(&definition)?;
            }
            state.native_work.ammunition_work.ammo = Some(definition);
        }
    }
    if melee != 0 {
        state.load_item_work(content, melee)?;
    }
    Ok(())
}

fn refusal<T>(index: i16) -> Result<T, RuleError> {
    Err(RuleError::RefusedAfterEffects {
        feedback: vec![NativeFeedback::Warning { index }],
    })
}

/// encounter21229be updates the main selection before matching the returned ID.
pub(crate) fn encounter_return(
    state: &mut Session,
    content: &Content,
    item: i16,
) -> Result<Option<i8>, RuleError> {
    clear_active_affected(state);
    // encounter2 1229ac keeps Items' returned ID in WORD44719a.
    state.native_work.fire_item = item;
    if item == 0 {
        return Ok(None);
    }
    crate::typed_encounter::select_item(&crate::exploration::typed_record(state, content)?, item)
        .map(Some)
}

pub(crate) fn clear_active_affected(state: &mut Session) {
    // updatemain's selectupdate13a234 consumes only the active party marks.
    let health = &mut state.native_work.health_context;
    health.affected[..state.warriors.len()].fill(0);
}
