//! A reaction check that resumes after the battle has ended.
use super::{ReactionFrame, ReactionPhase};
use crate::combat::runtime::death_return;
use crate::combat::runtime::reaction_calls::{
    AttackCall, GuardFrame, ReactionResult, ReactionReturn, turn_actor,
};
use crate::{Content, RuleError, Session, combat::runtime::neighbor_scan};

/// Runs a reaction check on the party's globals after the battle has ended, resuming from the phase
/// it paused in.
pub(super) fn run(
    frame: &mut ReactionFrame,
    state: &mut Session,
    content: &Content,
) -> Result<ReactionResult, RuleError> {
    frame.validate()?;
    let mut events = vec![];
    if !frame.started {
        state.globals.neighbors.departure = 0;
        frame.started = true;
        if turn_actor(state)? < 0 {
            frame.phase = ReactionPhase::Done(false);
        }
    }
    loop {
        match &mut frame.phase {
            ReactionPhase::Scan => {
                if !state.globals.kill_counts.reaction_admitted() {
                    frame.phase = ReactionPhase::Done(false);
                    continue;
                }
                let list = neighbor_scan::scan_out_of_combat(
                    state,
                    content,
                    frame.mode as usize,
                    &mut frame.size,
                )?;
                frame.phase = ReactionPhase::Guards(GuardFrame::new(
                    frame.actor,
                    frame.mode == 2,
                    list.len(),
                    frame.changed,
                )?);
            }
            ReactionPhase::Guards(guard) => {
                let returned = match guard.finish_out_of_combat(state, content, &mut events)? {
                    ReactionReturn::Returned(returned) => returned,
                    ReactionReturn::Terminated => {
                        return Ok(ReactionResult {
                            events,
                            interrupted: false,
                            terminated: true,
                        });
                    }
                };
                frame.changed = guard.changed();
                if returned {
                    frame.phase = ReactionPhase::Done(true);
                    continue;
                }
                frame.phase = ReactionPhase::AfterRefresh(guard.count());
                if frame.changed {
                    death_return::refresh_actor(state, content)?;
                }
            }
            ReactionPhase::AfterRefresh(count) => {
                if *count == 0 || frame.mode == 2 || turn_actor(state)? < 10 {
                    frame.phase = ReactionPhase::Done(false);
                    continue;
                }
                let index = (state.random.bound(i16::from(*count)) - 1) as usize;
                let target = state
                    .globals
                    .neighbors
                    .body(index, &state.globals.party_track)?;
                let slot = state.globals.monster_up;
                state
                    .globals
                    .monsters
                    .slot_mut(slot)?
                    .record
                    .record_mut()
                    .set_target(target);
                state.globals.weapon_traits.natural_attack_index = 0;
                frame.phase = ReactionPhase::Strike;
            }
            ReactionPhase::Strike => {
                let slot = state.globals.monster_up;
                let index = state.globals.weapon_traits.natural_attack_index;
                // The rate, attacks and movement are signed bytes here.
                let rate = state
                    .globals
                    .monsters
                    .slot(slot)?
                    .record
                    .record()
                    .attack_count();
                if index >= i16::from(rate / 2) {
                    frame.phase = ReactionPhase::Done(false);
                    continue;
                }
                let monster = &mut state.globals.monsters.slot_mut(slot)?.record;
                let actions = monster.record().action_points().wrapping_sub(2);
                monster.record_mut().set_action_points(actions);
                if actions < 0 {
                    if actions == -1 {
                        monster.record_mut().set_reaction_ready(true);
                    }
                    frame.phase = ReactionPhase::Done(false);
                    continue;
                }
                let movement = monster.record().movement().wrapping_sub(3);
                monster.record_mut().set_movement(movement);
                if actions < 2 {
                    monster.record_mut().set_reaction_ready(actions >= 1);
                }
                let target = monster.record().target()?;
                let index = usize::try_from(index)
                    .map_err(|_| RuleError::Invalid("negative natural-attack index".into()))?;
                let outcome = super::super::out_of_combat_attack::run(
                    state,
                    content,
                    AttackCall {
                        attacker: slot as i16 + 10,
                        recipient: target,
                        index,
                        turn_actor: turn_actor(state)?,
                    },
                )?;
                events.extend(outcome.events);
                if outcome.terminated {
                    return Ok(ReactionResult {
                        events,
                        interrupted: false,
                        terminated: true,
                    });
                }
                frame.phase = ReactionPhase::AwaitStrike(Some(outcome.returned));
            }
            ReactionPhase::AwaitStrike(result) => {
                let returned = result
                    .take()
                    .ok_or_else(|| RuleError::from("NPC attack return missing"))?;
                if returned {
                    frame.phase = ReactionPhase::Scan;
                } else {
                    let index = state.globals.weapon_traits.natural_attack_index;
                    state.globals.weapon_traits.natural_attack_index = index.wrapping_add(1);
                    frame.phase = ReactionPhase::Strike;
                }
            }
            ReactionPhase::Done(returned) => {
                return Ok(ReactionResult {
                    events,
                    interrupted: *returned,
                    terminated: false,
                });
            }
        }
    }
}
