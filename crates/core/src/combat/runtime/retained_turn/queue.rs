//! emptyque's killbody(0, index+1) reads combat mode before deferred dispatch.
use crate::{
    Content, Event, RuleError, Session,
    combat::runtime::{TurnFlow, TurnOutcome},
    feedback::NativeFeedback,
};

pub(in crate::combat::runtime) fn drain(
    state: &mut Session,
    content: &Content,
) -> Result<TurnOutcome, RuleError> {
    let queue = state
        .native_work
        .retained_death_queue
        .as_mut()
        .ok_or_else(|| RuleError::from("death queue missing after combat teardown"))?;
    queue.begin();
    state.native_work.death_processing.queue = 1;
    let mut events = vec![];
    while state
        .native_work
        .retained_death_queue
        .as_mut()
        .unwrap()
        .current()
        .is_some()
    {
        let health = &mut state.native_work.health_context;
        health.unavailable = health.unavailable.wrapping_add(1);
        events.push(Event::Sound {
            resource_id: state.random.bound(2).wrapping_add(506),
        });
        // Mission entry admits one to six warriors and no command empties the squad.
        let warrior = state
            .warriors
            .first()
            .ok_or("outdoor death queue with an empty squad")?;
        events.push(Event::Feedback(NativeFeedback::OutdoorBodyStatus {
            permanent: warrior.stamina()? < -9,
            wait: state.command_preferences.animation_delay,
        }));
        if state.warriors.len() as i16 - 1 < i16::from(health.unavailable) {
            events.extend(crate::mission::defeat(state, content)?);
            return Ok(TurnOutcome {
                events,
                flow: TurnFlow::Terminated,
            });
        }
        // param2 is nonzero: neither last_body nor combat-only records change.
        state
            .native_work
            .retained_death_queue
            .as_mut()
            .unwrap()
            .acknowledge()?;
    }
    state
        .native_work
        .retained_death_queue
        .as_mut()
        .unwrap()
        .finish();
    state.native_work.death_processing.queue = 0;
    Ok(TurnOutcome {
        events,
        flow: TurnFlow::Returned,
    })
}
