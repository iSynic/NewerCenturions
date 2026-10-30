//! Striking the body in an occupied cell; the guard loop's result does not stop it.
use super::{Control, Outcome};
use crate::combat::runtime::death_return;
use crate::combat::runtime::{out_of_combat_warrior, reaction_calls::AttackCall};
use crate::{Content, RuleError, Session};

/// A warrior strikes the body in the cell in melee; the attack is spent, a warrior whose stamina
/// has dropped below 1 loses its remaining attacks, and one with attacks left has its weapon
/// refreshed.
///
/// Original: combatchoice
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
    crate::combat::records::spend_attack_action(warrior)?;
    let outcome = out_of_combat_warrior::run(
        state,
        content,
        AttackCall {
            attacker: actor as i16,
            recipient: target,
            index: 0,
            turn_actor: actor as i16,
        },
    )?;
    if outcome.terminated {
        return Ok(Outcome {
            events: outcome.events,
            control: Control::Terminated,
        });
    }
    let actor = state.char_select_new;
    if state
        .warriors
        .get(actor)
        .ok_or_else(|| RuleError::from("selected warrior outside the squad"))?
        .stamina()?
        < 1
    {
        state.warriors[actor].record_mut().set_attacks(0);
    }
    if state.warriors[actor].record().attacks()? > 0 {
        death_return::refresh_body(state, content, actor as i16)?;
    }
    Ok(Outcome::returned(outcome.events))
}
