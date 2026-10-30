//! A monster's turn, resumed after a reaction's death story.
use super::neighbor_scan::{self, NeighborScan};
use super::reaction_calls::{AttackCall, GuardStep, ReactionContext};
use super::reaction_frame::ReactionFrame;
use crate::combat::map_alias;
use crate::{Content, Event, RuleError, Session};
use serde::{Deserialize, Serialize};
mod firearm;
mod firearm_selection;
mod global_spells;
mod movement;
mod pending;
mod scientist;
mod services;
mod target;
pub mod turn_flags;
mod turn_state;
use movement::MonsterMoveFrame;
use pending::GetupScope;
pub(super) use pending::owns_return as owns_pending_return;
pub(crate) use pending::resume as resume_pending;

/// A monster's turn in progress, resumable after a reaction's death script: the step it has
/// reached, the reaction check or move waiting to finish, and how far the search for a visible
/// target has got.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonsterActionFrame {
    phase: Phase,
    reaction: Option<ReactionFrame>,
    movement: Option<MonsterMoveFrame>,
    target_cursor: i16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parent: Option<Box<MonsterActionFrame>>,
}

/// The steps of a monster's turn: its decision to fire or cast, entering movement, the move,
/// placement in the new cell, the check after the step, the fallback casts when the move failed or
/// the movement ran out, the final shot, and waiting for the start-of-turn step that follows a
/// battle's end.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Phase {
    Decision,
    MovementEntry,
    Movement,
    Placement,
    AfterStep,
    FailedFallback,
    Exhausted,
    ExhaustedFallback,
    FinalFire,
    WaitingGetup(GetupScope),
    WaitingFinalGetup(GetupScope),
}

impl MonsterActionFrame {
    /// Whether the turn waits for the next turn to be set up.
    pub(crate) fn waiting_getup(&self) -> bool {
        matches!(
            self.phase,
            Phase::WaitingGetup(_) | Phase::WaitingFinalGetup(_)
        )
    }

    /// Refuses a saved monster turn that does not hold what its step needs.
    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        pending::validate(self)
    }

    /// Refuses one saved step of a monster's turn whose search position, move or reaction check
    /// does not fit its phase.
    fn validate_node(&self) -> Result<(), RuleError> {
        if !(0..=110).contains(&self.target_cursor) {
            return Err("invalid saved NPC target scan local".into());
        }
        if let Some(movement) = &self.movement {
            movement.validate()?;
        }
        match &self.reaction {
            Some(reaction)
                if matches!(
                    self.phase,
                    Phase::MovementEntry
                        | Phase::Placement
                        | Phase::AfterStep
                        | Phase::FailedFallback
                        | Phase::ExhaustedFallback
                ) && reaction.returned_attack() =>
            {
                reaction.validate()?;
                if (self.phase == Phase::Placement) != self.movement.is_some() {
                    return Err("NPC departure return has no movement locals".into());
                }
            }
            None if self.waiting_getup() && self.movement.is_none() => {}
            _ => return Err("saved NPC caller has no returning reaction or Getup".into()),
        }
        Ok(())
    }

    /// Starts a reaction check of the given mode for the acting monster, then continues at `next`.
    fn react(&mut self, state: &Session, mode: usize, next: Phase) -> Result<(), RuleError> {
        self.phase = next;
        self.reaction = Some(ReactionFrame::new(
            turn_state::body(state)?,
            mode,
            turn_state::direction(state)?,
        )?);
        Ok(())
    }
}

/// Starts the active monster's turn from its decision.
///
/// Original: combat
pub(crate) fn begin(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let combat = state.combat.as_ref().ok_or("combat missing")?;
    let body = combat
        .active_body()
        .filter(|body| *body >= 10)
        .ok_or("active enemy missing")?;
    combat.story_record((body - 10) as usize)?;
    run(
        state,
        content,
        MonsterActionFrame {
            phase: Phase::Decision,
            reaction: None,
            movement: None,
            target_cursor: 0,
            parent: None,
        },
    )
}

