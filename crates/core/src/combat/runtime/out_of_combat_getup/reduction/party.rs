//! Party conditions and deaths during getup after the battle has ended.
use crate::combat::natural_conditions;
use crate::{
    Content, Event, RuleError, Session, combat::runtime::TurnFlow, feedback::GameFeedback,
    story_health,
};
use centurions_content::records::Condition;

/// One warrior's conditions at the start of a round after the battle: regeneration heals, disease,
/// protection, speed and poison cost stamina, energy recovers, and confusion ends. A warrior who
/// falls is reported.
///
/// Original: getup
pub(super) fn apply(
    state: &mut Session,
    content: &Content,
    member: usize,
    events: &mut Vec<Event>,
) -> Result<TurnFlow, RuleError> {
    let warrior = state.warriors[member].record();
    let regeneration = warrior.condition(Condition::Regenerating)?;
    if regeneration != 0
        && warrior.stamina()? > -10
        && warrior.stamina()? < warrior.stamina_max()?
        && warrior.computer_control()? == 0
    {
        let context = &state.globals.health_context;
        let affected = context.affected[member];
        let mut healed = story_health::heal_outdoors(
            &state.warriors,
            member,
            regeneration.wrapping_abs(),
            false,
            context,
            &mut state.random,
        )?;
        if healed
            .events
            .contains(&story_health::HealthEvent::PartyLost)
        {
            // Healing marks the member affected after a party loss that never returns.
            healed.context.affected[member] = affected;
        }
        if commit_health(state, content, healed, events)? == TurnFlow::Terminated {
            return Ok(TurnFlow::Terminated);
        }
    }
    for condition in [
        Condition::Diseased,
        Condition::SeventhLevelProtection,
        Condition::Speedy,
        Condition::Poisoned,
    ] {
        let before = state.warriors[member].stamina()?;
        let record = state.warriors[member].record();
        let active = before > 0
            && record.condition(condition)? != 0
            && (!matches!(condition, Condition::Diseased | Condition::Poisoned)
                || record.condition(Condition::ComputerControl)? >= 0);
        if active {
            let packet = &mut state.globals.spell_info;
            centurions_content::records::SpellInfoRecordMut(packet).set_health_effect(7);
            state.globals.health_context.effect = 7;
        }
        let context = &state.globals.health_context;
        let outcome = match condition {
            Condition::Diseased => natural_conditions::disease_outdoors(
                &state.warriors,
                member,
                context,
                &mut state.random,
            )?,
            Condition::Poisoned => natural_conditions::poison_outdoors(
                &state.warriors,
                member,
                context,
                &mut state.random,
            )?,
            _ => {
                let amount = state.warriors[member].record().condition(condition)?;
                if amount == 0 || before < 1 {
                    continue;
                }
                let health = before.wrapping_sub(if matches!(condition, Condition::Speedy) {
                    2
                } else {
                    amount.wrapping_abs()
                });
                state.warriors[member].record_mut().set_stamina(health);
                let context = &mut state.globals.health_context;
                context.effect = 7;
                if health < 1 {
                    context.kill_party = context.kill_party.wrapping_add(1);
                    context.last_body = member as i16;
                    events.push(Event::Sound {
                        resource_id: state.random.bound(2).wrapping_add(506),
                    });
                    events.push(Event::Feedback(GameFeedback::OutdoorBodyStatus {
                        permanent: health < -9,
                        wait: state.command_preferences.animation_delay,
                    }));
                    if context.kill_party > state.warriors.len() as i8 - 1 {
                        events.extend(crate::mission::defeat(state, content)?);
                        return Ok(TurnFlow::Terminated);
                    }
                }
                if before > 0 && health < 1 {
                    flash(events, condition);
                }
                continue;
            }
        };
        if commit_health(state, content, outcome, events)? == TurnFlow::Terminated {
            return Ok(TurnFlow::Terminated);
        }
        if before > 0 && state.warriors[member].stamina()? < 1 {
            flash(events, condition);
        }
    }
    crate::story::reduce::energy(
        &mut state.warriors[member].bytes,
        state.energy_recovery_disabled,
    )?;
    if state.warriors[member].record().confusion_duration()? != 0
        && state.warriors[member].record().faction()? != 0
    {
        state.warriors[member].record_mut().set_faction(0);
        let health = &mut state.globals.health_context;
        health.kill_party = health.kill_party.wrapping_sub(1);
        state.globals.kill_counts.add_num_enemy(-1);
    }
    state.warriors[member]
        .record_mut()
        .decrement_temporary_conditions()?;
    Ok(TurnFlow::Returned)
}

/// Takes the outcome of a warrior's health change into the party and shows the sounds and the
/// status of any warrior who fell; Terminated when the party is lost.
fn commit_health(
    state: &mut Session,
    content: &Content,
    outcome: story_health::HealthOutcome,
    events: &mut Vec<Event>,
) -> Result<TurnFlow, RuleError> {
    state.warriors = outcome.warriors;
    state.globals.health_context = outcome.context;
    let mut status = None;
    for event in outcome.events {
        match event {
            story_health::HealthEvent::Defeated { permanent, .. } => status = Some(permanent),
            story_health::HealthEvent::Sound(resource_id) => {
                events.push(Event::Sound { resource_id });
                if let Some(permanent) = status.take() {
                    events.push(Event::Feedback(GameFeedback::OutdoorBodyStatus {
                        permanent,
                        wait: state.command_preferences.animation_delay,
                    }));
                }
            }
            story_health::HealthEvent::PartyLost => {
                events.extend(crate::mission::defeat(state, content)?);
                return Ok(TurnFlow::Terminated);
            }
            _ => return Err("unexpected retained reduction presentation".into()),
        }
    }
    Ok(TurnFlow::Returned)
}

/// The flash and sound that mark a condition costing a warrior stamina.
fn flash(events: &mut Vec<Event>, condition: Condition) {
    let (address, sound) = match condition {
        Condition::Diseased => (0x401960, 10122),
        Condition::SeventhLevelProtection => (0x401948, 10122),
        Condition::Speedy => (0x40193a, 10122),
        _ => (0x401922, 684),
    };
    events.push(Event::Feedback(GameFeedback::Flash {
        address,
        x: 100,
        y: 30,
        wait: 100,
        sound,
    }));
}
