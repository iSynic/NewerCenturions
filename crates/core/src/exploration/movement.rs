//! Movement for maintained exploration.
use super::{Regional, active_map, apply_region, regional_selection, run_script, unsupported};
use crate::{Event, MovementContinuation, Phase, RuleError, ScriptState, Session, travel};
use centurions_content::mission::OpeningMission;
use centurions_content::{Content, word};

pub fn manual_encounter(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    if state.phase != Phase::Exploration
        || state
            .exploration
            .as_ref()
            .is_none_or(|position| position.presentation.is_some() || position.script.is_some())
    {
        return Err(RuleError::Invalid(
            "encounter requires free exploration".into(),
        ));
    }
    let map = active_map(state, content)?;
    let position = state.exploration.as_mut().unwrap();
    let id = crate::story_manual_encounter::select(
        &mut position.region_state,
        (position.x, position.y),
        &mut state.random,
    )?;
    position.last_move = [0; 2];
    position.script = Some(ScriptState {
        record: map.script(id)?.to_vec(),
        cursor: 0,
        map_slot: 0,
        item_return: true,
        scheduled: true,
        menu: None,
        persist: false,
        origin: None,
    });
    run_script(state, content)
}

pub fn move_squad(
    state: &mut Session,
    content: &Content,
    dx: i8,
    dy: i8,
) -> Result<Vec<Event>, RuleError> {
    if state.phase != Phase::Exploration
        || !(-1..=1).contains(&dx)
        || !(-1..=1).contains(&dy)
        || (dx == 0 && dy == 0)
    {
        return Err(RuleError::Invalid(
            "movement requires an exploration direction".into(),
        ));
    }
    if !crate::field_controls::free(state) {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if let Some(events) = crate::field_controls::leave_camp(state, content, dx, dy)? {
        return Ok(events);
    }
    let map = active_map(state, content)?;
    let position = state.exploration.as_mut().unwrap();
    if position.presentation.is_some()
        || position.script.is_some()
        || state.movement_continuation.is_some()
    {
        return Err(RuleError::Invalid(
            "finish the current story page before moving".into(),
        ));
    }
    if let Some(control) = state.field_controls.as_mut() {
        control.camping = false;
        control.clinic = None;
    }
    state.shop_access = None;
    position.last_move = [dx, dy];
    let x = position.x + i32::from(dx);
    let y = position.y + i32::from(dy);
    let index = OpeningMission::index(x, y)?;
    let tile = position.floor[index];
    let property = if tile.unsigned_abs() > 200 || tile < 0 {
        map.default_tile
    } else {
        tile
    };
    let sound = if tile < 0 {
        82
    } else {
        map.tile_word(tile, 0)?
    };
    let mut effects = vec![Event::Sound { resource_id: sound }];
    if map.tile_word(property, 8)? == 1 {
        return Err(unsupported(
            "exploration.move",
            "boarding choice and boat movement",
        ));
    }
    let event_branch = position.events[index] > 0;
    let moved =
        event_branch || (map.tile_word(property, 4)? == 0 && map.tile_word(property, 8)? != 2);
    if moved {
        position.x = x;
        position.y = y;
        effects.push(Event::Moved { x, y });
    }
    if dx != 0 {
        position.facing = u8::from(dx > 0);
    }
    let ticks = if position.events[index] > 0 {
        if tile < 0 { 1 } else { map.tile_word(tile, 2)? }
    } else {
        effects.push(Event::Sound { resource_id: sound });
        if tile < 0 {
            3
        } else {
            map.tile_word(property, 2)?
        }
    };
    if ticks < 0 {
        return Err(unsupported(
            "exploration.move",
            "negative-time tile semantics",
        ));
    }
    let px = position.x as i16;
    let py = position.y as i16;
    state.movement_continuation = Some(MovementContinuation::Region {
        x: px,
        y: py,
        check: ticks != 0,
        header: event_branch,
    });
    effects.extend(crate::time::advance(
        state,
        content,
        ticks as u8,
        map.minutes_per_tick,
    )?);
    Ok(effects)
}

pub(crate) fn resume_movement(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    let mut effects = vec![];
    while state.phase == Phase::Exploration
        && state.combat_story.is_none()
        && state
            .exploration
            .as_ref()
            .is_some_and(|position| position.presentation.is_none() && position.script.is_none())
    {
        if state.time_continuation.is_some() {
            effects.extend(crate::time::resume(state, content)?);
            if state.time_continuation.is_some()
                || state.phase != Phase::Exploration
                || state
                    .exploration
                    .as_ref()
                    .is_none_or(|p| p.presentation.is_some() || p.script.is_some())
            {
                break;
            }
        }
        match state.movement_continuation.take() {
            Some(MovementContinuation::Region {
                x,
                y,
                check,
                header,
            }) => {
                state.movement_continuation = Some(if header {
                    MovementContinuation::Header
                } else {
                    MovementContinuation::Secrets
                });
                if check && word(&state.party_buffs, 14)? == 0 {
                    let region = regional_selection(
                        &mut state.exploration.as_mut().unwrap().region_state,
                        (x, y),
                        &mut state.random,
                    )?;
                    if matches!(region, Regional::Script(_)) {
                        state.movement_continuation =
                            Some(MovementContinuation::RegionalReturn { header });
                    }
                    effects.extend(apply_region(state, content, region)?);
                }
            }
            Some(MovementContinuation::RegionalReturn { header }) => {
                // timeclick15ea8c resumes buttonchoice after clearing the movement deltas.
                state.exploration.as_mut().unwrap().last_move = [0; 2];
                state.movement_continuation = Some(if header {
                    MovementContinuation::Header
                } else {
                    MovementContinuation::Secrets
                });
            }
            Some(MovementContinuation::Header) => {
                state.movement_continuation = Some(MovementContinuation::Secrets);
                let position = state.exploration.as_mut().unwrap();
                if let Some(header) = crate::walking_header::lookup(
                    &position.event_headers,
                    position.map,
                    position.x,
                    position.y,
                    &mut state.random,
                )? {
                    position.script = Some(ScriptState {
                        record: header.record,
                        cursor: 0,
                        map_slot: header.slot,
                        item_return: false,
                        scheduled: false,
                        menu: None,
                        persist: false,
                        origin: Some(crate::ScriptOrigin {
                            map: position.map,
                            x: position.x,
                            y: position.y,
                            level: state.scenario_level,
                        }),
                    });
                    effects.extend(run_script(state, content)?);
                    if effects
                        .iter()
                        .any(|event| matches!(event, Event::Message { .. }))
                    {
                        break;
                    }
                }
            }
            Some(MovementContinuation::Secrets) => {
                if crate::field_controls::automatic_search(state) {
                    continue;
                }
                let skill = travel::secret_search_skill(&state.warriors)?;
                let position = state.exploration.as_mut().unwrap();
                let search = travel::discover_secrets(
                    &position.floor,
                    (position.x as i16, position.y as i16),
                    skill,
                    &mut state.random,
                )?;
                for discovery in &search.discoveries {
                    effects.push(Event::Feedback(
                        crate::feedback::NativeFeedback::ResourceFlash {
                            resource_id: 138,
                            index: usize::from(discovery.ordinal),
                            x: 30,
                            y: 375,
                            wait: 0,
                            sound: 5000,
                        },
                    ));
                }
                position.floor = search.floor;
                if search.center_revealed {
                    // Packed (mode0, argument1) returns to the walking-header branch.
                    state.movement_continuation = Some(MovementContinuation::Header);
                }
            }
            None => break,
        }
    }
    Ok(effects)
}
