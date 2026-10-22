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
    let position = state.exploration.as_mut().unwrap();
    let script = position.script.as_mut().unwrap();
    match operation {
        Operation::CombatCleanup => {
            effects.extend(crate::story_cleanup::begin(state, content, parameter)?);
            if state.scripted_cleanup == Some(crate::story_cleanup::Stage::Survivors) {
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
            if state.round_dispatch.is_none() {
                return Err(unsupported("story.round_jump", "native round-slot caller"));
            }
            let arena = state.combat.as_mut().ok_or("round combat missing")?;
            let slot = arena
                .round_scripts
                .pending
                .as_ref()
                .ok_or("round program missing")?
                .slot;
            let extra = ExtraTable(
                &map.combat
                    .as_ref()
                    .ok_or("round tables missing")?
                    .encounters,
            )
            .record(parameter.unsigned_abs() as usize)?;
            let Some(target) = combat::round_stories::replacement(
                &mut arena.round_scripts,
                slot,
                extra,
                &mut state.random,
            )?
            else {
                effects.extend(finish_script(state, content, false)?);
                return Ok(ScriptStep::Pause);
            };
            script.restart_from(&map.script(target)?);
        }
        Operation::SelectedSpell | Operation::PartySpell => {
            let extra = ExtraTable(&map.combat.as_ref().unwrap().encounters)
                .record(parameter.unsigned_abs() as usize)?;
            effects.extend(crate::story_spell::apply(
                state,
                content,
                extra,
                operation == Operation::PartySpell,
            )?);
            if state
                .native_work
                .health_context
                .as_ref()
                .is_some_and(|health| health.unavailable >= state.warriors.len() as i8)
            {
                effects.extend(crate::mission::defeat(state, content));
                return Ok(ScriptStep::Pause);
            }
        }
        Operation::SelectPartyBySave => {
            let extra = ExtraTable(&map.combat.as_ref().unwrap().encounters)
                .record(parameter.unsigned_abs() as usize)?;
            let packet = state
                .combat
                .as_ref()
                .and_then(|c| c.attack_packet.as_ref())
                .or(state.native_work.attack_packet.as_ref());
            let context = crate::story_party::PartySaveContext {
                casting: false,
                packet_type: packet.map_or(0, |p| AttackPacket(p).saving_throw_mode() as i8),
                bonus: packet.map_or(0, |p| AttackPacket(p).save_bonus()),
                bonus_per_attack: packet
                    .map_or(0, |p| AttackPacket(p).save_bonus_per_attack() as i8),
                attack_count: state.native_work.attack_count,
                mind_shield: crate::combat::party_buffs::mind_shield(&state.party_buffs)?,
            };
            state.native_work.party_mask = crate::story_party::select_party(
                &state.warriors,
                state.selected_warrior,
                extra,
                context,
                &mut state.random,
            )?;
        }
        Operation::EditSchedule => {
            let extra_index = usize::try_from(parameter)
                .map_err(|_| RuleError::Invalid("negative schedule extra index".into()))?;
            let extra = ExtraTable(&map.combat.as_ref().unwrap().encounters).record(extra_index)?;
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
