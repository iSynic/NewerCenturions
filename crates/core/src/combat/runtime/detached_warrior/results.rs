//! Warrior attack result order differs from attack2: misses never call killbody.
use crate::{
    Content, Event, RuleError, Session,
    combat::{
        attack,
        runtime::{attacks::AttackOutcome, reaction_calls::AttackCall},
    },
    feedback::NativeFeedback,
};
use centurions_content::records::ItemDefinition;

pub(super) fn finish(
    state: &mut Session,
    content: &Content,
    call: AttackCall,
    actual: i16,
    result: attack::AttackResult,
    refusal: Option<i16>,
) -> Result<AttackOutcome, RuleError> {
    let mut events = vec![];
    if let Some(code) = refusal {
        state.native_work.neighbor_work.departure = 0;
        events.push(Event::Feedback(NativeFeedback::CombatResult {
            body: actual,
            code,
            amount: 0,
            audible: true,
        }));
    } else {
        if result.feedback.is_some() {
            let item = state
                .native_work
                .ammunition_work
                .item
                .as_ref()
                .ok_or_else(|| RuleError::from("ITEM work missing"))?;
            let item = ItemDefinition(item);
            let index = super::super::detached_attack::results::string_index(
                item.condition_feedback_index()?,
            );
            state.native_work.neighbor_work.departure = 0;
            events.push(Event::Feedback(NativeFeedback::CombatResourceResult {
                resource_id: 134,
                index,
            }));
            events.push(Event::Sound {
                resource_id: item.impact_sound()?,
            });
        }
        events.extend(
            result
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
        if result.killed {
            if super::super::outdoor_death::defeat(state, content, actual, &mut events)? {
                return Ok(AttackOutcome {
                    events,
                    returned: false,
                    terminated: true,
                });
            }
        } else {
            state.native_work.neighbor_work.departure = 0;
            if result.damage == 0 {
                events.push(Event::Sound { resource_id: 650 });
            } else {
                events.push(Event::Feedback(NativeFeedback::CombatResult {
                    body: actual,
                    code: 0,
                    amount: result.damage,
                    audible: false,
                }));
            }
        }
    }
    events.push(Event::AttackResolved {
        attacker: call.attacker,
        target: actual,
        damage: result.damage,
        defeated: result.killed,
        icon: crate::event::PresentationIcon(result.icon),
        flight: Default::default(),
    });
    Ok(AttackOutcome {
        events,
        returned: result.killed,
        terminated: false,
    })
}
