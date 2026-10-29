//! Getup's round boundary over shared work after the arena is detached.
use crate::combat::round_stories::RoundDispatch;
use crate::{Content, Event, RuleError, ScriptState, Session};

pub(super) enum Progress {
    Returned(Vec<Event>),
    Reduced(Vec<Event>),
    Terminated(Vec<Event>),
    Completed,
}

pub(super) fn begin(state: &mut Session, content: &Content) -> Result<Progress, RuleError> {
    let work = &mut state.native_work.round_work;
    let counter = work
        .counter
        .as_mut()
        .ok_or_else(|| RuleError::from("round counter missing after combat teardown"))?;
    *counter = counter.wrapping_add(1);
    if let Some(round) = state.native_work.turn_round.as_mut() {
        *round = round.wrapping_add(1);
    }
    work.slot = Some(0);
    for slot in 0..5 {
        work.slot = Some(slot as i16);
        let script = work
            .slots
            .ok_or_else(|| RuleError::from("round scripts missing after combat teardown"))?[slot];
        if script == 0 {
            continue;
        }
        work.processing = Some(1);
        let death_depth = RoundDispatch::death_depth(state);
        let mut record = state
            .land(content)
            .ok_or("round map missing")?
            .script(script)?
            .to_vec();
        let shared = state
            .exploration
            .as_ref()
            .and_then(|position| position.script.as_ref())
            .map(|script| script.record.clone())
            .or_else(|| {
                state
                    .turn_dispatch
                    .as_ref()
                    .filter(|frame| frame.returning)
                    .and_then(|frame| frame.work.clone())
            });
        let position = state.exploration.as_mut().ok_or("round position missing")?;
        if let Some(mut header) = shared {
            centurions_content::records::StoryRecordMut(&mut header).replace_body(&record);
            record = header;
        }
        state.round_dispatch = Some(RoundDispatch {
            retained: true,
            suspended_script: position.script.take(),
            parent: state.round_dispatch.take().map(Box::new),
            death_depth: Some(death_depth),
            ..Default::default()
        });
        position.script = Some(ScriptState::scheduled(record, 0));
        state.clear_story_entry_work();
        return crate::exploration::run_script(state, content).map(Progress::Returned);
    }
    work.slot = Some(5);
    let inactive = work
        .inactive_rounds
        .as_mut()
        .ok_or_else(|| RuleError::from("inactive rounds missing after combat teardown"))?;
    let previous = *inactive;
    *inactive = inactive.wrapping_add(1);
    if previous > 8 {
        state.native_work.victory_work.finish();
        state.native_work.early_booty = 1;
        return Ok(Progress::Completed);
    }
    state.native_work.initiative_work.cursor = 0;
    let outcome = super::reduction::apply(state, content)?;
    Ok(match outcome.flow {
        crate::combat::runtime::TurnFlow::Returned => Progress::Reduced(outcome.events),
        crate::combat::runtime::TurnFlow::Terminated => Progress::Terminated(outcome.events),
    })
}
