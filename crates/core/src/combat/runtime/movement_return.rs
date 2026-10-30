//! A warrior's move, resumed after a reaction's death story.
use super::reaction_calls::{GuardStep, ReactionContext};
use super::reaction_frame::ReactionFrame;
use crate::combat::map_alias;
use crate::combat::{
    interaction, occupied_cell,
    runtime::neighbor_scan::{self, NeighborScan},
};
use crate::{Event, RuleError, Session};
use centurions_content::{Content, mission::OpeningMission};
use serde::{Deserialize, Serialize};
mod out_of_combat;
pub(super) use out_of_combat::{
    OutOfCombatFieldOrigin, out_of_combat_field_target_mask, out_of_combat_flight,
    out_of_combat_ray,
};

/// What a death story that interrupted a reaction returns to: a warrior's step, a monster's action
/// or the monster health loop of the round.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CombatReturn {
    WarriorMove(Box<MoveFrame>),
    #[serde(rename = "NpcAction")]
    MonsterAction(Box<super::MonsterActionFrame>),
    Round(Box<super::rounds::RoundReturn>),
}

/// The offer made to a warrior who walked into a body after the battle ended: the body in the cell,
/// and whether the party member may exchange places (otherwise only attack it).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutOfCombatWalkChoice {
    pub body: i16,
    pub exchange: bool,
}

/// Carries out the answer to that offer, then settles any selected-treasure movement.
pub(crate) fn answer_out_of_combat_walk(
    state: &mut Session,
    content: &Content,
    exchange: Option<bool>,
) -> Result<Vec<Event>, RuleError> {
    let mut events = out_of_combat::answer(state, content, exchange)?;
    events.extend(super::selected_rewards::after_movement(state, content)?);
    Ok(events)
}

/// A warrior's step in progress, kept while a reaction's death script runs: who moves, which way,
/// the cell planned, its cost and how far the step has got.
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

/// What the step remembers for retrying after the battle has ended: the tile of the cell it was
/// heading for, the retries made and the been-attacked flag from before the step.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct MoveLocals {
    previous_tile: i16,
    retries: i16,
    damage_snapshot: u8,
}

/// What entering the planned cell does: step onto open ground, deal with a friend there (an
/// exchange or an attack, offered or taken automatically), or act on an occupied cell.
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

/// Where the step stands among its reaction checks: before the first check, after it, after the
/// departure check, or after the step is taken and the last check has run.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
enum MovePhase {
    Entry,
    AfterEntry,
    AfterDeparture,
    AfterStep,
}

impl CombatReturn {
    /// Refuses a saved return that does not fit its caller: a step must hold a returning guard
    /// attack, a monster action must not be waiting for the next turn to be set up.
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
            Self::MonsterAction(frame) => {
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
    /// Refuses a saved step whose warrior is not in the squad of six, whose direction is not a
    /// single step, or that holds a guard attack before the step has begun.
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

    /// The warrior who moves: the selected warrior once a death script has returned, else the
    /// mover.
    fn actor(&self, state: &Session) -> Result<usize, RuleError> {
        let actor = if self.callback_returned {
            state.char_select_new
        } else {
            self.actor
        };
        if actor >= state.warriors.len() {
            return Err(RuleError::from("walking warrior outside the squad"));
        }
        Ok(actor)
    }
}

/// Starts a warrior's step in `direction`. The direction is recorded before the first reaction
/// check.
///
/// Original: combatchoice
pub(crate) fn begin(
    state: &mut Session,
    content: &Content,
    actor: usize,
    direction: (i8, i8),
    plan: MovePlan,
) -> Result<Vec<Event>, RuleError> {
    // The direction is set before the first reaction.
    store_direction(state, [direction.0, direction.1]);
    let damage_snapshot = state.warriors[actor].record().been_attacked()? as u8;
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

/// Resumes the caller a death script interrupted: the round's health loop, a monster's action or
/// the warrior's step, followed by any selected-treasure movement.
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
        CombatReturn::MonsterAction(frame) => super::monster_return::resume(state, content, *frame),
        CombatReturn::Round(_) => unreachable!("round returns resume before movement"),
    }?;
    events.extend(super::selected_rewards::after_movement(state, content)?);
    Ok(events)
}

