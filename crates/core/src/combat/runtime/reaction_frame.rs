//! One reaction check, which pauses only when an attack's death story runs.
use super::reaction_calls::{AttackCall, GuardFrame, GuardStep, ReactionContext};
use crate::combat::{Combat, runtime::neighbor_scan};
use crate::{RandomState, RuleError};
use centurions_content::Warrior;
use serde::{Deserialize, Serialize};
mod out_of_combat;

/// One reaction check that pauses only while an attack's death script runs and picks up afterwards.
///
/// Original: checkforenemy
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReactionFrame {
    actor: i16,
    mode: u8,
    direction: (i8, i8),
    started: bool,
    changed: bool,
    size: i8,
    phase: ReactionPhase,
}

/// Where a reaction check stands: scanning the neighbors, the guards' strikes, choosing a target
/// after the guards (with the neighbor count), the monster's own strikes, waiting for such a strike
/// to return, or finished (with whether an attack returned).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
enum ReactionPhase {
    Scan,
    Guards(GuardFrame),
    AfterRefresh(i8),
    Strike,
    AwaitStrike(Option<bool>),
    Done(bool),
}

impl ReactionFrame {
    /// Starts a reaction check for `actor`. Mode 0 checks a body standing in place, mode 2 one
    /// leaving a cell in `direction`.
    pub(super) fn new(actor: i16, mode: usize, direction: (i8, i8)) -> Result<Self, RuleError> {
        if !matches!(mode, 0 | 2) {
            return Err("invalid reaction invocation mode".into());
        }
        Ok(Self {
            actor,
            mode: mode as u8,
            direction,
            started: false,
            changed: false,
            size: 0,
            phase: ReactionPhase::Scan,
        })
    }

