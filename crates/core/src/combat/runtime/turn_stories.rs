//! Session-owned return from getup's general newland call.
use super::turns::{finish_enemy_setup, initialize_active_turn, resume_getup, turn_feedback};
use crate::combat::final_turn_stories::TurnProgram;
use crate::{Event, RuleError, ScriptState, Session};
use centurions_content::Content;
use serde::{Deserialize, Serialize};
mod child;
pub use child::TurnChildReturn;
pub(crate) use child::{begin as begin_child, mark_cleanup, resume as resume_child};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurnCaller {
    Initial,
    #[default]
    Advance,
    Retained,
}

impl TurnCaller {
    fn advancing(&self) -> bool {
        *self == Self::Advance
    }
}

pub(crate) enum TurnInitialization {
    Ready(bool),
    Handled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnDispatch {
    #[serde(default, skip_serializing_if = "TurnCaller::advancing")]
    pub caller: TurnCaller,
    pub body: i16,
    pub script: i16,
    pub suspended_script: Option<ScriptState>,
    pub death_depth: usize,
    pub round_depth: usize,
    #[serde(default)]
    pub parent: Option<Box<TurnDispatch>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child: Option<TurnChildReturn>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub cleanup: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub returning: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub shared_record: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work: Option<Vec<u8>>,
}

impl TurnDispatch {
    fn round_depth(state: &Session) -> usize {
        std::iter::successors(state.round_dispatch.as_ref(), |frame| {
            frame.parent.as_deref()
        })
        .take(33)
        .count()
    }

    pub(crate) fn owns_script(&self, state: &Session) -> bool {
        !self.returning
            && !self.child.as_ref().is_some_and(|child| child.waiting)
            && self.at_depth(state)
    }

    fn at_depth(&self, state: &Session) -> bool {
        crate::combat::round_stories::RoundDispatch::death_depth(state) <= self.death_depth
            && Self::round_depth(state) <= self.round_depth
    }

    pub(crate) fn extended_return(&self) -> bool {
        self.child.is_some()
            || self.cleanup
            || self.returning
            || self.shared_record
            || self.work.is_some()
    }

    pub(crate) fn validate(&self, state: &Session, content: &Content) -> Result<(), RuleError> {
        let map = state.land(content).ok_or("turn story map missing")?;
        let mut current = Some(self);
        let mut depth = 0;
        while let Some(frame) = current {
            depth += 1;
            if depth > 32
                || !(10..110).contains(&frame.body)
                || map.script(frame.script).is_err()
                || frame.death_depth
                    > crate::combat::round_stories::RoundDispatch::death_depth(state)
                || frame.round_depth > Self::round_depth(state)
                || frame
                    .suspended_script
                    .as_ref()
                    .is_some_and(|script| !script.valid(content))
                || frame.work.as_ref().is_some_and(|record| record.len() != 42)
                || frame.returning && frame.child.as_ref().is_some_and(|child| child.waiting)
                || (frame.child.is_some()
                    || frame.cleanup
                    || frame.returning
                    || frame.shared_record)
                    && frame.work.is_none()
            {
                return Err("invalid enemy turn story caller".into());
            }
            if let Some(child) = &frame.child {
                child.validate(content, state)?;
            }
            current = frame.parent.as_deref();
        }
        if state.combat.is_none() && self.caller != TurnCaller::Retained && !self.extended_return()
            || state
                .exploration
                .as_ref()
                .is_none_or(|position| position.script.is_none() && !self.returning)
        {
            return Err("enemy turn story has no active combat script".into());
        }
        Ok(())
    }
}

pub(crate) fn initialize_session(
    state: &mut Session,
    content: &Content,
    caller: TurnCaller,
) -> Result<(Vec<Event>, TurnInitialization), RuleError> {
    if caller == TurnCaller::Initial && super::has_active_warrior(&state.warriors) {
        state.reset_npc_work();
    }
    if caller == TurnCaller::Initial
        && let Some(body) = state
            .combat
            .as_ref()
            .and_then(|combat| combat.active_body())
        && (0..6).contains(&body)
        && state.warriors[body as usize].record().active_flag() != 0
        && state.warriors[body as usize]
            .record()
            .turn_skip_duration()?
            == 0
    {
        // combatsetup11eda4 enters Getup's same party-selection suffix.
        state.selected_warrior = body as usize;
        state.set_warrior_selector(body as usize);
    }
    let mission = state.land(content).ok_or("combat tables missing")?;
    let combat = state.combat.as_mut().ok_or("combat missing")?;
    let body = combat.active_body().ok_or("active combatant missing")?;
    if caller == TurnCaller::Initial {
        state.native_work.walking_confusion = 0;
    }
    if body < 10 {
        let warrior = state
            .warriors
            .get_mut(body as usize)
            .ok_or("turn warrior outside represented records")?;
        if warrior.record().active_flag() != 0
            && warrior.record().turn_skip_duration()? == 0
            && warrior.record().faction()? != 0
        {
            state.native_work.walking_confusion = 1;
            warrior.record_mut().set_automatic_mode(1);
        }
        let (events, skipped) =
            initialize_active_turn(combat, &mut state.warriors, mission, &mut state.random)?;
        return Ok((events, TurnInitialization::Ready(skipped)));
    }
    let tables = mission.combat.as_ref().ok_or("turn tables missing")?;
    let (turn, program) = crate::combat::final_turn_stories::initialize_for_session(
        combat,
        &mut state.warriors,
        mission,
        &tables.monsters,
        &tables.encounters,
        &mut state.random,
    )?;
    if let Some(record) = turn.extra_record {
        state.native_work.extra_record = record;
    }
    let mut events = turn_feedback(mission, turn.effects)?;
    if let Some(program) = program {
        events.extend(begin(state, content, body, program, caller)?);
        return Ok((events, TurnInitialization::Handled));
    }
    let (events, skipped) = finish_enemy_setup(combat, body, events, turn.skipped_turn)?;
    Ok((events, TurnInitialization::Ready(skipped)))
}

pub(super) fn begin(
    state: &mut Session,
    content: &Content,
    body: i16,
    program: TurnProgram,
    caller: TurnCaller,
) -> Result<Vec<Event>, RuleError> {
    state.retain_monster_work();
    let death_depth = crate::combat::round_stories::RoundDispatch::death_depth(state);
    let round_depth = TurnDispatch::round_depth(state);
    let mut record = state
        .land(content)
        .ok_or("turn story map missing")?
        .script(program.script)?
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
                .and_then(|frame| frame.work.clone())
        });
    let position = state
        .exploration
        .as_mut()
        .ok_or("turn story position missing")?;
    // newland48 saves ACTION alone; newland2/107 save actions, parameters and level.
    let shared_record = position.script.as_ref().is_some_and(|script| {
        script.cursor.checked_sub(1).is_some_and(|cursor| {
            centurions_content::records::StoryRecord(&script.record)
                .instruction(usize::from(cursor))
                .is_ok_and(|(opcode, _)| opcode == 48)
        })
    });
    if let Some(mut retained) = shared {
        centurions_content::records::StoryRecordMut(&mut retained).replace_body(&record);
        record = retained;
    }
    state.turn_dispatch = Some(TurnDispatch {
        caller,
        body,
        script: program.script,
        suspended_script: position.script.take(),
        death_depth,
        round_depth,
        parent: state.turn_dispatch.take().map(Box::new),
        child: None,
        cleanup: false,
        returning: false,
        shared_record,
        work: Some(record.clone()),
    });
    position.script = Some(ScriptState::scheduled(record, program.cursor));
    state.clear_story_entry_work();
    crate::exploration::run_script(state, content)
}

