//! Pending Getup callers form a stack independent of the current child arena.
use super::{NpcActionFrame, Phase};
use crate::{Content, Event, RuleError, Session};
use serde::{Deserialize, Serialize};

const MAX_DEPTH: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct GetupScope {
    death_depth: usize,
    round_depth: usize,
    turn_depth: usize,
}

impl GetupScope {
    pub(super) fn capture(state: &Session) -> Self {
        Self {
            death_depth: crate::combat::round_stories::RoundDispatch::death_depth(state),
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

    fn reached(self, current: Self) -> bool {
        current.death_depth <= self.death_depth
            && current.round_depth <= self.round_depth
            && current.turn_depth <= self.turn_depth
    }

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

fn scope(frame: &NpcActionFrame) -> Option<GetupScope> {
    match frame.phase {
        Phase::WaitingGetup(scope) | Phase::WaitingFinalGetup(scope) => Some(scope),
        _ => None,
    }
}

pub(super) fn validate(frame: &NpcActionFrame) -> Result<(), RuleError> {
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
            // An inner call is entered behind a deeper owner of the outer Getup.
            // A single schema 47 frame needs no relation to current Session depths.
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

/// Execution deferral only. Validation can separately admit suspended history.
pub(in crate::combat::runtime) fn owns_return(state: &Session) -> bool {
    state
        .npc_action
        .as_deref()
        .and_then(scope)
        .is_some_and(|waiting| waiting.reached(GetupScope::capture(state)))
}

pub(super) fn obstructed(state: &Session, waiting: GetupScope) -> bool {
    !waiting.reached(GetupScope::capture(state))
        || state
            .exploration
            .as_ref()
            .is_some_and(|position| position.presentation.is_some())
        || state
            .scripted_cleanup
            .as_ref()
            .is_some_and(crate::story_cleanup::Stage::pauses)
        || state.survivors.is_some()
        || state.actor_replacement.is_some()
}

/// Called only after the actual Getup returned with an unfinished subowner.
pub(super) fn suspend(state: &mut Session, mut frame: NpcActionFrame) -> Result<(), RuleError> {
    if !frame.waiting_getup() || frame.parent.is_some() {
        return Err("NPC pending Getup installation has no fresh caller".into());
    }
    frame.parent = state.npc_action.take();
    if let Err(error) = frame.validate() {
        state.npc_action = frame.parent.take();
        return Err(error);
    }
    state.npc_action = Some(Box::new(frame));
    Ok(())
}

pub(crate) fn resume(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    loop {
        if state.mission.is_none() {
            state.npc_action = None;
            return Ok(events);
        }
        let Some(frame) = state.npc_action.as_deref() else {
            return Ok(events);
        };
        frame.validate()?;
        let waiting = scope(frame).ok_or("pending NPC caller is not waiting for Getup")?;
        if obstructed(state, waiting) {
            return Ok(events);
        }
        let mut frame = *state.npc_action.take().unwrap();
        // Restore the parent before the suffix can synchronously pause again.
        // Only the top caller can run; a blocked child is never skipped.
        state.npc_action = frame.parent.take();
        match frame.phase {
            Phase::WaitingFinalGetup(_) => {
                super::super::refresh_active_item_context(state, content)?;
                events.extend(super::super::movement_return::finish_npc_wrapper(
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
        // Every completed suffix removes one frame. A newly paused Getup has
        // an actual obstruction, so the next pass stops at that new head.
    }
}
