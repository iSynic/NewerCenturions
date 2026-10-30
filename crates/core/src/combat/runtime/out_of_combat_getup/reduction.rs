//! getup's round reduction after the battle has ended, by the party's outdoor rules.
use super::super::{TurnFlow, TurnOutcome};
use crate::combat::runtime::enemy_round::EnemyRoundStage;
use crate::story::allies::ALLY_BYTES;
use crate::{Content, Event, RuleError, Session, feedback::GameFeedback};
use centurions_content::records::EnemyRecordMut;
mod fields;
mod monsters;
mod party;
use self::fields::clear_usage;
pub(in crate::combat::runtime) use self::fields::collide;

/// Applies a round to the party after the battle has ended: buffs run down, each warrior's
/// conditions act, fields act on the party, bleeding wears the warriors down, and the monsters
/// follow.
///
/// Original: getup
pub(in crate::combat::runtime) fn apply(
    state: &mut Session,
    content: &Content,
) -> Result<TurnOutcome, RuleError> {
    crate::combat::party_buffs::reduce(&mut state.party_condition)?;
    let mut events = vec![];
    for member in 0..state.warriors.len() {
        if party::apply(state, content, member, &mut events)? == TurnFlow::Terminated {
            return Ok(TurnOutcome {
                events,
                flow: TurnFlow::Terminated,
            });
        }
        EnemyRecordMut::from_bytes(
            &mut state.holdover[member * ALLY_BYTES..(member + 1) * ALLY_BYTES],
        )
        .decrement_party_conditions();
    }
    for member in 0..state.warriors.len() {
        clear_usage(state)?;
        if state.warriors[member].stamina()? > 0 {
            let outcome = collide(state, content, member as i16)?;
            events.extend(outcome.events);
            if outcome.flow == TurnFlow::Terminated {
                return Ok(TurnOutcome {
                    events,
                    flow: TurnFlow::Terminated,
                });
            }
        }
        let warrior = &mut state.warriors[member];
        warrior.record_mut().clear_been_attacked();
        if warrior.record().bleeding() != 0 && warrior.stamina()? > -10 {
            let health = warrior.stamina()?.wrapping_sub(1);
            warrior.record_mut().set_stamina(health);
            if health < -9 {
                warrior.record_mut().set_bleeding(0);
                warrior.record_mut().record_bleeding_death();
                events.push(Event::Sound {
                    resource_id: state.random.bound(2).wrapping_add(506),
                });
                events.push(Event::Feedback(GameFeedback::Flash {
                    address: 0x403750,
                    x: 30,
                    y: 100,
                    wait: 180,
                    sound: 10122,
                }));
            }
        }
    }
    resume_monsters(state, content, (0, EnemyRoundStage::Start), events)
}

/// getup's monster health loop from the given monster record and stage, then the round's closing
/// flashes and sounds. A death story that interrupted the loop resumes it here.
///
/// Original: getup
pub(in crate::combat::runtime) fn resume_monsters(
    state: &mut Session,
    content: &Content,
    (from, stage): (usize, EnemyRoundStage),
    mut events: Vec<Event>,
) -> Result<TurnOutcome, RuleError> {
    for index in from..100 {
        let stage = if index == from {
            stage
        } else {
            EnemyRoundStage::Start
        };
        if monsters::conditions(state, content, (index, stage), &mut events)?
            == TurnFlow::Terminated
        {
            return Ok(TurnOutcome {
                events,
                flow: TurnFlow::Terminated,
            });
        }
        let living = state.globals.monsters.slot(index)?.record.stamina()? > 0;
        clear_usage(state)?;
        if living {
            let outcome = collide(state, content, index as i16 + 10)?;
            events.extend(outcome.events);
            if outcome.flow == TurnFlow::Terminated {
                return Ok(TurnOutcome {
                    events,
                    flow: TurnFlow::Terminated,
                });
            }
        }
        state
            .globals
            .monsters
            .slot_mut(index)?
            .record
            .record_mut()
            .clear_been_attacked();
    }
    if state
        .warriors
        .iter()
        .any(|warrior| warrior.record().bleeding() != 0)
    {
        events.push(Event::Feedback(GameFeedback::Flash {
            address: 0x403708,
            x: 30,
            y: 100,
            wait: 120,
            sound: 10122,
        }));
    }
    if !state.command_preferences.suppress_round_message_sound {
        events.push(Event::Feedback(GameFeedback::Flash {
            address: 0x4036f2,
            x: 30,
            y: 100,
            wait: -1,
            sound: 0,
        }));
    }
    events.extend([
        Event::Sound { resource_id: -139 },
        Event::Sound { resource_id: -139 },
    ]);
    if !state.command_preferences.suppress_round_message_sound {
        events.push(Event::Feedback(GameFeedback::Flash {
            address: 0x4036f1,
            x: 30,
            y: 100,
            wait: -1,
            sound: -139,
        }));
    }
    Ok(TurnOutcome {
        events,
        flow: TurnFlow::Returned,
    })
}