/// Finishes a monster's action after its death scripts returned, then the battle's rewards if they
/// are waiting.
pub(super) fn finish_monster_wrapper(
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

/// Whether a monster's action waits to resume, directly or inside a chain of death stories.
pub(super) fn monster_pending(state: &Session) -> bool {
    if state.monster_action.is_some() {
        return true;
    }
    let mut story = state.combat_story.as_ref();
    while let Some(frame) = story {
        if matches!(
            frame.return_to.as_deref(),
            Some(CombatReturn::MonsterAction(_))
        ) {
            return true;
        }
        story = frame.parent.as_deref();
    }
    false
}

/// The mission data of the current land, which reaction movement reads.
fn mission<'a>(state: &Session, content: &'a Content) -> Result<&'a OpeningMission, RuleError> {
    state
        .land(content)
        .ok_or_else(|| RuleError::Invalid("reaction movement map missing".into()))
}

/// Records the direction of the warrior's latest step.
fn store_direction(state: &mut Session, direction: [i8; 2]) {
    if let Some(position) = &mut state.exploration {
        position.last_move = direction;
    }
}

/// Starts the next scan of the step: modes 0 and 2 set up a reaction check for the warrior (0
/// before the move and again after it, 2 for the departure); mode 1 rescans the neighbors at once.
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
    neighbor_scan::begin(combat);
    let _ = neighbor_scan::scan(
        combat,
        &state.warriors,
        mode,
        if mode == 2 { frame.direction } else { (0, 0) },
        &mut NeighborScan {
            char_select_new: state.char_select_new,
            party_track: &mut state.globals.party_track,
            aliases: map_alias::session_map_aliases!(state, map),
            table: Some(&state.globals.warrior_table),
        },
    )?;
    Ok(())
}

/// Runs a warrior's step from its current phase until it ends or a reaction's death script suspends
/// it. Once the battle has ended the step finishes on the party's globals.
fn run(
    state: &mut Session,
    content: &Content,
    mut frame: MoveFrame,
) -> Result<Vec<Event>, RuleError> {
    frame.validate()?;
    let mut events = vec![];
    loop {
        if state.combat.is_none() {
            return out_of_combat::resume(state, content, frame);
        }
        if frame.guard.is_some() {
            match resume_guard(state, content, frame, &mut events)? {
                Some(next) => {
                    frame = next;
                    continue;
                }
                None => return Ok(events),
            }
        }
        match frame.phase {
            MovePhase::Entry => {
                frame.phase = MovePhase::AfterEntry;
                scan(state, content, &mut frame, 0)?;
            }
            MovePhase::AfterEntry => {
                if enter(state, content, &mut frame, &mut events)? {
                    return Ok(events);
                }
            }
            MovePhase::AfterDeparture => {
                if depart(state, content, &mut frame, &mut events)? {
                    return Ok(events);
                }
            }
            MovePhase::AfterStep => {
                let actor = frame.actor(state)?;
                if frame.interrupted || state.warriors[actor].record().in_battle() == 0 {
                    events.extend(super::end_turn(state, content)?);
                }
                return Ok(events);
            }
        }
    }
}

/// Advances the frame's reaction guard one step. `None` means the caller returns its
/// events: a direct death suspended this frame.
fn resume_guard(
    state: &mut Session,
    content: &Content,
    mut frame: MoveFrame,
    events: &mut Vec<Event>,
) -> Result<Option<MoveFrame>, RuleError> {
    let guard = frame.guard.as_mut().unwrap();
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
            buffs: &state.party_condition,
            buffers: &mut state.globals.loaded_items,
            display_writers: true,
            scan: Some(NeighborScan {
                char_select_new: state.char_select_new,
                party_track: &mut state.globals.party_track,
                aliases: map_alias::session_map_aliases!(state, map),
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
            let outcome = super::attacks::resolve_attack_result(
                combat,
                &mut state.warriors,
                &mut state.random,
                call.attacker,
                call.recipient,
                call.index,
                call.turn_actor,
                Some(&content.items),
                Some(
                    map.combat
                        .as_ref()
                        .map_or(&[][..], |tables| &tables.attacks),
                ),
                &state.party_condition,
                &mut state.globals.loaded_items,
                Some(state.globals.death_processing),
            )?;
            guard.finish_attack(outcome.returned)?;
            events.extend(outcome.events);
            super::store_last_defeated(state)?;
            if state.combat.as_ref().unwrap().direct_death.is_some() {
                events.extend(super::death::suspend_reaction(
                    state,
                    content,
                    call.turn_actor,
                    CombatReturn::WarriorMove(Box::new(frame)),
                )?);
                return Ok(None);
            }
        }
        GuardStep::Done(interrupted) => {
            frame.interrupted = interrupted;
            frame.guard = None;
        }
    }
    Ok(Some(frame))
}

