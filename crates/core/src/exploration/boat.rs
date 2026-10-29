//! buttonchoice1098de..109aee: boarding, shore attempts and movement returns.
use super::{Regional, active_map, apply_region, regional_selection};
use crate::{
    BoatMovement, BoatPhase, Event, MovementContinuation, Phase, Presentation, RuleError, Session,
};
use centurions_content::{Content, mission::OpeningMission};

pub(crate) fn aboard(state: &Session) -> bool {
    state.native_work.boat != 0
}

pub(super) fn start(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let map = active_map(state, content)?;
    let position = state.exploration.as_ref().unwrap();
    let [dx, dy] = position.last_move;
    let index = OpeningMission::index(position.x + i32::from(dx), position.y + i32::from(dy))?;
    let tile = position.floor[index];
    let event = position.events[index] > 0;
    let property = if !event && (tile.unsigned_abs() > 200 || tile < 0) {
        map.default_tile
    } else {
        tile.wrapping_abs()
    };
    let mut effects = vec![Event::Sound {
        resource_id: if tile < 0 {
            82
        } else {
            map.movement_sound(tile)?
        },
    }];
    let mut pending = BoatMovement {
        tile,
        property,
        phase: BoatPhase::Finish,
    };
    if event {
        effects.extend(move_party(state)?);
        let ticks = if tile < 0 {
            1
        } else {
            map.movement_time(tile)?
        } as u8;
        let position = state.exploration.as_ref().unwrap();
        pending.phase = BoatPhase::Region {
            x: position.x as i16,
            y: position.y as i16,
            check: ticks != 0,
        };
        state.movement_continuation = Some(MovementContinuation::Boating(pending));
        effects.extend(crate::time::advance(
            state,
            content,
            ticks,
            map.minutes_per_tick,
        )?);
    } else if !aboard(state) && map.boarding_field(property)? == 1 {
        effects.extend(question(state, pending, true));
    } else {
        if aboard(state) {
            if map.movement_blocker(property)? != 0 && map.boarding_field(property)? != 2 {
                effects.push(Event::Sound { resource_id: -148 });
                state.exploration.as_mut().unwrap().last_move = [0; 2];
            }
            if map.tile_word(property, 6)? != 0 {
                effects.push(Event::Sound { resource_id: -148 });
                state.exploration.as_mut().unwrap().last_move = [0; 2];
                let attempts = state.native_work.shore_attempts.wrapping_add(1);
                state.native_work.shore_attempts = attempts;
                if attempts > 2 {
                    state.native_work.shore_attempts = 0;
                    effects.extend(question(state, pending, false));
                    return Ok(effects);
                }
            }
        }
        effects.extend(finish(state, content, pending)?);
    }
    Ok(effects)
}

fn question(state: &mut Session, mut pending: BoatMovement, boarding: bool) -> Vec<Event> {
    pending.phase = if boarding {
        BoatPhase::Board
    } else {
        BoatPhase::Leave
    };
    state.movement_continuation = Some(MovementContinuation::Boating(pending));
    state.exploration.as_mut().unwrap().presentation =
        Some(Presentation::BoatQuestion { boarding });
    vec![Event::Sound { resource_id: 20005 }]
}

pub(super) fn answer(state: &mut Session, answer: bool) -> Result<Vec<Event>, RuleError> {
    let Some(MovementContinuation::Boating(mut pending)) = state.movement_continuation.take()
    else {
        return Err("boat answer has no movement return".into());
    };
    let boarding = match pending.phase {
        BoatPhase::Board => true,
        BoatPhase::Leave => false,
        _ => return Err("boat movement is not awaiting an answer".into()),
    };
    if state.exploration.as_ref().unwrap().presentation
        != Some(Presentation::BoatQuestion { boarding })
    {
        return Err("boat question differs from its movement return".into());
    }
    state.exploration.as_mut().unwrap().presentation = None;
    let mut effects = vec![];
    if answer {
        if !boarding {
            // 109978 clears the same WORD used by the player Psi movement gate.
            state.bandage_ready = 0;
        }
        state.native_work.boat = u8::from(boarding);
        if boarding {
            effects.extend(move_party(state)?);
        }
        let position = state.exploration.as_mut().unwrap();
        let index = OpeningMission::index(position.x, position.y)?;
        position.floor[index] = if boarding { 60 } else { 147 };
    }
    pending.phase = BoatPhase::Finish;
    state.movement_continuation = Some(MovementContinuation::Boating(pending));
    Ok(effects)
}

fn move_party(state: &mut Session) -> Result<Vec<Event>, RuleError> {
    let position = state.exploration.as_mut().unwrap();
    let [dx, dy] = position.last_move;
    let x = position.x.wrapping_add(i32::from(dx));
    let y = position.y.wrapping_add(i32::from(dy));
    OpeningMission::index(x, y)?;
    position.x = x;
    position.y = y;
    if let Some(control) = state.field_controls.as_mut() {
        control.camping = false;
        control.clinic = None;
    }
    state.shop_access = None;
    Ok(vec![Event::Moved { x, y }])
}

