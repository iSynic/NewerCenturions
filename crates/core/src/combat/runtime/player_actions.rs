//! Player actions in the original combat execution order.
use super::{
    alive, end_turn, finish_player_action, hostile, neighbors, position, refusal, resolve_attack,
};
use crate::combat::{Combat, lifecycle};
use crate::{Event, Phase, RandomState, RuleError, Session};
use centurions_content::{Content, Warrior};

pub fn select_enemy_target(
    c: &mut Combat,
    w: &[Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    body: i16,
) -> Result<i16, RuleError> {
    if body < 10 || body >= 10 + c.enemies.records.len() as i16 {
        return Err(RuleError::Invalid("invalid targeting enemy".into()));
    }
    let e = (body - 10) as usize;
    let mut target = c.enemies.records[e].record().target_body()?;
    if !alive(c, w, target) || !hostile(c, w, body, target)? {
        let mut selected = None;
        for _ in 0..10000 {
            let t = random.bound(c.enemies.records.len() as i16 + 10) - 1;
            if alive(c, w, t) && hostile(c, w, body, t)? {
                selected = Some(t);
                break;
            }
        }
        target = selected.ok_or_else(|| RuleError::Invalid("no eligible enemy target".into()))?;
    }
    let mut scan = 0;
    let selected_target = target;
    loop {
        c.enemies.records[e].record_mut().set_target_body(target);
        if lifecycle::can_see(c, mission, position(c, body), position(c, target), 0)? {
            break;
        }
        if scan >= 110 {
            target = selected_target;
            c.enemies.records[e].record_mut().set_target_body(target);
            break;
        }
        target = scan;
        scan += 1;
        while !alive(c, w, target) || !hostile(c, w, body, target)? {
            if scan >= 110 {
                target = selected_target;
                break;
            }
            target = scan;
            scan += 1;
        }
    }
    Ok(target)
}

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
    if warrior.record().active_flag() == 0 || warrior.record().stamina()? <= 0 {
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

pub fn player_attack(
    state: &mut Session,
    content: &Content,
    target: i16,
) -> Result<Vec<Event>, RuleError> {
    player_attack_inner(state, content, target, false)
}

pub fn player_attack_friend(
    state: &mut Session,
    content: &Content,
    target: i16,
) -> Result<Vec<Event>, RuleError> {
    if !crate::combat::interaction::is_friendly_neighbor(state, target)? {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::NativeFeedback::Silent],
        });
    }
    player_attack_inner(state, content, target, true)
}

pub(super) fn player_attack_inner(
    state: &mut Session,
    content: &Content,
    target: i16,
    friendly: bool,
) -> Result<Vec<Event>, RuleError> {
    let c = state
        .combat
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
    let body = c
        .active_body()
        .ok_or_else(|| RuleError::Invalid("active warrior missing".into()))?;
    if !(0..6).contains(&body)
        || !friendly && !neighbors(c, &state.warriors, body)?.contains(&target)
    {
        return Err(refusal(crate::feedback::CombatRefusal::OutOfRange));
    }
    let warrior = &mut state.warriors[body as usize];
    if warrior.record().action_points()? < 2 {
        return Ok(vec![]);
    }
    let movement = warrior.record().movement_points()?.wrapping_sub(3).max(0);
    let actions = warrior.record().action_points()?.wrapping_sub(2);
    warrior.record_mut().set_movement_points(movement);
    warrior.record_mut().set_action_points(actions);
    let mut events = resolve_attack(
        c,
        &mut state.warriors,
        &mut state.random,
        body,
        target,
        0,
        body,
        Some(&content.items),
        Some(
            &content
                .map(state.exploration.as_ref().unwrap().map)
                .unwrap()
                .combat
                .as_ref()
                .unwrap()
                .attacks,
        ),
        &state.party_buffs,
        &mut state.ammunition_work,
    )?;
    events.extend(finish_player_action(state, content, body)?);
    Ok(events)
}
