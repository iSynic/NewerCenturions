//! timeclick(0)'s calendar scan and newland(mode 1) return.
use super::scan_schedule;
use crate::{Event, Phase, RuleError, ScriptState, Session};
use centurions_content::Content;
use centurions_content::records::{ScheduleEntry, ScheduledScripts, StoryRecordMut};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ZeroTimeReturn {
    pub schedule_cursor: usize,
    pub suspended_script: Option<ScriptState>,
    /// Shared DOOR479eaa; the caller's cursor survives, its old body does not.
    pub work: Vec<u8>,
    /// A calendar script that crosses midnight again enters timeclick(0) recursively.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<Box<ZeroTimeReturn>>,
    /// Caller owners open when newland63 entered timeclick(0) (schema 56); absent
    /// for an exploration caller.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owners: Option<ZeroOwners>,
}

/// Turn, round and death story depths beneath a calendar frame.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ZeroOwners {
    pub turn: usize,
    pub round: usize,
    pub death: usize,
}

impl ZeroOwners {
    fn current(state: &Session) -> Self {
        Self {
            turn: std::iter::successors(state.turn_dispatch.as_ref(), |f| f.parent.as_deref())
                .take(33)
                .count(),
            round: std::iter::successors(state.round_dispatch.as_ref(), |f| f.parent.as_deref())
                .take(33)
                .count(),
            death: crate::combat::round_stories::RoundDispatch::death_depth(state),
        }
    }
}

impl ZeroTimeReturn {
    /// No caller owner has opened since this frame; its scripts and scan return here.
    pub(crate) fn innermost(&self, state: &Session) -> bool {
        let owners = self.owners.unwrap_or_default();
        let current = ZeroOwners::current(state);
        current.turn <= owners.turn
            && current.round <= owners.round
            && current.death <= owners.death
    }
}

pub(crate) fn validate_zero(state: &Session, content: &Content) -> Result<(), RuleError> {
    let frames = std::iter::successors(state.zero_time.as_ref(), |f| f.parent.as_deref());
    for (depth, frame) in frames.enumerate() {
        if depth >= 32
            || state
                .exploration
                .as_ref()
                .is_none_or(|p| frame.schedule_cursor > ScheduleEntry::count(&p.schedule))
            || state.time_continuation.is_some() && frame.owners.is_none()
            || frame
                .owners
                .is_some_and(|owners| owners.turn > 32 || owners.round > 32 || owners.death > 32)
            || frame.work.len() != 42
            || frame.parent.is_some() && frame.suspended_script.is_none()
            || frame
                .suspended_script
                .as_ref()
                .is_some_and(|s| !s.valid(content))
        {
            return Err("invalid zero-time calendar return".into());
        }
    }
    Ok(())
}

/// No hourly reduction, food recovery or regional rolls, even at midnight.
pub(crate) fn zero_tick(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let Some(position) = state.exploration.as_mut() else {
        return Ok(vec![]);
    };
    if position.hour != 24 {
        return Ok(vec![]);
    }
    position.day = position.day.wrapping_add(1);
    position.hour = 0;
    let mut events = vec![];
    let mut cursor = 0;
    if let Some(id) = scan_schedule(state, &mut cursor, &mut events)? {
        // timeclick runs the scheduled children synchronously under any newland63
        // caller; the caller's owner resumes after the scan. A menu caller keeps its
        // encounter continuation in the suspended script, as newland keeps it on the
        // stack. Every battle script is entered under a turn, round or death owner.
        let current = ZeroOwners::current(state);
        let owned = current != ZeroOwners::default();
        if state.combat.is_some() && !owned {
            return Err("calendar child under a battle script without a story owner".into());
        }
        let suspended_script = state.exploration.as_mut().unwrap().script.take();
        let work = suspended_script
            .as_ref()
            .map_or_else(|| vec![0; 42], |s| s.record.clone());
        let parent = state.zero_time.take().map(Box::new);
        state.zero_time = Some(ZeroTimeReturn {
            schedule_cursor: cursor,
            suspended_script,
            work,
            parent,
            owners: (owned || state.time_continuation.is_some()).then_some(current),
        });
        events.extend(dispatch(state, content, id)?);
    }
    Ok(events)
}

fn dispatch(state: &mut Session, content: &Content, id: i16) -> Result<Vec<Event>, RuleError> {
    let position = state
        .exploration
        .as_ref()
        .ok_or("calendar position missing")?;
    let map = state
        .map(content, position.map)
        .ok_or("calendar content missing")?;
    let body = ScheduledScripts(&map.scheduled_scripts).record(id.unsigned_abs())?;
    let position = state.exploration.as_mut().unwrap();
    let frame = state.zero_time.as_mut().unwrap();
    // Each child is a fresh newland call with its own locals and registers.
    let work = &mut frame.work;
    // newland140142..140172 sets the current destination before loaddoor2,
    // which retains the eight-byte header and replaces bytes8..41.
    let mut record = StoryRecordMut(work);
    record.relocate(position.map, position.x, position.y);
    record.replace_body(body);
    // loaddoor2 includes byte40 in the loaded body, after the caller's write.
    position.script = Some(ScriptState::scheduled(work.clone(), 0));
    crate::exploration::run_script(state, content)
}

pub(crate) fn retain_zero_work(state: &mut Session, record: &[u8]) {
    if let Some(frame) = state.zero_time.as_mut() {
        frame.work = record.to_vec();
    }
}

/// A calendar entered under a battle or time owner continues its scan once its
/// child has ended, before that owner resumes.
pub(crate) fn resume_owned(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    if state
        .zero_time
        .as_ref()
        .is_some_and(|frame| frame.owners.is_some() && frame.innermost(state))
    {
        resume(state, content)
    } else {
        Ok(vec![])
    }
}

pub(super) fn resume(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    loop {
        let owned = state
            .zero_time
            .as_ref()
            .is_some_and(|frame| frame.owners.is_some());
        if state.phase != Phase::Exploration && !owned
            || state
                .exploration
                .as_ref()
                .is_none_or(|p| p.script.is_some() || p.presentation.is_some())
        {
            return Ok(events);
        }
        let mut cursor = state.zero_time.as_ref().unwrap().schedule_cursor;
        let selected = scan_schedule(state, &mut cursor, &mut events)?;
        state.zero_time.as_mut().unwrap().schedule_cursor = cursor;
        if let Some(id) = selected {
            events.extend(dispatch(state, content, id)?);
            // Mission loss clears both the map and its caller.
            if state.zero_time.is_none() {
                return Ok(events);
            }
        } else {
            let frame = state.zero_time.take().unwrap();
            // The outer scan keeps the single DOOR the inner calendar left behind.
            state.zero_time = frame.parent.map(|mut parent| {
                parent.work = frame.work.clone();
                *parent
            });
            if let Some(mut caller) = frame.suspended_script {
                caller.record = frame.work;
                state.exploration.as_mut().unwrap().script = Some(caller);
                events.extend(crate::exploration::run_script(state, content)?);
            }
            return Ok(events);
        }
    }
}
