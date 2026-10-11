use crate::{Event, MapState, Presentation, RuleError, ScriptState, Session};
use centurions_content::{Content, Warrior, word};

pub fn stage_target(parameter: i16) -> Result<usize, RuleError> {
    let target = parameter.wrapping_abs().wrapping_sub(1);
    usize::try_from(target).map_err(|_| RuleError::Invalid("stage target is unavailable".into()))
}

pub fn restore_stage_warriors(warriors: &[Warrior]) -> Result<Vec<Warrior>, RuleError> {
    let mut restored = warriors.to_vec();
    for warrior in restored.iter_mut().take(warriors.len().saturating_sub(1)) {
        for (target, maximum) in [(0x50, 0x52), (0x58, 0x5a)] {
            let value = word(&warrior.bytes, maximum)?;
            warrior.bytes[target..target + 2].copy_from_slice(&value.to_be_bytes());
        }
    }
    Ok(restored)
}

pub(crate) fn begin_stage(
    state: &mut Session,
    content: &Content,
    parameter: i16,
) -> Result<Vec<Event>, RuleError> {
    let target = stage_target(parameter)?;
    let scenario = content
        .scenario
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("scenario unavailable".into()))?;
    let stage = scenario
        .stages
        .get(target)
        .filter(|s| !s.name.is_empty())
        .ok_or_else(|| RuleError::Invalid("stage unavailable".into()))?;
    let position = state
        .exploration
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("no active map".into()))?;
    let authored = content
        .map(stage.start.2)
        .ok_or_else(|| RuleError::Invalid("stage map unavailable".into()))?;
    let records = if position.map == stage.start.2 {
        position.map_state()
    } else {
        let records = state
            .visited_maps
            .remove(&stage.start.2)
            .unwrap_or_else(|| MapState::authored(authored));
        state
            .visited_maps
            .insert(position.map, position.map_state());
        records
    };
    position.restore_map(stage.start.2, records);
    position.x = stage.start.0;
    position.y = stage.start.1;
    state.mission_stage = target as i16;
    state.warriors = restore_stage_warriors(&state.warriors)?;
    state.allies = crate::story_allies::clear_stage_allies(&state.allies)?;
    state.ally_count = 0;
    position.script = if stage.start_script != 0 {
        Some(ScriptState {
            record: authored.script(stage.start_script)?.to_vec(),
            cursor: 0,
            map_slot: 0,
            scheduled: true,
            menu: None,
            persist: false,
            origin: None,
        })
    } else {
        None
    };
    position.presentation = Some(Presentation::Prologue {
        resource_id: 1000 + target as i16,
    });
    Ok(vec![
        Event::MissionEntered {
            title: stage.name.clone(),
            x: position.x,
            y: position.y,
        },
        Event::StoryPresented,
    ])
}
