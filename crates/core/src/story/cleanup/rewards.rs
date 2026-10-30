//! Combat cleanup reward modes, which return to the current script after Close.
use super::{Stage, combat, story_health, story_inventory};
use crate::{Content, Event, Presentation, RuleError, Session};

/// Begins the reward mode a scripted cleanup chose.
pub(super) fn begin(
    state: &mut Session,
    content: &Content,
    mode: i16,
) -> Result<Vec<Event>, RuleError> {
    combat::rewards::RewardMode::select(mode, &state.globals.treasure)?;
    let count = state
        .combat
        .as_ref()
        .ok_or("cleanup combat missing")?
        .roster
        .records
        .len() as i16;
    combat::runtime::reward_cleanup::validate(count)?;
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
    state.combat.as_mut().unwrap().kill_party = kill_party;
    let mut health =
        story_health::reselected_context(state.char_select_new, &state.globals.health_context);
    health.kill_party = kill_party;
    state.globals.health_context = health;
    state.clear_tactical_fields();
    state
        .booty_list
        .fill(centurions_content::InventoryRecord([0; 22]));
    state.scripted_cleanup = Some(Stage::RewardSurvivors(mode));
    if let Some(survivors) = combat::runtime::begin_survivors(state)? {
        events.extend(survivors);
    } else {
        events.extend(confirm(state, content)?);
    }
    Ok(events)
}

/// Confirms the survivors for a reward mode and hands out the rewards.
pub(super) fn confirm(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let Some(Stage::RewardSurvivors(mode)) = state.scripted_cleanup else {
        return Err("scripted reward mode missing".into());
    };
    let count = state
        .combat
        .as_ref()
        .ok_or("cleanup combat missing")?
        .roster
        .records
        .len() as i16;
    combat::runtime::reward_cleanup::validate(count)?;
    combat::runtime::store_last_defeated(state)?;
    let mut events = combat::runtime::synchronize_reward_survivors(state)?;
    let rewards = combat::rewards::prepare_mode(
        &state.warriors,
        &state
            .combat
            .as_ref()
            .ok_or("cleanup combat missing")?
            .roster
            .records,
        content,
        combat::rewards::RewardContext {
            difficulty: state.difficulty,
            deduction: 0,
            currency: state.party_currency,
            mode: combat::rewards::RewardMode::select(mode, &state.globals.treasure)?,
        },
        &mut state.random,
    )?;
    let count = state.combat.as_ref().unwrap().roster.records.len() as i16;
    combat::runtime::install_rewards(state, content, mode, rewards)?;
    combat::runtime::leave_reward_battle(state)?;
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
