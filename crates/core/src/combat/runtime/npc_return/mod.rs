//! NPC combat-loop locals across immediate checkforenemy/newland returns.
use super::reaction_calls::{GuardStep, ReactionContext};
use super::reaction_frame::ReactionFrame;
use crate::combat::neighbor_work::{self, NativeNeighbors};
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
mod work;
use movement::NpcMoveFrame;
use pending::GetupScope;
pub(super) use pending::owns_return as owns_pending_return;
pub(crate) use pending::resume as resume_pending;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NpcActionFrame {
    phase: Phase,
    reaction: Option<ReactionFrame>,
    movement: Option<NpcMoveFrame>,
    target_cursor: i16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parent: Option<Box<NpcActionFrame>>,
}

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

impl NpcActionFrame {
    pub(crate) fn waiting_getup(&self) -> bool {
        matches!(
            self.phase,
            Phase::WaitingGetup(_) | Phase::WaitingFinalGetup(_)
        )
    }

    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        pending::validate(self)
    }

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

    fn react(&mut self, state: &Session, mode: usize, next: Phase) -> Result<(), RuleError> {
        self.phase = next;
        self.reaction = Some(ReactionFrame::new(
            work::body(state)?,
            mode,
            work::direction(state)?,
        )?);
        Ok(())
    }
}

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
        NpcActionFrame {
            phase: Phase::Decision,
            reaction: None,
            movement: None,
            target_cursor: 0,
            parent: None,
        },
    )
}

pub(crate) fn resume(
    state: &mut Session,
    content: &Content,
    frame: NpcActionFrame,
) -> Result<Vec<Event>, RuleError> {
    frame.validate()?;
    if frame.waiting_getup() {
        return Err("NPC reaction return owns a Getup frame".into());
    }
    run(state, content, frame)
}

fn moving(state: &Session) -> Result<bool, RuleError> {
    if let Some(combat) = &state.combat {
        Ok(combat.victory_work.movement_admitted())
    } else {
        Ok(state.native_work.victory_work.movement_admitted())
    }
}

fn scan_entry(state: &mut Session, content: &Content) -> Result<(), RuleError> {
    let mission = work::mission(state, content)?;
    if let Some(combat) = &mut state.combat {
        neighbor_work::begin(combat);
        neighbor_work::scan(
            combat,
            &state.warriors,
            1,
            (0, 0),
            &mut NativeNeighbors {
                selected_warrior: state.selected_warrior,
                party_mask: &mut state.native_work.party_mask,
                aliases: crate::combat::map_alias::session_map_aliases!(state, mission),
                table: Some(&state.native_work.warrior_table),
            },
        )?;
    } else {
        state.native_work.neighbor_work.departure = 0;
        neighbor_work::scan_retained(state, content, 1, &mut 0)?;
    }
    Ok(())
}

fn fallback(
    state: &mut Session,
    content: &Content,
    failed: bool,
) -> Result<super::TurnOutcome, RuleError> {
    state.retain_npc_work();
    let selected = work::selected(state)?;
    if state.native_work.npc_work.attempts() > 1
        || work::record(state, selected)?.record().spell_chance()? == 0
        || state.native_work.npc_work.inhibited()
    {
        return Ok(super::TurnOutcome::default());
    }
    if failed {
        state.native_work.npc_work.add_attempts(2);
    } else {
        state.native_work.npc_work.attempts = 2;
    }
    work::record_mut(state, selected)?
        .record_mut()
        .set_movement_points(0);
    let record = work::record(state, selected)?.record();
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
    state.native_work.npc_work.add_attempts(1);
    // combat 115110 calls cleartarget after the attempt.
    state.clear_targets();
    Ok(super::TurnOutcome {
        events: outcome.events,
        flow: outcome.flow,
    })
}

