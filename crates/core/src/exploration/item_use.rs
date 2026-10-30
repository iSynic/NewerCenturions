//! Using an item while exploring.
use super::{active_map, finish_typed_selection, run_script};
use crate::combat::item_activation;
use crate::combat::item_activation::Caller;
use crate::combat::runtime;
use crate::{Event, Phase, Presentation, RuleError, ScriptState, Session};
use centurions_content::Content;
use centurions_content::records::ItemDefinition;

/// Uses an item from a warrior's pack while exploring.
pub fn use_item(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    item_activation::check_selected_slot(slot)?;
    if state
        .exploration
        .as_ref()
        .is_some_and(|p| matches!(p.presentation, Some(Presentation::Typed { .. })))
    {
        return use_encounter_item(state, content, warrior, slot);
    }
    let item = state
        .warriors
        .get(warrior)
        .ok_or("item warrior unavailable")?
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or("inventory item unavailable")?;
    if (700..=703).contains(&item.id()) && state.phase == Phase::Combat {
        return runtime::begin_medical_item(state, content, warrior, slot);
    }
    let definition = crate::ammunition::definition(content, item.id())?;
    if state.combat.is_none()
        && !matches!(
            ItemDefinition(&definition).item_type()?.wrapping_abs(),
            23 | 27
        )
        && ItemDefinition(&definition).spell()? > 1100
    {
        return crate::outdoor_psi::begin_item(state, content, warrior, slot);
    }
    if ItemDefinition(&definition).item_type()? == 27 {
        return crate::item_modifiers::use_item(state, content, warrior, slot);
    }
    if state.combat.is_some()
        && ItemDefinition(&definition).spell()? > 1100
        && ItemDefinition(&definition).item_type()?.wrapping_abs() != 23
    {
        return runtime::begin_medical_item(state, content, warrior, slot);
    }
    crate::story::item_use::reject_inert_item(&definition)?;
    if state.phase == Phase::Combat && ItemDefinition(&definition).item_type()?.wrapping_abs() == 23
    {
        return refuse_combat_script_item(state, content, warrior, item.id(), false);
    }
    if state.phase != Phase::Exploration
        || state
            .exploration
            .as_ref()
            .is_none_or(|p| p.presentation.is_some() || p.script.is_some())
    {
        return Err(RuleError::Invalid(
            "item use requires free exploration".into(),
        ));
    }
    let map = active_map(state, content)?;
    let races = &map.combat.as_ref().unwrap().race_moves;
    state.set_warrior_selector(warrior);
    let used = crate::story::item_use::use_scenario_item(
        &state.warriors,
        warrior,
        slot,
        content,
        races,
        crate::story::item_use::ScriptItemContext::Exploration,
        Some(&state.globals.inventory_record),
    )?;
    state.apply_equipment_globals(&used.global_writes)?;
    let id = state.warriors[warrior].inventory()?[slot].id();
    state.load_item(content, id)?;
    state.warriors = used.warriors;
    item_activation::finish_return(state, warrior, Caller::Ordinary);
    // Selection is refreshed before the item's script return is decoded.
    item_activation::clear_active_affected(state);
    let mut effects: Vec<Event> = used
        .warnings
        .into_iter()
        .map(|warning| Event::CollectionRejected { warning })
        .collect();
    effects.extend(
        used.sounds
            .into_iter()
            .map(|resource_id| Event::Sound { resource_id }),
    );
    if let Some(id) = used.script {
        state.exploration.as_mut().unwrap().script =
            Some(ScriptState::scheduled(map.script(id)?.to_vec(), 0));
        effects.extend(run_script(state, content)?);
    }
    Ok(effects)
}

/// Checks that the warrior may use an item now: in a battle only the warrior whose turn it is, with
/// an action point left.
pub(crate) fn check_item_actor(state: &Session, warrior: usize) -> Result<(), RuleError> {
    let actor = state
        .warriors
        .get(warrior)
        .ok_or("item warrior unavailable")?;
    if let Some(combat) = &state.combat {
        let warning = if combat.active_body() != Some(warrior as i16) {
            Some(10)
        } else if actor.record().attacks()? < 1 {
            Some(17)
        } else {
            None
        };
        if let Some(index) = warning {
            return Err(RuleError::Refused {
                feedback: vec![crate::feedback::GameFeedback::Warning { index }],
            });
        }
    }
    Ok(())
}

