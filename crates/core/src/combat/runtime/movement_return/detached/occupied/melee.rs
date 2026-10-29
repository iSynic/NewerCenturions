//! Direct occupied strikes ignore attack's ordinary guard-loop return value.
use super::{Control, Outcome};
use crate::combat::runtime::{detached_warrior, reaction_calls::AttackCall};
use crate::{Content, RuleError, Session};

pub(super) fn apply(
    state: &mut Session,
    content: &Content,
    actor: usize,
    target: i16,
) -> Result<Outcome, RuleError> {
    let warrior = state
        .warriors
        .get_mut(actor)
        .ok_or_else(|| RuleError::from("selected warrior outside the squad"))?;
    let movement = warrior.record().movement_points()?.wrapping_sub(3).max(0);
    let actions = warrior.record().action_points()?.wrapping_sub(2);
    warrior.record_mut().set_movement_points(movement);
    warrior.record_mut().set_action_points(actions);
    let outcome = detached_warrior::run(
        state,
        content,
        AttackCall {
            attacker: actor as i16,
            recipient: target,
            index: 0,
            retained_actor: actor as i16,
        },
    )?;
    if outcome.terminated {
        return Ok(Outcome {
            events: outcome.events,
            control: Control::Terminated,
        });
    }
    let actor = state.selected_warrior;
    if state
        .warriors
        .get(actor)
        .ok_or_else(|| RuleError::from("selected warrior outside the squad"))?
        .stamina()?
        < 1
    {
        state.warriors[actor].record_mut().set_action_points(0);
    }
    if state.warriors[actor].record().action_points()? > 0 {
        crate::combat::runtime::death_return::refresh_body(state, content, actor as i16)?;
    }
    Ok(Outcome::returned(outcome.events))
}
