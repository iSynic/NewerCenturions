//! The local guard-loop result and shared-list reads in checkforenemy110e96.
use super::{attacks, refresh_body_item};
use crate::combat::{
    Combat,
    neighbor_work::{self, NativeNeighbors},
};
use crate::{Event, RandomState, RuleError};
use centurions_content::Warrior;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub(crate) struct ReactionResult {
    pub events: Vec<Event>,
    pub interrupted: bool,
    pub terminated: bool,
}

pub(super) enum ReactionReturn {
    Returned(bool),
    Terminated,
}

pub(super) struct ReactionContext<'a> {
    pub items: Option<&'a [Vec<u8>]>,
    pub attacks: Option<&'a [u8]>,
    pub buffs: &'a [u8],
    pub buffers: &'a mut crate::ammunition::WorkBuffers,
    pub display_writers: bool,
    pub native: Option<NativeNeighbors<'a>>,
}

impl ReactionContext<'_> {
    pub fn actor(&self, combat: &Combat, fallback: i16) -> i16 {
        if self.native.is_some() {
            combat
                .order
                .get(combat.cursor)
                .copied()
                .unwrap_or(combat.cursor as i16)
        } else {
            fallback
        }
    }

    pub fn body(&self, combat: &Combat, list: &[i16], index: usize) -> Result<i16, RuleError> {
        if let Some(native) = &self.native {
            combat.neighbor_work.body(index, native.party_mask)
        } else {
            Ok(list[index])
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct AttackCall {
    pub attacker: i16,
    pub recipient: i16,
    pub index: usize,
    pub retained_actor: i16,
}

pub(super) fn attack(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    call: AttackCall,
    context: &mut ReactionContext<'_>,
) -> Result<attacks::AttackOutcome, RuleError> {
    attacks::resolve_attack_result(
        combat,
        warriors,
        random,
        call.attacker,
        call.recipient,
        call.index,
        context.actor(combat, call.retained_actor),
        context.items,
        context.attacks,
        context.buffs,
        context.buffers,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn guards(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    actor: i16,
    departure: bool,
    list: &[i16],
    changed: &mut bool,
    context: &mut ReactionContext<'_>,
) -> Result<ReactionResult, RuleError> {
    guards_with(
        combat,
        warriors,
        random,
        actor,
        departure,
        list,
        changed,
        context,
        &mut attack,
    )
}

#[allow(clippy::too_many_arguments)]
fn guards_with(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    actor: i16,
    departure: bool,
    list: &[i16],
    changed: &mut bool,
    context: &mut ReactionContext<'_>,
    strike: &mut impl FnMut(
        &mut Combat,
        &mut [Warrior],
        &mut RandomState,
        AttackCall,
        &mut ReactionContext<'_>,
    ) -> Result<attacks::AttackOutcome, RuleError>,
) -> Result<ReactionResult, RuleError> {
    let mut result = ReactionResult::default();
    let mut frame = GuardFrame::new(actor, departure, list.len(), *changed)?;
    loop {
        match frame.advance(combat, warriors, list, context)? {
            GuardStep::Refresh(body) => refresh_body_item(
                combat,
                warriors,
                context.items.unwrap(),
                body,
                context.buffers,
            )?,
            GuardStep::Attack(call) => {
                let outcome = strike(combat, warriors, random, call, context)?;
                result.events.extend(outcome.events);
                frame.finish_attack(outcome.returned)?;
            }
            GuardStep::Done(interrupted) => {
                *changed = frame.changed;
                result.interrupted = interrupted;
                return Ok(result);
            }
        }
    }
}

/// checkforenemy's caller locals; the neighbor list and actor globals stay shared.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuardFrame {
    actor: i16,
    departure: bool,
    count: i8,
    index: i8,
    changed: bool,
    interrupted: bool,
    phase: GuardPhase,
    result: Option<bool>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
enum GuardPhase {
    Select,
    Arm,
    AwaitAttack,
    Done,
}

pub(super) enum GuardStep {
    Refresh(i16),
    Attack(AttackCall),
    Done(bool),
}

impl GuardFrame {
    pub(super) fn new(
        actor: i16,
        departure: bool,
        count: usize,
        changed: bool,
    ) -> Result<Self, RuleError> {
        let count = i8::try_from(count)
            .ok()
            .filter(|count| *count <= 110)
            .ok_or("reaction count outside native caller range")?;
        Ok(Self {
            actor,
            departure,
            count,
            index: 0,
            changed,
            interrupted: false,
            phase: GuardPhase::Select,
            result: None,
        })
    }

    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        if !(0..=110).contains(&self.count)
            || !(0..=110).contains(&self.index)
            || !(-1..110).contains(&self.actor)
            || self.result.is_some() && self.phase != GuardPhase::AwaitAttack
            || self.phase == GuardPhase::AwaitAttack && self.index >= self.count
        {
            return Err("invalid saved guard caller".into());
        }
        Ok(())
    }

    pub(super) fn count(&self) -> i8 {
        self.count
    }

    pub(super) fn changed(&self) -> bool {
        self.changed
    }

    pub(crate) fn returned_attack(&self) -> bool {
        self.phase == GuardPhase::AwaitAttack && self.result.is_some()
    }

    pub(super) fn finish_attack(&mut self, returned: bool) -> Result<(), RuleError> {
        if self.phase != GuardPhase::AwaitAttack || self.result.is_some() {
            return Err("guard attack return has no waiting caller".into());
        }
        self.result = Some(returned);
        Ok(())
    }

    fn consume_return(&mut self) -> Result<(), RuleError> {
        let returned = self
            .result
            .take()
            .ok_or_else(|| RuleError::from("guard attack return missing"))?;
        self.index = self.index.wrapping_add(1);
        if returned {
            self.changed = false;
            self.count = 0;
            self.interrupted = true;
            self.phase = GuardPhase::Done;
        } else {
            self.phase = GuardPhase::Select;
        }
        Ok(())
    }

    pub(super) fn finish_retained(
        &mut self,
        state: &mut crate::Session,
        content: &centurions_content::Content,
        events: &mut Vec<Event>,
    ) -> Result<ReactionReturn, RuleError> {
        self.validate()?;
        loop {
            match self.phase {
                GuardPhase::AwaitAttack => self.consume_return()?,
                GuardPhase::Done => return Ok(ReactionReturn::Returned(self.interrupted)),
                GuardPhase::Select => {
                    if self.index >= self.count {
                        self.phase = GuardPhase::Done;
                        continue;
                    }
                    let other = state
                        .native_work
                        .neighbor_work
                        .body(self.index as usize, &state.native_work.party_mask)?;
                    let actor = retained_actor(state)?;
                    // Bodies 0..8 index the warrior table, the Upgrade records and the
                    // editor backup 0x44ecb0.
                    let eligible = if let Some(warrior) = usize::try_from(other)
                        .ok()
                        .filter(|_| other < 9)
                        .and_then(|other| state.table_record(other))
                    {
                        (self.departure || warrior.record().guarding()? != 0)
                            && warrior.record().turn_skip_duration()? == 0
                    } else if (10..110).contains(&other) {
                        let record = state
                            .native_work
                            .npc_backing
                            .slot((other - 10) as usize)?
                            .record
                            .record();
                        (self.departure || record.guarding() != 0)
                            && record.turn_skip_duration() == 0
                            && !record.removed()
                            && (actor > 6
                                || !matches!(
                                    state.warriors[state.selected_warrior]
                                        .record()
                                        .profession()?,
                                    4 | 5
                                ))
                    } else if other == 9 {
                        return Err(retained_unavailable(
                            "retained guard body 9 before the NPC array",
                        ));
                    } else if other >= 110 {
                        return Err(retained_unavailable(
                            "retained guard body past the NPC array",
                        ));
                    } else {
                        return Err(retained_unavailable(
                            "retained guard body before the warrior table",
                        ));
                    };
                    if eligible {
                        self.phase = GuardPhase::Arm;
                    } else {
                        self.index = self.index.wrapping_add(1);
                    }
                }
                GuardPhase::Arm => {
                    let other = state
                        .native_work
                        .neighbor_work
                        .body(self.index as usize, &state.native_work.party_mask)?;
                    if other < 10 {
                        super::detached_warrior::refresh_guard(state, content, other)?;
                        // checkforenemy clears guarding WORD +0x16 in whichever record.
                        let mut warrior = state
                            .table_record(other as usize)
                            .ok_or("retained guard outside the warrior table")?;
                        warrior.record_mut().set_guarding(0);
                        state.store_table_record(other as usize, warrior)?;
                    } else {
                        let slot = (other - 10) as usize;
                        let item = state
                            .native_work
                            .npc_backing
                            .slot(slot)?
                            .record
                            .record()
                            .selected_item()?;
                        if item != 0 {
                            state.load_item_work(content, item)?;
                        }
                        state
                            .native_work
                            .npc_backing
                            .slot_mut(slot)?
                            .record
                            .record_mut()
                            .set_reaction_ready(false);
                        state.native_work.weapon_work.natural_attack_index = 0;
                    }
                    self.changed = true;
                    let run = if other < 10 {
                        super::detached_warrior::run
                    } else {
                        super::detached_attack::run
                    };
                    let outcome = run(
                        state,
                        content,
                        AttackCall {
                            attacker: other,
                            recipient: retained_actor(state)?,
                            index: 0,
                            retained_actor: retained_actor(state)?,
                        },
                    )?;
                    events.extend(outcome.events);
                    if outcome.terminated {
                        return Ok(ReactionReturn::Terminated);
                    }
                    self.phase = GuardPhase::AwaitAttack;
                    self.result = Some(outcome.returned);
                }
            }
        }
    }

    pub(super) fn advance(
        &mut self,
        combat: &mut Combat,
        warriors: &mut [Warrior],
        list: &[i16],
        context: &mut ReactionContext<'_>,
    ) -> Result<GuardStep, RuleError> {
        self.validate()?;
        loop {
            match self.phase {
                GuardPhase::Select => {
                    if self.index >= self.count {
                        self.phase = GuardPhase::Done;
                        continue;
                    }
                    let other = context.body(combat, list, self.index as usize)?;
                    let actor = context.actor(combat, self.actor);
                    let eligible = if other < 10 {
                        neighbor_work::require_roster_body(combat, warriors, other)?;
                        let record = warriors[other as usize].record();
                        (self.departure || record.guarding()? != 0)
                            && record.turn_skip_duration()? == 0
                    } else {
                        let record = combat.story_record((other - 10) as usize)?.record();
                        (self.departure || record.guarding() != 0)
                            && record.turn_skip_duration() == 0
                            && !record.removed()
                            && (actor >= 10 || {
                                let selected = context
                                    .native
                                    .as_ref()
                                    .map_or(actor as usize, |native| native.selected_warrior);
                                let warrior = warriors.get(selected).ok_or_else(|| {
                                    RuleError::from("selected warrior outside the squad")
                                })?;
                                !matches!(warrior.record().profession()?, 4 | 5)
                            })
                    };
                    if !eligible {
                        self.index = self.index.wrapping_add(1);
                        continue;
                    }
                    neighbor_work::require_roster_body(combat, warriors, other)?;
                    neighbor_work::require_roster_body(combat, warriors, actor)?;
                    self.phase = GuardPhase::Arm;
                    if context.display_writers {
                        return Ok(GuardStep::Refresh(other));
                    }
                }
                GuardPhase::Arm => {
                    let other = context.body(combat, list, self.index as usize)?;
                    let actor = context.actor(combat, self.actor);
                    neighbor_work::require_roster_body(combat, warriors, other)?;
                    neighbor_work::require_roster_body(combat, warriors, actor)?;
                    self.changed = true;
                    if other < 10 {
                        warriors[other as usize].record_mut().set_guarding(0);
                    } else {
                        combat.enemies.records[(other - 10) as usize]
                            .record_mut()
                            .set_reaction_ready(false);
                        combat.weapon_work.natural_attack_index = 0;
                    }
                    self.phase = GuardPhase::AwaitAttack;
                    return Ok(GuardStep::Attack(AttackCall {
                        attacker: other,
                        recipient: actor,
                        index: 0,
                        retained_actor: actor,
                    }));
                }
                GuardPhase::AwaitAttack => {
                    self.consume_return()?;
                }
                GuardPhase::Done => return Ok(GuardStep::Done(self.interrupted)),
            }
        }
    }
}

pub(super) fn retained_actor(state: &crate::Session) -> Result<i16, RuleError> {
    state
        .native_work
        .initiative_work
        .current(state.native_work.selected_enemy)
}

pub(super) fn retained_unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.retained_reaction",
        dependency,
    }
}

#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../../../tests/native/reaction_return_reference.rs"]
mod tests;
