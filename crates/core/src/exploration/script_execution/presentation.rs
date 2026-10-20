//! Presentation script operations, preserving original dispatch order.
use super::super::{StoryBattle, begin_story_battle, dialogue, finish_script, unsupported};
use super::{Operation, ScriptStep};
use crate::{Event, Presentation, RuleError, Session};
use centurions_content::mission::OpeningMission;
use centurions_content::{Content, word};

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
        Operation::SuppressAllies => state.suppress_allies = parameter,
        Operation::SetMapPicture => {
            crate::story_map_picture::assign(&mut state.map_picture, parameter)
        }
        Operation::Registration => {}
        Operation::ChooseParty if parameter == 1 => {
            state.party_mask.fill(0);
            position.presentation = Some(Presentation::WarriorChoice { extra: -1 });
            effects.push(Event::StoryPresented);
            return Ok(ScriptStep::Pause);
        }
        Operation::NoAction => {}
        Operation::Dialogue => {
            let text = dialogue(map, &state.warriors, parameter)?;
            position.presentation = Some(Presentation::Dialogue {
                id: parameter,
                text,
            });
            effects.push(Event::StoryPresented);
            return Ok(ScriptStep::Pause);
        }
        Operation::Sound => effects.push(Event::Sound {
            resource_id: parameter,
        }),
        Operation::Delay => effects.push(Event::Delay {
            ticks: crate::story_delay::elapsed_ticks(parameter)?,
        }),
        Operation::Question => {
            let previous = cursor
                .checked_sub(1)
                .ok_or_else(|| RuleError::Invalid("question text missing".into()))?;
            let text = map.dialogue(word(&script.record, 24 + previous * 2)?)?;
            position.presentation = Some(Presentation::Question {
                extra: parameter,
                text,
            });
            effects.push(Event::Sound { resource_id: 20005 });
            effects.push(Event::StoryPresented);
            return Ok(ScriptStep::Pause);
        }
        Operation::EndScript => {
            effects.extend(finish_script(state, content, false)?);
            return Ok(ScriptStep::Pause);
        }
        Operation::PersistEvent => {
            script.persist = !script.scheduled;
        }
        Operation::Movie => {
            if !map.movies.contains_key(&parameter) && parameter != 1000 {
                return Err(unsupported("story.movie", "other TEXT resources"));
            }
            position.presentation = Some(Presentation::Prologue {
                resource_id: parameter,
            });
            effects.push(Event::StoryPresented);
            return Ok(ScriptStep::Pause);
        }
        Operation::Battle => {
            effects.extend(begin_story_battle(
                state,
                content,
                StoryBattle::Ordinary,
                parameter,
            )?);
            return Ok(ScriptStep::Pause);
        }
        _ => {
            return Err(unsupported(
                "story.opcode",
                "this story operation is not recovered",
            ));
        }
    }
    Ok(ScriptStep::Continue)
}
