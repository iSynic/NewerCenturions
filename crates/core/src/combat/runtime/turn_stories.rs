//! Returning to getup after a turn's story.
use super::turns::{finish_enemy_setup, initialize_active_turn, resume_getup, turn_feedback};
use crate::combat::final_turn_stories::TurnProgram;
use crate::combat::round_stories::RoundDispatch;
use crate::{Event, RuleError, ScriptState, Session};
use centurions_content::Content;
use serde::{Deserialize, Serialize};
mod child;
pub use child::TurnChildReturn;
pub(crate) use child::{begin as begin_child, mark_cleanup, resume as resume_child};

/// What set up the turn: the first turn of a battle, a turn reached by advancing, or one after the
/// battle has ended.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurnCaller {
    Initial,
    #[default]
    Advance,
    #[serde(rename = "Retained")]
    OutOfCombat,
}

impl TurnCaller {
    /// Whether the turn was reached by advancing.
    fn advancing(&self) -> bool {
        *self == Self::Advance
    }
}

/// How a turn's setup ended: ready (with whether the turn is skipped) or handled by a story.
pub(crate) enum TurnInitialization {
    Ready(bool),
    Handled,
}

/// A monster's turn script in progress: whose turn it is, the script, the script it interrupted,
/// how deep the death and round stories nest, and what a battle the script started must return to.
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
    #[serde(rename = "work")]
    pub script_record: Option<Vec<u8>>,
}

impl TurnDispatch {
    /// How deeply round scripts are nested.
    fn round_depth(state: &Session) -> usize {
        std::iter::successors(state.round_dispatch.as_ref(), |frame| {
            frame.parent.as_deref()
        })
        .take(33)
        .count()
    }

    /// Whether this turn script is the one that runs now: it has not returned, waits for no battle,
    /// and nothing nests deeper.
    pub(crate) fn owns_script(&self, state: &Session) -> bool {
        !self.returning
            && !self.child.as_ref().is_some_and(|child| child.waiting)
            && self.at_depth(state)
    }

    /// Whether the death and round stories run no deeper than when the script began.
    fn at_depth(&self, state: &Session) -> bool {
        RoundDispatch::death_depth(state) <= self.death_depth
            && Self::round_depth(state) <= self.round_depth
    }

    /// Whether the script returns through more than a plain end: a battle it started, cleanup,
    /// returning, a shared or saved script record.
    pub(crate) fn extended_return(&self) -> bool {
        self.child.is_some()
            || self.cleanup
            || self.returning
            || self.shared_record
            || self.script_record.is_some()
    }

