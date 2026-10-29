//! Battles script operations, preserving original dispatch order.
use super::super::{StoryBattle, begin_story_battle, finish_script, unsupported};
use super::{Operation, ScriptStep};
use crate::{Event, Presentation, RuleError, Session, combat, mission};
use centurions_content::Content;
use centurions_content::mission::OpeningMission;
use centurions_content::records::{AttackPacket, ExtraTable};
use centurions_content::word;

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
    let turn_owns = state
        .turn_dispatch
        .as_ref()
        .is_some_and(|frame| frame.owns_script(state));
    let retained_round = state
        .round_dispatch
        .as_ref()
        .is_some_and(|frame| frame.retained && frame.owns_script(state));
    let position = state.exploration.as_mut().unwrap();
    let script = position.script.as_mut().unwrap();
    match operation {
        Operation::CombatCleanup => {
            if turn_owns {
                crate::combat::runtime::mark_turn_cleanup(state);
            }
            effects.extend(crate::story_cleanup::begin(state, content, parameter)?);
            if state
                .scripted_cleanup
                .as_ref()
                .is_some_and(crate::story_cleanup::Stage::pauses)
            {
                return Ok(ScriptStep::Pause);
            }
        }
        Operation::UndoLastStep => {
            // newland141340 returns before door persistence and destination handling.
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
            let (events, progress) = crate::story_actor::begin(state, content, parameter)?;
            effects.extend(events);
            match progress {
                crate::story_actor::Progress::Waiting => return Ok(ScriptStep::Pause),
                crate::story_actor::Progress::Stop => {
                    effects.extend(finish_script(state, content, false)?);
                    return Ok(ScriptStep::Pause);
                }
                crate::story_actor::Progress::Continue => {}
            }
        }
        Operation::EndCombat
        | Operation::NeutralizeActors
        | Operation::SetFactionCondition
        | Operation::SummonActors
        | Operation::RemoveActorClass => {
            let (events, stopped) = crate::story_combat::apply(state, content, opcode, parameter)?;
            effects.extend(events);
            if stopped {
                effects.extend(finish_script(state, content, false)?);
                return Ok(ScriptStep::Pause);
            }
        }
        // newland121 destroys every NPC record, living or not, that carries each nonzero
        // creature-category code (BYTE 0xa8 + code - 1), only during a battle. Records
        // past the battle roster are taken as empty.
        Operation::DestroyCategories => {
            if state.phase == crate::Phase::Combat {
                let extra =
                    ExtraTable(&map.combat.as_ref().unwrap().encounters).by_magnitude(parameter)?;
                state.native_work.extra_record = extra.to_vec();
                let codes = (0..5)
                    .map(|i| word(extra, i * 2))
                    .collect::<Result<Vec<_>, _>>()?;
                let combat = state.combat.as_mut().ok_or("combat story roster missing")?;
                if codes.iter().any(|code| !(0..=120).contains(code)) {
                    return Err(unsupported("story.opcode", "category past the NPC record"));
                }
                if codes.iter().all(|&code| code == 0) && combat.enemies.records.len() < 100 {
                    return Err(unsupported("story.opcode", "empty NPC records destroyed"));
                }
                for index in 0..combat.enemies.records.len() {
                    let record = combat.enemies.records[index].record();
                    if codes
                        .iter()
                        .all(|&code| code == 0 || record.has_creature_category(code as usize - 1))
                    {
                        combat::lifecycle::destroy_enemy(combat, index)?;
                        effects.push(Event::CombatantRemoved {
                            body: index as i16 + 10,
                        });
                    }
                }
            }
        }
        Operation::ReplaceRoundScript => {
            // newland 0x7e has no caller test: outside a round frame it uses the shared
            // slot WORD408576 and the retained round work the last Getup left.
            let retained_round = retained_round
                || state.round_dispatch.is_none()
                    && state.combat.is_none()
                    && !state.native_work.round_work.is_unknown();
            // In a live battle with no round frame the shared WORD names the slot
            // Getup last scanned; the live round scripts are its arrays.
            let live_slot = if retained_round {
                None
            } else if state.round_dispatch.is_none() {
                state.native_work.round_work.slot
            } else {
                Some(
                    state
                        .combat
                        .as_ref()
                        .ok_or("round combat missing")?
                        .round_scripts
                        .pending
                        .as_ref()
                        .ok_or("round program missing")?
                        .slot
                        .into(),
                )
            };
            let extra = ExtraTable(
                &map.combat
                    .as_ref()
                    .ok_or("round tables missing")?
                    .encounters,
            )
            .record(parameter.unsigned_abs() as usize)?;
            state.native_work.extra_record = extra.to_vec();
            let target = if retained_round {
                let work = &mut state.native_work.round_work;
                combat::round_stories::replacement_slot(
                    work.counter,
                    work.slots.as_mut(),
                    work.slot,
                    extra,
                    &mut state.random,
                )?
            } else if let Some(arena) = state.combat.as_mut() {
                let scripts = &mut arena.round_scripts;
                combat::round_stories::replacement_slot(
                    Some(scripts.counter),
                    Some(&mut scripts.slots),
                    live_slot,
                    extra,
                    &mut state.random,
                )?
            } else {
                // Outside any battle the session has no round work; the chance arm reads none.
                combat::round_stories::replacement_slot(None, None, None, extra, &mut state.random)?
            };
            let Some(target) = target else {
                effects.extend(finish_script(state, content, false)?);
                return Ok(ScriptStep::Pause);
            };
            script.restart_from(&map.script(target)?);
        }
        Operation::SelectedSpell | Operation::PartySpell => {
            let extra = ExtraTable(&map.combat.as_ref().unwrap().encounters)
                .record(parameter.unsigned_abs() as usize)?;
            state.native_work.extra_record = extra.to_vec();
            effects.extend(crate::story_spell::apply(
                state,
                content,
                extra,
                operation == Operation::PartySpell,
            )?);
            if state.native_work.health_context.unavailable >= state.warriors.len() as i8 {
                effects.extend(crate::mission::defeat(state, content)?);
                return Ok(ScriptStep::Pause);
            }
        }
        Operation::SelectPartyBySave => {
            let extra = ExtraTable(&map.combat.as_ref().unwrap().encounters)
                .record(parameter.unsigned_abs() as usize)?;
            state.native_work.extra_record = extra.to_vec();
            let (packet, ready) = state.combat.as_ref().map_or(
                (&state.native_work.attack_packet, state.native_work.ready),
                |c| (&c.attack_packet, c.ready),
            );
            let mind_shield = crate::combat::party_buffs::mind_shield(&state.party_buffs)?;
            let context = crate::story_party::PartySaveContext {
                casting: false,
                packet_type: AttackPacket(packet).saving_throw_mode() as i8,
                bonus: AttackPacket(packet).save_bonus(),
                bonus_per_attack: AttackPacket(packet).save_bonus_per_attack() as i8,
                attack_count: state.native_work.attack_count,
                mind_shield,
            };
            state.native_work.party_mask = crate::story_party::select_party_with_ready(
                &state.warriors,
                state.selected_warrior,
                extra,
                context,
                ready,
                &mut state.random,
            )?;
        }
        Operation::EditSchedule => {
            let extra_index = usize::try_from(parameter)
                .map_err(|_| RuleError::Invalid("negative schedule extra index".into()))?;
            let extra = ExtraTable(&map.combat.as_ref().unwrap().encounters).record(extra_index)?;
            state.native_work.extra_record = extra.to_vec();
            crate::story_schedule::edit(&mut position.schedule, extra, position.day)?;
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
        _ => unreachable!("opcode routed to wrong story adapter"),
    }
    Ok(ScriptStep::Continue)
}