/// Resumes a monster's turn after a death script returned; refused while it still waits for the
/// next turn to be set up.
pub(crate) fn resume(
    state: &mut Session,
    content: &Content,
    frame: MonsterActionFrame,
) -> Result<Vec<Event>, RuleError> {
    frame.validate()?;
    if frame.waiting_getup() {
        return Err("NPC reaction return owns a Getup frame".into());
    }
    run(state, content, frame)
}

/// Whether monsters may still move, by the tally of hostile and defeated monsters.
fn moving(state: &Session) -> Result<bool, RuleError> {
    if let Some(combat) = &state.combat {
        Ok(combat.kill_counts.movement_admitted())
    } else {
        Ok(state.globals.kill_counts.movement_admitted())
    }
}

/// Rescans the surroundings of the acting monster on entering movement, in or out of the battle.
fn scan_entry(state: &mut Session, content: &Content) -> Result<(), RuleError> {
    let mission = turn_state::mission(state, content)?;
    if let Some(combat) = &mut state.combat {
        neighbor_scan::begin(combat);
        neighbor_scan::scan(
            combat,
            &state.warriors,
            1,
            (0, 0),
            &mut NeighborScan {
                char_select_new: state.char_select_new,
                party_track: &mut state.globals.party_track,
                aliases: map_alias::session_map_aliases!(state, mission),
                table: Some(&state.globals.warrior_table),
            },
        )?;
    } else {
        state.globals.neighbors.departure = 0;
        neighbor_scan::scan_out_of_combat(state, content, 1, &mut 0)?;
    }
    Ok(())
}

/// The spell a monster tries after its movement failed or ran out of points. Nothing happens when
/// the turn's spell-attempt count is above 1, the monster has no cast chance or it has attacked
/// this turn. Otherwise the attempt count is raised to 2 (by 2 after a failed move), the
/// monster's movement is set to 0, and it casts unless spells are disabled or blocked after
/// movement.
///
/// Original: combat
fn fallback(
    state: &mut Session,
    content: &Content,
    failed: bool,
) -> Result<super::TurnOutcome, RuleError> {
    state.store_monster_turn_flags();
    let selected = turn_state::selected(state)?;
    if state.globals.monster_turn_flags.no_spell() > 1
        || turn_state::record(state, selected)?
            .record()
            .cast_percent()?
            == 0
        || state.globals.monster_turn_flags.did_attack()
    {
        return Ok(super::TurnOutcome::default());
    }
    if failed {
        state.globals.monster_turn_flags.add_no_spell(2);
    } else {
        state.globals.monster_turn_flags.no_spell = 2;
    }
    turn_state::record_mut(state, selected)?
        .record_mut()
        .set_movement(0);
    let record = turn_state::record(state, selected)?.record();
    if state.enemy_spell_disabled != 0 || record.spell_blocked_after_movement() {
        return Ok(super::TurnOutcome::default());
    }
    let outcome = services::spell(state, content)?;
    if outcome.flow == super::TurnFlow::Terminated {
        return Ok(super::TurnOutcome {
            events: outcome.events,
            flow: outcome.flow,
        });
    }
    state.globals.monster_turn_flags.add_no_spell(1);
    state.clear_targets();
    Ok(super::TurnOutcome {
        events: outcome.events,
        flow: outcome.flow,
    })
}

