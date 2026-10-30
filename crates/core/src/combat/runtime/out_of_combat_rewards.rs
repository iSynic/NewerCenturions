//! Rewards granted after the battle has ended, while the monster records remain.
use super::booty_setup as reward;
use super::{ordinary_rewards, refit_after_combat, reward_context};
use crate::combat::{allies, rewards};
use crate::story::cleanup::{self as story_cleanup, Stage};
use crate::{Content, Event, Presentation, RuleError, Session, story_health, story_inventory};

/// When the monsters are gone, getup runs booty's early mode before going on. The rewards wait
/// behind any open window; then firearms are refitted and the survivors offered. No loot is dealt.
///
/// Original: getup, booty
pub(crate) fn begin_getup(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    if state.combat.is_some() || state.exploration.is_none() {
        return Err("Getup rewards require retained mission context".into());
    }
    if state.deferred_getup {
        return Err("retained Getup entered while its booty is deferred".into());
    }
    state.command_return = true;
    // Rewards wait until the open window closes.
    if modal_open(state) {
        // A paused story would resume before the rewards, which the game never does.
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
    reward::entry(state)?;
    let events = refit_after_combat(state, content)?;
    reward::early_out_of_combat(state)?;
    let records = state.globals.monsters.active_records()?;
    let parent = state.scripted_cleanup.take().map(Box::new);
    begin_selection(
        state,
        content,
        records,
        Stage::OutOfCombatGetupSurvivors { parent },
        events,
    )
}

/// Whether a window is open that the rewards must wait behind: the survivors screen or a message.
fn modal_open(state: &Session) -> bool {
    state.survivors.is_some()
        || state
            .exploration
            .as_ref()
            .is_some_and(|position| position.presentation.is_some())
}

/// Runs getup's deferred booty once its dialogue or survivor window has closed, before any paused
/// getup caller resumes.
///
/// Original: getup, booty
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

/// Refuses a saved deferred getup booty unless no battle runs, no script is paused and a window is
/// open.
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

/// Refuses a monster count that the rewards cannot work with.
pub(crate) fn validate_count(state: &Session) -> Result<(), RuleError> {
    super::reward_cleanup::validate(state.globals.monsters.num_mon)
}

/// Starts the rewards of a battle that ended inside a story. The reward mode is fixed, firearms are
/// refitted and, for a mode other than 2, the party's loot is normalized; then the survivors are
/// chosen.
///
/// Original: booty
pub(crate) fn begin(
    state: &mut Session,
    content: &Content,
    mode: i16,
    ordinary: bool,
) -> Result<Vec<Event>, RuleError> {
    let mode = reward::effective_mode(state, mode)?;
    validate_count(state)?;
    let mut records = state.globals.monsters.active_records()?;
    if mode != 2 {
        rewards::RewardMode::select(mode, &state.globals.treasure)?;
    }
    let events = refit_after_combat(state, content)?;
    if mode == 2 {
        story_cleanup::early_out_of_combat_prefix(state)?;
        records = state.globals.monsters.active_records()?;
    } else {
        let kill_party = state
            .warriors
            .iter()
            .filter(|warrior| {
                warrior.stamina().unwrap_or(0) < 1
                    || warrior.record().computer_control().unwrap_or(0) < 0
            })
            .count() as i8;
        state.warriors = story_inventory::normalize_story_loot(&state.warriors)?;
        let mut health =
            story_health::reselected_context(state.char_select_new, &state.globals.health_context);
        health.kill_party = kill_party;
        state.globals.health_context = health;
        state.clear_tactical_fields();
        state
            .booty_list
            .fill(centurions_content::InventoryRecord([0; 22]));
    }
    begin_selection(
        state,
        content,
        records,
        Stage::OutOfCombatSurvivors { mode, ordinary },
        events,
    )
}

/// Starts the first or second reward of a story battle with selected treasure, with the survivors
/// of that stage.
///
/// Original: booty
pub(super) fn begin_selected(
    state: &mut Session,
    content: &Content,
    requested: i16,
) -> Result<Vec<Event>, RuleError> {
    state.command_return = true;
    let selection = state.survivors.take();
    let mode = reward::effective_mode(state, requested)?;
    validate_count(state)?;
    if mode != 2 {
        rewards::RewardMode::select(mode, &state.globals.treasure)?;
    }
    reward::entry(state)?;
    let events = refit_after_combat(state, content)?;
    if mode == 2 {
        reward::early_out_of_combat(state)?;
    } else {
        reward::normalize_out_of_combat(state)?;
    }
    let records = state.globals.monsters.active_records()?;
    if let Some(selection) = selection {
        // Older selected saves can already be paused inside bodycount.
        state.survivors = Some(selection);
        state.scripted_cleanup = Some(Stage::OutOfCombatSelectedSurvivors { mode });
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
        Stage::OutOfCombatSelectedSurvivors { mode },
        events,
    )
}

/// Offers the survivors to the player when monsters are left over, otherwise goes straight to
/// confirming the reward.
fn begin_selection(
    state: &mut Session,
    content: &Content,
    mut records: Vec<centurions_content::combat::MonsterRecord>,
    stage: Stage,
    mut events: Vec<Event>,
) -> Result<Vec<Event>, RuleError> {
    state.scripted_cleanup = Some(stage);
    if state.suspend_allies == 0 {
        state.survivors = allies::prepare_survivors(&mut records)?;
    }
    state.globals.monsters.store_records(records);
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

/// Confirms the survivors and deals the reward by its mode. Mode 2 gives nothing and returns to the
/// caller; any other mode prepares the booty, installs it and opens the reward window with the
/// booty sound.
///
/// Original: booty
pub(crate) fn confirm(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let (mode, ordinary, getup, selected) = match state.scripted_cleanup.as_ref() {
        Some(Stage::OutOfCombatSurvivors { mode, ordinary }) => (*mode, *ordinary, false, false),
        Some(Stage::OutOfCombatGetupSurvivors { .. }) => (2, false, true, false),
        Some(Stage::OutOfCombatSelectedSurvivors { mode }) => (*mode, false, false, true),
        _ => return Err("retained survivor return missing".into()),
    };
    validate_count(state)?;
    let mut records = state.globals.monsters.active_records()?;
    if state.suspend_allies == 0 {
        let selection = state.survivors.take();
        let synchronized = allies::commit_survivors(
            &state.holdover,
            state.held_over,
            &mut records,
            selection.as_ref(),
        )?;
        state.holdover = synchronized.menu;
        state.held_over = synchronized.count;
    }
    reward_context::restore_weapons(state)?;
    state.globals.monsters.store_records(records.clone());
    state.exploration.as_mut().unwrap().presentation = None;
    if getup {
        // booty restores the allies' weapons, then returns straight to getup.
        let Some(Stage::OutOfCombatGetupSurvivors { parent }) = state.scripted_cleanup.take()
        else {
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
            deduction: 0,
            currency: state.party_currency,
            mode: rewards::RewardMode::select(mode, &state.globals.treasure)?,
        },
        &mut state.random,
    )?;
    // bodycount resets conditions and captures; the award loop writes only the
    // faction of each participating record.
    let award_count = if mode == 5 { 1 } else { records.len() };
    for record in records.iter_mut().take(award_count) {
        record.record_mut().set_faction(1);
    }
    let count = state.globals.monsters.num_mon;
    state.globals.monsters.store_records(records);
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