    /// Refuses a saved chain of turn scripts that is too deep or names a body, script or record
    /// that does not fit.
    pub(crate) fn validate(&self, state: &Session, content: &Content) -> Result<(), RuleError> {
        let map = state.land(content).ok_or("turn story map missing")?;
        let mut current = Some(self);
        let mut depth = 0;
        while let Some(frame) = current {
            depth += 1;
            if depth > 32
                || !(10..110).contains(&frame.body)
                || map.script(frame.script).is_err()
                || frame.death_depth > RoundDispatch::death_depth(state)
                || frame.round_depth > Self::round_depth(state)
                || frame
                    .suspended_script
                    .as_ref()
                    .is_some_and(|script| !script.valid(content))
                || frame
                    .script_record
                    .as_ref()
                    .is_some_and(|record| record.len() != 42)
                || frame.returning && frame.child.as_ref().is_some_and(|child| child.waiting)
                || (frame.child.is_some()
                    || frame.cleanup
                    || frame.returning
                    || frame.shared_record)
                    && frame.script_record.is_none()
            {
                return Err("invalid enemy turn story caller".into());
            }
            if let Some(child) = &frame.child {
                child.validate(content, state)?;
            }
            current = frame.parent.as_deref();
        }
        if state.combat.is_none()
            && self.caller != TurnCaller::OutOfCombat
            && !self.extended_return()
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

/// Sets up the turn of the active body. A warrior is put under computer control when it has changed
/// sides, and a monster runs its turn script when it has one, in which case the setup is handled by
/// the story.
///
/// Original: getup
pub(crate) fn initialize_session(
    state: &mut Session,
    content: &Content,
    caller: TurnCaller,
) -> Result<(Vec<Event>, TurnInitialization), RuleError> {
    if caller == TurnCaller::Initial && super::has_active_warrior(&state.warriors) {
        state.reset_monster_turn_flags();
    }
    if caller == TurnCaller::Initial
        && let Some(body) = state
            .combat
            .as_ref()
            .and_then(|combat| combat.active_body())
        && (0..6).contains(&body)
        && state.warriors[body as usize].record().in_battle() != 0
        && state.warriors[body as usize]
            .record()
            .turn_skip_duration()?
            == 0
    {
        // combatsetup selects the first warrior the same way getup does.
        state.char_select_new = body as usize;
        state.set_warrior_selector(body as usize);
    }
    let mission = state.land(content).ok_or("combat tables missing")?;
    let combat = state.combat.as_mut().ok_or("combat missing")?;
    let body = combat.active_body().ok_or("active combatant missing")?;
    if caller == TurnCaller::Initial {
        state.globals.possessed = 0;
    }
    if body < 10 {
        let warrior = state
            .warriors
            .get_mut(body as usize)
            .ok_or("turn warrior outside represented records")?;
        if warrior.record().in_battle() != 0
            && warrior.record().turn_skip_duration()? == 0
            && warrior.record().faction()? != 0
        {
            state.globals.possessed = 1;
            warrior.record_mut().set_computer_control(1);
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
        state.globals.extra_record = record;
    }
    let mut events = turn_feedback(mission, turn.effects)?;
    if let Some(program) = program {
        events.extend(begin(state, content, body, program, caller)?);
        return Ok((events, TurnInitialization::Handled));
    }
    let (events, skipped) = finish_enemy_setup(combat, body, events, turn.skipped_turn)?;
    Ok((events, TurnInitialization::Ready(skipped)))
}

/// Starts a monster's turn script, keeping the script it interrupts and the battle state it must
/// return to.
pub(super) fn begin(
    state: &mut Session,
    content: &Content,
    body: i16,
    program: TurnProgram,
    caller: TurnCaller,
) -> Result<Vec<Event>, RuleError> {
    state.store_monster_pick();
    let death_depth = RoundDispatch::death_depth(state);
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
                .and_then(|frame| frame.script_record.clone())
        });
    let position = state
        .exploration
        .as_mut()
        .ok_or("turn story position missing")?;
    // Story opcode 48 saves only the action; opcodes 2 and 107 also save its
    // parameters and level.
    let shared_record = position.script.as_ref().is_some_and(|script| {
        script.cursor.checked_sub(1).is_some_and(|cursor| {
            centurions_content::records::StoryRecord(&script.record)
                .instruction(usize::from(cursor))
                .is_ok_and(|(opcode, _)| opcode == 48)
        })
    });
    if let Some(mut story) = shared {
        centurions_content::records::StoryRecordMut(&mut story).replace_body(&record);
        record = story;
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
        script_record: Some(record.clone()),
    });
    position.script = Some(ScriptState::scheduled(record, program.cursor));
    state.clear_story_entry_byte();
    crate::exploration::run_script(state, content)
}

/// Ends a turn script. A battle it started must have returned; the monster's turn is then set up or
/// the scan for the next turn goes on.
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
        super::out_of_combat_getup::resume_enemy(state, content, caller != TurnCaller::Initial)?
    } else {
        finish_live(state, content, caller)?
    };
    events.extend(finish_returning(state, content)?);
    Ok(events)
}

/// Returns the interrupted script once nothing holds it up any longer: no survivors screen and no
/// message.
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
    if frame.caller != TurnCaller::OutOfCombat
        && state.combat.is_none()
        && !super::monster_return::owns_pending_return(state)
    {
        events.extend(super::finish_battle_wrapper(state, content)?);
    } else {
        events.extend(super::selected_rewards::after_movement(state, content)?);
    }
    events.extend(finish_returning(state, content)?);
    Ok(events)
}

/// Finishes a turn script inside the battle. A monster without stamina has its turn skipped; any
/// other is set up and, if its turn is not skipped, started.
fn finish_live(
    state: &mut Session,
    content: &Content,
    caller: TurnCaller,
) -> Result<Vec<Event>, RuleError> {
    let combat = state.combat.as_mut().ok_or("turn story combat missing")?;
    let selected = combat.monster_up;
    // Only a stamina of exactly zero counts here (see docs/source-notes/combat.md).
    if combat.story_record(selected)?.stamina()? == 0 {
        return resume_getup(state, content);
    }
    combat.macro_monster = 0;
    let record = combat.story_record_mut(selected)?;
    let (skipped, hostile_change) =
        crate::combat::initialize_enemy_record(&mut record.bytes, &mut state.random)?;
    combat.kill_counts.add_num_enemy(hostile_change);
    let body = combat
        .active_body()
        .ok_or("turn story initiative body missing")?;
    let (mut events, skipped) =
        super::turns::finish_monster_up_setup(combat, body, selected, vec![], skipped)?;
    if skipped {
        events.extend(resume_getup(state, content)?);
    } else if caller != TurnCaller::Initial {
        events.push(Event::TurnStarted {
            body,
            round: combat.combat_round,
        });
    }
    Ok(events)
}
