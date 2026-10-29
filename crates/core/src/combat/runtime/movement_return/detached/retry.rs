//! combatchoice11c000 retries involuntary movement using its retained locals.
use super::super::MoveLocals;
use super::{reaction, set_direction};
use crate::combat::runtime::reaction_calls::ReactionReturn;
use crate::{Content, Event, RuleError, Session};

pub(super) enum RetryStep {
    Attempt,
    Finished,
}

pub(super) fn involuntary(state: &Session) -> Result<bool, RuleError> {
    if state.native_work.walking_confusion != 0 {
        return Ok(true);
    }
    // A mission holds one to six warriors and the session admits only a selected
    // warrior inside them, so these reads always find its record.
    let warrior = state.warriors.get(state.selected_warrior).ok_or_else(|| {
        RuleError::from("selected warrior for involuntary walking outside the squad")
    })?;
    Ok(warrior.record().automatic_mode()? != 0)
}

pub(super) fn prepare(
    state: &mut Session,
    content: &Content,
    locals: &mut MoveLocals,
    events: &mut Vec<Event>,
) -> Result<RetryStep, RuleError> {
    if !involuntary(state)? || locals.previous_tile < 0 {
        return Ok(RetryStep::Attempt);
    }
    // The native WORD increments before testing the previous value or movement.
    let previous = locals.retries;
    locals.retries = previous.wrapping_add(1);
    let exhausted = if previous > 20 {
        true
    } else {
        state
            .warriors
            .get(state.selected_warrior)
            .ok_or_else(|| RuleError::from("selected warrior for walking retry outside the squad"))?
            .record()
            .movement_points()?
            < 1
    };
    if exhausted {
        set_direction(state, (0, 0))?;
        if matches!(
            reaction(state, content, 0, events)?,
            ReactionReturn::Terminated
        ) {
            return Ok(RetryStep::Finished);
        }
        state
            .warriors
            .get_mut(state.selected_warrior)
            .ok_or_else(|| {
                RuleError::from("selected warrior after walking retry reaction outside the squad")
            })?
            .record_mut()
            .set_guarding(1);
        events.extend(crate::combat::runtime::retained_turn::finish(
            state, content,
        )?);
        return Ok(RetryStep::Finished);
    }
    let dx = state.random.bound(3).wrapping_sub(2) as i8;
    let previous_y = super::direction(state)?.1;
    set_direction(state, (dx, previous_y))?;
    let dy = state.random.bound(3).wrapping_sub(2) as i8;
    set_direction(state, (dx, dy))?;
    Ok(RetryStep::Attempt)
}
