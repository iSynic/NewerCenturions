use centurions_content::Content;
use centurions_core::{Command, Event, Session, combat::Combat};

/// Compare a command against the same command resumed from a strict save checkpoint.
pub fn saved_command(command: Command, state: &mut Session, content: &Content) -> Vec<Event> {
    let mut resumed = super::checkpoint(state, content, "command");
    super::replay_command(command, state, &mut resumed, content)
}

/// The active combat state shared by native command comparisons.
pub fn current_combat(state: &Session) -> &Combat {
    state.combat.as_ref().unwrap()
}

/// Turn-story continuations must not announce an advance during initial entry.
pub fn assert_no_turn_started(events: &[Event]) {
    let started = |event: &Event| matches!(event, Event::TurnStarted { .. });
    assert!(!events.iter().any(started));
}
