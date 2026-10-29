//! Combat-zero ShowResults and the outdoor killbody suffix.
use crate::combat::{
    attack,
    runtime::{attacks::AttackOutcome, reaction_calls::AttackCall},
};
use crate::{Content, Event, RuleError, Session, feedback::NativeFeedback};
use centurions_content::records::ItemDefinition;

pub(super) fn finish(
    state: &mut Session,
    content: &Content,
    call: AttackCall,
    result: attack::AttackResult,
    reflected: bool,
) -> Result<AttackOutcome, RuleError> {
    let body = if reflected {
        call.attacker
    } else {
        call.recipient
    };
    let mut events = vec![];
    let mut physical = result.damage;
    let presented_hit = result.damage != 0 || result.feedback.is_some();
    let mut weapon_feedback = false;
    if let Some(feedback) = result.feedback {
        if matches!(feedback.code, -22 | -19 | -20 | -21 | -23 | -15 | -16) {
            physical = physical.wrapping_sub(feedback.amount);
        }
        let recipient = if feedback.recipient == attack::AttackRecipient::Attacker {
            call.attacker
        } else {
            body
        };
        if feedback.code == -40 {
            weapon_feedback = true;
        } else {
            events.push(Event::Feedback(NativeFeedback::CombatResult {
                body: recipient,
                code: feedback.code,
                amount: feedback.amount,
                audible: crate::feedback::combat_result(feedback.code)
                    .is_some_and(|(_, sound)| !result.sounds.contains(&sound)),
            }));
        }
    }
    events.extend(
        result
            .sounds
            .into_iter()
            .map(|resource_id| Event::Sound { resource_id }),
    );
    if weapon_feedback {
        let item = state
            .native_work
            .ammunition_work
            .item
            .as_ref()
            .ok_or_else(|| RuleError::from("ITEM work missing"))?;
        let item = ItemDefinition(item);
        let index = string_index(item.condition_feedback_index()?);
        events.push(Event::Feedback(NativeFeedback::CombatResourceResult {
            resource_id: 134,
            index,
        }));
        events.push(Event::Sound {
            resource_id: item.impact_sound()?,
        });
    }
    state.native_work.neighbor_work.departure = 0;
    if !presented_hit {
        events.push(Event::Sound { resource_id: 650 });
    } else {
        events.push(Event::Feedback(NativeFeedback::CombatResult {
            body,
            code: 0,
            amount: physical,
            audible: false,
        }));
    }
    let npc = usize::try_from(body.wrapping_sub(10)).ok();
    if let Some(npc) = npc.filter(|_| presented_hit && physical < 0) {
        let enemy = &mut state.native_work.npc_backing.slot_mut(npc)?.record;
        let maximum = enemy.maximum_stamina()?;
        if enemy.stamina()? > maximum {
            enemy.record_mut().set_stamina(maximum);
        }
    }
    let health = match npc {
        Some(npc) => state.native_work.npc_backing.slot(npc)?.record.stamina()?,
        None => state
            .table_record(body as usize)
            .ok_or("attack2 result body outside the warrior table")?
            .stamina()?,
    };
    let returned = health < 1;
    if returned && super::super::outdoor_death::defeat(state, content, body, &mut events)? {
        return Ok(AttackOutcome {
            events,
            returned: false,
            terminated: true,
        });
    }
    events.push(Event::AttackResolved {
        attacker: call.attacker,
        target: body,
        damage: result.damage,
        defeated: returned,
        icon: crate::event::PresentationIcon(result.icon),
        flight: Default::default(),
    });
    Ok(AttackOutcome {
        events,
        returned,
        terminated: false,
    })
}

/// GetIndString (CODE 2 +0x17c) clears the text, then leaves it empty for index 0
/// and, by its unsigned compare, for any negative index; index 0 carries that.
pub(in crate::combat::runtime) fn string_index(index: i16) -> usize {
    usize::try_from(index).unwrap_or(0)
}