/// Runs the monster's turn from its current step until it ends, a death script suspends it or the
/// battle ends.
///
/// Original: combat
fn run(
    state: &mut Session,
    content: &Content,
    mut frame: MonsterActionFrame,
) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    // Every cycle passes the decision. A repeated decision means the game never
    // leaves the loop; a monster's choices draw random numbers and keep it moving.
    let mut decisions = std::collections::HashSet::new();
    loop {
        if state.mission.is_none() {
            return Ok(events);
        }
        if frame.reaction.is_some() {
            match resume_reaction(state, content, frame, &mut events)? {
                Some(next) => {
                    frame = next;
                    continue;
                }
                None => return Ok(events),
            }
        }
        state.store_monster_turn_flags();
        match frame.phase {
            Phase::Decision => {
                match decide(state, content, &mut frame, &mut events, &mut decisions)? {
                    Step::Next => {}
                    Step::Return => return Ok(events),
                    Step::Finish => return finish(state, content, events),
                }
            }
            Phase::MovementEntry => {
                frame.target_cursor = 0;
                frame.phase = Phase::Movement;
            }
            Phase::Movement => {
                if !moving(state)? {
                    frame.phase = Phase::FinalFire;
                    continue;
                }
                let record = turn_state::record(state, turn_state::selected(state)?)?.record();
                if record.movement() <= 0
                    || record.action_points() <= 1
                    || record.turn_skip_duration() != 0
                {
                    frame.phase = Phase::Exhausted;
                    continue;
                }
                target::select(state, content, &mut frame.target_cursor)?;
                if turn_state::record(state, turn_state::selected(state)?)?
                    .record()
                    .fleeing_duration()
                    != 0
                {
                    scan_entry(state, content)?;
                }
                let mut movement = MonsterMoveFrame::begin(state)?;
                if !movement.prepare(state, content)? {
                    frame.react(state, 0, Phase::FailedFallback)?;
                    continue;
                }
                frame.movement = Some(movement);
                if turn_state::record(state, turn_state::selected(state)?)?
                    .record()
                    .fleeing_duration()
                    != 0
                {
                    frame.react(state, 2, Phase::Placement)?;
                } else {
                    frame.phase = Phase::Placement;
                }
            }
            Phase::Placement => {
                let movement = frame
                    .movement
                    .take()
                    .ok_or("NPC placement locals missing")?;
                events.extend(movement.place(state, content)?);
                let (suffix, terminated) = services::collide(state, content)?;
                events.extend(suffix);
                if terminated {
                    return Ok(events);
                }
                movement.edge(state)?;
                if turn_state::record(state, turn_state::selected(state)?)?
                    .record()
                    .turn_skip_duration()
                    != 0
                {
                    frame.phase = Phase::FinalFire;
                } else {
                    frame.react(state, 0, Phase::AfterStep)?;
                }
            }
            Phase::AfterStep => {
                if turn_state::body(state)? < 0 {
                    let scope = GetupScope::capture(state);
                    let suffix = if state.combat.is_some() {
                        super::end_turn(state, content)?
                    } else {
                        super::out_of_combat_getup::finish(state, content)?
                    };
                    events.extend(suffix);
                    if state.mission.is_none() {
                        return Ok(events);
                    }
                    if pending::obstructed(state, scope) {
                        frame.phase = Phase::WaitingGetup(scope);
                        pending::suspend(state, frame)?;
                        return Ok(events);
                    }
                }
                super::death_return::refresh_actor(state, content)?;
                frame.phase = Phase::Movement;
            }
            Phase::FailedFallback => {
                let outcome = fallback(state, content, true)?;
                events.extend(outcome.events);
                if outcome.flow == super::TurnFlow::Terminated {
                    return Ok(events);
                }
                frame.phase = Phase::FinalFire;
            }
            Phase::Exhausted => {
                if turn_state::record(state, turn_state::selected(state)?)?
                    .record()
                    .fleeing_duration()
                    != 0
                {
                    frame.react(state, 0, Phase::ExhaustedFallback)?;
                } else {
                    frame.phase = Phase::ExhaustedFallback;
                }
            }
            Phase::ExhaustedFallback => {
                let outcome = fallback(state, content, false)?;
                events.extend(outcome.events);
                if outcome.flow == super::TurnFlow::Terminated {
                    return Ok(events);
                }
                frame.phase = Phase::FinalFire;
            }
            Phase::FinalFire => {
                let record = turn_state::record(state, turn_state::selected(state)?)?.record();
                if record.missile_percent()? == 0
                    || record.action_points() < 2
                    || state.globals.monster_turn_flags.no_spell() > 1
                {
                    return finish(state, content, events);
                }
                state.globals.monster_turn_flags.add_no_spell(1);
                let result = services::firearm(state, content)?;
                events.extend(result.events);
                if result.flow == super::TurnFlow::Terminated {
                    return Ok(events);
                }
                if result.outcome == super::FireOutcome::Physical {
                    frame.phase = Phase::Decision;
                } else {
                    return finish(state, content, events);
                }
            }
            Phase::WaitingGetup(_) | Phase::WaitingFinalGetup(_) => {
                return Err("NPC Getup caller resumed before its child returned".into());
            }
        }
    }
}

