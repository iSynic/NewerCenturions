//! Paused getup callers, stacked apart from any nested battle.
use super::{MonsterActionFrame, Phase};
use crate::combat::round_stories::RoundDispatch;
use crate::{Content, Event, RuleError, Session};
use serde::{Deserialize, Serialize};

const MAX_DEPTH: usize = 32;

/// How deeply death scripts, round scripts and turn scripts are nested when a monster's turn begins
/// to wait for the next turn to be set up.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct GetupScope {
    death_depth: usize,
    round_depth: usize,
    turn_depth: usize,
}

impl GetupScope {
    /// Notes the nesting of the death, round and turn scripts now running.
    pub(super) fn capture(state: &Session) -> Self {
        Self {
            death_depth: RoundDispatch::death_depth(state),
            round_depth: std::iter::successors(state.round_dispatch.as_ref(), |frame| {
                frame.parent.as_deref()
            })
            .take(MAX_DEPTH + 1)
            .count(),
            turn_depth: std::iter::successors(state.turn_dispatch.as_ref(), |frame| {
                frame.parent.as_deref()
            })
            .take(MAX_DEPTH + 1)
            .count(),
        }
    }

    /// Whether the scripts that were nested at the wait have all returned.
    fn reached(self, current: Self) -> bool {
        current.death_depth <= self.death_depth
            && current.round_depth <= self.round_depth
            && current.turn_depth <= self.turn_depth
    }

    /// Refuses a saved nesting that is deeper than any real one.
    fn validate(self) -> Result<(), RuleError> {
        if self.death_depth > MAX_DEPTH
            || self.round_depth > MAX_DEPTH
            || self.turn_depth > MAX_DEPTH
        {
            return Err("invalid saved NPC Getup owner depth".into());
        }
        Ok(())
    }
}

/// The nesting a waiting monster turn expects to see end, if it waits.
fn scope(frame: &MonsterActionFrame) -> Option<GetupScope> {
    match frame.phase {
        Phase::WaitingGetup(scope) | Phase::WaitingFinalGetup(scope) => Some(scope),
        _ => None,
    }
}

/// Refuses a saved chain of waiting monster turns that is too long or whose links do not fit
/// together.
pub(super) fn validate(frame: &MonsterActionFrame) -> Result<(), RuleError> {
    let mut current = Some(frame);
    let mut inner_scope = None;
    let mut depth = 0;
    while let Some(frame) = current {
        depth += 1;
        if depth > MAX_DEPTH {
            return Err("saved NPC Getup return chain exceeds 32 frames".into());
        }
        frame.validate_node()?;
        let waiting = scope(frame);
        if waiting.is_none() && (depth > 1 || frame.parent.is_some()) {
            return Err("NPC reaction return cannot own pending Getup parents".into());
        }
        if let Some(waiting) = waiting {
            waiting.validate()?;
            // An inner getup starts behind a deeper owner of the outer one; a single
            // schema 47 frame needs no depth.
            if let Some(inner) = inner_scope
                && waiting.reached(inner)
            {
                return Err("saved NPC Getup parent has no deeper child owner".into());
            }
        }
        inner_scope = waiting;
        current = frame.parent.as_deref();
    }
    Ok(())
}

/// Whether the monster turn on top of the pending stack is waiting for the next turn to be set up
/// and the scripts it waited on have all returned.
pub(in crate::combat::runtime) fn owns_return(state: &Session) -> bool {
    state
        .monster_action
        .as_deref()
        .and_then(scope)
        .is_some_and(|waiting| waiting.reached(GetupScope::capture(state)))
}

/// Whether something still holds up a waiting turn: scripts not yet returned, a message on screen,
/// a cleanup that pauses, the survivors screen or a replaced actor.
pub(super) fn obstructed(state: &Session, waiting: GetupScope) -> bool {
    !waiting.reached(GetupScope::capture(state))
        || state
            .exploration
            .as_ref()
            .is_some_and(|position| position.presentation.is_some())
        || state
            .scripted_cleanup
            .as_ref()
            .is_some_and(crate::story::cleanup::Stage::pauses)
        || state.survivors.is_some()
        || state.actor_replacement.is_some()
}

/// Pushes a waiting monster turn onto the pending stack, above any turn already waiting. Refuses a
/// frame that is not a fresh waiting one.
pub(super) fn suspend(state: &mut Session, mut frame: MonsterActionFrame) -> Result<(), RuleError> {
    if !frame.waiting_getup() || frame.parent.is_some() {
        return Err("NPC pending Getup installation has no fresh caller".into());
    }
    frame.parent = state.monster_action.take();
    if let Err(error) = frame.validate() {
        state.monster_action = frame.parent.take();
        return Err(error);
    }
    state.monster_action = Some(Box::new(frame));
    Ok(())
}

/// Resumes the waiting monster turns, innermost first, once nothing holds them up any longer.
pub(crate) fn resume(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    loop {
        if state.mission.is_none() {
            state.monster_action = None;
            return Ok(events);
        }
        let Some(frame) = state.monster_action.as_deref() else {
            return Ok(events);
        };
        frame.validate()?;
        let waiting = scope(frame).ok_or("pending NPC caller is not waiting for Getup")?;
        if obstructed(state, waiting) {
            return Ok(events);
        }
        let mut frame = *state.monster_action.take().unwrap();
        // The parent returns before the rest can pause again. Only the top caller
        // runs; a blocked one is never skipped.
        state.monster_action = frame.parent.take();
        match frame.phase {
            Phase::WaitingFinalGetup(_) => {
                super::super::refresh_active_item_context(state, content)?;
                events.extend(super::super::movement_return::finish_monster_wrapper(
                    state, content,
                )?);
            }
            Phase::WaitingGetup(_) => {
                super::super::death_return::refresh_actor(state, content)?;
                frame.phase = Phase::Movement;
                events.extend(super::run(state, content, frame)?);
            }
            _ => return Err("pending NPC caller is not waiting for Getup".into()),
        }
        // Each finished caller removes one frame; a newly paused getup stops the
        // next pass.
    }
}
