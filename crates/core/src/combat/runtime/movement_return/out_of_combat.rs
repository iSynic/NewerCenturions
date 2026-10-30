//! A warrior's move that finishes after the battle has ended.
use super::super::reaction_calls::ReactionReturn;
use super::{MoveFrame, MovePhase, MovePlan, OutOfCombatWalkChoice, ReactionFrame, mission};
use crate::combat::map_alias;
use crate::combat::runtime::TurnFlow;
use crate::combat::runtime::out_of_combat_getup;
use crate::{Content, Event, RuleError, Session, combat::runtime::neighbor_scan};
mod casting;
pub(in crate::combat::runtime) use casting::{
    FieldOrigin as OutOfCombatFieldOrigin, field_target_mask as out_of_combat_field_target_mask,
    out_of_combat_flight, out_of_combat_ray,
};
mod occupied;
mod retry;
mod target_list;

/// A cell of the tactical map as column and row.
type Point = (i16, i16);

/// Finishes a warrior's step after the battle has ended. The suspended reaction completes first;
/// if it interrupted the warrior after the departure check or after the step, the turn ends.
/// Otherwise a step still at its first check starts its attempt (an involuntary mover retries
/// blocked or unaffordable steps), and a step past the departure check is taken.
///
/// Original: combatchoice
pub(super) fn resume(
    state: &mut Session,
    content: &Content,
    mut frame: MoveFrame,
) -> Result<Vec<Event>, RuleError> {
    let returned = frame
        .guard
        .as_mut()
        .ok_or("retained movement reaction missing")?
        .finish_after_teardown(state, content)?;
    let mut events = returned.events;
    if returned.terminated {
        return Ok(events);
    }
    match frame.phase {
        MovePhase::AfterStep => {
            if returned.interrupted {
                events.extend(out_of_combat_getup::finish(state, content)?);
            }
            return Ok(events);
        }
        MovePhase::AfterDeparture if returned.interrupted => {
            events.extend(out_of_combat_getup::finish(state, content)?);
            return Ok(events);
        }
        MovePhase::Entry => return Err("retained movement has no returned reaction".into()),
        _ => {}
    }
    let mut attempt = frame.phase == MovePhase::AfterEntry;
    if attempt {
        state.globals.neighbors.departure = 0;
        if crate::combat::runtime::reaction_calls::turn_actor(state)? >= 0 {
            neighbor_scan::scan_out_of_combat(state, content, 1, &mut 0)?;
        }
    }
    loop {
        if attempt {
            if retry::involuntary(state)? {
                let locals = frame
                    .locals
                    .as_mut()
                    .ok_or_else(|| RuleError::from("walking retry locals missing"))?;
                if matches!(
                    retry::prepare(state, content, locals, &mut events)?,
                    retry::RetryStep::Finished
                ) {
                    return Ok(events);
                }
            }
            let actor = frame.actor(state)?;
            let direction = direction(state)?;
            state.warriors[actor]
                .record_mut()
                .face_horizontal_step(direction.0);
            let (point, shift) = position(state, actor)?;
            let tile = tile(
                state,
                content,
                index(
                    i32::from(point.0) + i32::from(direction.0),
                    i32::from(point.1) + i32::from(direction.1),
                    shift,
                ),
            )?
            .wrapping_abs();
            if let Some(locals) = frame.locals.as_mut() {
                locals.previous_tile = tile;
            }
            let map = mission(state, content)?;
            let property = map.tactical_movement_property(tile).map_err(|_| {
                unsupported(
                    "retained walking movement property outside the tile property table 0x43167c",
                )
            })?;
            frame.cost = crate::rules::movement_cost(property, direction.0, direction.1);
            if tile <= 110 {
                let involuntary = retry::involuntary(state)?;
                let outcome = occupied::begin(state, content, actor, tile, involuntary)?;
                if complete_occupied(state, content, tile, outcome, &mut events)? {
                    continue;
                }
                return Ok(events);
            }
            let blocked = tile >= 1000
                && map.movement_blocker(tile - 1000).map_err(|_| {
                    unsupported(
                        "retained walking terrain blocker outside the tile property table 0x43167c",
                    )
                })? != 0;
            if blocked && retry::involuntary(state)? {
                continue;
            }
            if blocked
                || state.warriors[actor]
                    .record()
                    .movement()?
                    .wrapping_sub(frame.cost)
                    < 0
            {
                if blocked {
                    events.push(Event::Sound { resource_id: -129 });
                }
                set_direction(state, (0, 0))?;
                if react_or_end(state, content, 2, &mut events)? {
                    return Ok(events);
                }
                if finish(state, content, frame.cost, &mut events)? {
                    continue;
                }
                return Ok(events);
            }
            frame.plan = MovePlan::Ground { tile };
            if react_or_end(state, content, 2, &mut events)? {
                return Ok(events);
            }
        }
        if !matches!(frame.plan, MovePlan::Ground { .. }) {
            return Err("movement departure plan missing".into());
        }
        if step(state, content, &frame, &mut events)? {
            return Ok(events);
        }
        if !finish(state, content, frame.cost, &mut events)? {
            return Ok(events);
        }
        attempt = true;
    }
}

