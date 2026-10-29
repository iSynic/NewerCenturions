use crate::{Content, Event, Phase, RuleError, Session, combat, story_health, story_inventory};
mod rewards;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Stage {
    Survivors,
    Complete,
    OrdinarySurvivors,
    RewardSurvivors(i16),
    Rewards,
    RetainedSurvivors {
        mode: i16,
        ordinary: bool,
    },
    RetainedSelectedSurvivors {
        mode: i16,
    },
    RetainedGetupSurvivors {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parent: Option<Box<Stage>>,
    },
}

impl Stage {
    pub(crate) fn pauses(&self) -> bool {
        !matches!(self, Self::Complete)
    }
}

/// newland100 doubles its signed WORD parameter before calling booty.
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
    let mode = combat::reward_work::effective_mode(state, requested_mode)?;
    combat::runtime::begin_selected_cleanup(state)?;
    if let Some(combat) = state.combat.as_mut() {
        combat.victory_work.finish();
    } else {
        state.native_work.victory_work.finish();
    }
    if mode == 10 {
        return crate::loot_recovery::begin(
            state,
            content,
            crate::loot_recovery::RecoveryReturn::Scripted,
        );
    }
    if state.combat.is_none() {
        return combat::runtime::begin_retained_rewards(state, content, mode, false);
    }
    if mode != 2 {
        return rewards::begin(state, content, mode);
    }
    begin_early(state, content, Stage::Survivors)
}

pub(crate) fn begin_ordinary(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    begin_early(state, content, Stage::OrdinarySurvivors)
}

/// Early booty updates all100 backing slots independently of the active count.
pub(crate) fn early_retained_prefix(state: &mut Session) -> Result<(), RuleError> {
    let unavailable = state
        .warriors
        .iter()
        .filter(|warrior| {
            warrior.stamina().unwrap_or(0) < 1 || warrior.record().automatic_mode().unwrap_or(0) < 0
        })
        .count() as i8;
    state.warriors = story_inventory::normalize_story_loot(&state.warriors)?;
    let mut health =
        story_health::retained_context(state.selected_warrior, &state.native_work.health_context);
    health.unavailable = unavailable;
    state.native_work.health_context = health;
    state.clear_tactical_fields();
    state
        .booty_items
        .fill(centurions_content::InventoryRecord([0; 22]));
    for warrior in &mut state.warriors {
        warrior.record_mut().set_active_flag(1);
    }
    for slot in state.native_work.npc_backing.slots.iter_mut().flatten() {
        if slot.record.record().survivor_marker()? != -1 {
            slot.record.record_mut().set_stamina(0);
        }
    }
    Ok(())
}

fn begin_early(
    state: &mut Session,
    content: &Content,
    stage: Stage,
) -> Result<Vec<Event>, RuleError> {
    let mut events = combat::runtime::refit_after_combat(state, content)?;
    let unavailable = state
        .warriors
        .iter()
        .filter(|warrior| {
            warrior.stamina().unwrap_or(0) < 1 || warrior.record().automatic_mode().unwrap_or(0) < 0
        })
        .count() as i8;
    state.warriors = story_inventory::normalize_story_loot(&state.warriors)?;
    for warrior in &mut state.warriors {
        warrior.record_mut().set_active_flag(1);
    }
    let mut health =
        story_health::retained_context(state.selected_warrior, &state.native_work.health_context);
    health.unavailable = unavailable;
    state.native_work.health_context = health;
    let arena = state.combat.as_mut().ok_or("cleanup combat missing")?;
    arena.unavailable = unavailable;
    for record in &mut arena.enemies.records {
        if record.record().survivor_marker()? != -1 {
            record.record_mut().set_stamina(0);
        }
    }
    for slot in arena
        .retained_npcs
        .slots
        .iter_mut()
        .skip(arena.enemies.records.len())
        .flatten()
    {
        if slot.record.record().survivor_marker()? != -1 {
            slot.record.record_mut().set_stamina(0);
        }
    }
    state.clear_tactical_fields();
    state
        .booty_items
        .fill(centurions_content::InventoryRecord([0; 22]));
    state.scripted_cleanup = Some(stage);
    if let Some(presentation) = combat::runtime::begin_survivors(state)? {
        events.extend(presentation);
    } else {
        events.extend(confirm(state, content)?);
    }
    Ok(events)
}

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
    combat::runtime::retain_last_defeated(state)?;
    let mut events = combat::runtime::synchronize_reward_survivors(state)?;
    state.exploration.as_mut().unwrap().presentation = None;
    combat::runtime::detach_reward_combat(state)?;
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

pub(crate) fn complete(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    state.native_work.battle_rewards_suppressed = 1;
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
    // Current mutable records are the reconstruction's read-only replacement for loadland.
    Ok(vec![Event::Moved {
        x: position.x,
        y: position.y,
    }])
}

pub(crate) fn validate(state: &Session) -> Result<(), RuleError> {
    if let Some(Stage::RetainedSelectedSurvivors { mode }) = state.scripted_cleanup {
        combat::runtime::validate_retained_reward_count(state)?;
        if state.phase != Phase::Exploration
            || state.combat.is_some()
            || state.selected_battle_treasure.is_none()
            || state.survivors.is_none()
            || !matches!(mode, 0..=2)
        {
            return Err("invalid selected survivor return".into());
        }
    }
    if let Some(Stage::RetainedGetupSurvivors { parent }) = &state.scripted_cleanup {
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
    if let Some(Stage::RetainedSurvivors { mode, ordinary }) = state.scripted_cleanup {
        combat::runtime::validate_retained_reward_count(state)?;
        if state.phase != Phase::Exploration
            || state.combat.is_some()
            || state.survivors.is_none()
            || ordinary && state.ordinary_battle_rewards.is_none()
            || mode == 10
        {
            return Err("invalid retained survivor return".into());
        }
        if mode != 2 {
            combat::rewards::RewardMode::select(mode, &state.native_work.treasure_record)?;
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
