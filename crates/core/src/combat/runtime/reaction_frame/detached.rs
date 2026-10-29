//! checkforenemy resumes its locals against the current shared work after teardown.
use super::{ReactionFrame, ReactionPhase};
use crate::combat::runtime::reaction_calls::{
    AttackCall, GuardFrame, ReactionResult, ReactionReturn, retained_actor,
};
use crate::{Content, RuleError, Session, combat::neighbor_work};

pub(super) fn run(
    frame: &mut ReactionFrame,
    state: &mut Session,
    content: &Content,
) -> Result<ReactionResult, RuleError> {
    frame.validate()?;
    let mut events = vec![];
    if !frame.started {
        state.native_work.neighbor_work.departure = 0;
        frame.started = true;
        if retained_actor(state)? < 0 {
            frame.phase = ReactionPhase::Done(false);
        }
    }
    loop {
        match &mut frame.phase {
            ReactionPhase::Scan => {
                if !state.native_work.victory_work.reaction_admitted() {
                    frame.phase = ReactionPhase::Done(false);
                    continue;
                }
                let list = neighbor_work::scan_retained(
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
                let returned = match guard.finish_retained(state, content, &mut events)? {
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
                    crate::combat::runtime::death_return::refresh_actor(state, content)?;
                }
            }
            ReactionPhase::AfterRefresh(count) => {
                if *count == 0 || frame.mode == 2 || retained_actor(state)? < 10 {
                    frame.phase = ReactionPhase::Done(false);
                    continue;
                }
                let index = (state.random.bound(i16::from(*count)) - 1) as usize;
                let target = state
                    .native_work
                    .neighbor_work
                    .body(index, &state.native_work.party_mask)?;
                let slot = state.native_work.selected_enemy;
                state
                    .native_work
                    .npc_backing
                    .slot_mut(slot)?
                    .record
                    .record_mut()
                    .set_target_body(target);
                state.native_work.weapon_work.natural_attack_index = 0;
                frame.phase = ReactionPhase::Strike;
            }
            ReactionPhase::Strike => {
                let slot = state.native_work.selected_enemy;
                let index = state.native_work.weapon_work.natural_attack_index;
                // The original rate and AP/MP fields are signed BYTEs here.
                let rate = state
                    .native_work
                    .npc_backing
                    .slot(slot)?
                    .record
                    .record()
                    .actions_per_round();
                if index >= i16::from(rate / 2) {
                    frame.phase = ReactionPhase::Done(false);
                    continue;
                }
                let enemy = &mut state.native_work.npc_backing.slot_mut(slot)?.record;
                let actions = enemy.record().action_points().wrapping_sub(2);
                enemy.record_mut().set_action_points(actions);
                if actions < 0 {
                    if actions == -1 {
                        enemy.record_mut().set_reaction_ready(true);
                    }
                    frame.phase = ReactionPhase::Done(false);
                    continue;
                }
                let movement = enemy.record().movement_points().wrapping_sub(3);
                enemy.record_mut().set_movement_points(movement);
                if actions < 2 {
                    enemy.record_mut().set_reaction_ready(actions >= 1);
                }
                let target = enemy.record().target_body()?;
                let index = usize::try_from(index)
                    .map_err(|_| RuleError::Invalid("negative natural-attack index".into()))?;
                let outcome = super::super::detached_attack::run(
                    state,
                    content,
                    AttackCall {
                        attacker: slot as i16 + 10,
                        recipient: target,
                        index,
                        retained_actor: retained_actor(state)?,
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
                    let index = state.native_work.weapon_work.natural_attack_index;
                    state.native_work.weapon_work.natural_attack_index = index.wrapping_add(1);
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
