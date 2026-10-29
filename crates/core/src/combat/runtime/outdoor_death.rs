//! killbody's combat-zero suffix and nonreturning party-loss boundary.
use crate::{Content, Event, RuleError, Session, combat::outdoor_health, feedback::NativeFeedback};

/// Returns true when partyloss enters MainLoop instead of returning to its caller.
pub(super) fn defeat(
    state: &mut Session,
    content: &Content,
    body: i16,
    events: &mut Vec<Event>,
) -> Result<bool, RuleError> {
    let health = &mut state.native_work.health_context;
    health.last_body = body;
    health.unavailable = health.unavailable.wrapping_add(1);
    events.push(Event::Sound {
        resource_id: state.random.bound(2).wrapping_add(506),
    });
    // updatecharshort rejects NPC indices before touching the warrior/profile record.
    let stamina = outdoor_health::health(state, content, body)?;
    events.push(Event::Feedback(NativeFeedback::OutdoorBodyStatus {
        permanent: stamina < -9,
        wait: state.command_preferences.animation_delay,
    }));
    if state.native_work.health_context.unavailable > state.warriors.len() as i8 - 1 {
        events.extend(crate::mission::defeat(state, content)?);
        return Ok(true);
    }
    Ok(false)
}
