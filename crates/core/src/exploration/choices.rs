//! Choices for maintained exploration.
use super::{active_map, choose_avoidance, finish_dispatch, run_script, typed_record, unsupported};
use crate::{
    Event, Exploration, Phase, Presentation, RuleError, Session, mission, story, typed_encounter,
};
use centurions_content::mission::OpeningMission;
use centurions_content::{Content, span};

pub fn choose_warrior(
    state: &mut Session,
    content: &Content,
    index: usize,
) -> Result<Vec<Event>, RuleError> {
    let map = active_map(state, content)?;
    let warrior = state
        .warriors
        .get(index)
        .ok_or_else(|| RuleError::Invalid("warrior unavailable".into()))?;
    if warrior.stamina()? < 1 {
        return Err(RuleError::Invalid("choose a living warrior".into()));
    }
    let position = state
        .exploration
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("no active mission".into()))?;
    let Some(Presentation::WarriorChoice { extra }) = position.presentation.as_ref() else {
        return Err(RuleError::Invalid("no warrior choice active".into()));
    };
    let selected = crate::story_skill::single_warrior(&state.warriors, index)?;
    state.party_mask = selected;
    state.selected_warrior = index;
    if *extra == -1 {
        position.presentation = None;
        return run_script(state, content);
    }
    let record = span(
        &map.combat.as_ref().unwrap().encounters,
        *extra as usize * 10,
        10,
    )?;
    let plan = crate::story_skill::predicate(record, &state.warriors, &selected)?;
    let script = position
        .script
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("choice script missing".into()))?;
    if let crate::story_skill::SkillPlan::ReplaceScript { script: id, .. } = plan {
        script.restart_from(&map.script(id)?);
    }
    position.presentation = None;
    run_script(state, content)
}

pub fn choose(
    state: &mut Session,
    content: &Content,
    answer: bool,
) -> Result<Vec<Event>, RuleError> {
    if let Some(Presentation::Avoidance { region, .. }) = state
        .exploration
        .as_ref()
        .and_then(|p| p.presentation.as_ref())
    {
        return choose_avoidance(state, content, *region as usize, answer);
    }
    if let Some(Presentation::StageChange { parameter }) = state
        .exploration
        .as_ref()
        .and_then(|p| p.presentation.as_ref())
    {
        let parameter = *parameter;
        if answer {
            return mission::begin_stage(state, content, parameter);
        }
        let position = state.exploration.as_mut().unwrap();
        position.presentation = None;
        position.script = None;
        return Ok(vec![]);
    }
    let map = active_map(state, content)?;
    let position = state
        .exploration
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("no active mission".into()))?;
    let Some(Presentation::Question { extra, .. }) = position.presentation.as_ref() else {
        return Err(RuleError::Invalid("no question active".into()));
    };
    let record = span(
        &map.combat.as_ref().unwrap().encounters,
        *extra as usize * 10,
        10,
    )?;
    match typed_encounter::question_plan(record, answer)? {
        typed_encounter::QuestionPlan::Continue => {}
        typed_encounter::QuestionPlan::MoveAway => {
            let movement = back_out(position)?;
            position.presentation = None;
            position.script = None;
            let mut events = vec![movement];
            if let Some(closed) = finish_dispatch(state, content)? {
                events.extend(closed);
            }
            return Ok(events);
        }
        typed_encounter::QuestionPlan::ReplaceScript(id) => {
            let replacement = map.script(id)?;
            let script = position
                .script
                .as_mut()
                .ok_or_else(|| RuleError::Invalid("question script missing".into()))?;
            script.restart_from(&replacement);
        }
        typed_encounter::QuestionPlan::MenuBranch(branch) => {
            let script = position
                .script
                .as_mut()
                .ok_or_else(|| RuleError::Invalid("question script missing".into()))?;
            let id = script
                .menu
                .as_ref()
                .ok_or_else(|| RuleError::Invalid("question menu missing".into()))?
                .id;
            let table = if state.menus.is_empty() {
                &map.menus
            } else {
                &state.menus
            };
            script.record[8..40].copy_from_slice(&story::dispatch_menu_branch(
                span(
                    table,
                    id as usize * story::MENU_RECORD_BYTES,
                    story::MENU_RECORD_BYTES,
                )?,
                branch as i8 + 1,
            )?);
            script.cursor = 0;
        }
        typed_encounter::QuestionPlan::TypedBranch(branch) => {
            let script = position.script.as_mut().ok_or("question script missing")?;
            let menu = script
                .menu
                .as_ref()
                .filter(|m| m.typed)
                .ok_or("question typed continuation missing")?;
            let record = span(
                if state.typed_encounters.is_empty() {
                    &map.typed_encounters
                } else {
                    &state.typed_encounters
                },
                menu.id as usize * typed_encounter::RECORD_BYTES,
                typed_encounter::RECORD_BYTES,
            )?;
            if let typed_encounter::TypedContinuation::Branch { body, .. } =
                typed_encounter::continuation(record, branch as i8 + 1, menu.remaining)?
            {
                script.record[8..40].copy_from_slice(&body);
                script.cursor = 0;
            }
        }
        typed_encounter::QuestionPlan::EndScript => position.script = None,
    }
    position.presentation = None;
    run_script(state, content)
}

