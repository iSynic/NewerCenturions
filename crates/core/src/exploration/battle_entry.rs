//! Battle entry for maintained exploration.
use crate::{Event, RuleError, Session, combat};
use centurions_content::Content;

#[derive(Clone, Copy)]
pub(crate) enum StoryBattle {
    Ordinary,
    Group,
    Selected,
    Npc,
}

pub(crate) fn begin_story_battle(
    state: &mut Session,
    content: &Content,
    entry: StoryBattle,
    parameter: i16,
) -> Result<Vec<Event>, RuleError> {
    let turn_owns = state
        .turn_dispatch
        .as_ref()
        .is_some_and(|frame| frame.owns_script(state));
    let round_owns = state
        .round_dispatch
        .as_ref()
        .is_some_and(|frame| frame.owns_script(state));
    let prior_round = state.round_dispatch.clone();
    let prior_turn = state.turn_dispatch.clone();
    let prior_cleanup = state.scripted_cleanup.clone();
    let prior_selected = state.selected_battle_treasure.clone();
    if prior_selected.is_some() && !round_owns && !turn_owns && state.combat_story.is_none() {
        return Err(RuleError::Unsupported {
            action: "story.selected_battle",
            dependency: "child battle without a suspended selected caller",
        });
    }
    let outer = (round_owns
        && ((state.combat.is_some()
            && prior_cleanup != Some(crate::story_cleanup::Stage::Complete))
            || prior_selected.is_some()))
    .then(|| combat::round_stories::RoundBattleReturn {
        after_message: state.battle_after_message,
        treasure: state.battle_treasure,
        rewards: state.ordinary_battle_rewards.clone(),
        selected: prior_selected.clone(),
        group: state.battle_group_script,
        npc: state.battle_npc.clone(),
    });
    let death_depth = combat::round_stories::RoundDispatch::death_depth(state);
    if let Some(frame) = state
        .round_dispatch
        .as_mut()
        .filter(|frame| !turn_owns && !frame.child_battle)
    {
        frame.child_battle = true;
        frame.outer_battle = outer;
        frame.death_depth.get_or_insert(death_depth);
    }
    if turn_owns {
        combat::runtime::begin_turn_child(state)?;
    }
    if prior_cleanup == Some(crate::story_cleanup::Stage::Complete) {
        state.scripted_cleanup = None;
    }
    let mut continuation = state.combat_story.take();
    let prior_wait = continuation.as_ref().map(|story| story.waiting_child);
    if let Some(story) = continuation.as_mut().filter(|_| !round_owns && !turn_owns) {
        story.waiting_child = true;
        story.outer_selected = prior_selected.clone();
    }
    state.selected_battle_treasure = None;
    let events = match entry {
        StoryBattle::Ordinary => combat::begin(state, content, parameter),
        StoryBattle::Group => combat::begin_group(state, content, parameter),
        StoryBattle::Selected if turn_owns || round_owns || continuation.is_some() => {
            combat::begin_selected_child(state, content, parameter)
        }
        StoryBattle::Selected => combat::begin_selected_battle(state, content, parameter),
        StoryBattle::Npc => combat::begin_npc(state, content, parameter),
    }?;
    // A terminal battle can discard the mission before entry returns. Its
    // former caller belongs to that discarded context and must not be restored.
    if state.exploration.is_none() {
        return Ok(events);
    }
    if !events
        .iter()
        .any(|event| matches!(event, Event::CombatStarted { .. }))
    {
        state.round_dispatch = prior_round;
        state.turn_dispatch = prior_turn;
        state.scripted_cleanup = prior_cleanup;
        state.selected_battle_treasure = prior_selected;
        if let Some(story) = continuation.as_mut() {
            story.waiting_child = prior_wait.unwrap();
        }
    }
    combat::runtime::retain_death_parent(state, continuation);
    Ok(events)
}
