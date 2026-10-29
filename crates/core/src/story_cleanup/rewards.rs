//! newland100 reward modes which return to the current script after Close.
use super::{Stage, combat, story_health, story_inventory};
use crate::{Content, Event, Presentation, RuleError, Session};

pub(super) fn begin(
    state: &mut Session,
    content: &Content,
    mode: i16,
) -> Result<Vec<Event>, RuleError> {
    combat::rewards::RewardMode::select(mode, &state.native_work.treasure_record)?;
    let count = state
        .combat
        .as_ref()
        .ok_or("cleanup combat missing")?
        .enemies
        .records
        .len() as i16;
    combat::runtime::reward_cleanup::validate(count)?;
    let mut events = combat::runtime::refit_after_combat(state, content)?;
    let unavailable = state
        .warriors
        .iter()
        .filter(|warrior| {
            warrior.stamina().unwrap_or(0) < 1 || warrior.record().automatic_mode().unwrap_or(0) < 0
        })
        .count() as i8;
    state.warriors = story_inventory::normalize_story_loot(&state.warriors)?;
    state.combat.as_mut().unwrap().unavailable = unavailable;
    let mut health =
        story_health::retained_context(state.selected_warrior, &state.native_work.health_context);
    health.unavailable = unavailable;
    state.native_work.health_context = health;
    state.clear_tactical_fields();
    state
        .booty_items
        .fill(centurions_content::InventoryRecord([0; 22]));
    state.scripted_cleanup = Some(Stage::RewardSurvivors(mode));
    if let Some(survivors) = combat::runtime::begin_survivors(state)? {
        events.extend(survivors);
    } else {
        events.extend(confirm(state, content)?);
    }
    Ok(events)
}

pub(super) fn confirm(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let Some(Stage::RewardSurvivors(mode)) = state.scripted_cleanup else {
        return Err("scripted reward mode missing".into());
    };
    let count = state
        .combat
        .as_ref()
        .ok_or("cleanup combat missing")?
        .enemies
        .records
        .len() as i16;
    combat::runtime::reward_cleanup::validate(count)?;
    combat::runtime::retain_last_defeated(state)?;
    let mut events = combat::runtime::synchronize_reward_survivors(state)?;
    let rewards = combat::rewards::prepare_mode(
        &state.warriors,
        &state
            .combat
            .as_ref()
            .ok_or("cleanup combat missing")?
            .enemies
            .records,
        content,
        combat::rewards::RewardContext {
            difficulty: state.difficulty,
            npc_denominator: 0,
            currency: state.party_currency,
            mode: combat::rewards::RewardMode::select(mode, &state.native_work.treasure_record)?,
        },
        &mut state.random,
    )?;
    let count = state.combat.as_ref().unwrap().enemies.records.len() as i16;
    combat::runtime::install_rewards(state, content, mode, rewards)?;
    combat::runtime::detach_reward_combat(state)?;
    combat::runtime::reward_cleanup::finish(state, count, mode)?;
    if let Some(story) = state.combat_story.as_mut() {
        story.child_returned = true;
    }
    state.scripted_cleanup = Some(Stage::Rewards);
    state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
    events.push(Event::Sound {
        resource_id: crate::feedback::BOOTY_SOUND,
    });
    Ok(events)
}
