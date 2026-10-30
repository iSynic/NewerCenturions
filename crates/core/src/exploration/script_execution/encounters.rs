//! Encounter menus and the script branches that jump within them.
use super::super::{active_table, present_menu, present_typed, unsupported};
use super::{ENCOUNTER_SOUND, ExtraIndex, Operation, ScriptStep, load_extra};
use crate::{Event, RuleError, Session, story, story_allies, story_inventory, story_variables};
use centurions_content::mission::OpeningMission;
use centurions_content::records::StoryRecord;
use centurions_content::records::{
    AllyClassBranchExtra, ChanceBranchExtra, DisableChoiceExtra, EncounterMenuMut, EnemyRecord,
    PaymentBranchExtra, RandomBranchExtra, StoryRecordMut,
};
use centurions_content::span;

/// Warning string 50, shown when the party cannot pay (as at the clinic).
const INSUFFICIENT_CREDITS_WARNING: i16 = 50;
// A script cursor names the next of the record's eight actions to run.
/// An unpaid condition -1 payment continues at action 6.
const UNPAID_PAYMENT_ACTION: u8 = 6;
/// A LastAction jump continues at the record's last action.
const LAST_ACTION: u8 = (StoryRecord::ACTIONS - 1) as u8;
/// A passed chance branch with action -2 leaves no action to run.
const END_OF_ACTIONS: u8 = StoryRecord::ACTIONS as u8;

