//! Battles script operations, preserving original dispatch order.
use super::super::{StoryBattle, begin_story_battle, finish_script, unsupported};
use super::{Operation, ScriptStep};
use crate::{Event, Presentation, RuleError, Session, combat, mission};
use centurions_content::mission::OpeningMission;
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
            let extra = span(
                &map.combat
                    .as_ref()
                    .ok_or("round tables missing")?
                    .encounters,
                parameter.unsigned_abs() as usize * 10,
                10,
            )?;
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
            let extra = span(
                &map.combat.as_ref().unwrap().encounters,
                parameter.unsigned_abs() as usize * 10,
                10,
            )?;
            effects.extend(crate::story_spell::apply(
                state,
                content,
                extra,
                operation == Operation::PartySpell,
            )?);
            if state
                .health_context
                .as_ref()
                .is_some_and(|health| health.unavailable >= state.warriors.len() as i8)
            {
                effects.extend(crate::mission::defeat(state, content));
                return Ok(ScriptStep::Pause);
            }
        }
        Operation::SelectPartyBySave => {
            let extra = span(
                &map.combat.as_ref().unwrap().encounters,
                parameter.unsigned_abs() as usize * 10,
                10,
            )?;
            let packet = state
                .combat
                .as_ref()
                .and_then(|c| c.attack_packet.as_ref())
                .or(state.attack_packet.as_ref());
            let context = crate::story_party::PartySaveContext {
                casting: false,
                packet_type: packet.map_or(0, |p| p[9] as i8),
                bonus: packet.map_or(0, |p| p[6] as i8),
                bonus_per_attack: packet.map_or(0, |p| p[8] as i8),
                attack_count: state.attack_count,
                mind_shield: word(&state.party_buffs, 14)? != 0,
            };
            state.party_mask = crate::story_party::select_party(
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
            let extra = span(
                &map.combat.as_ref().unwrap().encounters,
                extra_index * 10,
                10,
            )?;
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
