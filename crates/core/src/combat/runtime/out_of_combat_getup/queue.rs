//! Emptying the death queue after the battle has ended.
use crate::{
    Content, Event, RuleError, Session,
    combat::runtime::{TurnFlow, TurnOutcome},
    feedback::GameFeedback,
};

/// Empties the death queue after the battle has ended. Each death costs the party one more fallen
/// warrior, plays its sound and shows the status of the first warrior; the party is defeated when
/// too many have fallen.
///
/// Original: emptyque
pub(in crate::combat::runtime) fn drain(
    state: &mut Session,
    content: &Content,
) -> Result<TurnOutcome, RuleError> {
    let queue = state
        .globals
        .outer_todo_queue
        .as_mut()
        .ok_or_else(|| RuleError::from("death queue missing after combat teardown"))?;
    queue.begin();
    state.globals.death_processing.doing_queue = 1;
    let mut events = vec![];
    while state
        .globals
        .outer_todo_queue
        .as_mut()
        .unwrap()
        .current()
        .is_some()
    {
        let health = &mut state.globals.health_context;
        health.kill_party = health.kill_party.wrapping_add(1);
        events.push(Event::Sound {
            resource_id: state.random.bound(2).wrapping_add(506),
        });
        // Mission entry admits one to six warriors and no command empties the squad.
        let warrior = state
            .warriors
            .first()
            .ok_or("outdoor death queue with an empty squad")?;
        events.push(Event::Feedback(GameFeedback::OutdoorBodyStatus {
            permanent: warrior.stamina()? < -9,
            wait: state.command_preferences.animation_delay,
        }));
        if state.warriors.len() as i16 - 1 < i16::from(health.kill_party) {
            events.extend(crate::mission::defeat(state, content)?);
            return Ok(TurnOutcome {
                events,
                flow: TurnFlow::Terminated,
            });
        }
        // A queued death changes neither last_body nor the combat records.
        state
            .globals
            .outer_todo_queue
            .as_mut()
            .unwrap()
            .acknowledge()?;
    }
    state.globals.outer_todo_queue.as_mut().unwrap().finish();
    state.globals.death_processing.doing_queue = 0;
    Ok(TurnOutcome {
        events,
        flow: TurnFlow::Returned,
    })
}
