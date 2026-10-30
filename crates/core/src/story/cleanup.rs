use crate::{Content, Event, Phase, RuleError, Session, combat, story_health, story_inventory};
mod rewards;

/// How far a scripted battle cleanup has got: choosing survivors (for a script cleanup or an
/// ordinary battle), choosing survivors for a reward mode, showing the rewards, or complete. The
/// `OutOfCombat` forms are the cleanups run outside a battle.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Stage {
    Survivors,
    Complete,
    OrdinarySurvivors,
    RewardSurvivors(i16),
    Rewards,
    #[serde(rename = "RetainedSurvivors")]
    OutOfCombatSurvivors {
        mode: i16,
        ordinary: bool,
    },
    #[serde(rename = "RetainedSelectedSurvivors")]
    OutOfCombatSelectedSurvivors {
        mode: i16,
    },
    #[serde(rename = "RetainedGetupSurvivors")]
    OutOfCombatGetupSurvivors {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parent: Option<Box<Stage>>,
    },
}

impl Stage {
    /// Whether the cleanup waits for the player.
    pub(crate) fn pauses(&self) -> bool {
        !matches!(self, Self::Complete)
    }
}

/// Combat cleanup doubles its signed parameter before calling booty.
///
/// Original: newland
pub(crate) fn begin(
    state: &mut Session,
    content: &Content,
    parameter: i16,
) -> Result<Vec<Event>, RuleError> {
    let requested_mode = parameter.wrapping_mul(2);
    let pending =
        state.scripted_cleanup.is_some() && state.scripted_cleanup != Some(Stage::Complete);
    if !matches!(state.phase, Phase::Combat | Phase::Exploration) || pending {
        return Err(RuleError::Invalid(
            "scripted combat cleanup is not available".into(),
        ));
    }
    let mode = combat::runtime::booty_setup::effective_mode(state, requested_mode)?;
    combat::runtime::begin_selected_cleanup(state)?;
    if let Some(combat) = state.combat.as_mut() {
        combat.kill_counts.finish();
    } else {
        state.globals.kill_counts.finish();
    }
    if mode == 10 {
        return crate::loot_recovery::begin(
            state,
            content,
            crate::loot_recovery::RecoveryReturn::Scripted,
        );
    }
    if state.combat.is_none() {
        return combat::runtime::begin_out_of_combat_rewards(state, content, mode, false);
    }
    if mode != 2 {
        return rewards::begin(state, content, mode);
    }
    begin_early(state, content, Stage::Survivors)
}

/// Begins the cleanup of an ordinary battle early, choosing survivors.
pub(crate) fn begin_ordinary(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    begin_early(state, content, Stage::OrdinarySurvivors)
}

/// Early booty updates all 100 monster slots, whatever the active count.
pub(crate) fn early_out_of_combat_prefix(state: &mut Session) -> Result<(), RuleError> {
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
    for warrior in &mut state.warriors {
        warrior.record_mut().set_in_battle(1);
    }
    for slot in state.globals.monsters.slots.iter_mut().flatten() {
        if slot.record.record().survivor_marker()? != -1 {
            slot.record.record_mut().set_stamina(0);
        }
    }
    Ok(())
}

/// Begins a cleanup that ends the battle early: the warriors are refitted, monsters that are not
/// survivors are defeated, and the player chooses survivors.
fn begin_early(
    state: &mut Session,
    content: &Content,
    stage: Stage,
) -> Result<Vec<Event>, RuleError> {
    let mut events = combat::runtime::refit_after_combat(state, content)?;
    let kill_party = state
        .warriors
        .iter()
        .filter(|warrior| {
            warrior.stamina().unwrap_or(0) < 1
                || warrior.record().computer_control().unwrap_or(0) < 0
        })
        .count() as i8;
    state.warriors = story_inventory::normalize_story_loot(&state.warriors)?;
    for warrior in &mut state.warriors {
        warrior.record_mut().set_in_battle(1);
    }
    let mut health =
        story_health::reselected_context(state.char_select_new, &state.globals.health_context);
    health.kill_party = kill_party;
    state.globals.health_context = health;
    let arena = state.combat.as_mut().ok_or("cleanup combat missing")?;
    arena.kill_party = kill_party;
    for record in &mut arena.roster.records {
        if record.record().survivor_marker()? != -1 {
            record.record_mut().set_stamina(0);
        }
    }
    for slot in arena
        .monsters
        .slots
        .iter_mut()
        .skip(arena.roster.records.len())
        .flatten()
    {
        if slot.record.record().survivor_marker()? != -1 {
            slot.record.record_mut().set_stamina(0);
        }
    }
    state.clear_tactical_fields();
    state
        .booty_list
        .fill(centurions_content::InventoryRecord([0; 22]));
    state.scripted_cleanup = Some(stage);
    if let Some(presentation) = combat::runtime::begin_survivors(state)? {
        events.extend(presentation);
    } else {
        events.extend(confirm(state, content)?);
    }
    Ok(events)
}