/// The second entry scan, then the planned cell: a friend's exchange or attack, an
/// occupied cell's reload, or the ground step's blocker and cost checks before the
/// departure scan. True when the move ends here.
fn enter(
    state: &mut Session,
    content: &Content,
    frame: &mut MoveFrame,
    events: &mut Vec<Event>,
) -> Result<bool, RuleError> {
    scan(state, content, frame, 1)?;
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
            Ok(true)
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
            Ok(true)
        }
        MovePlan::Ground { tile } => {
            state.warriors[actor]
                .record_mut()
                .face_horizontal_step(frame.direction.0);
            let map = mission(state, content)?;
            if map.movement_blocker(tile - 1000)? != 0 {
                store_direction(state, [0, 0]);
                events.extend([
                    Event::Sound { resource_id: -129 },
                    Event::Sound { resource_id: -130 },
                ]);
                return Ok(true);
            }
            frame.cost = crate::rules::movement_cost(
                map.tactical_movement_property(tile)?,
                frame.direction.0,
                frame.direction.1,
            );
            if state.warriors[actor]
                .record()
                .movement()?
                .wrapping_sub(frame.cost)
                < 0
            {
                store_direction(state, [0, 0]);
                events.push(Event::Sound { resource_id: -130 });
                return Ok(true);
            }
            frame.phase = MovePhase::AfterDeparture;
            scan(state, content, frame, 2)?;
            Ok(false)
        }
    }
}

/// The ground step after the departure scan. The turn ends instead if the departure check
/// interrupted it or the warrior is out of the battle. Otherwise it spends the cost, moves the
/// warrior's map and squad cells, plays the landing sound, resolves field collisions and starts the
/// scan after the step. True when the move ends here.
fn depart(
    state: &mut Session,
    content: &Content,
    frame: &mut MoveFrame,
    events: &mut Vec<Event>,
) -> Result<bool, RuleError> {
    let actor = frame.actor(state)?;
    if frame.interrupted || state.warriors[actor].record().in_battle() == 0 {
        events.extend(super::end_turn(state, content)?);
        return Ok(true);
    }
    let MovePlan::Ground { tile } = frame.plan else {
        return Err("movement departure plan missing".into());
    };
    let remaining = state.warriors[actor]
        .record()
        .movement()?
        .wrapping_sub(frame.cost);
    state.warriors[actor].record_mut().set_movement(remaining);
    let combat = state.combat.as_mut().unwrap();
    let position = combat.squad.pos[actor];
    let destination = (
        position.0.wrapping_add(i16::from(frame.direction.0)),
        position.1.wrapping_add(i16::from(frame.direction.1)),
    );
    let origin = (combat.field.map_shift.0 + 5, combat.field.map_shift.1 + 5);
    let current = OpeningMission::index(
        i32::from(position.0 + origin.0),
        i32::from(position.1 + origin.1),
    )?;
    let next = OpeningMission::index(
        i32::from(destination.0 + origin.0),
        i32::from(destination.1 + origin.1),
    )?;
    combat.field.tiles[current] = combat.squad.char_under[actor];
    combat.squad.char_under[actor] = combat.field.tiles[next];
    combat.field.tiles[next] = actor as i16;
    combat.squad.pos[actor] = destination;
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
    events.extend(super::collide_fields_with_refresh(
        state.combat.as_mut().unwrap(),
        &mut state.warriors,
        map,
        &mut state.globals.attack_count,
        &mut state.random,
        (
            actor as i16,
            crate::combat::spells::loaded_item_restriction(&state.globals.loaded_items)?,
            state.char_select_new,
            &mut state.bandage_ready,
        ),
        &mut state.globals.party_track,
        Some((&content.items, &mut state.globals.loaded_items)),
    )?);
    if state.warriors[actor].stamina()? < 1 {
        state.warriors[actor].record_mut().set_action_points(0);
        state.warriors[actor].record_mut().set_movement(0);
        return Ok(true);
    }
    frame.phase = MovePhase::AfterStep;
    scan(state, content, frame, 0)?;
    Ok(false)
}