fn finish(
    state: &mut Session,
    content: &Content,
    pending: BoatMovement,
) -> Result<Vec<Event>, RuleError> {
    let map = active_map(state, content)?;
    if state.exploration.as_ref().unwrap().last_move != [0; 2] {
        state.native_work.shore_attempts = 0;
    }
    let boat = aboard(state);
    let boarding = map.boarding_field(pending.property)?;
    let mut effects = vec![];
    if (!boat && map.movement_blocker(pending.property)? == 0 && boarding != 2)
        || (boat && boarding == 2)
    {
        effects.extend(move_party(state)?);
    }
    let (sound, ticks) = if pending.tile < 0 {
        (82, 3)
    } else if pending.property > 0 {
        (
            map.movement_sound(pending.property)?,
            map.movement_time(pending.property)?,
        )
    } else {
        state.movement_continuation = Some(MovementContinuation::Secrets);
        return Ok(effects);
    };
    effects.push(Event::Sound { resource_id: sound });
    let position = state.exploration.as_ref().unwrap();
    state.movement_continuation = Some(MovementContinuation::Region {
        x: position.x as i16,
        y: position.y as i16,
        check: ticks as u8 != 0,
        header: false,
    });
    effects.extend(crate::time::advance(
        state,
        content,
        ticks as u8,
        map.minutes_per_tick,
    )?);
    Ok(effects)
}

pub(super) fn resume(
    state: &mut Session,
    content: &Content,
    mut pending: BoatMovement,
) -> Result<Vec<Event>, RuleError> {
    match pending.phase {
        BoatPhase::Region { x, y, check } => {
            pending.phase = BoatPhase::Header;
            state.movement_continuation = Some(MovementContinuation::Boating(pending.clone()));
            if check {
                let battles = super::regions::battles_allowed(state)?;
                let region = regional_selection(
                    &mut state.exploration.as_mut().unwrap().region_state,
                    (x, y),
                    battles,
                    &mut state.random,
                )?;
                if matches!(region, Regional::Script(_)) {
                    pending.phase = BoatPhase::RegionalReturn;
                    state.movement_continuation = Some(MovementContinuation::Boating(pending));
                }
                return apply_region(state, content, region);
            }
        }
        BoatPhase::RegionalReturn => {
            state.exploration.as_mut().unwrap().last_move = [0; 2];
            pending.phase = BoatPhase::Header;
            state.movement_continuation = Some(MovementContinuation::Boating(pending));
        }
        BoatPhase::Header => {
            pending.phase = BoatPhase::AfterHeader;
            state.movement_continuation = Some(MovementContinuation::Boating(pending));
            return super::movement::walking_header(state, content);
        }
        BoatPhase::AfterHeader => {
            let map = active_map(state, content)?;
            if pending.tile.wrapping_abs() < 600 && map.boarding_field(pending.property)? == 1 {
                return Ok(question(state, pending, true));
            }
            state.movement_continuation = Some(MovementContinuation::Secrets);
        }
        BoatPhase::Finish => return finish(state, content, pending),
        BoatPhase::Board | BoatPhase::Leave => {
            return Err("boat question lost its presentation".into());
        }
    }
    Ok(vec![])
}

pub(crate) fn validate(state: &Session) -> Result<(), RuleError> {
    let mut page = state
        .exploration
        .as_ref()
        .and_then(|p| p.presentation.as_ref());
    while let Some(Presentation::Messages { next, .. }) = page {
        page = next.as_deref();
    }
    let question = match page {
        Some(Presentation::BoatQuestion { boarding }) => Some(*boarding),
        _ => None,
    };
    let pending = match state.movement_continuation.as_ref() {
        Some(MovementContinuation::Boating(pending)) => Some(pending),
        _ => None,
    };
    let expected = pending.and_then(|p| match p.phase {
        BoatPhase::Board => Some(true),
        BoatPhase::Leave => Some(false),
        _ => None,
    });
    if question != expected
        || (pending.is_some() && state.exploration.is_none())
        || (expected.is_some()
            && (state.phase != Phase::Exploration
                || state.time_continuation.is_some()
                || state.zero_time.is_some()
                || state
                    .exploration
                    .as_ref()
                    .is_none_or(|p| p.script.is_some())))
    {
        return Err("invalid boat movement return".into());
    }
    if let Some(pending) = pending {
        if !(0..300).contains(&pending.property) && pending.tile.wrapping_abs() < 600 {
            return Err("boat movement tile properties unavailable".into());
        }
        if let BoatPhase::Region { x, y, .. } = pending.phase {
            OpeningMission::index(i32::from(x), i32::from(y))?;
        }
    }
    Ok(())
}
