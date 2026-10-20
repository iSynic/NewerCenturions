//! Party script operations, preserving original dispatch order.
use super::super::present_treasure;
use super::{Operation, ScriptStep};
use crate::{
    Event, Presentation, RuleError, Session, story, story_allies, story_inventory, story_party,
};
use centurions_content::mission::OpeningMission;
use centurions_content::records::{FatigueChange, RecoveryRestrictions};
use centurions_content::{Content, span, word};

pub(super) fn execute(
    operation: Operation,
    parameter: i16,
    state: &mut Session,
    content: &Content,
    map: &OpeningMission,
    cursor: usize,
    effects: &mut Vec<Event>,
) -> Result<ScriptStep, RuleError> {
    let position = state.exploration.as_mut().unwrap();
    let script = position.script.as_mut().unwrap();
    match operation {
        Operation::AdjustStatistics => {
            let extra = span(
                &map.combat
                    .as_ref()
                    .ok_or("story combat tables missing")?
                    .encounters,
                parameter.unsigned_abs() as usize * 10,
                10,
            )?;
            state.warriors =
                story_party::adjust_statistics(&state.warriors, &state.party_mask, extra)?;
        }
        Operation::Confiscate => {
            effects.extend(crate::confiscation::apply(state, content, parameter)?);
            if state.exploration.as_ref().unwrap().presentation.is_some() {
                return Ok(ScriptStep::Pause);
            }
        }
        Operation::Shop => {
            if crate::shop::visit(state, content, parameter)? {
                effects.push(Event::StoryPresented);
                return Ok(ScriptStep::Pause);
            }
        }
        Operation::Clinic => {
            let extra = span(
                &map.combat
                    .as_ref()
                    .ok_or("clinic extra records missing")?
                    .encounters,
                parameter.unsigned_abs() as usize * 10,
                10,
            )?;
            let percent = word(extra, 0)?;
            let sound = word(extra, 2)?;
            let text = word(extra, 4)?;
            if text != 0 {
                return Err(RuleError::Unsupported {
                    action: "story.clinic",
                    dependency: "clinic introductory textbox caller",
                });
            }
            if sound != 0 {
                effects.push(Event::Sound { resource_id: sound });
            }
            effects.extend(crate::clinic::visit(state, percent)?);
            return Ok(ScriptStep::Pause);
        }
        Operation::RevealNote => {
            let index = usize::from(parameter.unsigned_abs());
            let flag = state
                .notes
                .get_mut(index)
                .ok_or("map or note index unavailable")?;
            *flag = 1;
            if parameter > -1 {
                effects.push(Event::Feedback(crate::feedback::NativeFeedback::Flash {
                    address: 0x401c28,
                    x: 30,
                    y: 355,
                    wait: 0,
                    sound: 30005,
                }));
            }
        }
        Operation::SelectedHealth | Operation::PartyHealth => {
            let extra = span(
                &map.combat.as_ref().unwrap().encounters,
                parameter.unsigned_abs() as usize * 10,
                10,
            )?;
            let context = crate::story_health::context(
                &state.warriors,
                state.selected_warrior,
                state.health_context.as_ref(),
            );
            let outcome = crate::story_health::party_health(
                &state.warriors,
                extra,
                operation == Operation::PartyHealth,
                &state.party_mask,
                &context,
                &mut state.random,
            )?;
            state.warriors = outcome.warriors;
            state.health_context = Some(outcome.context);
            let mut party_lost = false;
            for event in outcome.events {
                match event {
                    crate::story_health::HealthEvent::Flash { address, sound } => {
                        effects.push(Event::Feedback(crate::feedback::NativeFeedback::Flash {
                            address,
                            x: 100,
                            y: 30,
                            wait: 100,
                            sound,
                        }))
                    }
                    crate::story_health::HealthEvent::Animation { effect, .. } => {
                        effects.push(Event::OutdoorEffect {
                            effect: i16::from(effect),
                            x: position.x as i16,
                            y: position.y as i16,
                            size: 32,
                            delay: 0,
                        })
                    }
                    crate::story_health::HealthEvent::Sound(resource_id) => {
                        effects.push(Event::Sound { resource_id })
                    }
                    crate::story_health::HealthEvent::Dialogue(id) => {
                        position.presentation = Some(Presentation::Dialogue {
                            id,
                            text: map.dialogue(id)?,
                        });
                        effects.push(Event::StoryPresented);
                    }
                    crate::story_health::HealthEvent::Defeated { warrior, .. } => {
                        if let Some(combat) = state.combat.as_mut() {
                            crate::combat::lifecycle::kill_body(
                                combat,
                                &mut state.warriors,
                                warrior as i16,
                            )?;
                        }
                    }
                    crate::story_health::HealthEvent::PartyLost => {
                        party_lost = true;
                    }
                }
            }
            if party_lost {
                effects.extend(crate::mission::defeat(state, content));
                return Ok(ScriptStep::Pause);
            }
            if position.presentation.is_some() {
                return Ok(ScriptStep::Pause);
            }
        }
        Operation::FilterParty => {
            let extra = span(
                &map.combat.as_ref().unwrap().encounters,
                parameter.unsigned_abs() as usize * 10,
                10,
            )?;
            state.party_mask = crate::story_party::filter_party(
                &state.warriors,
                state.party_mask,
                cursor,
                extra,
                &mut state.random,
            )?;
        }
        Operation::SetRecoveryRestrictions => {
            let extra = span(
                &map.combat
                    .as_ref()
                    .ok_or("script extra records unavailable")?
                    .encounters,
                parameter.unsigned_abs() as usize * 10,
                10,
            )?;
            let restrictions = RecoveryRestrictions::decode(extra);
            state.psi_disabled = restrictions.psi;
            state.enemy_spell_disabled = restrictions.enemy_spells;
            state.energy_recovery_disabled = restrictions.energy;
        }
        Operation::ChangeFatigue => {
            let extra = span(
                &map.combat.as_ref().unwrap().encounters,
                parameter.unsigned_abs() as usize * 10,
                10,
            )?;
            position.fatigue = match FatigueChange::decode(extra)? {
                FatigueChange::Set(value) => value,
                // Native order divides first, then multiplies with WORD wrapping.
                FatigueChange::Percent(percent) => (percent / 100).wrapping_mul(position.fatigue),
                FatigueChange::Keep => position.fatigue,
            };
        }
        Operation::SetMissionFlag => {
            state.flags = story::set_mission_flag(&state.flags, parameter)?;
        }
        Operation::AddMissionScore => {
            state.mission_score = story::add_mission_score(state.mission_score, parameter);
        }
        Operation::AddAlly => {
            let definition = span(
                &map.combat.as_ref().unwrap().monsters,
                parameter as usize * story_allies::ALLY_BYTES,
                story_allies::ALLY_BYTES,
            )?;
            let addition = story_allies::add_ally(
                &state.allies,
                state.ally_count,
                definition,
                state.difficulty,
                &mut state.random,
            )?;
            state.allies = addition.records;
            state.ally_count = addition.count;
            if let Some(warning) = addition.warning {
                effects.push(Event::CollectionRejected { warning });
            } else {
                effects.push(Event::AllyAdded {
                    definition: parameter,
                });
            }
        }
        Operation::RemoveAllies => {
            let removed = crate::story_remove_allies::remove_class(
                &state.allies,
                state.ally_count,
                parameter,
            )?;
            state.allies = removed.records;
            state.ally_count = removed.count;
        }
        Operation::CopyMapEvent => {
            // newland1434d4 copies the current map's mutable header, not Data ED3.
            let slot = usize::try_from(parameter)
                .map_err(|_| RuleError::Invalid("negative event header slot".into()))?;
            let replacement = span(&position.event_headers, slot * 42, 42)?;
            script.record[8..41].copy_from_slice(&replacement[8..41]);
            script.cursor = 0;
        }
        Operation::ReplaceScript => {
            let replacement = map.script(parameter)?;
            script.restart_from(&replacement);
        }
        Operation::LimitedShopOrTreasure => {
            let extra = span(
                &map.combat.as_ref().unwrap().encounters,
                parameter.unsigned_abs() as usize * 10,
                10,
            )?;
            let id = word(extra, 0)?;
            if id <= 0 {
                crate::shop::visit_restricted(state, content, extra)?;
                effects.push(Event::StoryPresented);
            } else {
                state.loot_limit = if word(extra, 4)?.wrapping_add(1) != 0 {
                    Some(word(extra, 2)?)
                } else {
                    None
                };
                effects.extend(present_treasure(state, content, id)?);
            }
            return Ok(ScriptStep::Pause);
        }
        Operation::Treasure => {
            effects.extend(present_treasure(state, content, parameter)?);
            return Ok(ScriptStep::Pause);
        }
        Operation::Experience => {
            let awarded =
                story_inventory::award_story_experience(&state.warriors, i32::from(parameter))?;
            state.warriors = awarded.warriors;
            position.presentation = Some(Presentation::Loot);
            effects.push(Event::StoryPresented);
            return Ok(ScriptStep::Pause);
        }
        _ => unreachable!("opcode routed to wrong story adapter"),
    }
    Ok(ScriptStep::Continue)
}
