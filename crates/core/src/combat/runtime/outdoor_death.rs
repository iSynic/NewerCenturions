//! Deaths outside combat, and the party loss that ends the game.
use crate::{Content, Event, RuleError, Session, combat::outdoor_health, feedback::GameFeedback};

/// Returns true when the party is lost and play returns to the main loop.
///
/// Original: killbody, partyloss
pub(super) fn defeat(
    state: &mut Session,
    content: &Content,
    body: i16,
    events: &mut Vec<Event>,
) -> Result<bool, RuleError> {
    let health = &mut state.globals.health_context;
    health.last_body = body;
    health.kill_party = health.kill_party.wrapping_add(1);
    events.push(Event::Sound {
        resource_id: state.random.bound(2).wrapping_add(506),
    });
    // Only warrior bodies update the character panel.
    let stamina = outdoor_health::health(state, content, body)?;
    events.push(Event::Feedback(GameFeedback::OutdoorBodyStatus {
        permanent: stamina < -9,
        wait: state.command_preferences.animation_delay,
    }));
    if state.globals.health_context.kill_party > state.warriors.len() as i8 - 1 {
        events.extend(crate::mission::defeat(state, content)?);
        return Ok(true);
    }
    Ok(false)
}
