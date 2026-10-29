//! Detached health, healing caps and outdoor death feedback.
use super::{Context, NpcCredit, Outcome};
use crate::{Event, RandomState, RuleError, feedback::NativeFeedback, story_health};
use centurions_content::records::AttackPacket;

#[derive(PartialEq, Eq)]
pub(super) enum HealthReturn {
    Applied { defeated: bool },
    PartyLost,
}

pub(super) fn apply(
    outcome: &mut Outcome,
    body: usize,
    damage: i16,
    context: &Context<'_>,
    random: &mut RandomState,
) -> Result<HealthReturn, RuleError> {
    let attack = AttackPacket(context.spell.packet);
    if body >= 10 {
        if damage == 0 {
            return Ok(HealthReturn::Applied { defeated: false });
        }
        if attack.defense_category() != 9 {
            outcome.events.push(Event::Sound {
                resource_id: attack.impact_sound()?,
            });
        }
        let record = &mut outcome.bodies.enemies[body - 10];
        let after = record.stamina()?.wrapping_sub(damage);
        record.record_mut().mark_damaged();
        record.record_mut().set_stamina(after);
        // resolvespell 0x151c80..0x151d60 credits the hit before killbody enters MainLoop.
        outcome.npc_credit.push(NpcCredit {
            damage,
            hit: attack.defense_category() == 9,
            killed: after < 1,
        });
        if after < 1 {
            return defeat(outcome, body, None, context, random);
        }
        outcome.events.push(Event::Sound {
            resource_id: context.spell.weapon_sound,
        });
        animation(
            &mut outcome.events,
            context,
            attack.effect_animation().wrapping_abs(),
        );
        // ShowResults0 caps surviving negative-damage hits after field feedback.
        if damage < 0 {
            let record = &mut outcome.bodies.enemies[body - 10];
            let maximum = record.maximum_stamina()?;
            if record.stamina()? > maximum {
                record.record_mut().set_stamina(maximum);
            }
        }
        outcome.show_result(body, 0, damage);
        return Ok(HealthReturn::Applied { defeated: false });
    }
    let before = outcome.bodies.warriors[body].stamina()?;
    if damage == 0 || before <= -10 {
        return Ok(HealthReturn::Applied { defeated: false });
    }
    if attack.defense_category() != 9 {
        outcome.events.push(Event::Sound {
            resource_id: attack.impact_sound()?,
        });
    }
    if damage < 0 {
        let mut healed = story_health::heal_record(
            &outcome.bodies.warriors,
            outcome.party,
            body,
            damage.wrapping_abs(),
            true,
            &outcome.health,
            random,
        )?;
        if healed
            .events
            .contains(&story_health::HealthEvent::PartyLost)
        {
            // heal 0x1287c2 writes this mark after nonreturning partyloss 0x14913e.
            healed.context.affected[body] = outcome.health.affected[body];
        }
        outcome.bodies.warriors = healed.warriors;
        outcome.health = healed.context;
        outcome.bodies.healed = true;
        let mut status = None;
        for event in healed.events {
            match event {
                story_health::HealthEvent::Animation { effect, .. } => {
                    animation(&mut outcome.events, context, i16::from(effect))
                }
                story_health::HealthEvent::Sound(resource_id) => {
                    outcome.events.push(Event::Sound { resource_id });
                    if let Some(permanent) = status.take() {
                        outcome
                            .events
                            .push(Event::Feedback(NativeFeedback::OutdoorBodyStatus {
                                permanent,
                                wait: context.animation_delay,
                            }));
                    }
                }
                story_health::HealthEvent::Defeated { permanent, .. } => {
                    status = Some(permanent);
                }
                story_health::HealthEvent::PartyLost => {
                    return Ok(HealthReturn::PartyLost);
                }
                _ => return Err("unexpected detached field healing event".into()),
            }
        }
        return Ok(HealthReturn::Applied {
            defeated: before > 0 && outcome.bodies.warriors[body].stamina()? < 1,
        });
    }
    if before < 0 {
        return Ok(HealthReturn::Applied { defeated: false });
    }
    let after = before.wrapping_sub(damage);
    outcome.bodies.warriors[body]
        .record_mut()
        .set_stamina(after);
    outcome.bodies.warriors[body].record_mut().mark_damaged();
    outcome.health.affected[body] = -1;
    let defeated = before > 0 && after < 1;
    if defeated {
        return defeat(outcome, body, Some(after), context, random);
    }
    animation(
        &mut outcome.events,
        context,
        attack.effect_animation().wrapping_abs(),
    );
    outcome.show_result(body, effect_code(context), damage);
    Ok(HealthReturn::Applied { defeated })
}

pub(super) fn defeat(
    outcome: &mut Outcome,
    body: usize,
    stamina: Option<i16>,
    context: &Context<'_>,
    random: &mut RandomState,
) -> Result<HealthReturn, RuleError> {
    outcome.health.last_body = body as i16;
    outcome.health.unavailable = outcome.health.unavailable.wrapping_add(1);
    outcome.events.push(Event::Sound {
        resource_id: random.bound(2).wrapping_add(506),
    });
    let stamina = match stamina {
        Some(stamina) => stamina,
        None => context.outdoor_health.health(body as i16)?,
    };
    outcome
        .events
        .push(Event::Feedback(NativeFeedback::OutdoorBodyStatus {
            permanent: stamina < -9,
            wait: context.animation_delay,
        }));
    Ok(if outcome.health.unavailable > outcome.party as i8 - 1 {
        HealthReturn::PartyLost
    } else {
        HealthReturn::Applied { defeated: true }
    })
}

fn effect_code(context: &Context<'_>) -> i16 {
    match AttackPacket(context.spell.packet)
        .effect()
        .unwrap()
        .wrapping_abs()
    {
        57 => 0,
        effect => effect,
    }
}

pub(super) fn animation(events: &mut Vec<Event>, context: &Context<'_>, effect: i16) {
    events.push(Event::OutdoorEffect {
        effect,
        x: context.location.0,
        y: context.location.1,
        size: 32,
        delay: 0,
    });
}