/// Runs the encounter script operations: menus and typed prompts, chance, random, flag, variable,
/// payment, item, difficulty, tile and ally-class branches, and the operations that disable a
/// choice or end the encounter.
pub(super) fn execute(
    operation: Operation,
    parameter: i16,
    state: &mut Session,
    map: &OpeningMission,
    effects: &mut Vec<Event>,
) -> Result<ScriptStep, RuleError> {
    let position = state.exploration.as_mut().unwrap();
    let script = position.script.as_mut().unwrap();
    match operation {
        // Restores the record that ran the encounter and resumes after its action
        // without the action-count check; the encounter loop ends.
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
            script.menu = None;
            effects.push(Event::Sound {
                resource_id: ENCOUNTER_SOUND,
            });
            let table = active_table(&state.menus, &map.menus);
            position.presentation = present_menu(
                map,
                table,
                script,
                parameter,
                &mut state.globals.encounter_headers,
                true,
            )?;
            if position.presentation.is_some() {
                effects.push(Event::StoryPresented);
                return Ok(ScriptStep::Pause);
            }
        }
        Operation::TypedMenu => {
            script.menu = None;
            position.presentation = Some(present_typed(
                map,
                active_table(&state.typed_encounters, &map.typed_encounters),
                script,
                parameter,
                &mut state.globals.encounter_headers,
                true,
            )?);
            effects.push(Event::Sound {
                resource_id: ENCOUNTER_SOUND,
            });
            effects.push(Event::StoryPresented);
            return Ok(ScriptStep::Pause);
        }
        Operation::PartyItemBranch
        | Operation::UpdateVariable
        | Operation::VariableBranch
        | Operation::RandomBranch
        | Operation::AllyClassBranch => {
            let extra = load_extra(&mut state.globals, map, ExtraIndex::Signed(parameter), None)?;
            let continuation = match operation {
                Operation::PartyItemBranch => {
                    story_variables::party_item_condition(&state.warriors, extra)?
                }
                Operation::UpdateVariable => {
                    let update = story_variables::update_variable(&state.flags, extra)?;
                    state.flags = update.flags;
                    update.continuation
                }
                Operation::VariableBranch => {
                    story_variables::variable_predicate(&state.flags, extra)?.continuation
                }
                Operation::RandomBranch => {
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
                        .holdover
                        .as_chunks::<{ story_allies::ALLY_BYTES }>()
                        .0
                        .iter()
                        .take(state.held_over)
                        .any(|record| {
                            i16::from(EnemyRecord::from_bytes(record).monster_id())
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
                    let table = active_table(&state.menus, &map.menus);
                    position.presentation = present_menu(
                        map,
                        table,
                        script,
                        id,
                        &mut state.globals.encounter_headers,
                        true,
                    )?;
                    if position.presentation.is_some() {
                        effects.push(Event::StoryPresented);
                        return Ok(ScriptStep::Pause);
                    }
                }
                story_variables::StoryContinuation::TypedEncounter(id) => {
                    script.menu = None;
                    position.presentation = Some(present_typed(
                        map,
                        active_table(&state.typed_encounters, &map.typed_encounters),
                        script,
                        id,
                        &mut state.globals.encounter_headers,
                        true,
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
            let original = menu.original.clone();
            state.globals.encounter_headers.disable_choice(parameter)?;
            let header = state.globals.encounter_headers.header(false);
            if state.menus.is_empty() {
                state.menus = map.menus.clone();
            }
            let offset = id as usize * story::MENU_RECORD_BYTES;
            span(&state.menus, offset, header.len())?;
            state.menus[offset..offset + header.len()].copy_from_slice(header);
            script.record = original;
            position.presentation = present_menu(
                map,
                &state.menus,
                script,
                id,
                &mut state.globals.encounter_headers,
                false,
            )?;
            if position.presentation.is_some() {
                effects.push(Event::StoryPresented);
                return Ok(ScriptStep::Pause);
            }
        }
        // Rewrites simple encounter e1 in the encounter file with choice e2
        // disabled.
        Operation::DisableEncounterChoice => {
            let extra = load_extra(&mut state.globals, map, ExtraIndex::Signed(parameter), None)?;
            let extra = DisableChoiceExtra(extra);
            let choice = extra.choice()?;
            let offset = usize::try_from(extra.encounter()?)
                .ok()
                .map(|id| id * story::MENU_RECORD_BYTES)
                .filter(|offset| offset + story::MENU_RECORD_BYTES <= map.menus.len())
                .filter(|_| (1..=10).contains(&choice))
                .ok_or_else(|| unsupported("story.opcode", "choice outside the encounters"))?;
            if state.menus.is_empty() {
                state.menus = map.menus.clone();
            }
            EncounterMenuMut(&mut state.menus[offset..]).clear_choice(choice as usize);
        }
        Operation::PaymentBranch
        | Operation::ItemPrerequisiteBranch
        | Operation::ChanceBranch
        | Operation::FlagBranch
        | Operation::DifficultyBranch
        | Operation::TileBranch => {
            let extra = load_extra(
                &mut state.globals,
                map,
                ExtraIndex::Magnitude(parameter),
                None,
            )?;
            let jump = if operation == Operation::PaymentBranch {
                let payment = crate::story::inventory::payment::pay(
                    &crate::story::inventory::payment::PaymentState {
                        party_gold: state.party_currency,
                        highest_party_index: state.warriors.len() as i8 - 1,
                        warriors: state.warriors.clone(),
                    },
                    PaymentBranchExtra(extra).cost()?,
                )?;
                state.party_currency = payment.state.party_gold;
                state.warriors = payment.state.warriors;
                for effect in payment.effects {
                    if let crate::story::inventory::payment::PaymentEffect::InsufficientCredits =
                        effect
                    {
                        effects.push(Event::Feedback(crate::feedback::GameFeedback::Warning {
                            index: INSUFFICIENT_CREDITS_WARNING,
                        }));
                    }
                }
                let paid = payment.paid;
                let condition = PaymentBranchExtra(extra);
                if !paid && condition.condition()? == -1 {
                    script.cursor = UNPAID_PAYMENT_ACTION;
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
                    // Passes at difficulty e1 or harder.
                    Operation::DifficultyBranch => chance.percent()? <= state.difficulty,
                    // Folds the party's floor value into -999..999 and keeps it
                    // in the encounter-caller slot. It never compares it with e1,
                    // so the branch always passes.
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
                            script.cursor = END_OF_ACTIONS;
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
                        StoryRecordMut(&mut script.record).write_actions(
                            &story::dispatch_menu_branch(
                                state.globals.encounter_headers.header(false),
                                branch as i8 + 1,
                            )?,
                        );
                        script.cursor = start;
                    }
                    story::ConditionalJump::TypedBranch { branch, start } => {
                        StoryRecordMut(&mut script.record).write_actions(
                            &story::dispatch_menu_branch(
                                state.globals.encounter_headers.header(true),
                                branch as i8 + 1,
                            )?,
                        );
                        script.cursor = start;
                    }
                    story::ConditionalJump::EndMenu => {
                        position.script = None;
                    }
                    story::ConditionalJump::LastAction => {
                        script.cursor = LAST_ACTION;
                    }
                }
            }
        }
        _ => unreachable!("opcode routed to wrong story adapter"),
    }
    Ok(ScriptStep::Continue)
}
