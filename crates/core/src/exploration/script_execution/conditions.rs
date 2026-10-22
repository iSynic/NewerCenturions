//! Conditions script operations, preserving original dispatch order.
use super::super::{present_menu, present_typed, unsupported};
use super::{Operation, ScriptStep};
use crate::{
    Event, MapState, Presentation, RuleError, Session, story, story_allies, story_inventory,
    story_variables, travel, typed_encounter,
};
use centurions_content::mission::OpeningMission;
use centurions_content::records::StoryRecord;
use centurions_content::records::{
    AllyClassBranchExtra, ChanceBranchExtra, EnemyRecord, ExtraTable, ItemConsumptionExtra,
    ItemDefinition, MapEffectExtra, MapMutationExtra, ObjectRectangleExtra, PaymentBranchExtra,
    RandomBranchExtra, RegionChangeExtra, RegionTableMut, StoryRecordMut, TravelExtra,
};
use centurions_content::{Content, span, word};

pub(super) fn execute(
    operation: Operation,
    parameter: i16,
    state: &mut Session,
    content: &Content,
    map: &OpeningMission,
    _cursor: usize,
    effects: &mut Vec<Event>,
) -> Result<ScriptStep, RuleError> {
    let opcode = operation.native();
    let position = state.exploration.as_mut().unwrap();
    let script = position.script.as_mut().unwrap();
    match operation {
        Operation::ChangeRegion => {
            let extra =
                ExtraTable(&map.combat.as_ref().unwrap().encounters).record(parameter as usize)?;
            let change = RegionChangeExtra(extra);
            let target = change.map()?;
            let authored = content
                .map(target)
                .ok_or_else(|| RuleError::Invalid("region target map unavailable".into()))?;
            if !(0..20).contains(&change.region()?) {
                return Err(RuleError::Invalid("region index unavailable".into()));
            }
            let regions = if target == position.map {
                &mut position.region_state
            } else {
                &mut state
                    .visited_maps
                    .entry(target)
                    .or_insert_with(|| MapState::authored(authored))
                    .regions
            };
            let region = change.region()? as usize;
            let mut regions = RegionTableMut(regions);
            regions.set_encounter_chance(region, change.encounter_chance());
            let minimum = change.battle_minimum()?;
            if minimum >= 0 {
                regions.set_battle_minimum(region, minimum);
            }
            let maximum = change.battle_maximum()?;
            if maximum >= 0 {
                regions.set_battle_maximum(region, maximum);
            }
        }
        Operation::ClearObjects => {
            let extra = ExtraTable(&map.combat.as_ref().unwrap().encounters)
                .record(parameter.unsigned_abs() as usize)?;
            let rectangle = ObjectRectangleExtra(extra);
            let target = rectangle.map()?;
            let authored = content
                .map_at(state.mission.as_deref(), state.scenario_level, target)
                .ok_or("object clearing map unavailable")?;
            let first = rectangle.first()?;
            let last = rectangle.last()?;
            OpeningMission::index(first.0, first.1)?;
            OpeningMission::index(last.0, last.1)?;
            let objects = if target == position.map {
                &mut position.objects
            } else {
                &mut state
                    .visited_maps
                    .entry(target)
                    .or_insert_with(|| MapState::authored(authored))
                    .objects
            };
            for x in first.0..=last.0 {
                for y in first.1..=last.1 {
                    objects[OpeningMission::index(x, y)?] = 0;
                }
            }
        }
        Operation::ReplaceMapEvent | Operation::ChangeTiles | Operation::SetEventChances => {
            let extra =
                ExtraTable(&map.combat.as_ref().unwrap().encounters).record(parameter as usize)?;
            let target = MapMutationExtra(extra).map()?;
            let authored = content
                .map(target)
                .ok_or_else(|| RuleError::Invalid("map mutation target unavailable".into()))?;
            if operation == Operation::ChangeTiles {
                let mutation = travel::tile_mutation(extra)?;
                if target == position.map {
                    let layer = match mutation.layer {
                        travel::TileLayer::Floor => &mut position.floor,
                        travel::TileLayer::Object => &mut position.objects,
                    };
                    *layer = travel::apply_tile(layer, &mutation)?;
                } else {
                    let records = state
                        .visited_maps
                        .entry(target)
                        .or_insert_with(|| MapState::authored(authored));
                    let layer = match mutation.layer {
                        travel::TileLayer::Floor => &mut records.floor,
                        travel::TileLayer::Object => &mut records.objects,
                    };
                    *layer = travel::apply_tile(layer, &mutation)?;
                }
            } else {
                let headers = if target == position.map {
                    &mut position.event_headers
                } else {
                    &mut state
                        .visited_maps
                        .entry(target)
                        .or_insert_with(|| MapState::authored(authored))
                        .event_headers
                };
                *headers = if operation == Operation::SetEventChances {
                    travel::set_event_chances(headers, extra)?
                } else {
                    let replacement = travel::event_replacement(extra)?;
                    travel::replace_event_script(
                        headers,
                        replacement.slot,
                        &map.script(replacement.script)?,
                    )?
                };
            }
        }
        Operation::Pause => {
            position.presentation = Some(Presentation::ScriptPause);
            effects.push(Event::Feedback(crate::feedback::script_pause()));
            return Ok(ScriptStep::Pause);
        }
        Operation::MapEffect => {
            let extra =
                ExtraTable(&map.combat.as_ref().unwrap().encounters).record(parameter as usize)?;
            let effect = MapEffectExtra(extra);
            effects.push(Event::OutdoorEffect {
                effect: effect.effect()?,
                x: effect.x()?,
                y: effect.y()?,
                size: effect.size()?,
                delay: effect.delay()?,
            });
        }
        Operation::ConsumePartyItems => {
            let tables = map.combat.as_ref().unwrap();
            let extra = ExtraTable(&tables.encounters).record(parameter as usize)?;
            let consumed = story_inventory::consume_party_items_retained(
                &state.warriors,
                content,
                &tables.race_moves,
                extra,
                state.native_work.inventory_record.as_ref(),
            )?;
            if let Some(definition) = consumed.definition {
                state.native_work.loaded_item_field32 =
                    Some(ItemDefinition(&definition).retained_field32()?);
                state.native_work.ammunition_work.item = Some(definition);
            } else if consumed.consumed != 0 {
                let consumption = ItemConsumptionExtra(extra);
                let loaded = if consumption.mode()? == 3 {
                    consumption.replacement()?
                } else {
                    consumption.item()?
                };
                state.native_work.loaded_item_field32 = Some(
                    state
                        .native_work
                        .ammunition_work
                        .load_item(content, loaded)?,
                );
            }
            state.native_work.inventory_record = consumed.record;
            state.warriors = consumed.warriors;
            effects.extend(consumed.events);
        }
        Operation::ChooseWarrior => {
            position.presentation = Some(Presentation::WarriorChoice { extra: parameter });
            effects.push(Event::StoryPresented);
            return Ok(ScriptStep::Pause);
        }
        // newland45, and newland20 which then starts any event at the destination.
        Operation::Travel | Operation::TeleportActivate => {
            let extra =
                ExtraTable(&map.combat.as_ref().unwrap().encounters).record(parameter as usize)?;
            let destination = TravelExtra(extra);
            let target = destination.map(position.map)?;
            let x = destination.x(position.x)?;
            let y = destination.y(position.y)?;
            OpeningMission::index(x, y)?;
            super::lands::enter_land(state, content, target)?;
            let position = state.exploration.as_mut().unwrap();
            position.x = x;
            position.y = y;
            let script = position.script.as_mut().unwrap();
            StoryRecordMut(&mut script.record).relocate(position.map, x, y);
            effects.push(Event::Moved { x, y });
            if destination.sound()? != 0 {
                effects.push(Event::Sound {
                    resource_id: destination.sound()?,
                });
            }
            let paused = destination.dialogue()? != 0;
            if paused {
                position.presentation = Some(Presentation::Dialogue {
                    id: destination.dialogue()?,
                    text: map.dialogue(destination.dialogue()?)?,
                });
                effects.push(Event::StoryPresented);
            }
            if operation == Operation::TeleportActivate
                && !super::lands::activate_destination(state)?
            {
                effects.extend(super::super::abandon_script(state, content)?);
                return Ok(ScriptStep::Pause);
            }
            if paused {
                return Ok(ScriptStep::Pause);
            }
        }
        // newland34 restores the encounter caller's record and resumes after its
        // action without the action-count check; the encounter loop ends.
        Operation::BreakEncounter => {
            let resume = script
                .caller_action
                .map(|action| action.wrapping_add(1))
                .filter(|action| {
                    script.menu.is_some() && (0..StoryRecord::ACTIONS as i16).contains(action)
                })
                .ok_or_else(|| unsupported("story.opcode", "encounter return without a caller"))?;
            script.record = script.menu.take().unwrap().original;
            script.cursor = resume as u8;
        }
        Operation::Menu => {
            effects.push(Event::Sound { resource_id: 20005 });
            let table = if state.menus.is_empty() {
                &map.menus
            } else {
                &state.menus
            };
            position.presentation = present_menu(map, table, script, parameter)?;
            if position.presentation.is_some() {
                effects.push(Event::StoryPresented);
                return Ok(ScriptStep::Pause);
            }
        }
        Operation::TypedMenu => {
            script.menu = None;
            position.presentation = Some(present_typed(
                map,
                if state.typed_encounters.is_empty() {
                    &map.typed_encounters
                } else {
                    &state.typed_encounters
                },
                script,
                parameter,
            )?);
            effects.push(Event::Sound { resource_id: 20005 });
            effects.push(Event::StoryPresented);
            return Ok(ScriptStep::Pause);
        }
        Operation::PartyItemBranch
        | Operation::UpdateVariable
        | Operation::VariableBranch
        | Operation::RandomBranch
        | Operation::AllyClassBranch => {
            let extra =
                ExtraTable(&map.combat.as_ref().unwrap().encounters).record(parameter as usize)?;
            let continuation = match opcode {
                21 => story_variables::party_item_condition(&state.warriors, extra)?,
                76 => {
                    let update = story_variables::update_variable(&state.flags, extra)?;
                    state.flags = update.flags;
                    update.continuation
                }
                77 => story_variables::variable_predicate(&state.flags, extra)?.continuation,
                85 => {
                    let random = RandomBranchExtra(extra);
                    let sound = random.sound()?;
                    if sound != 0 {
                        effects.push(Event::Sound { resource_id: sound });
                    }
                    let id = random.message()?;
                    if id != 0 {
                        effects.push(Event::Message {
                            id,
                            text: map.dialogue(id)?,
                        });
                    }
                    let target = state.random.range(random.minimum()?, random.maximum()?);
                    story_variables::branch(random.mode()?, target)
                }
                _ => {
                    let ally = AllyClassBranchExtra(extra);
                    let matched = state
                        .allies
                        .as_chunks::<{ story_allies::ALLY_BYTES }>()
                        .0
                        .iter()
                        .take(state.ally_count)
                        .any(|record| {
                            i16::from(EnemyRecord::from_bytes(record).definition_id())
                                == ally.class().unwrap()
                        });
                    if matched || ally.unmatched_gate()? == 0 {
                        story_variables::branch(
                            ally.mode()?,
                            if matched {
                                ally.matched_target()?
                            } else {
                                ally.unmatched_target()?
                            },
                        )
                    } else {
                        story_variables::StoryContinuation::Continue
                    }
                }
            };
            match continuation {
                story_variables::StoryContinuation::Continue => {}
                story_variables::StoryContinuation::ReplaceScript(id) => {
                    script.restart_from(&map.script(id)?);
                }
                story_variables::StoryContinuation::Encounter(id) => {
                    script.menu = None;
                    let table = if state.menus.is_empty() {
                        &map.menus
                    } else {
                        &state.menus
                    };
                    position.presentation = present_menu(map, table, script, id)?;
                    if position.presentation.is_some() {
                        effects.push(Event::StoryPresented);
                        return Ok(ScriptStep::Pause);
                    }
                }
                story_variables::StoryContinuation::TypedEncounter(id) => {
                    script.menu = None;
                    position.presentation = Some(present_typed(
                        map,
                        if state.typed_encounters.is_empty() {
                            &map.typed_encounters
                        } else {
                            &state.typed_encounters
                        },
                        script,
                        id,
                    )?);
                    effects.push(Event::StoryPresented);
                    return Ok(ScriptStep::Pause);
                }
                story_variables::StoryContinuation::EndScript => {
                    position.script = None;
                    return Ok(ScriptStep::Pause);
                }
            }
        }
        Operation::DisableMenuEntry => {
            let menu = script.menu.as_ref().ok_or_else(|| {
                RuleError::Invalid("menu mutation has no active encounter".into())
            })?;
            let id = menu.id;
            if state.menus.is_empty() {
                state.menus = map.menus.clone();
            }
            let offset = id as usize * story::MENU_RECORD_BYTES;
            let replacement = story::disable_menu_entry(
                span(&state.menus, offset, story::MENU_RECORD_BYTES)?,
                parameter,
            )?;
            state.menus[offset..offset + story::MENU_RECORD_BYTES].copy_from_slice(&replacement);
            script.record = menu.original.clone();
            position.presentation = present_menu(map, &state.menus, script, id)?;
            if position.presentation.is_some() {
                effects.push(Event::StoryPresented);
                return Ok(ScriptStep::Pause);
            }
        }
        // newland41 rewrites the first 106 bytes of simple encounter e1 in the
        // encounter file with choice byte 96 + e2 - 1 cleared.
        Operation::DisableEncounterChoice => {
            let extra =
                ExtraTable(&map.combat.as_ref().unwrap().encounters).record(parameter as usize)?;
            let choice = word(extra, 2)?;
            let offset = usize::try_from(word(extra, 0)?)
                .ok()
                .map(|id| id * story::MENU_RECORD_BYTES)
                .filter(|offset| offset + story::MENU_RECORD_BYTES <= map.menus.len())
                .filter(|_| (1..=10).contains(&choice))
                .ok_or_else(|| unsupported("story.opcode", "choice outside the encounters"))?;
            if state.menus.is_empty() {
                state.menus = map.menus.clone();
            }
            state.menus[offset + 95 + choice as usize] = 0;
        }
        Operation::PaymentBranch
        | Operation::ItemPrerequisiteBranch
        | Operation::ChanceBranch
        | Operation::FlagBranch
        | Operation::DifficultyBranch
        | Operation::TileBranch => {
            let extra = ExtraTable(&map.combat.as_ref().unwrap().encounters)
                .record(usize::from(parameter.unsigned_abs()))?;
            let jump = if operation == Operation::PaymentBranch {
                let payment = crate::story_inventory::payment::pay(
                    &crate::story_inventory::payment::PaymentState {
                        party_gold: state.party_currency,
                        highest_party_index: state.warriors.len() as i8 - 1,
                        warriors: state.warriors.clone(),
                    },
                    PaymentBranchExtra(extra).cost()?,
                )?;
                state.party_currency = payment.state.party_gold;
                state.warriors = payment.state.warriors;
                for effect in payment.effects {
                    if let crate::story_inventory::payment::PaymentEffect::InsufficientCredits =
                        effect
                    {
                        effects.push(Event::Feedback(crate::feedback::NativeFeedback::Warning {
                            index: 50,
                        }));
                    }
                }
                let paid = payment.paid;
                let condition = PaymentBranchExtra(extra);
                if !paid && condition.condition()? == -1 {
                    script.cursor = 6;
                }
                if matches!(condition.condition()?, 2)
                    || (condition.condition()? == 0 && !paid)
                    || (condition.condition()? == 1 && paid)
                {
                    story::condition_jump(extra)?
                } else {
                    None
                }
            } else if operation == Operation::ItemPrerequisiteBranch {
                if story_inventory::item_prerequisite_matches(&state.warriors, extra)? {
                    story::condition_jump(extra)?
                } else {
                    None
                }
            } else if matches!(
                operation,
                Operation::ChanceBranch | Operation::DifficultyBranch | Operation::TileBranch
            ) {
                let chance = ChanceBranchExtra(extra);
                let passed = match operation {
                    Operation::ChanceBranch => state.random.bound(100) <= chance.percent()?,
                    // newland58 passes at difficulty e1 or harder.
                    Operation::DifficultyBranch => chance.percent()? <= state.difficulty,
                    // newland59 folds the party's floor word into -999..999 in the
                    // encounter-caller local and never compares it with e1.
                    _ => {
                        let index = OpeningMission::index(position.x, position.y)?;
                        let mut tile = position.floor[index];
                        for _ in 0..3 {
                            if tile > 999 {
                                tile -= 1000;
                            }
                        }
                        for _ in 0..3 {
                            if tile < -999 {
                                tile += 1000;
                            }
                        }
                        script.caller_action = Some(tile);
                        true
                    }
                };
                if passed {
                    match chance.action()? {
                        -2 => {
                            script.cursor = 8;
                            None
                        }
                        1 => story::condition_jump(extra)?,
                        2 => Some(story::ConditionalJump::EndMenu),
                        _ => None,
                    }
                } else {
                    None
                }
            } else {
                story::flag_condition_jump(&state.flags, extra)?
            };
            if let Some(jump) = jump {
                match jump {
                    story::ConditionalJump::ReplaceScript(id) => {
                        let replacement = map.script(id)?;
                        script.restart_from(&replacement);
                    }
                    story::ConditionalJump::MenuBranch { branch, start } => {
                        let id = script
                            .menu
                            .as_ref()
                            .ok_or_else(|| {
                                RuleError::Invalid("conditional branch has no active menu".into())
                            })?
                            .id;
                        let table = if state.menus.is_empty() {
                            &map.menus
                        } else {
                            &state.menus
                        };
                        let menu = span(
                            table,
                            id as usize * story::MENU_RECORD_BYTES,
                            story::MENU_RECORD_BYTES,
                        )?;
                        StoryRecordMut(&mut script.record)
                            .write_actions(&story::dispatch_menu_branch(menu, branch as i8 + 1)?);
                        script.cursor = start;
                    }
                    story::ConditionalJump::TypedBranch { branch, start } => {
                        let menu = script
                            .menu
                            .as_ref()
                            .filter(|m| m.typed)
                            .ok_or("conditional branch has no active typed encounter")?;
                        let (id, eliminated) = (menu.id, menu.eliminated);
                        let table = if state.typed_encounters.is_empty() {
                            &map.typed_encounters
                        } else {
                            &state.typed_encounters
                        };
                        let mut record = span(
                            table,
                            id as usize * typed_encounter::RECORD_BYTES,
                            typed_encounter::RECORD_BYTES,
                        )?
                        .to_vec();
                        typed_encounter::eliminate_branches(&mut record, eliminated);
                        StoryRecordMut(&mut script.record).write_actions(
                            &story::dispatch_menu_branch(&record, branch as i8 + 1)?,
                        );
                        script.cursor = start;
                    }
                    story::ConditionalJump::EndMenu => {
                        position.script = None;
                        return Ok(ScriptStep::Pause);
                    }
                    story::ConditionalJump::LastAction => {
                        script.cursor = 7;
                    }
                }
            }
        }
        _ => unreachable!("opcode routed to wrong story adapter"),
    }
    Ok(ScriptStep::Continue)
}
