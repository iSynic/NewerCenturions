//! The guard loop's result and its reads of the shared neighbor list.
use super::{attacks, refresh_body_item};
use crate::combat::{
    Combat,
    runtime::neighbor_scan::{self, NeighborScan},
};
use crate::{Event, RandomState, RuleError};
use centurions_content::Warrior;
use serde::{Deserialize, Serialize};

/// What a reaction check produced: its events, whether the acting body was interrupted by a kill,
/// and whether the battle ended.
#[derive(Default)]
pub(crate) struct ReactionResult {
    pub events: Vec<Event>,
    pub interrupted: bool,
    pub terminated: bool,
}

/// How a resumed reaction check ended: it finished, saying whether a kill interrupted the acting
/// body, or the battle ended.
pub(super) enum ReactionReturn {
    Returned(bool),
    Terminated,
}

/// What a reaction needs besides the battle itself: the item and attack tables, the buff table, the
/// loaded item buffers, whether to refresh the shown weapon, and the neighbor scan when one runs.
pub(super) struct ReactionContext<'a> {
    pub items: Option<&'a [Vec<u8>]>,
    pub attacks: Option<&'a [u8]>,
    pub buffs: &'a [u8],
    pub buffers: &'a mut crate::ammunition::LoadedItems,
    pub display_writers: bool,
    pub scan: Option<NeighborScan<'a>>,
}

impl ReactionContext<'_> {
    /// The body whose turn it is: from the battle order while scanning, else `fallback`.
    pub fn actor(&self, combat: &Combat, fallback: i16) -> i16 {
        if self.scan.is_some() {
            combat
                .battle_order
                .get(combat.battle_order_index)
                .copied()
                .unwrap_or(combat.battle_order_index as i16)
        } else {
            fallback
        }
    }

    /// The body at `index` of the neighbor list, read from the shared scan when one runs.
    pub fn body(&self, combat: &Combat, list: &[i16], index: usize) -> Result<i16, RuleError> {
        if let Some(scan) = &self.scan {
            combat.neighbors.body(index, scan.party_track)
        } else {
            Ok(list[index])
        }
    }
}

/// One reaction attack: who strikes whom with which natural attack, during whose turn.
#[derive(Clone, Copy)]
pub(super) struct AttackCall {
    pub attacker: i16,
    pub recipient: i16,
    pub index: usize,
    pub turn_actor: i16,
}

/// Resolves a reaction's attack without a death story.
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
        context.actor(combat, call.turn_actor),
        context.items,
        context.attacks,
        context.buffs,
        context.buffers,
        None,
    )
}

/// Gives each neighbor that is guarding one free attack against the acting body; on a departure
/// every neighbor in the list qualifies. A neighbor skipping its turns, a removed monster, or a
/// monster facing a warrior of MOS 4 or 5 does not react. A kill ends the loop and
/// interrupts the acting body.
///
/// Original: checkforenemy
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

/// The guard loop with the strike supplied by the caller.
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

/// The reaction loop's own state; the neighbor list and actor globals stay shared.
///
/// Original: checkforenemy
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

/// Where a guard's reaction stands: choosing the next neighbor, readying it, waiting for its attack
/// to return, or finished.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
enum GuardPhase {
    Select,
    Arm,
    AwaitAttack,
    Done,
}

/// What the guard loop asks of its caller next: refresh a neighbor's weapon, resolve an attack, or
/// finish (interrupted or not).
pub(super) enum GuardStep {
    Refresh(i16),
    Attack(AttackCall),
    Done(bool),
}

impl GuardFrame {
    /// Starts a guard loop over `count` neighbors, at most 110.
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

    /// Refuses a saved guard loop whose counts, actor or phase disagree.
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

    /// How many neighbors the guard loop covers.
    pub(super) fn count(&self) -> i8 {
        self.count
    }

    /// Whether a guard has reacted; a kill clears it again.
    pub(super) fn changed(&self) -> bool {
        self.changed
    }

    /// Whether a guard's attack has returned and waits to be consumed.
    pub(crate) fn returned_attack(&self) -> bool {
        self.phase == GuardPhase::AwaitAttack && self.result.is_some()
    }

    /// Records how the waiting guard attack returned: `true` when it killed and so ended the
    /// exchange.
    pub(super) fn finish_attack(&mut self, returned: bool) -> Result<(), RuleError> {
        if self.phase != GuardPhase::AwaitAttack || self.result.is_some() {
            return Err("guard attack return has no waiting caller".into());
        }
        self.result = Some(returned);
        Ok(())
    }

    /// Passes on from the neighbor that just attacked; if its attack returned, the loop ends
    /// interrupted.
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

