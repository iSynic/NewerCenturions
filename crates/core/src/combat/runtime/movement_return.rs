//! Warrior movement locals while checkforenemy returns through killbody/newland.
use super::reaction_calls::{GuardStep, ReactionContext};
use super::reaction_frame::ReactionFrame;
use crate::combat::{
    interaction,
    neighbor_work::{self, NativeNeighbors},
    occupied_cell,
};
use crate::{Event, RuleError, Session};
use centurions_content::{Content, mission::OpeningMission};
use serde::{Deserialize, Serialize};
mod detached;
pub(super) use detached::{
    RetainedFieldOrigin, retained_field_target_mask, retained_flight, retained_ray,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CombatReturn {
    WarriorMove(Box<MoveFrame>),
    NpcAction(Box<super::NpcActionFrame>),
    Round(Box<super::rounds::RoundReturn>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetainedWalkChoice {
    pub body: i16,
    pub exchange: bool,
}

pub(crate) fn answer_retained_walk(
    state: &mut Session,
    content: &Content,
    exchange: Option<bool>,
) -> Result<Vec<Event>, RuleError> {
    let mut events = detached::answer(state, content, exchange)?;
    events.extend(super::selected_rewards::after_movement(state, content)?);
    Ok(events)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MoveFrame {
    actor: usize,
    direction: (i8, i8),
    plan: MovePlan,
    phase: MovePhase,
    cost: i16,
    guard: Option<ReactionFrame>,
    interrupted: bool,
    callback_returned: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    locals: Option<MoveLocals>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct MoveLocals {
    previous_tile: i16,
    retries: i16,
    damage_snapshot: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum MovePlan {
    Ground {
        tile: i16,
    },
    Friend {
        tile: i16,
        exchange: bool,
        automatic: bool,
    },
    Occupied {
        tile: i16,
    },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
enum MovePhase {
    Entry,
    AfterEntry,
    AfterDeparture,
    AfterStep,
}

impl CombatReturn {
    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        match self {
            Self::WarriorMove(frame) => {
                frame.validate()?;
                if !frame
                    .guard
                    .as_ref()
                    .is_some_and(ReactionFrame::returned_attack)
                {
                    return Err("saved movement has no returning guard attack".into());
                }
                Ok(())
            }
            Self::NpcAction(frame) => {
                frame.validate()?;
                if frame.waiting_getup() {
                    return Err("saved NPC reaction has a Getup return owner".into());
                }
                Ok(())
            }
            Self::Round(frame) => frame.validate(),
        }
    }
}

impl MoveFrame {
    fn validate(&self) -> Result<(), RuleError> {
        if self.actor >= 6
            || !(-1..=1).contains(&self.direction.0)
            || !(-1..=1).contains(&self.direction.1)
            || self.direction == (0, 0)
            || self.phase == MovePhase::Entry && self.guard.is_some()
        {
            return Err("invalid saved warrior movement caller".into());
        }
        if let Some(guard) = &self.guard {
            guard.validate()?;
        }
        Ok(())
    }

    fn actor(&self, state: &Session) -> Result<usize, RuleError> {
        let actor = if self.callback_returned {
            state.selected_warrior
        } else {
            self.actor
        };
        if actor >= state.warriors.len() {
            return Err(RuleError::from("walking warrior outside the squad"));
        }
        Ok(actor)
    }
}

pub(crate) fn begin(
    state: &mut Session,
    content: &Content,
    actor: usize,
    direction: (i8, i8),
    plan: MovePlan,
) -> Result<Vec<Event>, RuleError> {
    // combatchoice writes the shared signed direction before its first reaction.
    retain_direction(state, [direction.0, direction.1]);
    let damage_snapshot = state.warriors[actor].record().damage_marker()? as u8;
    run(
        state,
        content,
        MoveFrame {
            actor,
            direction,
            plan,
            phase: MovePhase::Entry,
            cost: 0,
            guard: None,
            interrupted: false,
            callback_returned: false,
            locals: Some(MoveLocals {
                previous_tile: -1,
                retries: -1,
                damage_snapshot,
            }),
        },
    )
}

pub(crate) fn resume(
    state: &mut Session,
    content: &Content,
    caller: CombatReturn,
) -> Result<Vec<Event>, RuleError> {
    if let CombatReturn::Round(frame) = caller {
        return super::rounds::resume_round(state, content, *frame);
    }
    super::selected_rewards::begin_movement(state);
    let mut events = match caller {
        CombatReturn::WarriorMove(mut frame) => {
            frame.callback_returned = true;
            run(state, content, *frame)
        }
        CombatReturn::NpcAction(frame) => super::npc_return::resume(state, content, *frame),
        CombatReturn::Round(_) => unreachable!("round returns resume before movement"),
    }?;
    events.extend(super::selected_rewards::after_movement(state, content)?);
    Ok(events)
}

pub(super) fn finish_npc_wrapper(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    let mut events = super::selected_rewards::after_getup(state, content)?;
    events.extend(super::selected_rewards::after_movement(state, content)?);
    if state.combat.is_none()
        && state.ordinary_battle_rewards.is_some()
        && !super::selected_rewards::movement_pending(state)
        && state
            .exploration
            .as_ref()
            .is_some_and(|p| p.presentation.is_none())
    {
        events.extend(super::finish_battle_wrapper(state, content)?);
    }
    Ok(events)
}

pub(super) fn npc_pending(state: &Session) -> bool {
    if state.npc_action.is_some() {
        return true;
    }
    let mut story = state.combat_story.as_ref();
    while let Some(frame) = story {
        if matches!(frame.return_to.as_deref(), Some(CombatReturn::NpcAction(_))) {
            return true;
        }
        story = frame.parent.as_deref();
    }
    false
}

fn mission<'a>(state: &Session, content: &'a Content) -> Result<&'a OpeningMission, RuleError> {
    state
        .land(content)
        .ok_or_else(|| RuleError::Invalid("reaction movement map missing".into()))
}

fn retain_direction(state: &mut Session, direction: [i8; 2]) {
    if let Some(position) = &mut state.exploration {
        position.last_move = direction;
    }
}

fn scan(
    state: &mut Session,
    content: &Content,
    frame: &mut MoveFrame,
    mode: usize,
) -> Result<(), RuleError> {
    let map = mission(state, content)?;
    let combat = state
        .combat
        .as_mut()
        .ok_or_else(|| RuleError::from("reaction scan without a live combat"))?;
    if mode != 1 {
        frame.guard = Some(ReactionFrame::new(
            frame.actor as i16,
            mode,
            frame.direction,
        )?);
        return Ok(());
    }
    neighbor_work::begin(combat);
    let _ = neighbor_work::scan(
        combat,
        &state.warriors,
        mode,
        if mode == 2 { frame.direction } else { (0, 0) },
        &mut NativeNeighbors {
            selected_warrior: state.selected_warrior,
            party_mask: &mut state.native_work.party_mask,
            aliases: crate::combat::map_alias::session_map_aliases!(state, map),
            table: Some(&state.native_work.warrior_table),
        },
    )?;
    Ok(())
}

fn run(
    state: &mut Session,
    content: &Content,
    mut frame: MoveFrame,
) -> Result<Vec<Event>, RuleError> {
    frame.validate()?;
    let mut events = vec![];
    loop {
        if state.combat.is_none() {
            return detached::resume(state, content, frame);
        }
        if let Some(guard) = &mut frame.guard {
            let map = mission(state, content)?;
            let combat = state.combat.as_mut().unwrap();
            let step = guard.advance(
                combat,
                &mut state.warriors,
                &mut state.random,
                &mut ReactionContext {
                    items: Some(&content.items),
                    attacks: Some(
                        map.combat
                            .as_ref()
                            .map_or(&[][..], |tables| &tables.attacks),
                    ),
                    buffs: &state.party_buffs,
                    buffers: &mut state.native_work.ammunition_work,
                    display_writers: true,
                    native: Some(NativeNeighbors {
                        selected_warrior: state.selected_warrior,
                        party_mask: &mut state.native_work.party_mask,
                        aliases: crate::combat::map_alias::session_map_aliases!(state, map),
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
                    let outcome = super::attacks::resolve_attack_result(
                        combat,
                        &mut state.warriors,
                        &mut state.random,
                        call.attacker,
                        call.recipient,
                        call.index,
                        call.retained_actor,
                        Some(&content.items),
                        Some(
                            map.combat
                                .as_ref()
                                .map_or(&[][..], |tables| &tables.attacks),
                        ),
                        &state.party_buffs,
                        &mut state.native_work.ammunition_work,
                        Some(state.native_work.death_processing),
                    )?;
                    guard.finish_attack(outcome.returned)?;
                    events.extend(outcome.events);
                    super::retain_last_defeated(state)?;
                    if state.combat.as_ref().unwrap().direct_death.is_some() {
                        events.extend(super::death::suspend_reaction(
                            state,
                            content,
                            call.retained_actor,
                            CombatReturn::WarriorMove(Box::new(frame)),
                        )?);
                        return Ok(events);
                    }
                }
                GuardStep::Done(interrupted) => {
                    frame.interrupted = interrupted;
                    frame.guard = None;
                }
            }
            continue;
        }
        match frame.phase {
            MovePhase::Entry => {
                frame.phase = MovePhase::AfterEntry;
                scan(state, content, &mut frame, 0)?;
            }
            MovePhase::AfterEntry => {
                scan(state, content, &mut frame, 1)?;
                let actor = frame.actor(state)?;
                if let Some(locals) = frame.locals.as_mut() {
                    locals.previous_tile = match frame.plan {
                        MovePlan::Ground { tile }
                        | MovePlan::Friend { tile, .. }
                        | MovePlan::Occupied { tile } => tile,
                    };
                }
                match frame.plan {
                    MovePlan::Friend {
                        tile,
                        exchange,
                        automatic,
                    } => {
                        state.warriors[actor]
                            .record_mut()
                            .face_horizontal_step(frame.direction.0);
                        if automatic {
                            events.extend(if exchange {
                                interaction::exchange(state, tile)?
                            } else {
                                super::player_attack_friend(state, content, tile)?
                            });
                        } else {
                            events.extend([
                                Event::Sound { resource_id: 20005 },
                                if exchange {
                                    Event::ExchangeOffered { body: tile }
                                } else {
                                    Event::FriendAttackOffered { body: tile }
                                },
                            ]);
                        }
                        return Ok(events);
                    }
                    MovePlan::Occupied { tile } => {
                        events.extend(occupied_cell::resume_reload(
                            state,
                            content,
                            mission(state, content)?,
                            actor,
                            tile,
                            frame.direction.0,
                        )?);
                        return Ok(events);
                    }
                    MovePlan::Ground { tile } => {
                        state.warriors[actor]
                            .record_mut()
                            .face_horizontal_step(frame.direction.0);
                        let map = mission(state, content)?;
                        if map.movement_blocker(tile - 1000)? != 0 {
                            retain_direction(state, [0, 0]);
                            events.extend([
                                Event::Sound { resource_id: -129 },
                                Event::Sound { resource_id: -130 },
                            ]);
                            return Ok(events);
                        }
                        frame.cost = crate::rules::movement_cost(
                            map.tactical_movement_property(tile)?,
                            frame.direction.0,
                            frame.direction.1,
                        );
                        if state.warriors[actor]
                            .record()
                            .movement_points()?
                            .wrapping_sub(frame.cost)
                            < 0
                        {
                            retain_direction(state, [0, 0]);
                            events.push(Event::Sound { resource_id: -130 });
                            return Ok(events);
                        }
                        frame.phase = MovePhase::AfterDeparture;
                        scan(state, content, &mut frame, 2)?;
                    }
                }
            }
            MovePhase::AfterDeparture => {
                let actor = frame.actor(state)?;
                if frame.interrupted || state.warriors[actor].record().active_flag() == 0 {
                    events.extend(super::end_turn(state, content)?);
                    return Ok(events);
                }
                let MovePlan::Ground { tile } = frame.plan else {
                    return Err("movement departure plan missing".into());
                };
                let remaining = state.warriors[actor]
                    .record()
                    .movement_points()?
                    .wrapping_sub(frame.cost);
                state.warriors[actor]
                    .record_mut()
                    .set_movement_points(remaining);
                let combat = state.combat.as_mut().unwrap();
                let position = combat.squad.positions[actor];
                let destination = (
                    position.0.wrapping_add(i16::from(frame.direction.0)),
                    position.1.wrapping_add(i16::from(frame.direction.1)),
                );
                let origin = (combat.map.shift.0 + 5, combat.map.shift.1 + 5);
                let current = OpeningMission::index(
                    i32::from(position.0 + origin.0),
                    i32::from(position.1 + origin.1),
                )?;
                let next = OpeningMission::index(
                    i32::from(destination.0 + origin.0),
                    i32::from(destination.1 + origin.1),
                )?;
                combat.map.tiles[current] = combat.squad.under_tiles[actor];
                combat.squad.under_tiles[actor] = combat.map.tiles[next];
                combat.map.tiles[next] = actor as i16;
                combat.squad.positions[actor] = destination;
                events.extend([
                    Event::TacticalMoved {
                        body: actor as i16,
                        x: destination.0,
                        y: destination.1,
                    },
                    Event::Sound {
                        resource_id: mission(state, content)?.movement_sound(tile - 1000)?,
                    },
                ]);
                let map = mission(state, content)?;
                events.extend(super::collide_fields_with_work(
                    state.combat.as_mut().unwrap(),
                    &mut state.warriors,
                    map,
                    &mut state.native_work.attack_count,
                    &mut state.random,
                    (
                        actor as i16,
                        crate::combat::spells::retained_item_alias(
                            &state.native_work.ammunition_work,
                        )?,
                        state.selected_warrior,
                        &mut state.bandage_ready,
                    ),
                    &mut state.native_work.party_mask,
                    Some((&content.items, &mut state.native_work.ammunition_work)),
                )?);
                if state.warriors[actor].stamina()? < 1 {
                    state.warriors[actor].record_mut().set_action_points(0);
                    state.warriors[actor].record_mut().set_movement_points(0);
                    return Ok(events);
                }
                frame.phase = MovePhase::AfterStep;
                scan(state, content, &mut frame, 0)?;
            }
            MovePhase::AfterStep => {
                let actor = frame.actor(state)?;
                if frame.interrupted || state.warriors[actor].record().active_flag() == 0 {
                    events.extend(super::end_turn(state, content)?);
                }
                return Ok(events);
            }
        }
    }
}
