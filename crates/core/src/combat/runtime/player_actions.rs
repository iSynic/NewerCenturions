//! A warrior's ordinary combat actions.
use super::{end_turn, finish_player_action, neighbors, refusal};
use crate::combat::Combat;
use crate::{Event, Phase, RandomState, RuleError, Session};
use centurions_content::{Content, Warrior};

/// Chooses the target a body attacks next.
pub fn select_enemy_target(
    combat: &mut Combat,
    warriors: &[Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    body: i16,
) -> Result<i16, RuleError> {
    crate::combat::targeting::select_actor_target(combat, warriors, mission, random, body)
}

/// The active warrior guards for the rest of the turn: a guard sound plays and the turn ends.
pub fn player_guard(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    if state.phase != Phase::Combat
        || state
            .combat_story
            .as_ref()
            .is_some_and(|story| !story.waiting_child)
        || state
            .exploration
            .as_ref()
            .is_none_or(|position| position.presentation.is_some())
    {
        return Err(RuleError::Invalid(
            "guard requires an unobstructed player combat turn".into(),
        ));
    }
    let body = state
        .combat
        .as_ref()
        .and_then(Combat::active_body)
        .filter(|body| (0..6).contains(body))
        .ok_or_else(|| RuleError::Invalid("active warrior missing".into()))?;
    let warrior = state
        .warriors
        .get_mut(body as usize)
        .ok_or_else(|| RuleError::Invalid("active warrior missing".into()))?;
    if warrior.record().in_battle() == 0 || warrior.record().stamina()? <= 0 {
        return Err(RuleError::Invalid(
            "guard requires a living participant".into(),
        ));
    }
    if warrior.record().action_points()? < 2 {
        return Err(RuleError::Invalid(
            "guard requires a current player input turn".into(),
        ));
    }
    warrior.record_mut().set_guarding(1);
    let mut events = vec![Event::Sound { resource_id: 10121 }];
    events.extend(end_turn(state, content)?);
    Ok(events)
}

/// The active warrior attacks a neighboring enemy.
pub fn player_attack(
    state: &mut Session,
    content: &Content,
    target: i16,
) -> Result<Vec<Event>, RuleError> {
    player_attack_inner(state, content, target, false)
}

/// The active warrior attacks a friendly neighbor; refused silently when the target is not one.
pub fn player_attack_friend(
    state: &mut Session,
    content: &Content,
    target: i16,
) -> Result<Vec<Event>, RuleError> {
    if !crate::combat::interaction::is_friendly_neighbor(state, target)? {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::GameFeedback::Silent],
        });
    }
    player_attack_inner(state, content, target, true)
}

/// The active warrior attacks a neighbor, spending an attack action. A friendly monster that is
/// attacked turns hostile and joins the count of enemies. A target out of reach is refused.
///
/// Original: attack
pub(super) fn player_attack_inner(
    state: &mut Session,
    content: &Content,
    target: i16,
    friendly: bool,
) -> Result<Vec<Event>, RuleError> {
    let combat = state
        .combat
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
    let body = combat
        .active_body()
        .ok_or_else(|| RuleError::Invalid("active warrior missing".into()))?;
    if !(0..6).contains(&body)
        || !friendly && !neighbors(combat, &state.warriors, body)?.contains(&target)
    {
        return Err(refusal(crate::feedback::CombatRefusal::OutOfRange));
    }
    if friendly
        && target > 10
        && combat.roster.records[(target - 10) as usize]
            .record()
            .faction()?
            == 0
    {
        combat.roster.records[(target - 10) as usize]
            .record_mut()
            .set_faction(1);
        combat.kill_counts.add_num_enemy(1);
    }
    let warrior = &mut state.warriors[body as usize];
    if warrior.record().action_points()? < 2 {
        return Ok(vec![]);
    }
    crate::combat::records::spend_attack_action(warrior)?;
    let mut events = super::attacks::resolve_attack_with_death(
        combat,
        &mut state.warriors,
        &mut state.random,
        body,
        target,
        0,
        body,
        Some(&content.items),
        Some(
            &crate::Session::land_in(
                content,
                state.mission.as_deref(),
                state.scenario_level,
                state.exploration.as_ref(),
            )
            .unwrap()
            .combat
            .as_ref()
            .unwrap()
            .attacks,
        ),
        &state.party_condition,
        &mut state.globals.loaded_items,
        Some(state.globals.death_processing),
    )?;
    events.extend(finish_player_action(state, content, body)?);
    Ok(events)
}
