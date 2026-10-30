//! Script operations that start, change or end battles.
use super::super::{StoryBattle, begin_story_battle, finish_script, unsupported};
use super::{ExtraIndex, Operation, ScriptStep, load_extra};
use crate::{Event, Presentation, RuleError, Session, combat, mission};
use centurions_content::Content;
use centurions_content::mission::OpeningMission;
use centurions_content::records::{SpellInfoRecord, StoryRecord};
use centurions_content::word;

/// Runs the script operations that start, change or end battles: group, selected and NPC fights,
/// summoning and replacing actors, destroying or neutralizing them, party and selected spells,
/// ending the fight, and the stage, schedule and cleanup operations that go with them.
pub(super) fn execute(
    operation: Operation,
    parameter: i16,
    state: &mut Session,
    content: &Content,
    map: &OpeningMission,
    effects: &mut Vec<Event>,
) -> Result<ScriptStep, RuleError> {
    let turn_owns = state
        .turn_dispatch
        .as_ref()
        .is_some_and(|frame| frame.owns_script(state));
    let out_of_combat_round = state
        .round_dispatch
        .as_ref()
        .is_some_and(|frame| frame.out_of_combat && frame.owns_script(state));
    let position = state.exploration.as_mut().unwrap();
    let script = position.script.as_mut().unwrap();
    match operation {
        Operation::CombatCleanup => {
            if turn_owns {
                crate::combat::runtime::mark_turn_cleanup(state);
            }
            effects.extend(crate::story::cleanup::begin(state, content, parameter)?);
            if state
                .scripted_cleanup
                .as_ref()
                .is_some_and(crate::story::cleanup::Stage::pauses)
            {
                return Ok(ScriptStep::Pause);
            }
        }
        Operation::UndoLastStep => {
            // Undoing the last step returns before event persistence and the
            // destination.
            position.x -= i32::from(position.last_move[0]);
            position.y -= i32::from(position.last_move[1]);
            OpeningMission::index(position.x, position.y)?;
            position.script = None;
            state.movement_continuation = None;
            state.time_continuation = None;
            effects.push(Event::Moved {
                x: position.x,
                y: position.y,
            });
            if turn_owns {
                effects.extend(super::super::finish_dispatch(state, content)?.unwrap_or_default());
            }
            return Ok(ScriptStep::Pause);
        }
        Operation::ReplaceActor => {
            let (events, progress) = crate::story::actor::begin(state, content, parameter)?;
            effects.extend(events);
            match progress {
                crate::story::actor::Progress::Waiting => return Ok(ScriptStep::Pause),
                crate::story::actor::Progress::Stop => {
                    effects.extend(finish_script(state, content, false)?);
                    return Ok(ScriptStep::Pause);
                }
                crate::story::actor::Progress::Continue => {}
            }
        }
        Operation::EndCombat
        | Operation::NeutralizeActors
        | Operation::SetFactionCondition
        | Operation::SummonActors
        | Operation::RemoveActorClass => {
            let (events, stopped) =
                crate::story::combat::apply(state, content, operation, parameter)?;
            effects.extend(events);
            if stopped {
                effects.extend(finish_script(state, content, false)?);
                return Ok(ScriptStep::Pause);
            }
        }
        // During a battle, destroys every monster, living or not, of each nonzero
        // creature category. Records past the battle roster count as empty.
        Operation::DestroyCategories => {
            if state.phase == crate::Phase::Combat {
                let extra = load_extra(
                    &mut state.globals,
                    map,
                    ExtraIndex::Magnitude(parameter),
                    None,
                )?;
                let codes = (0..5)
                    .map(|i| word(extra, i * 2))
                    .collect::<Result<Vec<_>, _>>()?;
                let combat = state.combat.as_mut().ok_or("combat story roster missing")?;
                if codes.iter().any(|code| !(0..=120).contains(code)) {
                    return Err(unsupported("story.opcode", "category past the NPC record"));
                }
                if codes.iter().all(|&code| code == 0) && combat.roster.records.len() < 100 {
                    return Err(unsupported("story.opcode", "empty NPC records destroyed"));
                }
                for index in 0..combat.roster.records.len() {
                    let record = combat.roster.records[index].record();
                    if codes
                        .iter()
                        .all(|&code| code == 0 || record.has_creature_category(code as usize - 1))
                    {
                        combat::lifecycle::destroy_monster(combat, index)?;
                        effects.push(Event::CombatantRemoved {
                            body: index as i16 + 10,
                        });
                    }
                }
            }
        }
        Operation::ReplaceRoundScript => {
            // The operation does not check who ran it: outside a round script it
            // uses the slot and round state the last getup left.
            let out_of_combat_round = out_of_combat_round
                || state.round_dispatch.is_none()
                    && state.combat.is_none()
                    && !state.globals.round_scripts.is_unknown();
            // In a live battle with no round script the slot is the one getup last
            // scanned, read from the live round scripts.
            let live_slot = if out_of_combat_round {
                None
            } else if state.round_dispatch.is_none() {
                state.globals.round_scripts.slot
            } else {
                Some(
                    state
                        .combat
                        .as_ref()
                        .ok_or("round combat missing")?
                        .round_macros
                        .pending
                        .as_ref()
                        .ok_or("round program missing")?
                        .slot
                        .into(),
                )
            };
            let extra = load_extra(
                &mut state.globals,
                map,
                ExtraIndex::Magnitude(parameter),
                Some("round tables missing"),
            )?;
            let target = if out_of_combat_round {
                let scripts = &mut state.globals.round_scripts;
                combat::round_stories::replacement_slot(
                    scripts.counter,
                    scripts.slots.as_mut(),
                    scripts.slot,
                    extra,
                    &mut state.random,
                )?
            } else if let Some(arena) = state.combat.as_mut() {
                let scripts = &mut arena.round_macros;
                combat::round_stories::replacement_slot(
                    Some(scripts.counter),
                    Some(&mut scripts.slots),
                    live_slot,
                    extra,
                    &mut state.random,
                )?
            } else {
                // Outside any battle there is no round state for the chance to read.
                combat::round_stories::replacement_slot(None, None, None, extra, &mut state.random)?
            };
            let Some(target) = target else {
                effects.extend(finish_script(state, content, false)?);
                return Ok(ScriptStep::Pause);
            };
            script.restart_from(&map.script(target)?);
        }
        Operation::SelectedSpell | Operation::PartySpell => {
            let extra = load_extra(
                &mut state.globals,
                map,
                ExtraIndex::Magnitude(parameter),
                None,
            )?;
            effects.extend(crate::story::spell::apply(
                state,
                content,
                extra,
                operation == Operation::PartySpell,
            )?);
            if state.globals.health_context.kill_party >= state.warriors.len() as i8 {
                effects.extend(crate::mission::defeat(state, content)?);
                return Ok(ScriptStep::Pause);
            }
        }
        Operation::SelectPartyBySave => {
            let extra = load_extra(
                &mut state.globals,
                map,
                ExtraIndex::Magnitude(parameter),
                None,
            )?;
            let (packet, in_spell) = state
                .combat
                .as_ref()
                .map_or((&state.globals.spell_info, state.globals.in_spell), |c| {
                    (&c.spell_info, c.in_spell)
                });
            let mind_shield = crate::combat::party_buffs::mind_shield(&state.party_condition)?;
            let context = crate::story::party::PartySaveContext {
                casting: false,
                packet_type: SpellInfoRecord(packet).cannot() as i8,
                bonus: SpellInfoRecord(packet).save_bonus(),
                bonus_per_attack: SpellInfoRecord(packet).save_bonus_per_attack() as i8,
                attack_count: state.globals.attack_count,
                mind_shield,
            };
            state.globals.party_track = crate::story::party::select_party_with_ready(
                &state.warriors,
                state.char_select_new,
                extra,
                context,
                in_spell,
                &mut state.random,
            )?;
        }
        Operation::EditSchedule => {
            let extra_index = usize::try_from(parameter)
                .map_err(|_| RuleError::Invalid("negative schedule extra index".into()))?;
            let extra = load_extra(
                &mut state.globals,
                map,
                ExtraIndex::Exact(extra_index),
                None,
            )?;
            crate::story::schedule::edit(&mut position.schedule, extra, position.day)?;
        }
        Operation::GroupBattle => {
            effects.extend(begin_story_battle(
                state,
                content,
                StoryBattle::Group,
                parameter,
            )?);
            return Ok(ScriptStep::Pause);
        }
        Operation::SelectedBattle => {
            effects.extend(begin_story_battle(
                state,
                content,
                StoryBattle::Selected,
                parameter,
            )?);
            return Ok(ScriptStep::Pause);
        }
        Operation::NpcBattle => {
            effects.extend(begin_story_battle(
                state,
                content,
                StoryBattle::Npc,
                parameter,
            )?);
            return Ok(ScriptStep::Pause);
        }
        Operation::StageTransition => {
            if parameter > 0 {
                position.presentation = Some(Presentation::StageChange { parameter });
                effects.push(Event::StoryPresented);
                return Ok(ScriptStep::Pause);
            }
            effects.extend(mission::begin_stage(state, content, parameter)?);
            return Ok(ScriptStep::Pause);
        }
        Operation::ContinueIfMonster => {
            let combat = state.combat.as_ref().ok_or_else(|| {
                unsupported("story.opcode", "monster test outside a battle macro")
            })?;
            if !crate::story::combat::monster_present(combat, parameter)? {
                // The script returns at once; the battle script's owner then finishes.
                script.cursor = StoryRecord::ACTIONS as u8;
            }
        }
        _ => unreachable!("opcode routed to wrong story adapter"),
    }
    Ok(ScriptStep::Continue)
}