pub(crate) fn finish(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let mut frame = state
        .turn_dispatch
        .take()
        .ok_or("turn story caller missing")?;
    if frame.child.as_ref().is_some_and(|child| child.waiting) {
        return Err("enemy turn child battle has not returned".into());
    }
    let caller = frame.caller;
    frame.returning = true;
    state.turn_dispatch = Some(frame);
    let mut events = if state.combat.is_none() {
        super::retained_turn::resume_enemy(state, content, caller != TurnCaller::Initial)?
    } else {
        finish_live(state, content, caller)?
    };
    events.extend(finish_returning(state, content)?);
    Ok(events)
}

pub(crate) fn finish_returning(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    if !state
        .turn_dispatch
        .as_ref()
        .is_some_and(|frame| frame.returning && frame.at_depth(state))
        || state.survivors.is_some()
        || state
            .exploration
            .as_ref()
            .is_none_or(|position| position.presentation.is_some())
    {
        return Ok(events);
    }
    let frame = state.turn_dispatch.take().unwrap();
    state.turn_dispatch = frame.parent.map(|parent| *parent);
    if state.exploration.is_none() {
        return Ok(events);
    }
    state.exploration.as_mut().unwrap().script = frame.suspended_script;
    if let Some(child) = frame.child {
        child.restore(state);
    }
    if frame.caller != TurnCaller::Retained
        && state.combat.is_none()
        && !super::npc_return::owns_pending_return(state)
    {
        events.extend(super::finish_battle_wrapper(state, content)?);
    } else {
        events.extend(super::selected_rewards::after_movement(state, content)?);
    }
    events.extend(finish_returning(state, content)?);
    Ok(events)
}

fn finish_live(
    state: &mut Session,
    content: &Content,
    caller: TurnCaller,
) -> Result<Vec<Event>, RuleError> {
    let combat = state.combat.as_mut().ok_or("turn story combat missing")?;
    let selected = combat.selected_enemy;
    // getup126a06 tests equality with zero, before clearing the script origin.
    if combat.story_record(selected)?.stamina()? == 0 {
        return resume_getup(state, content);
    }
    combat.script_enemy = 0;
    let record = combat.story_record_mut(selected)?;
    let (skipped, hostile_change) =
        crate::combat::initialize_enemy_record(&mut record.bytes, &mut state.random)?;
    combat.victory_work.add_total(hostile_change);
    let body = combat
        .active_body()
        .ok_or("turn story initiative body missing")?;
    let (mut events, skipped) =
        super::turns::finish_selected_enemy_setup(combat, body, selected, vec![], skipped)?;
    if skipped {
        events.extend(resume_getup(state, content)?);
    } else if caller != TurnCaller::Initial {
        events.push(Event::TurnStarted {
            body,
            round: combat.round,
        });
    }
    Ok(events)
}
