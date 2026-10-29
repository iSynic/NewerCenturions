//! Movement for maintained exploration.
use super::{Regional, active_map, apply_region, regional_selection, run_script};
use crate::{Event, MovementContinuation, Phase, RuleError, ScriptState, Session, travel};
use centurions_content::Content;

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
        item_return: true,
        ..ScriptState::scheduled(map.script(id)?.to_vec(), 0)
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
    // buttonchoice 0x109526 refuses any move above fatigue 134 while camping is
    // allowed, before it breaks camp.
    if state.exploration.as_ref().unwrap().fatigue > 134 && !state.native_work.camp_disabled {
        return Err(RuleError::Refused {
            feedback: vec![
                crate::feedback::NativeFeedback::Warning { index: 54 },
                crate::feedback::NativeFeedback::Tutorial {
                    first: 80,
                    last: 80,
                },
            ],
        });
    }
    if let Some(events) = crate::field_controls::leave_camp(state, content, dx, dy)? {
        return Ok(events);
    }
    if let Some(control) = state.field_controls.as_mut() {
        control.camping = false;
        control.clinic = None;
    }
    state.shop_access = None;
    let position = state.exploration.as_mut().unwrap();
    if dx != 0 {
        position.facing = u8::from(dx > 0);
        state.native_work.boat_facing = if dx < 0 { -1 } else { 0 };
    }
    position.last_move = [dx, dy];
    super::boat::start(state, content)
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
        if state.time_continuation.is_some() || state.zero_time.is_some() {
            effects.extend(crate::time::resume(state, content)?);
            if state.time_continuation.is_some()
                || state.zero_time.is_some()
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
                if check {
                    let battles = super::regions::battles_allowed(state)?;
                    let region = regional_selection(
                        &mut state.exploration.as_mut().unwrap().region_state,
                        (x, y),
                        battles,
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
                effects.extend(walking_header(state, content)?);
                if effects
                    .iter()
                    .any(|event| matches!(event, Event::Message { .. }))
                {
                    break;
                }
            }
            Some(MovementContinuation::Boating(pending)) => {
                effects.extend(super::boat::resume(state, content, pending)?);
                if effects
                    .iter()
                    .any(|event| matches!(event, Event::Message { .. }))
                {
                    break;
                }
            }
            Some(MovementContinuation::Secrets) => {
                // buttonchoice109afe clears WORD443316 before checkforsecret.
                state.bandage_ready = 0;
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

pub(super) fn walking_header(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    let position = state.exploration.as_mut().unwrap();
    let found = crate::walking_header::lookup(
        &position.event_headers,
        position.map,
        position.x,
        position.y,
        &mut state.random,
    )?;
    if let Some(row) = found.matched {
        state.native_work.header_row = row as i16;
    }
    if let Some(header) = found.header {
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
            caller_action: None,
            chained: false,
            registers: (position.x as i16, position.y as i16),
            persist_cell: (0, 0),
        });
        return run_script(state, content);
    }
    Ok(vec![])
}
