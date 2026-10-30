//! A warrior's attack results; unlike a monster's, a miss never calls killbody.
use crate::{
    Content, Event, RuleError, Session,
    combat::{
        attack,
        runtime::{attacks::AttackOutcome, reaction_calls::AttackCall},
    },
    feedback::GameFeedback,
};
use centurions_content::records::ItemDefinition;

/// Shows the result of a warrior's attack and defeats the body that fell. A body is defeated only
/// when the attack killed it, and a blow that does no damage gives only a sound, with no damage
/// message.
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
        state.globals.neighbors.departure = 0;
        events.push(Event::Feedback(GameFeedback::CombatResult {
            body: actual,
            code,
            amount: 0,
            audible: true,
        }));
    } else {
        if result.feedback.is_some() {
            let item = state
                .globals
                .loaded_items
                .item
                .as_ref()
                .ok_or_else(|| RuleError::from("ITEM work missing"))?;
            let item = ItemDefinition(item);
            let index = super::super::out_of_combat_attack::results::string_index(
                item.condition_feedback_index()?,
            );
            state.globals.neighbors.departure = 0;
            events.push(Event::Feedback(GameFeedback::CombatResourceResult {
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
            state.globals.neighbors.departure = 0;
            if result.damage == 0 {
                events.push(Event::Sound { resource_id: 650 });
            } else {
                events.push(Event::Feedback(GameFeedback::CombatResult {
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