fn run(
    state: &mut Session,
    content: &Content,
    mut frame: NpcActionFrame,
) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    // Every action cycle passes Decision. A repeated Decision state means the
    // original never leaves the loop; NPC choices draw RNG and keep it moving.
    let mut decisions = std::collections::HashSet::new();
    loop {
        if state.mission.is_none() {
            return Ok(events);
        }
        if frame.reaction.is_some() {
            if state.combat.is_none() {
                let outcome = frame
                    .reaction
                    .as_mut()
                    .unwrap()
                    .finish_after_teardown(state, content)?;
                events.extend(outcome.events);
                if outcome.terminated {
                    return Ok(events);
                }
                frame.reaction = None;
                continue;
            }
            let mission = work::mission(state, content)?;
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
                    buffs: &state.party_buffs,
                    buffers: &mut state.native_work.ammunition_work,
                    display_writers: true,
                    native: Some(NativeNeighbors {
                        selected_warrior: state.selected_warrior,
                        party_mask: &mut state.native_work.party_mask,
                        aliases: crate::combat::map_alias::session_map_aliases!(state, mission),
                        table: Some(&state.native_work.warrior_table),
                    }),
                },
            )?;
            match step {
                GuardStep::Refresh(body) => super::refresh_body_item(
                    combat,
                    &mut state.warriors,
                    &content.items,
                    body,
                    &mut state.native_work.ammunition_work,
                )?,
                GuardStep::Attack(call) => {
                    let previous = work::party_hp(state)?;
                    let outcome = super::attacks::resolve_attack_result(
                        state.combat.as_mut().unwrap(),
                        &mut state.warriors,
                        &mut state.random,
                        call.attacker,
                        call.recipient,
                        call.index,
                        call.retained_actor,
                        Some(&content.items),
                        Some(
                            mission
                                .combat
                                .as_ref()
                                .map_or(&[][..], |tables| &tables.attacks),
                        ),
                        &state.party_buffs,
                        &mut state.native_work.ammunition_work,
                        Some(state.native_work.death_processing),
                    )?;
                    frame
                        .reaction
                        .as_mut()
                        .unwrap()
                        .finish_attack(outcome.returned)?;
                    work::health(state, &outcome.events, previous)?;
                    events.extend(outcome.events);
                    state.retain_npc_work();
                    if state
                        .combat
                        .as_ref()
                        .is_some_and(|combat| combat.direct_death.is_some())
                    {
                        events.extend(super::death::suspend_reaction(
                            state,
                            content,
                            call.retained_actor,
                            super::CombatReturn::NpcAction(Box::new(frame)),
                        )?);
                        return Ok(events);
                    }
                }
                GuardStep::Done(_) => frame.reaction = None,
            }
            continue;
        }
        state.retain_npc_work();
        match frame.phase {
            Phase::Decision => {
                let digest = centurions_content::digest(&serde_json::to_vec(&(&frame, &*state))?);
                if !decisions.insert(digest) {
                    return Err(work::missing(
                        "NPC actions repeat without change; the original loop never ends",
                    ));
                }
                let selected = work::selected(state)?;
                let record = work::record(state, selected)?.record();
                let ap = record.action_points();
                if ap < 2 {
                    return finish(state, content, events);
                }
                let fleeing = record.fleeing_duration() != 0;
                let firearm_chance = record.firearm_chance()?;
                work::record_mut(state, selected)?
                    .record_mut()
                    .set_reaction_ready(ap >= 1);
                if fleeing {
                    frame.phase = Phase::MovementEntry;
                    continue;
                }
                if state.random.bound(100) <= firearm_chance {
                    let result = services::firearm(state, content)?;
                    events.extend(result.events);
                    if result.flow == super::TurnFlow::Terminated {
                        return Ok(events);
                    }
                    if result.outcome == super::FireOutcome::Physical {
                        continue;
                    }
                    state.native_work.npc_work.add_attempts(1);
                    // combat 115110 calls cleartarget after the attempt.
                    state.clear_targets();
                    if result.outcome == super::FireOutcome::Spell {
                        frame.phase = Phase::FinalFire;
                        continue;
                    }
                } else {
                    let record = work::record(state, work::selected(state)?)?.record();
                    let spell_chance = record.spell_chance()?;
                    let blocked =
                        state.enemy_spell_disabled != 0 || record.spell_blocked_before_movement();
                    if state.random.bound(100) <= spell_chance {
                        if blocked {
                            frame.phase = Phase::MovementEntry;
                            continue;
                        }
                        let outcome = services::spell(state, content)?;
                        events.extend(outcome.events);
                        if outcome.flow == super::TurnFlow::Terminated {
                            return Ok(events);
                        }
                        state.native_work.npc_work.add_attempts(1);
                        // combat 115110 calls cleartarget after the attempt.
                        state.clear_targets();
                        if outcome.cast {
                            frame.phase = Phase::FinalFire;
                            continue;
                        }
                    }
                }
                let record = work::record(state, work::selected(state)?)?.record();
                if record.turn_skip_duration() == 0 {
                    frame.react(state, 0, Phase::MovementEntry)?;
                } else {
                    frame.phase = Phase::MovementEntry;
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
                let record = work::record(state, work::selected(state)?)?.record();
                if record.movement_points() <= 0
                    || record.action_points() <= 1
                    || record.turn_skip_duration() != 0
                {
                    frame.phase = Phase::Exhausted;
                    continue;
                }
                target::select(state, content, &mut frame.target_cursor)?;
                if work::record(state, work::selected(state)?)?
                    .record()
                    .fleeing_duration()
                    != 0
                {
                    scan_entry(state, content)?;
                }
                let mut movement = NpcMoveFrame::begin(state)?;
                if !movement.prepare(state, content)? {
                    frame.react(state, 0, Phase::FailedFallback)?;
                    continue;
                }
                frame.movement = Some(movement);
                if work::record(state, work::selected(state)?)?
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
                if work::record(state, work::selected(state)?)?
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
                if work::body(state)? < 0 {
                    let scope = GetupScope::capture(state);
                    let suffix = if state.combat.is_some() {
                        super::end_turn(state, content)?
                    } else {
                        super::retained_turn::finish(state, content)?
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
                if work::record(state, work::selected(state)?)?
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
                let record = work::record(state, work::selected(state)?)?.record();
                if record.firearm_chance()? == 0
                    || record.action_points() < 2
                    || state.native_work.npc_work.attempts() > 1
                {
                    return finish(state, content, events);
                }
                state.native_work.npc_work.add_attempts(1);
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

fn finish(
    state: &mut Session,
    content: &Content,
    mut events: Vec<Event>,
) -> Result<Vec<Event>, RuleError> {
    state.retain_npc_work();
    let scope = GetupScope::capture(state);
    if state.combat.is_some() {
        let actor = work::selected(state)? as i16 + 10;
        events.extend(super::finish_player_action(state, content, actor)?);
    } else {
        events.extend(super::retained_turn::finish(state, content)?);
    }
    if state.mission.is_none() {
        return Ok(events);
    }
    if pending::obstructed(state, scope) {
        pending::suspend(
            state,
            NpcActionFrame {
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
    events.extend(super::movement_return::finish_npc_wrapper(state, content)?);
    Ok(events)
}
