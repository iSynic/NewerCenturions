//! booty callers which retain NPC records after clearing the combat flag.
use super::{ordinary_rewards, refit_after_combat, reward_context};
use crate::combat::{allies, rewards};
use crate::story_cleanup::{self, Stage};
use crate::{Content, Event, Presentation, RuleError, Session, story_health, story_inventory};

/// Getup calls booty(2,0) at 0x125b34 and then branches to its epilogue at 0x125b3e.
pub(crate) fn begin_getup(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    if state.combat.is_some() || state.exploration.is_none() {
        return Err("Getup rewards require retained mission context".into());
    }
    if state.deferred_getup {
        return Err("retained Getup entered while its booty is deferred".into());
    }
    state.command_return = true;
    // The original window was dismissed before Getup ran; booty waits for its close.
    if modal_open(state) {
        // A paused script would resume before booty, which native never reaches.
        if state
            .exploration
            .as_ref()
            .is_some_and(|position| position.script.is_some())
        {
            return Err("retained Getup reached behind a paused script".into());
        }
        state.deferred_getup = true;
        return Ok(vec![]);
    }
    crate::combat::reward_work::entry(state)?;
    let events = refit_after_combat(state, content)?;
    crate::combat::reward_work::early_retained(state)?;
    let records = state.native_work.npc_backing.active_records()?;
    let parent = state.scripted_cleanup.take().map(Box::new);
    begin_selection(
        state,
        content,
        records,
        Stage::RetainedGetupSurvivors { parent },
        events,
    )
}

fn modal_open(state: &Session) -> bool {
    state.survivors.is_some()
        || state
            .exploration
            .as_ref()
            .is_some_and(|position| position.presentation.is_some())
}

/// Runs a deferred Getup booty(2,0) once its dialogue or survivor window has closed,
/// before any suspended Getup caller resumes.
pub(crate) fn resume_deferred_getup(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    if !state.deferred_getup {
        return Ok(vec![]);
    }
    if state.mission.is_none() {
        state.deferred_getup = false;
        return Ok(vec![]);
    }
    if modal_open(state) {
        return Ok(vec![]);
    }
    state.deferred_getup = false;
    begin_getup(state, content)
}

pub(crate) fn validate_deferred_getup(state: &Session) -> Result<(), RuleError> {
    if state.deferred_getup
        && (state.combat.is_some()
            || state
                .exploration
                .as_ref()
                .is_none_or(|position| position.script.is_some())
            || !modal_open(state))
    {
        return Err("invalid deferred Getup booty".into());
    }
    Ok(())
}

pub(crate) fn validate_count(state: &Session) -> Result<(), RuleError> {
    super::reward_cleanup::validate(state.native_work.npc_backing.active_count)
}

pub(crate) fn begin(
    state: &mut Session,
    content: &Content,
    mode: i16,
    ordinary: bool,
) -> Result<Vec<Event>, RuleError> {
    let mode = crate::combat::reward_work::effective_mode(state, mode)?;
    validate_count(state)?;
    let mut records = state.native_work.npc_backing.active_records()?;
    if mode != 2 {
        rewards::RewardMode::select(mode, &state.native_work.treasure_record)?;
    }
    let events = refit_after_combat(state, content)?;
    if mode == 2 {
        story_cleanup::early_retained_prefix(state)?;
        records = state.native_work.npc_backing.active_records()?;
    } else {
        let unavailable = state
            .warriors
            .iter()
            .filter(|warrior| {
                warrior.stamina().unwrap_or(0) < 1
                    || warrior.record().automatic_mode().unwrap_or(0) < 0
            })
            .count() as i8;
        state.warriors = story_inventory::normalize_story_loot(&state.warriors)?;
        let mut health = story_health::retained_context(
            state.selected_warrior,
            &state.native_work.health_context,
        );
        health.unavailable = unavailable;
        state.native_work.health_context = health;
        state.clear_tactical_fields();
        state
            .booty_items
            .fill(centurions_content::InventoryRecord([0; 22]));
    }
    begin_selection(
        state,
        content,
        records,
        Stage::RetainedSurvivors { mode, ordinary },
        events,
    )
}