pub(super) fn back_out(position: &mut Exploration) -> Result<Event, RuleError> {
    let [dx, dy] = position.last_move;
    let forward = OpeningMission::index(position.x + i32::from(dx), position.y + i32::from(dy))?;
    let undone = crate::story_move_away::move_away(
        position.x,
        position.y,
        dx,
        dy,
        position.floor[forward],
        [0; 6],
    );
    OpeningMission::index(undone.x, undone.y)?;
    position.x = undone.x;
    position.y = undone.y;
    Ok(Event::Moved {
        x: undone.x,
        y: undone.y,
    })
}

pub fn cancel_menu(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let position = state
        .exploration
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("no active mission".into()))?;
    if !matches!(
        position.presentation,
        Some(Presentation::Menu { .. } | Presentation::Typed { .. })
    ) {
        return Err(RuleError::Invalid("no recovered menu active".into()));
    }
    let script = position
        .script
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("menu script missing".into()))?;
    if let Some(menu) = &script.menu
        && !script.scheduled
    {
        position.event_headers[script.map_slot * 42..(script.map_slot + 1) * 42]
            .copy_from_slice(&menu.original);
    }
    // buttonchoice10a5a8 marks the manual caller; newland142cca skips its menu backstep.
    let backstep =
        matches!(position.presentation, Some(Presentation::Typed { .. })) || !script.item_return;
    let mut events =
        if state.phase == Phase::Exploration && backstep && position.last_move != [0; 2] {
            vec![back_out(position)?]
        } else {
            vec![]
        };
    position.presentation = None;
    position.script = None;
    if let Some(closed) = finish_dispatch(state, content)? {
        events.extend(closed);
    }
    Ok(events)
}

pub fn select_menu(
    state: &mut Session,
    content: &Content,
    index: usize,
) -> Result<Vec<Event>, RuleError> {
    let map = active_map(state, content)?;
    let position = state
        .exploration
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("no active mission".into()))?;
    let Some(Presentation::Menu { id, .. }) = position.presentation.as_ref() else {
        return Err(RuleError::Invalid("no menu active".into()));
    };
    let table = if state.menus.is_empty() {
        &map.menus
    } else {
        &state.menus
    };
    let record = span(
        table,
        *id as usize * story::MENU_RECORD_BYTES,
        story::MENU_RECORD_BYTES,
    )?;
    let selection = story::select_menu_entry(record, index)?;
    let script = position
        .script
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("menu script missing".into()))?;
    if script.menu.is_none() {
        return Err(unsupported(
            "story.select_menu",
            "legacy menu without a recovered continuation",
        ));
    }
    script.record = story::replace_menu_branch(&script.record, &selection)?;
    script.cursor = 0;
    position.presentation = None;
    run_script(state, content)
}

pub(crate) fn finish_typed_selection(
    state: &mut Session,
    content: &Content,
    selected: i8,
) -> Result<Vec<Event>, RuleError> {
    let record = typed_record(state, content)?.to_vec();
    let position = state.exploration.as_mut().unwrap();
    let script = position
        .script
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("typed script missing".into()))?;
    let remaining = script
        .menu
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("typed continuation missing".into()))?
        .remaining;
    match typed_encounter::continuation(&record, selected, remaining)? {
        typed_encounter::TypedContinuation::Branch { body, .. } => {
            script.record[8..40].copy_from_slice(&body);
            script.cursor = 0;
        }
        typed_encounter::TypedContinuation::Cancel => return cancel_menu(state, content),
        typed_encounter::TypedContinuation::Abort => {
            position.script = None;
        }
    }
    position.presentation = None;
    run_script(state, content)
}
