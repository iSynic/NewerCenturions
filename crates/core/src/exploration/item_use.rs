//! Item use for maintained exploration.
use super::{active_map, finish_typed_selection, run_script, typed_record, unsupported};
use crate::{
    Event, MovementContinuation, Phase, Presentation, RuleError, ScriptState, Session,
    typed_encounter,
};
use centurions_content::{Content, word};

pub fn use_item(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
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
        return crate::combat::runtime::begin_medical_item(state, content, warrior, slot);
    }
    let definition = crate::ammunition::definition(content, item.id())?;
    if state.combat.is_none() && word(&definition, 26)? == 28 && word(&definition, 44)? > 1100 {
        return crate::outdoor_psi::begin_item(state, content, warrior, slot);
    }
    if word(&definition, 26)? == 27 {
        return crate::item_modifiers::use_item(state, content, warrior, slot);
    }
    crate::story_item_use::reject_inert_item(&definition)?;
    if state.phase == Phase::Combat && word(&definition, 26)?.wrapping_abs() == 23 {
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
    let used = crate::story_item_use::use_scenario_item(
        &state.warriors,
        warrior,
        slot,
        content,
        races,
        crate::story_item_use::ScriptItemContext::Exploration,
        state.inventory_record.as_ref(),
    )?;
    let id = state.warriors[warrior].inventory()?[slot].id();
    state.load_item_work(content, id)?;
    state.warriors = used.warriors;
    state.selected_warrior = warrior;
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
        state.exploration.as_mut().unwrap().script = Some(ScriptState {
            record: map.script(id)?.to_vec(),
            cursor: 0,
            map_slot: 0,
            item_return: false,
            scheduled: true,
            menu: None,
            persist: false,
            origin: None,
        });
        effects.extend(run_script(state, content)?);
    }
    Ok(effects)
}

pub(crate) fn check_item_actor(state: &Session, warrior: usize) -> Result<(), RuleError> {
    let actor = state
        .warriors
        .get(warrior)
        .ok_or("item warrior unavailable")?;
    if let Some(combat) = &state.combat {
        let warning = if combat.active_body() != Some(warrior as i16) {
            Some(10)
        } else if word(&actor.bytes, 0x40)? < 1 {
            Some(17)
        } else {
            None
        };
        if let Some(index) = warning {
            return Err(RuleError::Refused {
                feedback: vec![crate::feedback::NativeFeedback::Warning { index }],
            });
        }
    }
    Ok(())
}

pub(super) fn refuse_combat_script_item(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    item: i16,
    encounter: bool,
) -> Result<Vec<Event>, RuleError> {
    check_item_actor(state, warrior)?;
    state.load_item_work(content, item)?;
    let mut feedback = Vec::new();
    if !encounter && word(state.ammunition_work.item.as_ref().unwrap(), 44)? < 1100 {
        feedback.extend([88, 89].map(|index| crate::feedback::NativeFeedback::Warning { index }));
    }
    feedback.push(crate::feedback::NativeFeedback::Warning { index: 52 });
    Err(RuleError::RefusedAfterEffects { feedback })
}

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
    let selected = typed_encounter::select_item(typed_record(state, content)?, item.id())?;
    let definition = crate::ammunition::definition(content, item.id())?;
    let class = word(&definition, 26)?.wrapping_abs();
    if word(&definition, 26)? == 27 {
        return crate::item_modifiers::use_item(state, content, warrior, slot);
    }
    if class == 23 {
        if state.combat.is_some() {
            return refuse_combat_script_item(state, content, warrior, item.id(), true);
        }
        let frame = state
            .exploration
            .as_ref()
            .unwrap()
            .script
            .as_ref()
            .ok_or("typed item caller missing")?;
        let manual = frame.item_return;
        let walking =
            !frame.scheduled && frame.origin.is_some() && state.time_continuation.is_none();
        let calendar = frame.scheduled
            && state.time_continuation.is_some()
            && state.combat_story.is_none()
            && state.round_dispatch.is_none();
        let regional = frame.scheduled
            && frame.origin.is_none()
            && state.time_continuation.is_none()
            && state.combat_story.is_none()
            && state.round_dispatch.is_none()
            && matches!(
                state.movement_continuation,
                Some(MovementContinuation::RegionalReturn { .. })
            );
        let shared_death = state
            .combat_story
            .as_ref()
            .is_some_and(|story| story.shared_dispatch);
        let completed_round = state.round_dispatch.is_some()
            && state.scripted_cleanup == Some(crate::story_cleanup::Stage::Complete);
        if !manual && !walking && !calendar && !regional && !shared_death && !completed_round {
            return Err(unsupported(
                "story.encounter_item",
                "script-item return caller",
            ));
        }
        let map = active_map(state, content)?;
        let used = crate::story_item_use::use_scenario_item(
            &state.warriors,
            warrior,
            slot,
            content,
            &map.combat
                .as_ref()
                .ok_or("item movement tables unavailable")?
                .race_moves,
            crate::story_item_use::ScriptItemContext::Encounter,
            state.inventory_record.as_ref(),
        )?;
        // Only manual buttonchoice10a9ee decodes the script-item return.
        let record = if manual {
            Some(
                map.script(used.return_code.wrapping_abs().wrapping_sub(100))?
                    .to_vec(),
            )
        } else {
            None
        };
        state.load_item_work(content, item.id())?;
        state.warriors = used.warriors;
        state.selected_warrior = warrior;
        state.party_mask[warrior] = 1;
        let position = state.exploration.as_mut().unwrap();
        position.presentation = None;
        position.script = record.map(|record| ScriptState {
            record,
            cursor: 0,
            map_slot: 0,
            item_return: false,
            scheduled: true,
            menu: None,
            persist: false,
            origin: None,
        });
        let mut effects: Vec<_> = used
            .sounds
            .into_iter()
            .map(|resource_id| Event::Sound { resource_id })
            .collect();
        if manual || shared_death || completed_round {
            effects.extend(run_script(state, content)?);
        }
        return Ok(effects);
    }
    let activating =
        word(&definition, 44)? > 1100 && (word(&item.0, 2)? > 0 || word(&definition, 20)? < 0);
    if activating && class == 28 && state.combat.is_none() {
        return crate::outdoor_psi::begin_item(state, content, warrior, slot);
    }
    if class == 27 || activating {
        return Err(unsupported(
            "story.encounter_item",
            "activating item encounter caller",
        ));
    }
    // Items12b04a reloads the item and clears targets;12c422 selects its owner.
    state.load_item_work(content, item.id())?;
    state.party_mask.fill(0);
    state.party_mask[warrior] = 1;
    state.selected_warrior = warrior;
    finish_typed_selection(state, content, selected)
}
