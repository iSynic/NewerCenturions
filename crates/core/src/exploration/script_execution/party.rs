//! Script operations that change the party, its members and their gear.
use super::super::{present_treasure, present_treasure_record};
use super::{ExtraIndex, Operation, ScriptStep, load_extra};
use crate::combat::spells::RestrictionMemoryView;
use crate::{
    Event, Presentation, RuleError, Session, story, story_allies, story_inventory, story_party,
};
use centurions_content::mission::OpeningMission;
use centurions_content::records::{
    ClinicExtra, DropCreditsExtra, EventHeaderTable, FatigueChange, GiveConditionExtra,
    ItemConsumptionExtra, ItemDefinition, LimitedTreasureExtra, PickByCasteExtra, PickByTraitExtra,
    RandomTreasureExtra, RecoveryRestrictions, StoryRecordMut, VictoryPointsExtra,
};
use centurions_content::{Content, span};

/// Runs the script operations that change the party: its members and conditions, gear, money,
/// allies, shops and treasure, fatigue, experience and healing, and the mission's score and flags.
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
            let extra = load_extra(
                &mut state.globals,
                map,
                ExtraIndex::Magnitude(parameter),
                Some("story combat tables missing"),
            )?;
            state.warriors =
                story_party::adjust_statistics(&state.warriors, &state.globals.party_track, extra)?;
        }
        Operation::Confiscate => {
            // The restore loop counts its items in the slot that holds the
            // encounter's caller.
            let restored = state
                .confiscated
                .as_ref()
                .filter(|saved| parameter == 0 && saved.active)
                .map(|saved| {
                    let slots = &saved.items[..state.warriors.len() * 100];
                    slots.iter().filter(|item| item.id() != 0).count() as i16
                });
            effects.extend(crate::confiscation::apply(state, content, parameter)?);
            if restored.is_some() {
                let position = state.exploration.as_mut().unwrap();
                position.script.as_mut().unwrap().caller_action = restored;
            }
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
            let extra = load_extra(
                &mut state.globals,
                map,
                ExtraIndex::Magnitude(parameter),
                Some("clinic extra records missing"),
            )?;
            let clinic = ClinicExtra(extra);
            let percent = clinic.percent()?;
            let sound = clinic.sound()?;
            let text = clinic.text()?;
            state.field_controls.get_or_insert_default().clinic = Some(percent);
            if sound != 0 {
                effects.push(Event::Sound { resource_id: sound });
            }
            if text != 0 {
                let text_body = super::super::dialogue(map, &state.warriors, text)?;
                state.exploration.as_mut().unwrap().presentation =
                    Some(Presentation::ClinicEntry { percent });
                effects.push(Event::Message {
                    id: text,
                    text: text_body,
                });
                return Ok(ScriptStep::Pause);
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
                effects.push(Event::Feedback(crate::feedback::GameFeedback::Flash {
                    address: 0x401c28,
                    x: 30,
                    y: 355,
                    wait: 0,
                    sound: 30005,
                }));
            }
        }
        Operation::SelectedHealth | Operation::PartyHealth => {
            let extra = load_extra(
                &mut state.globals,
                map,
                ExtraIndex::Magnitude(parameter),
                None,
            )?;
            let context = crate::story::health::reselected_context(
                state.char_select_new,
                &state.globals.health_context,
            );
            let outcome = crate::story::health::party_health(
                &state.warriors,
                extra,
                operation == Operation::PartyHealth,
                &state.globals.party_track,
                &context,
                &mut state.random,
            )?;
            state.warriors = outcome.warriors;
            state.globals.health_context = outcome.context;
            let mut party_lost = false;
            for event in outcome.events {
                match event {
                    crate::story::health::HealthEvent::Flash { address, sound } => {
                        effects.push(Event::Feedback(crate::feedback::GameFeedback::Flash {
                            address,
                            x: 100,
                            y: 30,
                            wait: 100,
                            sound,
                        }))
                    }
                    crate::story::health::HealthEvent::Animation { effect, .. } => {
                        effects.push(Event::OutdoorEffect {
                            effect: i16::from(effect),
                            x: position.x as i16,
                            y: position.y as i16,
                            size: 32,
                            delay: 0,
                        })
                    }
                    crate::story::health::HealthEvent::Sound(resource_id) => {
                        effects.push(Event::Sound { resource_id })
                    }
                    crate::story::health::HealthEvent::Dialogue(id) => {
                        position.presentation = Some(Presentation::Dialogue {
                            id,
                            text: map.dialogue(id)?,
                        });
                        effects.push(Event::StoryPresented);
                    }
                    crate::story::health::HealthEvent::Defeated { warrior, .. } => {
                        if let Some(combat) = state.combat.as_mut() {
                            crate::combat::lifecycle::kill_body(
                                combat,
                                &mut state.warriors,
                                warrior as i16,
                            )?;
                        }
                    }
                    crate::story::health::HealthEvent::PartyLost => {
                        party_lost = true;
                    }
                }
            }
            if party_lost {
                effects.extend(crate::mission::defeat(state, content)?);
                return Ok(ScriptStep::Pause);
            }
            if position.presentation.is_some() {
                return Ok(ScriptStep::Pause);
            }
        }
        Operation::FilterParty => {
            let extra = load_extra(
                &mut state.globals,
                map,
                ExtraIndex::Magnitude(parameter),
                None,
            )?;
            let memory = RestrictionMemoryView::ordinary_startup(
                &map.item_cache,
                crate::combat::spells::loaded_item_restriction(&state.globals.loaded_items)?,
            )
            .with_alias_owners(crate::combat::warrior_alias::session_owners!(state))
            .with_definitions(&content.items);
            state.globals.party_track = crate::story::party::filter_party(
                &state.warriors,
                state.globals.party_track,
                cursor,
                extra,
                &mut state.random,
                memory,
            )?;
        }
        Operation::SetRecoveryRestrictions => {
            let extra = load_extra(
                &mut state.globals,
                map,
                ExtraIndex::Magnitude(parameter),
                Some("script extra records unavailable"),
            )?;
            let restrictions = RecoveryRestrictions::decode(extra);
            state.psi_disabled = restrictions.psi;
            state.enemy_spell_disabled = restrictions.enemy_spells;
            state.energy_recovery_disabled = restrictions.energy;
        }
        Operation::ChangeFatigue => {
            let extra = load_extra(
                &mut state.globals,
                map,
                ExtraIndex::Magnitude(parameter),
                None,
            )?;
            position.fatigue = match FatigueChange::decode(extra)? {
                FatigueChange::Set(value) => value,
                // Divides first, then multiplies with 16-bit wrapping.
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
                &state.holdover,
                state.held_over,
                definition,
                state.difficulty,
                &mut state.random,
            )?;
            state.holdover = addition.records;
            state.held_over = addition.count;
            if let Some(warning) = addition.warning {
                effects.push(Event::CollectionRejected { warning });
            } else {
                effects.push(Event::AllyAdded {
                    definition: parameter,
                });
            }
        }
        Operation::RemoveAllies => {
            let removed = crate::story::remove_allies::remove_class(
                &state.holdover,
                state.held_over,
                parameter,
            )?;
            state.holdover = removed.records;
            state.held_over = removed.count;
        }
        Operation::CopyMapEvent => {
            // Copies from the current land's changed headers, not the scenario's.
            let slot = usize::try_from(parameter)
                .map_err(|_| RuleError::Invalid("negative event header slot".into()))?;
            let replacement = EventHeaderTable(&position.event_headers).record(slot)?;
            StoryRecordMut(&mut script.record).copy_actions_and_byte40(replacement);
            script.cursor = 0;
        }
        Operation::ReplaceScript => {
            let replacement = map.script(parameter)?;
            script.restart_from(&replacement);
        }
        Operation::LimitedShopOrTreasure => {
            let extra = load_extra(
                &mut state.globals,
                map,
                ExtraIndex::Magnitude(parameter),
                None,
            )?;
            let limited = LimitedTreasureExtra(extra);
            let id = limited.treasure()?;
            if id <= 0 {
                crate::shop::visit_restricted(state, content, extra)?;
                effects.push(Event::StoryPresented);
            } else {
                state.loot_limit = if limited.limit_mode()?.wrapping_add(1) != 0 {
                    Some(limited.limit()?)
                } else {
                    None
                };
                effects.extend(present_treasure(state, content, id)?);
            }
            return Ok(ScriptStep::Pause);
        }
        // Clears a positive duration of condition e2 for every member, then adds e3
        // for members chosen by e1 (all, picked, stamina above -10) with sound e4,
        // the portrait effect and showresults code e2 + 1.
        Operation::GiveCondition => {
            let extra = load_extra(&mut state.globals, map, ExtraIndex::Signed(parameter), None)?;
            story_party::give_condition(
                &mut state.warriors,
                &state.globals.party_track,
                GiveConditionExtra(extra),
                effects,
            )?;
        }
        // Clears the picks, then picks members by trait.
        Operation::PickByTrait => {
            let extra = load_extra(&mut state.globals, map, ExtraIndex::Signed(parameter), None)?;
            state.globals.party_track = story_party::pick_by_trait(
                &state.warriors,
                PickByTraitExtra(extra),
                |race, code| super::branches::race_class(map, race, code),
                |caste| super::branches::caste_class(map, caste),
            )?;
        }
        Operation::PickByCaste => {
            let extra = load_extra(&mut state.globals, map, ExtraIndex::Signed(parameter), None)?;
            state.globals.party_track =
                story_party::pick_by_caste(&state.warriors, PickByCasteExtra(extra))?;
        }
        // Clears the carried credits of every member, or of picked members when e2
        // is set; the wealth type e1 is not read.
        Operation::DropMemberCredits => {
            let extra = load_extra(&mut state.globals, map, ExtraIndex::Signed(parameter), None)?;
            let picked_only = DropCreditsExtra(extra).picked_only()?;
            for (member, warrior) in state.warriors.iter_mut().enumerate() {
                if !picked_only || state.globals.party_track[member] != 0 {
                    warrior.record_mut().set_carried_credits(0);
                }
            }
        }
        Operation::TakeVictoryPoints => {
            let extra = load_extra(&mut state.globals, map, ExtraIndex::Signed(parameter), None)?;
            story_party::take_victory_points(
                &mut state.warriors,
                &state.globals.party_track,
                cursor,
                VictoryPointsExtra(extra),
            )?;
        }
        // Empties every member's inventory, then redraws.
        Operation::DropAllEquipment => {
            let mut warriors = state.warriors.clone();
            let mut loaded = None;
            let mut globals = vec![];
            for warrior in &mut warriors {
                let (emptied, last, writes) =
                    crate::equipment::drop_all(warrior, content, map, effects)?;
                *warrior = emptied;
                loaded = last.or(loaded);
                globals.extend(writes);
            }
            state.apply_equipment_globals(&globals)?;
            state.warriors = warriors;
            if let Some(item) = loaded {
                state.load_item(content, item)?;
            }
        }
        Operation::AlterShop => {
            let extra = load_extra(&mut state.globals, map, ExtraIndex::Signed(parameter), None)?;
            crate::shop::alter(state, content, extra)?;
        }
        Operation::RandomTreasure => {
            let extra = load_extra(&mut state.globals, map, ExtraIndex::Signed(parameter), None)?;
            let (count, record) = crate::story::treasure::random_items(
                &mut state.random,
                RandomTreasureExtra(extra),
            )?;
            let position = state.exploration.as_mut().unwrap();
            position.script.as_mut().unwrap().caller_action = Some(count);
            effects.extend(present_treasure_record(state, content, &record)?);
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
            // The amount is the experience the treasure window hands out.
            state.booty_tally = crate::BootyTally {
                credits: 0,
                share: awarded.share,
                experience: i32::from(parameter),
            };
            let position = state.exploration.as_mut().unwrap();
            position.presentation = Some(Presentation::Loot);
            effects.push(Event::StoryPresented);
            return Ok(ScriptStep::Pause);
        }
        Operation::ConsumePartyItems => {
            let tables = map.combat.as_ref().unwrap();
            let extra = load_extra(&mut state.globals, map, ExtraIndex::Signed(parameter), None)?;
            let consumed = story_inventory::consume_party_items_with_record(
                &state.warriors,
                content,
                &tables.race_moves,
                extra,
                Some(&state.globals.inventory_record),
            )?;
            state.apply_equipment_globals(&consumed.global_writes)?;
            if let Some(definition) = consumed.definition {
                state.globals.loaded_item_field32 =
                    ItemDefinition(&definition).ammunition_caliber()?;
                state.globals.loaded_items.item = Some(definition);
            } else if consumed.consumed != 0 {
                let consumption = ItemConsumptionExtra(extra);
                let loaded = if consumption.mode()? == 3 {
                    consumption.replacement()?
                } else {
                    consumption.item()?
                };
                state.globals.loaded_item_field32 =
                    state.globals.loaded_items.load_item(content, loaded)?;
            }
            if let Some(record) = consumed.record {
                state.globals.inventory_record = record;
            }
            state.warriors = consumed.warriors;
            effects.extend(consumed.events);
        }
        Operation::ResetExperience => {
            effects.extend(super::super::loot::reset_experience(state, content)?);
            return Ok(ScriptStep::Pause);
        }
        _ => unreachable!("opcode routed to wrong story adapter"),
    }
    Ok(ScriptStep::Continue)
}
