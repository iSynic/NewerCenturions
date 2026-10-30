//! Using an item outdoors from the Items screen, then returning to a typed
//! encounter or the inventory.
use super::{Selection, choice_limit, resolve};
use crate::combat::item_activation::{self, Caller};
use crate::{Content, Event, Phase, Presentation, RuleError, Session};
use centurions_content::records::SpellInfoRecord;

/// Begins using an item outdoors from the Items screen, with the owner's warrior as the user.
pub(crate) fn begin_item(
    state: &mut Session,
    content: &Content,
    owner: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    if state.phase != Phase::Exploration
        || state.combat.is_some()
        || state.exploration.as_ref().is_none_or(|position| {
            (position.presentation.is_some() || position.script.is_some())
                && !matches!(position.presentation, Some(Presentation::Typed { .. }))
        })
    {
        return Err("item use requires free exploration".into());
    }
    crate::exploration::check_item_actor(state, owner)?;
    // The inventory owner is chosen apart from the warrior whose turn it is.
    state.set_warrior_selector(owner);
    super::begin_item_selected(state, content, owner, slot)
}

/// Uses an item of the chosen inventory owner, with the work after the screen.
///
/// Original: items
pub(crate) fn begin_selected(
    state: &mut Session,
    content: &Content,
    owner: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    let caller =
        if state.exploration.as_ref().is_some_and(|position| {
            matches!(position.presentation, Some(Presentation::Typed { .. }))
        }) {
            Caller::Encounter
        } else {
            Caller::Ordinary
        };
    let activation = item_activation::prepare_selected(state, content, owner, slot, caller)?;
    if activation.spell == 0 {
        return Err(RuleError::RefusedAfterEffects { feedback: vec![] });
    }
    let packet = &state.globals.spell_info;
    let mode = SpellInfoRecord(packet).target_type();
    let power = state.globals.attack_count;
    let targets = choice_limit(mode, power, state.warriors.len());
    state.outdoor_psi = Some(Selection {
        actor: owner,
        spell: activation.spell,
        power,
        remaining: targets,
        item: Some(activation.item),
    });
    let mut events = activation.events;
    if mode == 5 {
        state.globals.party_track[owner] = 1;
        events.extend(resolve(state, content)?);
    } else if (mode as i8) > 2 || targets >= state.warriors.len() as i16 {
        state.globals.party_track[..state.warriors.len()].fill(1);
        events.extend(resolve(state, content)?);
    } else {
        events.push(Event::Sound { resource_id: 6001 });
    }
    Ok(events)
}

/// Returns to the typed encounter or the inventory when the item's cast is finished.
pub(super) fn finish_return(
    state: &mut Session,
    content: &Content,
    pending: &Selection,
) -> Result<Vec<Event>, RuleError> {
    if state
        .exploration
        .as_ref()
        .is_some_and(|position| matches!(position.presentation, Some(Presentation::Typed { .. })))
    {
        item_activation::finish_return(state, pending.actor, Caller::Encounter);
        let mut events = vec![];
        if let Some(selected) = item_activation::encounter_return(
            state,
            content,
            pending.item.ok_or("outdoor item return missing")?,
        )? {
            events.extend(crate::exploration::finish_typed_selection(
                state, content, selected,
            )?);
        }
        Ok(events)
    } else {
        // The owner's information is refreshed before the screen returns; this
        // path does not reset the marks.
        item_activation::refresh_inventory_info_items(state, content, pending.actor)?;
        state.globals.item_used = 0;
        state.set_in_spell(0);
        Ok(vec![])
    }
}