/// Refuses an item a script's battle or encounter cannot use, with warning 52; outside an
/// encounter, an item whose spell is below 1100 also gets warnings 88 and 89.
pub(super) fn refuse_combat_script_item(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    item: i16,
    encounter: bool,
) -> Result<Vec<Event>, RuleError> {
    check_item_actor(state, warrior)?;
    state.load_item(content, item)?;
    let mut feedback = Vec::new();
    if !encounter
        && ItemDefinition(state.globals.loaded_items.item.as_ref().unwrap()).spell()? < 1100
    {
        feedback.extend([88, 89].map(|index| crate::feedback::GameFeedback::Warning { index }));
    }
    feedback.push(crate::feedback::GameFeedback::Warning { index: 52 });
    Err(RuleError::RefusedAfterEffects { feedback })
}

/// Uses an item while a typed encounter prompt is showing.
pub(super) fn use_encounter_item(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    check_item_actor(state, warrior)?;
    let item = state.warriors[warrior]
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or("inventory item unavailable")?;
    let definition = crate::ammunition::definition(content, item.id())?;
    let class = ItemDefinition(&definition).item_type()?.wrapping_abs();
    if ItemDefinition(&definition).item_type()? == 27 {
        return crate::item_modifiers::use_item(state, content, warrior, slot);
    }
    if state.combat.is_some() && class == 23 {
        return refuse_combat_script_item(state, content, warrior, item.id(), true);
    }
    if state.combat.is_some() {
        let used = item_activation::activate(state, content, warrior, slot, Caller::Encounter)?;
        let mut events = used.events;
        if let Some(selected) = item_activation::encounter_return(state, content, used.item)? {
            events.extend(finish_typed_selection(state, content, selected)?);
        }
        return Ok(events);
    }
    if class == 23 {
        state.set_warrior_selector(warrior);
        let frame = state
            .exploration
            .as_ref()
            .unwrap()
            .script
            .as_ref()
            .ok_or("typed item caller missing")?;
        let manual = frame.item_return;
        let map = active_map(state, content)?;
        let used = crate::story::item_use::use_scenario_item(
            &state.warriors,
            warrior,
            slot,
            content,
            &map.combat
                .as_ref()
                .ok_or("item movement tables unavailable")?
                .race_moves,
            crate::story::item_use::ScriptItemContext::Encounter,
            Some(&state.globals.inventory_record),
        )?;
        state.apply_equipment_globals(&used.global_writes)?;
        // Items and typed encounters leave the script return for the fire button.
        state.globals.item_used = used.return_code;
        // Only an item used by hand decodes its script return.
        let record = if manual {
            Some(
                map.script(used.return_code.wrapping_abs().wrapping_sub(100))?
                    .to_vec(),
            )
        } else {
            None
        };
        state.load_item(content, item.id())?;
        state.warriors = used.warriors;
        item_activation::finish_return(state, warrior, Caller::Encounter);
        item_activation::clear_active_affected(state);
        let position = state.exploration.as_mut().unwrap();
        position.presentation = None;
        position.script = record.map(|record| ScriptState::scheduled(record, 0));
        let mut effects: Vec<_> = used
            .sounds
            .into_iter()
            .map(|resource_id| Event::Sound { resource_id })
            .collect();
        effects.extend(run_script(state, content)?);
        return Ok(effects);
    }
    let activating = ItemDefinition(&definition).spell()? > 1100
        && (item.charge()? > 0 || ItemDefinition(&definition).supplied_quantity()? < 0);
    if activating && state.combat.is_none() {
        return crate::outdoor_psi::begin_item(state, content, warrior, slot);
    }
    let used = item_activation::activate(state, content, warrior, slot, Caller::Encounter)?;
    let mut events = used.events;
    if let Some(selected) = item_activation::encounter_return(state, content, used.item)? {
        events.extend(finish_typed_selection(state, content, selected)?);
    }
    Ok(events)
}