/// Takes the ground step: movement is paid, the warrior and the tile under it change cells, the
/// landing sound plays and any field in the new cell acts. True when the step ends the turn.
fn step(
    state: &mut Session,
    content: &Content,
    frame: &MoveFrame,
    events: &mut Vec<Event>,
) -> Result<bool, RuleError> {
    let actor = frame.actor(state)?;
    let remaining = state.warriors[actor]
        .record()
        .movement()?
        .wrapping_sub(frame.cost);
    state.warriors[actor].record_mut().set_movement(remaining);
    let direction = direction(state)?;
    let (point, shift) = position(state, actor)?;
    let destination = (
        point.0.wrapping_add(i16::from(direction.0)),
        point.1.wrapping_add(i16::from(direction.1)),
    );
    let current = index(i32::from(point.0), i32::from(point.1), shift);
    let next = index(
        i32::from(point.0) + i32::from(direction.0),
        i32::from(point.1) + i32::from(direction.1),
        shift,
    );
    let under = state.globals.battlefield.char_under[actor];
    write(state, current, under)?;
    let arrived = tile(state, content, next)?;
    state.globals.battlefield.char_under[actor] = arrived;
    write(state, next, actor as i16)?;
    let battlefield = &mut state.globals.battlefield;
    battlefield.pos[actor] = destination;
    events.push(Event::TacticalMoved {
        body: actor as i16,
        x: destination.0,
        y: destination.1,
    });
    let sound = mission(state, content)?
        .movement_sound(arrived.wrapping_sub(1000))
        .map_err(|_| {
            unsupported("retained walking terrain sound outside the tile property table 0x43167c")
        })?;
    events.push(Event::Sound { resource_id: sound });
    let collision = out_of_combat_getup::collide_fields(state, content, actor as i16)?;
    events.extend(collision.events);
    if collision.flow == TurnFlow::Terminated {
        return Ok(true);
    }
    let actor = state.char_select_new;
    if state.warriors[actor].stamina()? < 1 {
        state.warriors[actor].record_mut().set_attacks(0);
        state.warriors[actor].record_mut().set_movement(0);
        return Ok(true);
    }
    Ok(false)
}

/// Closes a step: one that cost more movement than remained, or has no direction, is cancelled with
/// the blocked sound, or, for an involuntary mover, returns true to be retried. Then the check
/// after the step runs.
fn finish(
    state: &mut Session,
    content: &Content,
    cost: i16,
    events: &mut Vec<Event>,
) -> Result<bool, RuleError> {
    let actor = state.char_select_new;
    if state.warriors[actor]
        .record()
        .movement()?
        .wrapping_sub(cost)
        < 0
        || direction(state)? == (0, 0)
    {
        set_direction(state, (0, 0))?;
        if retry::involuntary(state)? {
            return Ok(true);
        }
        events.push(Event::Sound { resource_id: -130 });
    }
    react_or_end(state, content, 0, events)?;
    Ok(false)
}

/// Runs a reaction check and ends the turn when it interrupted the warrior. True when the turn is
/// over, because the check ended the battle or interrupted the warrior.
fn react_or_end(
    state: &mut Session,
    content: &Content,
    mode: usize,
    events: &mut Vec<Event>,
) -> Result<bool, RuleError> {
    match reaction(state, content, mode, events)? {
        ReactionReturn::Terminated => Ok(true),
        ReactionReturn::Returned(true) => {
            events.extend(out_of_combat_getup::finish(state, content)?);
            Ok(true)
        }
        ReactionReturn::Returned(false) => Ok(false),
    }
}

