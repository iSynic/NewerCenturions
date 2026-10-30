//! getup's end of round after the battle has ended.
use crate::combat::round_stories::RoundDispatch;
use crate::combat::runtime::TurnFlow;
use crate::{Content, Event, RuleError, ScriptState, Session};

/// How a round start ended after the battle: a round script took over, the party and monsters were
/// reduced, the reduction ended the battle, or the battle is over for want of action.
pub(super) enum Progress {
    Returned(Vec<Event>),
    Reduced(Vec<Event>),
    Terminated(Vec<Event>),
    Completed,
}

/// Starts a round after the battle has ended. The round counters advance and the next round script
/// runs; a script that starts a story takes over. After ten rounds without an attack the battle
/// ends with the early-booty flag set; otherwise the battle order restarts and the party and
/// monsters are reduced.
///
/// Original: getup
pub(super) fn begin(state: &mut Session, content: &Content) -> Result<Progress, RuleError> {
    let scripts = &mut state.globals.round_scripts;
    let counter = scripts
        .counter
        .as_mut()
        .ok_or_else(|| RuleError::from("round counter missing after combat teardown"))?;
    *counter = counter.wrapping_add(1);
    if let Some(round) = state.globals.turn_round.as_mut() {
        *round = round.wrapping_add(1);
    }
    scripts.slot = Some(0);
    for slot in 0..5 {
        scripts.slot = Some(slot as i16);
        let script = scripts
            .slots
            .ok_or_else(|| RuleError::from("round scripts missing after combat teardown"))?[slot];
        if script == 0 {
            continue;
        }
        scripts.processing = Some(1);
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
                    .and_then(|frame| frame.script_record.clone())
            });
        let position = state.exploration.as_mut().ok_or("round position missing")?;
        if let Some(mut header) = shared {
            centurions_content::records::StoryRecordMut(&mut header).replace_body(&record);
            record = header;
        }
        state.round_dispatch = Some(RoundDispatch {
            out_of_combat: true,
            suspended_script: position.script.take(),
            parent: state.round_dispatch.take().map(Box::new),
            death_depth: Some(death_depth),
            ..Default::default()
        });
        position.script = Some(ScriptState::scheduled(record, 0));
        state.clear_story_entry_byte();
        return crate::exploration::run_script(state, content).map(Progress::Returned);
    }
    scripts.slot = Some(5);
    let inactive = scripts
        .blank_round
        .as_mut()
        .ok_or_else(|| RuleError::from("inactive rounds missing after combat teardown"))?;
    let previous = *inactive;
    *inactive = inactive.wrapping_add(1);
    if previous > 8 {
        state.globals.kill_counts.finish();
        state.globals.coward = 1;
        return Ok(Progress::Completed);
    }
    state.globals.battle_order.index = 0;
    let outcome = super::reduction::apply(state, content)?;
    Ok(match outcome.flow {
        TurnFlow::Returned => Progress::Reduced(outcome.events),
        TurnFlow::Terminated => Progress::Terminated(outcome.events),
    })
}