/// Where the action loop goes after a phase.
enum Step {
    /// Run the loop again from the frame's phase.
    Next,
    /// Return the events gathered so far.
    Return,
    /// Leave through `finish`.
    Finish,
}

/// Advances the frame's returning reaction one guard step. `None` means the caller
/// returns its events: the reaction terminated combat or suspended this frame.
fn resume_reaction(
    state: &mut Session,
    content: &Content,
    mut frame: MonsterActionFrame,
    events: &mut Vec<Event>,
) -> Result<Option<MonsterActionFrame>, RuleError> {
    if state.combat.is_none() {
        let outcome = frame
            .reaction
            .as_mut()
            .unwrap()
            .finish_after_teardown(state, content)?;
        events.extend(outcome.events);
        if outcome.terminated {
            return Ok(None);
        }
        frame.reaction = None;
        return Ok(Some(frame));
    }
    let mission = turn_state::mission(state, content)?;
    let combat = state.combat.as_mut().unwrap();
    let step = frame.reaction.as_mut().unwrap().advance(
        combat,
        &mut state.warriors,
        &mut state.random,
        &mut ReactionContext {
            items: Some(&content.items),
            attacks: Some(
                mission
                    .combat
                    .as_ref()
                    .map_or(&[][..], |tables| &tables.attacks),
            ),
            buffs: &state.party_condition,
            buffers: &mut state.globals.loaded_items,
            display_writers: true,
            scan: Some(NeighborScan {
                char_select_new: state.char_select_new,
                party_track: &mut state.globals.party_track,
                aliases: map_alias::session_map_aliases!(state, mission),
                table: Some(&state.globals.warrior_table),
            }),
        },
    )?;
    match step {
        GuardStep::Refresh(body) => super::refresh_body_item(
            combat,
            &mut state.warriors,
            &content.items,
            body,
            &mut state.globals.loaded_items,
        )?,
        GuardStep::Attack(call) => {
            return reaction_attack(state, content, mission, frame, call, events);
        }
        GuardStep::Done(_) => frame.reaction = None,
    }
    Ok(Some(frame))
}

/// The reaction's attack and its effect on stamina, then a death that pauses this
/// turn.
fn reaction_attack(
    state: &mut Session,
    content: &Content,
    mission: &centurions_content::mission::OpeningMission,
    mut frame: MonsterActionFrame,
    call: AttackCall,
    events: &mut Vec<Event>,
) -> Result<Option<MonsterActionFrame>, RuleError> {
    let previous = turn_state::party_hp(state)?;
    let outcome = super::attacks::resolve_attack_result(
        state.combat.as_mut().unwrap(),
        &mut state.warriors,
        &mut state.random,
        call.attacker,
        call.recipient,
        call.index,
        call.turn_actor,
        Some(&content.items),
        Some(
            mission
                .combat
                .as_ref()
                .map_or(&[][..], |tables| &tables.attacks),
        ),
        &state.party_condition,
        &mut state.globals.loaded_items,
        Some(state.globals.death_processing),
    )?;
    frame
        .reaction
        .as_mut()
        .unwrap()
        .finish_attack(outcome.returned)?;
    turn_state::health(state, &outcome.events, previous)?;
    events.extend(outcome.events);
    state.store_monster_turn_flags();
    if state
        .combat
        .as_ref()
        .is_some_and(|combat| combat.direct_death.is_some())
    {
        events.extend(super::death::suspend_reaction(
            state,
            content,
            call.turn_actor,
            super::CombatReturn::MonsterAction(Box::new(frame)),
        )?);
        return Ok(None);
    }
    Ok(Some(frame))
}

