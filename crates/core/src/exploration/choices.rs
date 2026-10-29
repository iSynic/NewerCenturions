//! Choices for maintained exploration.
use super::{active_map, choose_avoidance, finish_dispatch, run_script, typed_record, unsupported};
use crate::{
    Event, Exploration, Phase, Presentation, RuleError, Session, mission, story, typed_encounter,
};
use centurions_content::Content;
use centurions_content::mission::OpeningMission;
use centurions_content::records::{EventHeaderTableMut, ExtraTable, StoryRecordMut};

pub fn choose_warrior(
    state: &mut Session,
    content: &Content,
    index: usize,
) -> Result<Vec<Event>, RuleError> {
    if let Some(Presentation::PartyPick {
        remaining,
        unconscious,
        invert,
    }) = state
        .exploration
        .as_ref()
        .and_then(|p| p.presentation.clone())
    {
        if index >= state.warriors.len() {
            return Err(RuleError::Invalid("warrior unavailable".into()));
        }
        return pick_member(state, content, index, remaining, unconscious, invert);
    }
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
    state.native_work.party_mask = selected;
    state.selected_warrior = index;
    // Sessions saved before opcode 14 used the shared pick hold its single pick here.
    if *extra == -1 {
        position.presentation = None;
        return run_script(state, content);
    }
    let record = ExtraTable(&map.combat.as_ref().unwrap().encounters).record(*extra as usize)?;
    state.native_work.extra_record = record.to_vec();
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

/// getchoice125112: an unpicked member takes the next countdown number (sound 141),
/// a picked one is released and later picks move up (sound 144); dead members are
/// refused (sound 6000) unless the pick allows them. The script resumes when the
/// countdown is spent, after -14's inversion.
fn pick_member(
    state: &mut Session,
    content: &Content,
    index: usize,
    remaining: i16,
    unconscious: bool,
    invert: bool,
) -> Result<Vec<Event>, RuleError> {
    if !unconscious && state.warriors[index].stamina()? < 1 {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::NativeFeedback::Sound { resource_id: 6000 }],
        });
    }
    state.selected_warrior = index;
    let mask = &mut state.native_work.party_mask;
    let (sound, remaining) = if mask[index] == 0 {
        mask[index] = remaining;
        (141, remaining - 1)
    } else {
        let order = mask[index];
        mask[index] = 0;
        for pick in mask.iter_mut().take(state.warriors.len()) {
            if *pick != 0 && *pick < order {
                *pick += 1;
            }
        }
        (144, remaining + 1)
    };
    let mut events = vec![Event::Sound { resource_id: sound }];
    let position = state.exploration.as_mut().unwrap();
    if remaining > 0 {
        position.presentation = Some(Presentation::PartyPick {
            remaining,
            unconscious,
            invert,
        });
        return Ok(events);
    }
    position.presentation = None;
    if invert {
        for pick in mask.iter_mut().take(state.warriors.len()) {
            *pick = i16::from(*pick == 0);
        }
    }
    events.extend(run_script(state, content)?);
    Ok(events)
}

pub fn choose(
    state: &mut Session,
    content: &Content,
    answer: bool,
) -> Result<Vec<Event>, RuleError> {
    if matches!(
        state
            .exploration
            .as_ref()
            .and_then(|p| p.presentation.as_ref()),
        Some(Presentation::BoatQuestion { .. })
    ) {
        return super::boat::answer(state, answer);
    }
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
    let record = ExtraTable(&map.combat.as_ref().unwrap().encounters).record(*extra as usize)?;
    state.native_work.extra_record = record.to_vec();
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
            StoryRecordMut(&mut script.record).write_actions(&story::dispatch_menu_branch(
                state.native_work.encounter_work.header(false),
                branch as i8 + 1,
            )?);
            script.cursor = 0;
        }
        typed_encounter::QuestionPlan::TypedBranch(branch) => {
            let script = position.script.as_mut().ok_or("question script missing")?;
            let record = state.native_work.encounter_work.header(true);
            StoryRecordMut(&mut script.record)
                .write_actions(&story::dispatch_menu_branch(record, branch as i8 + 1)?);
            script.cursor = 0;
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
        EventHeaderTableMut(&mut position.event_headers).replace(script.map_slot, &menu.original);
    }
    if let Some(menu) = &script.menu {
        let (id, typed) = (menu.id, menu.typed);
        super::menus::save_encounter(state, content, id, typed)?;
    }
    let position = state.exploration.as_mut().unwrap();
    let script = position.script.as_ref().unwrap();
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
    let position = state
        .exploration
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("no active mission".into()))?;
    let Some(Presentation::Menu { .. }) = position.presentation.as_ref() else {
        return Err(RuleError::Invalid("no menu active".into()));
    };
    let record = state.native_work.encounter_work.record(false)?;
    let selection = story::select_menu_entry(&record, index)?;
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
    let record = typed_record(state, content)?;
    let position = state.exploration.as_mut().unwrap();
    let script = position
        .script
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("typed script missing".into()))?;
    let menu = script
        .menu
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("typed continuation missing".into()))?;
    let remaining = menu.remaining;
    match typed_encounter::continuation(&record, selected, remaining)? {
        typed_encounter::TypedContinuation::Branch { body, .. } => {
            StoryRecordMut(&mut script.record).write_actions(&body);
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
