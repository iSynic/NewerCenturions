//! Turns of warriors on automatic.
use super::{do_reload, end_turn, finish_player_action, hostile, neighbors, player_shoot_action};
use crate::{Event, Phase, RuleError, Session, feedback::GameFeedback};
use centurions_content::Content;
use centurions_content::records::ItemDefinition;

/// Starts the active monster's turn; its movement, firearm and spell steps follow as the turn
/// resumes.
pub fn enemy_turn(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    super::monster_return::begin(state, content)
}

/// The warrior whose turn the game plays itself, during a battle: the active, standing warrior
/// who is on automatic, under computer control, on the other faction or fleeing, or any active
/// warrior while the party is possessed.
pub fn controlled_warrior(state: &Session) -> Option<usize> {
    let body = state.combat.as_ref()?.active_body()?;
    let member = usize::try_from(body).ok()?;
    let warrior = state.warriors.get(member)?;
    (state.phase == Phase::Combat
        && warrior.record().in_battle() != 0
        && warrior.stamina().ok()? > 0
        && (state.combat.as_ref()?.do_auto[member]
            || state.globals.possessed != 0
            || warrior
                .record()
                .computer_control()
                .is_ok_and(|value| value != 0)
            || warrior.record().faction().is_ok_and(|value| value != 0)
            || warrior
                .record()
                .fleeing_duration()
                .is_ok_and(|value| value != 0)))
    .then_some(member)
}

/// Plays a warrior on automatic; each command completes a move, an attack or the turn. A warrior
/// with a firearm in hand refits it when his pack's first slot is spent, or when he has too few
/// action points left for a shot; a refit that leaves him one action point shows the tutorial on
/// action points and lets him fire with it. He then fires at his chosen target before he looks to
/// move, unless he is retreating without having refitted.
///
/// Original: combatchoice
pub fn controlled_turn(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let member = controlled_warrior(state).ok_or("no controlled warrior turn")?;
    if state
        .exploration
        .as_ref()
        .is_none_or(|p| p.presentation.is_some())
        || state
            .combat_story
            .as_ref()
            .is_some_and(|s| !s.waiting_child)
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    state.globals.possessed = 1;
    state.warriors[member].record_mut().set_computer_control(1);
    let firearm = choose_firearm(state, content, member)?;
    let mut events = vec![];
    if firearm {
        let warrior = state.warriors[member].record();
        let empty = warrior.inventory_quantity(0)? == 0;
        let fleeing = warrior.fleeing_duration()? != 0;
        let reload = if empty {
            warrior.toggle()
        } else if fleeing {
            false
        } else {
            state.globals.reload_shot = 0;
            state.warriors[member].record().action_points()? < 2
        };
        if reload {
            state.globals.reload_shot = 1;
            events = do_reload(state, content, member)?;
            let warrior = state.warriors[member].record();
            let points = warrior.action_points()?;
            if warrior.computer_control()? == 0 || points < 1 {
                events.extend(finish_player_action(state, content, member as i16)?);
                return Ok(events);
            }
            if points < 2 {
                events.push(Event::Feedback(GameFeedback::Tutorial {
                    first: 79,
                    last: 80,
                }));
            }
        }
        let target = state.warriors[member].record().target()?;
        let combat = state.combat.as_ref().unwrap();
        if (reload || !fleeing)
            && combat.body_alive(&state.warriors, target)
            && hostile(combat, &state.warriors, member as i16, target)?
        {
            match player_shoot_action(state, content, target) {
                Ok(shot) => {
                    events.extend(shot);
                    events.extend(finish_player_action(state, content, member as i16)?);
                    return Ok(events);
                }
                Err(RuleError::Refused { .. }) => {}
                Err(error) => return Err(error),
            }
        }
    }
    events.extend(advance(state, content, member)?);
    Ok(events)
}