/// The decision: the repeat guard, then the shot or spell and the reaction before
/// movement.
///
/// Original: combat
fn decide(
    state: &mut Session,
    content: &Content,
    frame: &mut MonsterActionFrame,
    events: &mut Vec<Event>,
    decisions: &mut std::collections::HashSet<String>,
) -> Result<Step, RuleError> {
    let digest = centurions_content::digest(&serde_json::to_vec(&(&*frame, &*state))?);
    if !decisions.insert(digest) {
        return Err(turn_state::missing(
            "NPC actions repeat without change; the original loop never ends",
        ));
    }
    let selected = turn_state::selected(state)?;
    let record = turn_state::record(state, selected)?.record();
    let ap = record.action_points();
    if ap < 2 {
        return Ok(Step::Finish);
    }
    let fleeing = record.fleeing_duration() != 0;
    let firearm_chance = record.missile_percent()?;
    turn_state::record_mut(state, selected)?
        .record_mut()
        .set_reaction_ready(ap >= 1);
    if fleeing {
        frame.phase = Phase::MovementEntry;
        return Ok(Step::Next);
    }
    if state.random.bound(100) <= firearm_chance {
        let result = services::firearm(state, content)?;
        events.extend(result.events);
        if result.flow == super::TurnFlow::Terminated {
            return Ok(Step::Return);
        }
        if result.outcome == super::FireOutcome::Physical {
            return Ok(Step::Next);
        }
        state.globals.monster_turn_flags.add_no_spell(1);
        state.clear_targets();
        if result.outcome == super::FireOutcome::Spell {
            frame.phase = Phase::FinalFire;
            return Ok(Step::Next);
        }
    } else {
        let record = turn_state::record(state, turn_state::selected(state)?)?.record();
        let spell_chance = record.cast_percent()?;
        let blocked = state.enemy_spell_disabled != 0 || record.spell_blocked_before_movement();
        if state.random.bound(100) <= spell_chance {
            if blocked {
                frame.phase = Phase::MovementEntry;
                return Ok(Step::Next);
            }
            let outcome = services::spell(state, content)?;
            events.extend(outcome.events);
            if outcome.flow == super::TurnFlow::Terminated {
                return Ok(Step::Return);
            }
            state.globals.monster_turn_flags.add_no_spell(1);
            state.clear_targets();
            if outcome.cast {
                frame.phase = Phase::FinalFire;
                return Ok(Step::Next);
            }
        }
    }
    let record = turn_state::record(state, turn_state::selected(state)?)?.record();
    if record.turn_skip_duration() == 0 {
        frame.react(state, 0, Phase::MovementEntry)?;
    } else {
        frame.phase = Phase::MovementEntry;
    }
    Ok(Step::Next)
}

/// Ends the monster's turn: the action finishes, a following turn that cannot start yet is waited
/// for, and the next turn is set up.
fn finish(
    state: &mut Session,
    content: &Content,
    mut events: Vec<Event>,
) -> Result<Vec<Event>, RuleError> {
    state.store_monster_turn_flags();
    let scope = GetupScope::capture(state);
    if state.combat.is_some() {
        let actor = turn_state::selected(state)? as i16 + 10;
        events.extend(super::finish_player_action(state, content, actor)?);
    } else {
        events.extend(super::out_of_combat_getup::finish(state, content)?);
    }
    if state.mission.is_none() {
        return Ok(events);
    }
    if pending::obstructed(state, scope) {
        pending::suspend(
            state,
            MonsterActionFrame {
                phase: Phase::WaitingFinalGetup(scope),
                reaction: None,
                movement: None,
                target_cursor: 0,
                parent: None,
            },
        )?;
        return Ok(events);
    }
    super::refresh_active_item_context(state, content)?;
    events.extend(super::movement_return::finish_monster_wrapper(
        state, content,
    )?);
    Ok(events)
}
