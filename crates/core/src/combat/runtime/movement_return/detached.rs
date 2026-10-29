//! combatchoice's saved walking suffix over shared arrays after arena teardown.
use super::super::reaction_calls::ReactionReturn;
use super::{MoveFrame, MovePhase, MovePlan, ReactionFrame, RetainedWalkChoice, mission};
use crate::{Content, Event, RuleError, Session, combat::neighbor_work};
mod casting;
pub(in crate::combat::runtime) use casting::{
    FieldOrigin as RetainedFieldOrigin, field_target_mask as retained_field_target_mask,
    retained_flight, retained_ray,
};
mod occupied;
mod retry;
mod target_list;

type Point = (i16, i16);

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
                events.extend(crate::combat::runtime::retained_turn::finish(
                    state, content,
                )?);
            }
            return Ok(events);
        }
        MovePhase::AfterDeparture if returned.interrupted => {
            events.extend(crate::combat::runtime::retained_turn::finish(
                state, content,
            )?);
            return Ok(events);
        }
        MovePhase::Entry => return Err("retained movement has no returned reaction".into()),
        _ => {}
    }
    let mut attempt = frame.phase == MovePhase::AfterEntry;
    if attempt {
        state.native_work.neighbor_work.departure = 0;
        if crate::combat::runtime::reaction_calls::retained_actor(state)? >= 0 {
            neighbor_work::scan_retained(state, content, 1, &mut 0)?;
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
                    .movement_points()?
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

fn step(
    state: &mut Session,
    content: &Content,
    frame: &MoveFrame,
    events: &mut Vec<Event>,
) -> Result<bool, RuleError> {
    let actor = frame.actor(state)?;
    let remaining = state.warriors[actor]
        .record()
        .movement_points()?
        .wrapping_sub(frame.cost);
    state.warriors[actor]
        .record_mut()
        .set_movement_points(remaining);
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
    let under = state.native_work.tactical_work.under_tiles[actor];
    write(state, current, under)?;
    let arrived = tile(state, content, next)?;
    state.native_work.tactical_work.under_tiles[actor] = arrived;
    write(state, next, actor as i16)?;
    let work = &mut state.native_work.tactical_work;
    work.positions[actor] = destination;
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
    let collision =
        crate::combat::runtime::retained_turn::collide_fields(state, content, actor as i16)?;
    events.extend(collision.events);
    if collision.flow == crate::combat::runtime::TurnFlow::Terminated {
        return Ok(true);
    }
    let actor = state.selected_warrior;
    if state.warriors[actor].stamina()? < 1 {
        state.warriors[actor].record_mut().set_action_points(0);
        state.warriors[actor].record_mut().set_movement_points(0);
        return Ok(true);
    }
    Ok(false)
}

fn finish(
    state: &mut Session,
    content: &Content,
    cost: i16,
    events: &mut Vec<Event>,
) -> Result<bool, RuleError> {
    let actor = state.selected_warrior;
    if state.warriors[actor]
        .record()
        .movement_points()?
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

fn react_or_end(
    state: &mut Session,
    content: &Content,
    mode: usize,
    events: &mut Vec<Event>,
) -> Result<bool, RuleError> {
    match reaction(state, content, mode, events)? {
        ReactionReturn::Terminated => Ok(true),
        ReactionReturn::Returned(true) => {
            events.extend(crate::combat::runtime::retained_turn::finish(
                state, content,
            )?);
            Ok(true)
        }
        ReactionReturn::Returned(false) => Ok(false),
    }
}

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
            state.retained_walk_choice = Some(RetainedWalkChoice {
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

pub(super) fn answer(
    state: &mut Session,
    content: &Content,
    exchange: Option<bool>,
) -> Result<Vec<Event>, RuleError> {
    let pending = state
        .retained_walk_choice
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

fn reaction(
    state: &mut Session,
    content: &Content,
    mode: usize,
    events: &mut Vec<Event>,
) -> Result<ReactionReturn, RuleError> {
    let body = crate::combat::runtime::reaction_calls::retained_actor(state)?;
    let mut frame = ReactionFrame::new(body, mode, direction(state)?)?;
    let outcome = frame.finish_after_teardown(state, content)?;
    events.extend(outcome.events);
    Ok(if outcome.terminated {
        ReactionReturn::Terminated
    } else {
        ReactionReturn::Returned(outcome.interrupted)
    })
}

fn direction(state: &Session) -> Result<(i8, i8), RuleError> {
    let value = state
        .exploration
        .as_ref()
        .ok_or_else(|| RuleError::from("walk outside a mission"))?
        .last_move;
    Ok((value[0], value[1]))
}

fn set_direction(state: &mut Session, value: (i8, i8)) -> Result<(), RuleError> {
    state
        .exploration
        .as_mut()
        .ok_or_else(|| RuleError::from("walk outside a mission"))?
        .last_move = [value.0, value.1];
    Ok(())
}

fn position(state: &Session, actor: usize) -> Result<(Point, Point), RuleError> {
    let work = &state.native_work.tactical_work;
    let map = &work.map;
    let point = work.positions[actor];
    Ok((point, map.shift))
}

/// The walker's linear map index, column × 90 + row, without clipping.
fn index(x: i32, y: i32, shift: (i16, i16)) -> i32 {
    let x = x.wrapping_add(i32::from(shift.0)).wrapping_add(5);
    let y = y.wrapping_add(i32::from(shift.1)).wrapping_add(5);
    x.wrapping_mul(90).wrapping_add(y)
}

fn tile(state: &Session, content: &Content, index: i32) -> Result<i16, RuleError> {
    let aliases = crate::combat::map_alias::session_map_aliases!(state, content content);
    aliases
        .read(&state.native_work.tactical_work.map, index)?
        .ok_or_else(|| {
            unsupported(
                "retained walking read outside the map and the represented memory beside it",
            )
        })
}

fn write(state: &mut Session, index: i32, value: i16) -> Result<(), RuleError> {
    let land = state
        .exploration
        .as_mut()
        .map(|position| (&mut position.objects[..], &mut position.events[..]));
    let map = &mut state.native_work.tactical_work.map;
    if crate::combat::map_alias::write(map, land, index, value) {
        Ok(())
    } else {
        Err(unsupported(
            "retained walking writes beside the tactical map outside its edge WORD and land arrays",
        ))
    }
}

fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.retained_walk",
        dependency,
    }
}