/// A warrior on automatic who has not fired leaves the field from its edge, waits on guard without
/// the movement or action points to act, or picks a hostile target and moves toward it, away from
/// it while retreating, attacking whatever stands in the way.
///
/// Original: combatchoice
fn advance(state: &mut Session, content: &Content, member: usize) -> Result<Vec<Event>, RuleError> {
    let combat = state.combat.as_ref().unwrap();
    let edge_point = combat.squad.pos[member];
    let absolute = (
        edge_point.0 + combat.field.map_shift.0 + 5,
        edge_point.1 + combat.field.map_shift.1 + 5,
    );
    if !(1..=88).contains(&absolute.0) || !(1..=88).contains(&absolute.1) {
        return crate::combat::retreat::leave_edge(state, content, member);
    }
    if state.warriors[member].record().movement()? < 1
        || state.warriors[member].record().action_points()? < 2
    {
        state.warriors[member].record_mut().set_guarding(1);
        return end_turn(state, content);
    }
    let neighbors = neighbors(combat, &state.warriors, member as i16)?;
    let mut target = neighbors
        .first()
        .copied()
        .unwrap_or(state.warriors[member].record().target()?);
    if !combat.body_alive(&state.warriors, target)
        || !hostile(combat, &state.warriors, member as i16, target)?
    {
        let eligible = (0..110_i16).any(|body| {
            combat.body_alive(&state.warriors, body)
                && hostile(combat, &state.warriors, member as i16, body).unwrap_or(false)
        });
        if !eligible {
            return end_turn(state, content);
        }
        let mut selected = None;
        for _ in 0..10_000 {
            let candidate = state.random.range(0, 109);
            if combat.body_alive(&state.warriors, candidate)
                && hostile(combat, &state.warriors, member as i16, candidate)?
            {
                selected = Some(candidate);
                break;
            }
        }
        target = selected.ok_or("automatic target selection budget exceeded")?;
    }
    let to = combat.body_position(target);
    let from = combat.body_position(member as i16);
    let fleeing = state.warriors[member].record().fleeing_duration()? != 0;
    let mut direction = (
        (to.0 - from.0).signum() as i8,
        (to.1 - from.1).signum() as i8,
    );
    if fleeing {
        direction = (-direction.0, -direction.1);
    }
    state.warriors[member].record_mut().set_target(target);
    if !fleeing && state.warriors[member].record().toggle() {
        match player_shoot_action(state, content, target) {
            Ok(mut events) => {
                events.extend(finish_player_action(state, content, member as i16)?);
                return Ok(events);
            }
            Err(RuleError::Refused { .. }) => {}
            Err(error) => return Err(error),
        }
    }
    for _ in 0..21 {
        if direction != (0, 0) {
            let before = state.combat.as_ref().unwrap().squad.pos[member];
            let events = crate::combat::move_controlled(state, content, direction.0, direction.1)?;
            if state.combat.as_ref().is_none_or(|c| {
                c.active_body() != Some(member as i16) || c.squad.pos[member] != before
            }) || events
                .iter()
                .any(|e| matches!(e, Event::AttackResolved { .. }))
            {
                return Ok(events);
            }
        }
        direction = (
            state.random.bound(3) as i8 - 2,
            state.random.bound(3) as i8 - 2,
        );
    }
    state.warriors[member].record_mut().set_guarding(1);
    end_turn(state, content)
}

/// A warrior under computer control who carries a firearm takes it up when the first item in his
/// pack holds rounds and he is not retreating. Looking at the firearm loads it and its spell as
/// the item and spell being worked on. Returns whether he carries one.
///
/// Original: tomissle, combatchoice
fn choose_firearm(
    state: &mut Session,
    content: &Content,
    member: usize,
) -> Result<bool, RuleError> {
    let firearm = state.warriors[member].record().equipped_weapon()?;
    if firearm == 0 {
        return Ok(false);
    }
    state.load_item(content, firearm)?;
    let spell = ItemDefinition(
        state
            .globals
            .loaded_items
            .item
            .as_ref()
            .ok_or("firearm item work missing")?,
    )
    .spell()?;
    crate::items::load_spell(state, content, spell)?;
    let warrior = state.warriors[member].record();
    if warrior.inventory_quantity(0)? != 0 && warrior.fleeing_duration()? == 0 {
        state.warriors[member].record_mut().set_toggle(true);
    }
    Ok(true)
}