pub(super) fn begin_selected(
    state: &mut Session,
    content: &Content,
    requested: i16,
) -> Result<Vec<Event>, RuleError> {
    state.command_return = true;
    let selection = state.survivors.take();
    let mode = crate::combat::reward_work::effective_mode(state, requested)?;
    validate_count(state)?;
    if mode != 2 {
        rewards::RewardMode::select(mode, &state.native_work.treasure_record)?;
    }
    crate::combat::reward_work::entry(state)?;
    let events = refit_after_combat(state, content)?;
    if mode == 2 {
        crate::combat::reward_work::early_retained(state)?;
    } else {
        crate::combat::reward_work::normalize_retained(state)?;
    }
    let records = state.native_work.npc_backing.active_records()?;
    if let Some(selection) = selection {
        // Older selected saves can already be paused inside bodycount.
        state.survivors = Some(selection);
        state.scripted_cleanup = Some(Stage::RetainedSelectedSurvivors { mode });
        return confirm(state, content).map(|returned| {
            let mut events = events;
            events.extend(returned);
            events
        });
    }
    begin_selection(
        state,
        content,
        records,
        Stage::RetainedSelectedSurvivors { mode },
        events,
    )
}

fn begin_selection(
    state: &mut Session,
    content: &Content,
    mut records: Vec<centurions_content::combat::MonsterRecord>,
    stage: Stage,
    mut events: Vec<Event>,
) -> Result<Vec<Event>, RuleError> {
    state.scripted_cleanup = Some(stage);
    if state.suppress_allies == 0 {
        state.survivors = allies::prepare_survivors(&mut records)?;
    }
    state.native_work.npc_backing.store_records(records);
    if state.survivors.is_some() {
        state.exploration.as_mut().unwrap().presentation = Some(Presentation::Survivors);
        events.push(Event::Sound {
            resource_id: 29999_i16.wrapping_add(state.random.bound(4)),
        });
        events.push(Event::StoryPresented);
    } else {
        events.extend(confirm(state, content)?);
    }
    Ok(events)
}

pub(crate) fn confirm(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let (mode, ordinary, getup, selected) = match state.scripted_cleanup.as_ref() {
        Some(Stage::RetainedSurvivors { mode, ordinary }) => (*mode, *ordinary, false, false),
        Some(Stage::RetainedGetupSurvivors { .. }) => (2, false, true, false),
        Some(Stage::RetainedSelectedSurvivors { mode }) => (*mode, false, false, true),
        _ => return Err("retained survivor return missing".into()),
    };
    validate_count(state)?;
    let mut records = state.native_work.npc_backing.active_records()?;
    if state.suppress_allies == 0 {
        let selection = state.survivors.take();
        let synchronized = allies::commit_survivors(
            &state.allies,
            state.ally_count,
            &mut records,
            selection.as_ref(),
        )?;
        state.allies = synchronized.menu;
        state.ally_count = synchronized.count;
    }
    reward_context::restore_weapons(state)?;
    state.native_work.npc_backing.store_records(records.clone());
    state.exploration.as_mut().unwrap().presentation = None;
    if getup {
        // booty 0x105ac4 restores ally weapons, then returns directly to Getup.
        let Some(Stage::RetainedGetupSurvivors { parent }) = state.scripted_cleanup.take() else {
            unreachable!("Getup caller changed during survivor commit");
        };
        state.scripted_cleanup = parent.map(|stage| *stage);
        return super::selected_rewards::after_getup(state, content);
    }
    if mode == 2 {
        if selected {
            state.scripted_cleanup = None;
            return super::selected_rewards::returned(state, content);
        }
        if ordinary {
            state.scripted_cleanup = None;
            state.reload_battle_land()?;
            return ordinary_rewards::finish(state, content);
        }
        let mut events = story_cleanup::complete(state, content)?;
        events.extend(crate::exploration::run_script(state, content)?);
        return Ok(events);
    }
    let prepared = rewards::prepare_mode(
        &state.warriors,
        &records,
        content,
        rewards::RewardContext {
            difficulty: state.difficulty,
            npc_denominator: 0,
            currency: state.party_currency,
            mode: rewards::RewardMode::select(mode, &state.native_work.treasure_record)?,
        },
        &mut state.random,
    )?;
    // Bodycount owns condition/capture normalization, including its suppression branch.
    // The award loop itself writes only faction in the participating records.
    let award_count = if mode == 5 { 1 } else { records.len() };
    for record in records.iter_mut().take(award_count) {
        record.record_mut().set_faction(1);
    }
    let count = state.native_work.npc_backing.active_count;
    state.native_work.npc_backing.store_records(records);
    ordinary_rewards::install(state, content, mode, prepared)?;
    super::reward_cleanup::finish(state, count, mode)?;
    if !selected && let Some(story) = state.combat_story.as_mut() {
        story.child_returned = true;
    }
    if selected {
        state.scripted_cleanup = None;
        super::selected_rewards::defer(state)?;
    } else if ordinary {
        state.scripted_cleanup = None;
        ordinary_rewards::defer(state);
    } else {
        state.scripted_cleanup = Some(Stage::Rewards);
    }
    state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
    Ok(vec![Event::Sound {
        resource_id: crate::feedback::BOOTY_SOUND,
    }])
}
