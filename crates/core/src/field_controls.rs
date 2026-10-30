//! The outdoor control buttons and the search for secrets.
use crate::{Content, Event, Phase, RuleError, Session, feedback::GameFeedback, story_allies};
use centurions_content::records::{EnemyRecord, EnemyRecordMut, WarriorRecord};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// The outdoor control buttons.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Action {
    Camp,
    SearchMode,
    Search,
    Rest,
    Heal,
    Clinic,
}

/// The outdoor controls' state: whether the party is camping, whether search mode is on, the
/// clinic's price percentage when one is available, and the steps still to run.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct State {
    pub camping: bool,
    pub searching: bool,
    pub clinic: Option<i16>,
    pending: VecDeque<Step>,
}

/// One step of a control button's action: time passing, a region check, a search for secrets,
/// healing, or a move.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
enum Step {
    Time(u8),
    Region(i16),
    Search,
    Heal,
    Move(i8, i8),
}

/// Whether the field controls can be used: the party is exploring with nothing on screen and
/// nothing waiting to resume.
pub fn free(state: &Session) -> bool {
    state.phase == Phase::Exploration
        && state
            .exploration
            .as_ref()
            .is_some_and(|p| p.presentation.is_none() && p.script.is_none())
        && state.medical_continuation.is_none()
        && state.outdoor_psi.is_none()
        && state.time_continuation.is_none()
        && state.zero_time.is_none()
        && state.movement_continuation.is_none()
}

/// Checks that the queued steps are ones the controls can queue: at most five, with time of 1 to
/// 10 ticks, a region divisor of 1 or 4, and a move of one square at most that goes somewhere.
pub(crate) fn validate(state: &Session) -> Result<(), RuleError> {
    if let Some(control) = &state.field_controls
        && (control.pending.len() > 5
            || control.pending.iter().any(|step| match step {
                Step::Time(ticks) => !(1..=10).contains(ticks),
                Step::Region(divisor) => !matches!(divisor, 1 | 4),
                Step::Move(dx, dy) => {
                    !(-1..=1).contains(dx) || !(-1..=1).contains(dy) || (*dx == 0 && *dy == 0)
                }
                _ => false,
            }))
    {
        return Err("invalid outdoor control continuation".into());
    }
    Ok(())
}

/// Starts a control button's action, refused when the controls are not free or steps are still
/// queued. The button's tutorial page is shown first, even when the action is then refused:
/// Search and Heal are refused with a warning above fatigue 134. Camp, Rest, Search and Heal queue
/// their steps and run them; Clinic enters the clinic at the recorded price percentage.
pub(crate) fn start(
    state: &mut Session,
    content: &Content,
    action: Action,
) -> Result<Vec<Event>, RuleError> {
    if !free(state)
        || state
            .field_controls
            .as_ref()
            .is_some_and(|c| !c.pending.is_empty())
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let fatigue = state.exploration.as_ref().unwrap().fatigue;
    let tutorial = match action {
        Action::Camp => Some(4),
        Action::Rest => Some(5),
        Action::Search => Some(6),
        Action::SearchMode => Some(7),
        Action::Heal => Some(8),
        Action::Clinic => None,
    };
    let feedback: Vec<_> = tutorial
        .into_iter()
        .map(|index| GameFeedback::Tutorial {
            first: index,
            last: index,
        })
        .collect();
    if matches!(action, Action::Search | Action::Heal) && fatigue > 134 {
        let mut feedback = feedback;
        feedback.push(GameFeedback::Warning { index: 54 });
        return Err(RuleError::Refused { feedback });
    }
    if action == Action::Clinic {
        let percent = state
            .field_controls
            .as_ref()
            .and_then(|c| c.clinic)
            .ok_or_else(|| RuleError::Refused { feedback: vec![] })?;
        return crate::clinic::visit(state, percent);
    }
    let control = state.field_controls.get_or_insert_default();
    let mut events: Vec<_> = feedback.into_iter().map(Event::Feedback).collect();
    match action {
        Action::SearchMode => {
            control.searching = !control.searching;
            events.push(Event::Sound {
                resource_id: if control.searching { 208 } else { 205 },
            });
        }
        // Camping can always be broken, but not started while disabled.
        Action::Camp if !control.camping && state.globals.camp_disabled => {
            events.push(Event::Feedback(crate::feedback::camp_flash(0x403f24)));
        }
        Action::Camp => {
            control.camping = !control.camping;
            if control.camping {
                control.clinic = None;
                state.shop_access = None;
                control
                    .pending
                    .extend([Step::Time(3), Step::Time(2), Step::Region(1)]);
            } else {
                control.pending.push_back(Step::Time(2));
            }
            events.push(Event::Sound {
                resource_id: if control.camping { 131 } else { 141 },
            });
        }
        Action::Search => {
            events.push(Event::Sound { resource_id: 6001 });
            control.pending.extend([
                Step::Time(4),
                Step::Region(1),
                Step::Search,
                Step::Time(1),
                Step::Region(1),
            ]);
        }
        Action::Rest => {
            events.push(Event::Sound { resource_id: 6001 });
            state.exploration.as_mut().unwrap().fatigue = (fatigue - 2).clamp(4, 135);
            control
                .pending
                .extend([Step::Time(4), Step::Time(1), Step::Region(1)]);
        }
        Action::Heal => {
            events.push(Event::Sound { resource_id: 10105 });
            control
                .pending
                .extend([Step::Time(1), Step::Region(4), Step::Heal]);
        }
        Action::Clinic => unreachable!(),
    }
    state.exploration.as_mut().unwrap().last_move = [0; 2];
    events.extend(resume(state, content)?);
    Ok(events)
}