/// Applies what moving onto an occupied cell decided: retry the step (true), ask the player a
/// question, or cast.
fn complete_occupied(
    state: &mut Session,
    content: &Content,
    target: i16,
    outcome: occupied::Outcome,
    events: &mut Vec<Event>,
) -> Result<bool, RuleError> {
    events.extend(outcome.events);
    match outcome.control {
        occupied::Control::Retry => return Ok(true),
        occupied::Control::Question(question) => {
            state.out_of_combat_walk_choice = Some(OutOfCombatWalkChoice {
                body: target,
                exchange: question == occupied::Question::ExchangeOrAttack,
            });
        }
        occupied::Control::Cast(request) => {
            casting::apply(state, content, request, events)?;
        }
        occupied::Control::Returned | occupied::Control::Terminated => {}
    }
    Ok(false)
}

/// Applies the player's answer to the offer made at an occupied cell; cancelling the offer does
/// nothing.
pub(super) fn answer(
    state: &mut Session,
    content: &Content,
    exchange: Option<bool>,
) -> Result<Vec<Event>, RuleError> {
    let pending = state
        .out_of_combat_walk_choice
        .take()
        .ok_or("no retained occupied-cell choice")?;
    let question = if pending.exchange {
        occupied::Question::ExchangeOrAttack
    } else {
        occupied::Question::AttackFriendly
    };
    let choice = match exchange {
        None => occupied::Choice::Cancel,
        Some(true) if pending.exchange => occupied::Choice::Second,
        Some(false) if pending.exchange => occupied::Choice::First,
        Some(false) => occupied::Choice::Second,
        Some(true) => return Err("this occupied-cell question has no exchange choice".into()),
    };
    let outcome = occupied::answer(state, content, pending.body, question, choice)?;
    let mut events = vec![];
    if complete_occupied(state, content, pending.body, outcome, &mut events)? {
        return Err("occupied-cell answer unexpectedly resumed walking".into());
    }
    Ok(events)
}

/// Runs a reaction check of the given mode for the body whose turn it is.
fn reaction(
    state: &mut Session,
    content: &Content,
    mode: usize,
    events: &mut Vec<Event>,
) -> Result<ReactionReturn, RuleError> {
    let body = crate::combat::runtime::reaction_calls::turn_actor(state)?;
    let mut frame = ReactionFrame::new(body, mode, direction(state)?)?;
    let outcome = frame.finish_after_teardown(state, content)?;
    events.extend(outcome.events);
    Ok(if outcome.terminated {
        ReactionReturn::Terminated
    } else {
        ReactionReturn::Returned(outcome.interrupted)
    })
}

/// The direction of the warrior's latest step.
fn direction(state: &Session) -> Result<(i8, i8), RuleError> {
    let value = state
        .exploration
        .as_ref()
        .ok_or_else(|| RuleError::from("walk outside a mission"))?
        .last_move;
    Ok((value[0], value[1]))
}

/// Sets the direction of the warrior's latest step.
fn set_direction(state: &mut Session, value: (i8, i8)) -> Result<(), RuleError> {
    state
        .exploration
        .as_mut()
        .ok_or_else(|| RuleError::from("walk outside a mission"))?
        .last_move = [value.0, value.1];
    Ok(())
}

/// The warrior's cell and the shift of the tactical map.
fn position(state: &Session, actor: usize) -> Result<(Point, Point), RuleError> {
    let battlefield = &state.globals.battlefield;
    let map = &battlefield.field;
    let point = battlefield.pos[actor];
    Ok((point, map.map_shift))
}

/// The walker's linear map index, column × 90 + row, without clipping.
fn index(x: i32, y: i32, shift: (i16, i16)) -> i32 {
    let x = x.wrapping_add(i32::from(shift.0)).wrapping_add(5);
    let y = y.wrapping_add(i32::from(shift.1)).wrapping_add(5);
    x.wrapping_mul(90).wrapping_add(y)
}

/// The tile at a map index, including the cells beside the map that a step can reach.
fn tile(state: &Session, content: &Content, index: i32) -> Result<i16, RuleError> {
    let aliases = map_alias::session_map_aliases!(state, content content);
    aliases
        .read(&state.globals.battlefield.field, index)?
        .ok_or_else(|| {
            unsupported(
                "retained walking read outside the map and the represented memory beside it",
            )
        })
}

/// Writes a tile at a map index, refused beyond the map and the land's arrays.
fn write(state: &mut Session, index: i32, value: i16) -> Result<(), RuleError> {
    let land = state
        .exploration
        .as_mut()
        .map(|position| (&mut position.objects[..], &mut position.events[..]));
    let map = &mut state.globals.battlefield.field;
    if map_alias::write(map, land, index, value) {
        Ok(())
    } else {
        Err(unsupported(
            "retained walking writes beside the tactical map outside its edge WORD and land arrays",
        ))
    }
}

/// The refusal for a step that depends on an unrecovered part of the game.
fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.retained_walk",
        dependency,
    }
}