/// Confirms the survivors the player chose. A reward mode hands out its rewards, an ordinary battle
/// leaves the battle and finishes its rewards, and any other cleanup completes.
pub(crate) fn confirm(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    if matches!(state.scripted_cleanup, Some(Stage::RewardSurvivors(_))) {
        return rewards::confirm(state, content);
    }
    let ordinary = state.scripted_cleanup == Some(Stage::OrdinarySurvivors);
    if !ordinary && state.scripted_cleanup != Some(Stage::Survivors) {
        return Err(RuleError::Invalid(
            "scripted survivor cleanup is not pending".into(),
        ));
    }
    combat::runtime::store_last_defeated(state)?;
    let mut events = combat::runtime::synchronize_reward_survivors(state)?;
    state.exploration.as_mut().unwrap().presentation = None;
    combat::runtime::leave_reward_battle(state)?;
    if ordinary {
        state.scripted_cleanup = None;
        state.reload_battle_land()?;
        return combat::runtime::finish_early_rewards(state, content).map(|finished| {
            events.extend(finished);
            events
        });
    }
    events.extend(complete(state, content)?);
    Ok(events)
}

/// Completes the cleanup: rewards are suppressed for the battle and the party returns to the land
/// it fought on.
pub(crate) fn complete(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    state.globals.battle_rewards_suppressed = 1;
    state.scripted_cleanup = Some(Stage::Complete);
    if let Some(story) = state.combat_story.as_mut() {
        story.child_returned = true;
    }
    state.reload_battle_land()?;
    let position = state.exploration.as_mut().ok_or("cleanup map missing")?;
    position
        .map_state()
        .validate(content.map(position.map).ok_or("cleanup content missing")?)?;
    position.presentation = None;
    // The current land records stand in for reloading the land.
    Ok(vec![Event::Moved {
        x: position.x,
        y: position.y,
    }])
}

/// Checks that a pending cleanup agrees with the battle and rewards it needs.
pub(crate) fn validate(state: &Session) -> Result<(), RuleError> {
    if let Some(Stage::OutOfCombatSelectedSurvivors { mode }) = state.scripted_cleanup {
        combat::runtime::validate_out_of_combat_reward_count(state)?;
        if state.phase != Phase::Exploration
            || state.combat.is_some()
            || state.selected_battle_treasure.is_none()
            || state.survivors.is_none()
            || !matches!(mode, 0..=2)
        {
            return Err("invalid selected survivor return".into());
        }
    }
    if let Some(Stage::OutOfCombatGetupSurvivors { parent }) = &state.scripted_cleanup {
        if state.phase != Phase::Exploration
            || state.combat.is_some()
            || state.survivors.is_none()
            || state.exploration.as_ref().is_none_or(|position| {
                position.presentation != Some(crate::Presentation::Survivors)
            })
        {
            return Err("invalid retained Getup survivor return".into());
        }
        if parent.as_deref().is_some_and(|stage| {
            *stage != Stage::Complete
                || state
                    .exploration
                    .as_ref()
                    .is_none_or(|position| position.script.is_none())
        }) {
            return Err("invalid suspended Getup cleanup caller".into());
        }
    }
    if let Some(Stage::OutOfCombatSurvivors { mode, ordinary }) = state.scripted_cleanup {
        combat::runtime::validate_out_of_combat_reward_count(state)?;
        if state.phase != Phase::Exploration
            || state.combat.is_some()
            || state.survivors.is_none()
            || ordinary && state.ordinary_battle_rewards.is_none()
            || mode == 10
        {
            return Err("invalid retained survivor return".into());
        }
        if mode != 2 {
            combat::rewards::RewardMode::select(mode, &state.globals.treasure)?;
        }
    }
    if state.scripted_cleanup == Some(Stage::Rewards) {
        let mut page = state
            .exploration
            .as_ref()
            .and_then(|p| p.presentation.as_ref());
        while let Some(crate::Presentation::Messages { next, .. }) = page {
            page = next.as_deref();
        }
        if state.phase != Phase::Exploration
            || state.combat.is_some()
            || !matches!(page, Some(crate::Presentation::Loot))
        {
            return Err("scripted reward continuation missing".into());
        }
    }
    match state.scripted_cleanup {
        Some(Stage::Survivors | Stage::OrdinarySurvivors | Stage::RewardSurvivors(_))
            if state.phase != Phase::Combat || state.survivors.is_none() =>
        {
            Err(RuleError::Invalid(
                "cleanup survivor continuation missing".into(),
            ))
        }
        Some(Stage::Complete)
            if state.phase != Phase::Exploration
                || state.combat.is_some()
                || state
                    .exploration
                    .as_ref()
                    .is_none_or(|p| p.script.is_none()) =>
        {
            Err(RuleError::Invalid(
                "cleanup script continuation missing".into(),
            ))
        }
        _ => Ok(()),
    }
}

/// Closes the rewards window of a cleanup, completes it and resumes the script.
pub(crate) fn close_rewards(
    state: &mut Session,
    content: &Content,
) -> Result<Option<Vec<Event>>, RuleError> {
    if state.scripted_cleanup != Some(Stage::Rewards) {
        return Ok(None);
    }
    let mut events = complete(state, content)?;
    events.extend(crate::exploration::run_script(state, content)?);
    Ok(Some(events))
}
