//! Acting-warrior and combat-body lookups shared by desktop input and presentation.
//!
//! Lookups into the six roster slots stop at six; lookups into the squad itself
//! stop at the squad's size.
use centurions_core::Session;

/// Combat body numbers below 10 are warriors; 10 and above are enemies.
pub fn is_warrior_body(body: i16) -> bool {
    body < 10
}

/// Whether `body` indexes a warrior in the current roster.
pub fn is_roster_warrior(state: &Session, body: i16) -> bool {
    (0..state.warriors.len() as i16).contains(&body)
}

/// The active combat body when it indexes a warrior in the current roster.
pub fn active_warrior(state: &Session) -> Option<i16> {
    state
        .combat
        .as_ref()
        .and_then(|combat| combat.active_body())
        .filter(|body| is_roster_warrior(state, *body))
}

/// The active combat body when it is one of the six roster slots.
pub fn active_slot(state: &Session) -> Option<i16> {
    state
        .combat
        .as_ref()
        .and_then(|combat| combat.active_body())
        .filter(|body| (0..6).contains(body))
}

/// The active roster warrior, or `selected` when none is acting.
pub fn acting_warrior(state: &Session, selected: usize) -> usize {
    active_warrior(state).map_or(selected, |body| body as usize)
}

/// The active roster slot, or `selected` when none is acting.
pub fn acting_slot(state: &Session, selected: usize) -> usize {
    active_slot(state).map_or(selected, |body| body as usize)
}