/// Runs the queued steps in order until the controls are no longer free (a step put something on
/// screen) or none remain.
pub(crate) fn resume(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    while free(state) {
        let Some(step) = state
            .field_controls
            .as_mut()
            .and_then(|c| c.pending.pop_front())
        else {
            break;
        };
        match step {
            Step::Time(ticks) => {
                let minutes = crate::exploration::active_map(state, content)?.minutes_per_tick;
                events.extend(crate::time::advance(state, content, ticks, minutes)?);
            }
            Step::Region(divisor) => {
                events.extend(crate::exploration::check_region(state, content, divisor)?)
            }
            Step::Search => {
                let position = state.exploration.as_mut().unwrap();
                let search = crate::travel::discover_secrets(
                    &position.floor,
                    (position.x as i16, position.y as i16),
                    100,
                    &mut state.random,
                )?;
                position.floor = search.floor;
                events.extend(search.discoveries.into_iter().map(|found| {
                    Event::Feedback(GameFeedback::ResourceFlash {
                        resource_id: 138,
                        index: usize::from(found.ordinal),
                        x: 30,
                        y: 375,
                        wait: 0,
                        sound: 5000,
                    })
                }));
                if search.center_revealed {
                    state.movement_continuation = Some(crate::MovementContinuation::Header);
                    events.extend(crate::exploration::resume_movement(state, content)?);
                }
            }
            Step::Heal => heal(state, content, &mut events)?,
            // The party moves only while fatigue is below 135 or camping is
            // disabled; otherwise the move waits without a warning.
            Step::Move(dx, dy) => {
                if state.exploration.as_ref().unwrap().fatigue < 135 || state.globals.camp_disabled
                {
                    events.extend(crate::exploration::move_squad(state, content, dx, dy)?)
                }
            }
        }
    }
    Ok(events)
}

/// Breaks camp when the party moves: time passes, the region is checked, and the move follows.
/// Returns nothing when the party is not camping.
pub(crate) fn leave_camp(
    state: &mut Session,
    content: &Content,
    dx: i8,
    dy: i8,
) -> Result<Option<Vec<Event>>, RuleError> {
    if let Some(control) = state.field_controls.as_mut()
        && control.camping
    {
        control.camping = false;
        control.pending.extend([
            Step::Time(5),
            Step::Time(10),
            Step::Region(1),
            Step::Move(dx, dy),
        ]);
        Ok(Some(resume(state, content)?))
    } else {
        Ok(None)
    }
}

/// Queues a search for secrets after the party's step when search mode is on. Returns whether one
/// was queued.
pub(crate) fn automatic_search(state: &mut Session) -> bool {
    if let Some(control) = state.field_controls.as_mut()
        && control.searching
    {
        control
            .pending
            .extend([Step::Time(4), Step::Region(1), Step::Search]);
        true
    } else {
        false
    }
}

/// Each warrior who can heal and cast spends 10 psi points, while holding more than 9, on every
/// squad member or held-over ally hurt but above -10 stamina, healing a random 1 to 8. Squad
/// members of race 3 are not healed.
fn heal(state: &mut Session, content: &Content, events: &mut Vec<Event>) -> Result<(), RuleError> {
    for caster in 0..state.warriors.len() {
        if !state.warriors[caster].record().can_heal()
            || !crate::combat::player_psi::can_cast(
                &state.warriors,
                caster as i16,
                state.psi_disabled,
            )?
        {
            continue;
        }
        for target in 0..state.warriors.len() + state.held_over {
            if state.warriors[caster].record().psi_points()? <= 9 {
                break;
            }
            let needs_heal = if target < state.warriors.len() {
                let bytes = &state.warriors[target].bytes;
                WarriorRecord::from_bytes(bytes).stamina()? > -10
                    && WarriorRecord::from_bytes(bytes).stamina()?
                        < WarriorRecord::from_bytes(bytes).stamina_max()?
                    && WarriorRecord::from_bytes(bytes).race()? != 3
            } else {
                let bytes =
                    &state.holdover[(target - state.warriors.len()) * story_allies::ALLY_BYTES..];
                EnemyRecord::from_bytes(bytes).stamina()? > -10
                    && EnemyRecord::from_bytes(bytes).stamina()?
                        < EnemyRecord::from_bytes(bytes).stamina_max()?
            };
            if !needs_heal {
                continue;
            }
            let psi = state.warriors[caster].record().psi_points()? - 10;
            state.warriors[caster].record_mut().set_psi_points(psi);
            let amount = state.random.bound(8);
            if target < state.warriors.len() {
                let context = crate::story::health::reselected_context(
                    state.char_select_new,
                    &state.globals.health_context,
                );
                let healed = crate::story::health::heal_outdoors(
                    &state.warriors,
                    target,
                    amount,
                    false,
                    &context,
                    &mut state.random,
                )?;
                state.warriors = healed.warriors;
                state.globals.health_context = healed.context;
                for effect in healed.events {
                    match effect {
                        crate::story::health::HealthEvent::Sound(resource_id) => {
                            events.push(Event::Sound { resource_id })
                        }
                        crate::story::health::HealthEvent::PartyLost => {
                            events.extend(crate::mission::defeat(state, content)?);
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            } else {
                let bytes = &mut state.holdover
                    [(target - state.warriors.len()) * story_allies::ALLY_BYTES..];
                let stamina = EnemyRecord::from_bytes(bytes)
                    .stamina()?
                    .wrapping_add(amount)
                    .min(EnemyRecord::from_bytes(bytes).stamina_max()?);
                EnemyRecordMut::from_bytes(bytes).set_stamina(stamina);
            }
        }
    }
    Ok(())
}