    /// Refuses a saved reaction check whose mode, actor, direction or counts are out of range.
    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        if !matches!(self.mode, 0 | 2)
            || !(-1..110).contains(&self.actor)
            || !(-1..=1).contains(&self.direction.0)
            || !(-1..=1).contains(&self.direction.1)
        {
            return Err("invalid saved reaction invocation".into());
        }
        match &self.phase {
            ReactionPhase::Guards(guard) => guard.validate(),
            ReactionPhase::AfterRefresh(count) if !(0..=110).contains(count) => {
                Err("invalid reaction local count".into())
            }
            _ => Ok(()),
        }
    }

    /// Whether an attack of this check has returned and waits to be consumed.
    pub(crate) fn returned_attack(&self) -> bool {
        match &self.phase {
            ReactionPhase::Guards(guard) => guard.returned_attack(),
            ReactionPhase::AwaitStrike(result) => result.is_some(),
            _ => false,
        }
    }

    /// Finishes the check against the party's globals after the battle ended during its pause.
    pub(super) fn finish_after_teardown(
        &mut self,
        state: &mut crate::Session,
        content: &centurions_content::Content,
    ) -> Result<super::reaction_calls::ReactionResult, RuleError> {
        out_of_combat::run(self, state, content)
    }

    /// Records how the waiting attack returned: `true` when it killed.
    pub(super) fn finish_attack(&mut self, returned: bool) -> Result<(), RuleError> {
        match &mut self.phase {
            ReactionPhase::Guards(guard) => guard.finish_attack(returned),
            ReactionPhase::AwaitStrike(result) if result.is_none() => {
                *result = Some(returned);
                Ok(())
            }
            _ => Err("reaction attack return has no waiting caller".into()),
        }
    }

    /// Steps the check inside a battle until it needs a weapon refresh or an attack from its
    /// caller, or finishes.
    pub(super) fn advance(
        &mut self,
        combat: &mut Combat,
        warriors: &mut [Warrior],
        random: &mut RandomState,
        context: &mut ReactionContext<'_>,
    ) -> Result<GuardStep, RuleError> {
        self.validate()?;
        if context.scan.is_none() {
            return Err(unsupported("native reaction work context"));
        }
        if !self.started {
            neighbor_scan::begin(combat);
            self.started = true;
            if context.actor(combat, self.actor) < 0 {
                self.phase = ReactionPhase::Done(false);
            }
        }
        loop {
            match &mut self.phase {
                ReactionPhase::Scan => {
                    if !combat.kill_counts.reaction_admitted() {
                        self.phase = ReactionPhase::Done(false);
                        continue;
                    }
                    let list = neighbor_scan::scan_with_size(
                        combat,
                        warriors,
                        self.mode as usize,
                        self.direction,
                        context.scan.as_mut().unwrap(),
                        &mut self.size,
                    )?;
                    self.phase = ReactionPhase::Guards(GuardFrame::new(
                        self.actor,
                        self.mode == 2,
                        list.len(),
                        self.changed,
                    )?);
                }
                ReactionPhase::Guards(guard) => {
                    match guard.advance(combat, warriors, &[], context)? {
                        GuardStep::Done(returned) => {
                            self.changed = guard.changed();
                            if returned {
                                self.phase = ReactionPhase::Done(true);
                                continue;
                            }
                            let count = guard.count();
                            self.phase = ReactionPhase::AfterRefresh(count);
                            if self.changed {
                                return Ok(GuardStep::Refresh(context.actor(combat, self.actor)));
                            }
                        }
                        step => return Ok(step),
                    }
                }
                ReactionPhase::AfterRefresh(count) => {
                    if *count == 0 || self.mode == 2 || context.actor(combat, self.actor) < 10 {
                        self.phase = ReactionPhase::Done(false);
                        continue;
                    }
                    let index = (random.bound(i16::from(*count)) - 1) as usize;
                    let target = context.body(combat, &[], index)?;
                    let monster = combat.monster_up;
                    neighbor_scan::require_roster_body(combat, warriors, monster as i16 + 10)?;
                    combat.roster.records[monster]
                        .record_mut()
                        .set_target(target);
                    combat.weapon_traits.natural_attack_index = 0;
                    self.phase = ReactionPhase::Strike;
                }
                ReactionPhase::Strike => {
                    let monster = combat.monster_up;
                    neighbor_scan::require_roster_body(combat, warriors, monster as i16 + 10)?;
                    let index = combat.weapon_traits.natural_attack_index;
                    let record = combat.roster.records[monster].record();
                    if index >= i16::from(record.attack_count() / 2) {
                        self.phase = ReactionPhase::Done(false);
                        continue;
                    }
                    let actions = record.action_points().wrapping_sub(2);
                    let record = &mut combat.roster.records[monster];
                    record.record_mut().set_action_points(actions);
                    if actions < 0 {
                        if actions == -1 {
                            record.record_mut().set_reaction_ready(true);
                        }
                        self.phase = ReactionPhase::Done(false);
                        continue;
                    }
                    let movement = record.record().movement().wrapping_sub(3);
                    record.record_mut().set_movement(movement);
                    if actions < 2 {
                        record.record_mut().set_reaction_ready(actions >= 1);
                    }
                    let target = record.record().target()?;
                    neighbor_scan::require_roster_body(combat, warriors, target)?;
                    let index = usize::try_from(index)
                        .map_err(|_| RuleError::Invalid("negative natural-attack index".into()))?;
                    self.phase = ReactionPhase::AwaitStrike(None);
                    return Ok(GuardStep::Attack(AttackCall {
                        attacker: monster as i16 + 10,
                        recipient: target,
                        index,
                        turn_actor: context.actor(combat, self.actor),
                    }));
                }
                ReactionPhase::AwaitStrike(result) => {
                    let returned = result
                        .take()
                        .ok_or_else(|| RuleError::from("NPC attack return missing"))?;
                    if returned {
                        self.phase = ReactionPhase::Scan;
                    } else {
                        combat.weapon_traits.natural_attack_index =
                            combat.weapon_traits.natural_attack_index.wrapping_add(1);
                        self.phase = ReactionPhase::Strike;
                    }
                }
                ReactionPhase::Done(returned) => return Ok(GuardStep::Done(*returned)),
            }
        }
    }
}

/// The refusal for a reaction that depends on an unrecovered part of the game.
fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.reaction",
        dependency,
    }
}