    /// Runs the guard loop on the party's globals after the battle has ended. A neighbor reacts
    /// when it is guarding (or the body is departing) and is not skipping its turns; a monster must
    /// also be present and does not react to a warrior of MOS 4 or 5. The reacting neighbor
    /// stops guarding and attacks the acting body.
    pub(super) fn finish_out_of_combat(
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
                        .globals
                        .neighbors
                        .body(self.index as usize, &state.globals.party_track)?;
                    let actor = turn_actor(state)?;
                    // Bodies 0..8 are the warrior table, the Upgrade records and the
                    // editor backup.
                    let eligible = if let Some(warrior) = usize::try_from(other)
                        .ok()
                        .filter(|_| other < 9)
                        .and_then(|other| state.table_record(other))
                    {
                        (self.departure || warrior.record().guarding()? != 0)
                            && warrior.record().turn_skip_duration()? == 0
                    } else if (10..110).contains(&other) {
                        let record = state
                            .globals
                            .monsters
                            .slot((other - 10) as usize)?
                            .record
                            .record();
                        (self.departure || record.guarding() != 0)
                            && record.turn_skip_duration() == 0
                            && !record.removed()
                            && (actor > 6
                                || !matches!(
                                    state.warriors[state.char_select_new].record().mos()?,
                                    4 | 5
                                ))
                    } else if other == 9 {
                        return Err(out_of_combat_unavailable(
                            "retained guard body 9 before the NPC array",
                        ));
                    } else if other >= 110 {
                        return Err(out_of_combat_unavailable(
                            "retained guard body past the NPC array",
                        ));
                    } else {
                        return Err(out_of_combat_unavailable(
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
                        .globals
                        .neighbors
                        .body(self.index as usize, &state.globals.party_track)?;
                    if other < 10 {
                        super::out_of_combat_warrior::refresh_guard(state, content, other)?;
                        // A reacting body stops guarding, whichever record it names.
                        let mut warrior = state
                            .table_record(other as usize)
                            .ok_or("retained guard outside the warrior table")?;
                        warrior.record_mut().set_guarding(0);
                        state.store_table_record(other as usize, warrior)?;
                    } else {
                        let slot = (other - 10) as usize;
                        let item = state
                            .globals
                            .monsters
                            .slot(slot)?
                            .record
                            .record()
                            .weapon()?;
                        if item != 0 {
                            state.load_item(content, item)?;
                        }
                        state
                            .globals
                            .monsters
                            .slot_mut(slot)?
                            .record
                            .record_mut()
                            .set_reaction_ready(false);
                        state.globals.weapon_traits.natural_attack_index = 0;
                    }
                    self.changed = true;
                    let run = if other < 10 {
                        super::out_of_combat_warrior::run
                    } else {
                        super::out_of_combat_attack::run
                    };
                    let outcome = run(
                        state,
                        content,
                        AttackCall {
                            attacker: other,
                            recipient: turn_actor(state)?,
                            index: 0,
                            turn_actor: turn_actor(state)?,
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

    /// Steps the guard loop inside a battle until it needs a weapon refresh or an attack from its
    /// caller, or finishes. A reacting neighbor stops guarding.
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
                        neighbor_scan::require_roster_body(combat, warriors, other)?;
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
                                    .scan
                                    .as_ref()
                                    .map_or(actor as usize, |scan| scan.char_select_new);
                                let warrior = warriors.get(selected).ok_or_else(|| {
                                    RuleError::from("selected warrior outside the squad")
                                })?;
                                !matches!(warrior.record().mos()?, 4 | 5)
                            })
                    };
                    if !eligible {
                        self.index = self.index.wrapping_add(1);
                        continue;
                    }
                    neighbor_scan::require_roster_body(combat, warriors, other)?;
                    neighbor_scan::require_roster_body(combat, warriors, actor)?;
                    self.phase = GuardPhase::Arm;
                    if context.display_writers {
                        return Ok(GuardStep::Refresh(other));
                    }
                }
                GuardPhase::Arm => {
                    let other = context.body(combat, list, self.index as usize)?;
                    let actor = context.actor(combat, self.actor);
                    neighbor_scan::require_roster_body(combat, warriors, other)?;
                    neighbor_scan::require_roster_body(combat, warriors, actor)?;
                    self.changed = true;
                    if other < 10 {
                        warriors[other as usize].record_mut().set_guarding(0);
                    } else {
                        combat.roster.records[(other - 10) as usize]
                            .record_mut()
                            .set_reaction_ready(false);
                        combat.weapon_traits.natural_attack_index = 0;
                    }
                    self.phase = GuardPhase::AwaitAttack;
                    return Ok(GuardStep::Attack(AttackCall {
                        attacker: other,
                        recipient: actor,
                        index: 0,
                        turn_actor: actor,
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

/// The body whose turn it is, from the battle order.
pub(super) fn turn_actor(state: &crate::Session) -> Result<i16, RuleError> {
    state.globals.battle_order.current(state.globals.monster_up)
}

/// The refusal for a guard reaction that depends on an unrecovered part of the game.
pub(super) fn out_of_combat_unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.retained_reaction",
        dependency,
    }
}

#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../../../tests/native/reaction_return_reference.rs"]
mod tests;
