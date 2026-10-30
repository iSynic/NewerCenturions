//! Running a land script's operations.
use super::{
    active_map, active_table, finish_dispatch, finish_script, present_menu, present_typed,
    store_script_stamp,
};
use crate::combat::runtime;
use crate::{Event, RuleError, Session};
use centurions_content::Content;

mod operation;
use centurions_content::mission::OpeningMission;
use centurions_content::records::{ExtraTable, StoryRecord};
use operation::Adapter;
pub(crate) use operation::Operation;

/// Sound 20005, played as a script presents a simple or typed encounter.
const ENCOUNTER_SOUND: i16 = 20005;

mod battles;
mod branches;
mod encounters;
mod lands;
mod map_mutation;
mod party;
mod presentation;

/// How a script parameter selects its EXTRA record.
#[derive(Clone, Copy)]
pub(super) enum ExtraIndex {
    /// `parameter as usize`: a negative parameter selects past the table.
    Signed(i16),
    /// The parameter's magnitude.
    Magnitude(i16),
    /// An index the caller has already converted.
    Exact(usize),
}

/// Reads a script's extra record and keeps a copy as the shared extra row. With
/// `missing` absent, a map without combat tables panics; otherwise it returns
/// that error.
pub(super) fn load_extra<'a>(
    globals: &mut crate::Globals,
    map: &'a OpeningMission,
    index: ExtraIndex,
    missing: Option<&'static str>,
) -> Result<&'a [u8], RuleError> {
    let tables = match missing {
        Some(message) => map.combat.as_ref().ok_or(message)?,
        None => map.combat.as_ref().unwrap(),
    };
    let index = match index {
        ExtraIndex::Signed(parameter) => parameter as usize,
        ExtraIndex::Magnitude(parameter) => usize::from(parameter.unsigned_abs()),
        ExtraIndex::Exact(index) => index,
    };
    let extra = ExtraTable(&tables.encounters).record(index)?;
    globals.extra_record = extra.to_vec();
    Ok(extra)
}

/// Whether the script goes on to its next action or stops to wait for the player.
#[derive(PartialEq, Eq)]
pub(super) enum ScriptStep {
    Continue,
    Pause,
}

/// Runs the land script action by action until it ends, needs the player, or is handed to a battle.
/// A script that has used all its actions repeats its menu while the menu has choices left. A
/// script that runs more than 128 actions in one call is refused.
pub(crate) fn run_script(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    runtime::resume_turn_child(state)?;
    let mut effects = Vec::new();
    for _ in 0..128 {
        if state.zero_time.is_some()
            || state.combat_story.is_some()
            || state.turn_dispatch.is_some()
        {
            let record = state
                .exploration
                .as_ref()
                .and_then(|p| p.script.as_ref())
                .map(|script| script.record.clone());
            if let Some(record) = record {
                super::store_turn_header(state, &record);
                crate::time::store_zero_record(state, &record);
                runtime::store_death_header(state, &record);
            }
        }
        if let Some(stamp) = state
            .exploration
            .as_ref()
            .and_then(|p| p.script.as_ref())
            .and_then(|s| StoryRecord(&s.record).dispatch_stamp())
        {
            store_script_stamp(state, stamp);
        }
        let map = active_map(state, content)?;
        let position = state.exploration.as_mut().unwrap();
        let Some(script) = position.script.as_mut() else {
            if let Some(closed) = finish_dispatch(state, content)? {
                effects.extend(closed);
            }
            return Ok(effects);
        };
        if usize::from(script.cursor) >= StoryRecord::ACTIONS {
            if let Some(menu) = script.menu.as_mut() {
                let before = menu.remaining;
                menu.remaining = before.wrapping_sub(1);
                if before > 1 {
                    let id = menu.id;
                    if menu.typed {
                        position.presentation = Some(present_typed(
                            map,
                            active_table(&state.typed_encounters, &map.typed_encounters),
                            script,
                            id,
                            &mut state.globals.encounter_headers,
                            false,
                        )?);
                        effects.push(Event::StoryPresented);
                        return Ok(effects);
                    }
                    let table = active_table(&state.menus, &map.menus);
                    effects.push(Event::Sound { resource_id: 20005 });
                    position.presentation = present_menu(
                        map,
                        table,
                        script,
                        id,
                        &mut state.globals.encounter_headers,
                        false,
                    )?;
                    if position.presentation.is_some() {
                        effects.push(Event::StoryPresented);
                        return Ok(effects);
                    }
                    continue;
                }
            }
            effects.extend(finish_script(state, content, true)?);
            return Ok(effects);
        }
        let cursor = usize::from(script.cursor);
        let (opcode, parameter) = StoryRecord(&script.record).instruction(cursor)?;
        let operation = Operation::from_opcode(opcode);
        script.cursor += 1;
        crate::trace::story(
            "exploration",
            script.map_slot as i32,
            cursor,
            opcode,
            parameter,
        );
        let step = match operation.adapter() {
            Adapter::Battles => {
                battles::execute(operation, parameter, state, content, map, &mut effects)?
            }
            Adapter::Party => party::execute(
                operation,
                parameter,
                state,
                content,
                map,
                cursor,
                &mut effects,
            )?,
            Adapter::Branches => {
                branches::execute(operation, parameter, state, content, map, &mut effects)?
            }
            Adapter::LandExtra => {
                lands::execute(operation, parameter, state, content, map, &mut effects)?
            }
            Adapter::Travel => {
                lands::travel(operation, parameter, state, content, map, &mut effects)?
            }
            Adapter::MapMutation => {
                map_mutation::execute(operation, parameter, state, content, map)?
            }
            Adapter::Encounters => {
                encounters::execute(operation, parameter, state, map, &mut effects)?
            }
            Adapter::Presentation => presentation::execute(
                operation,
                parameter,
                state,
                content,
                map,
                cursor,
                &mut effects,
            )?,
        };
        if step == ScriptStep::Pause {
            return Ok(effects);
        }
    }
    Err(RuleError::Invalid(
        "story exceeds instruction budget".into(),
    ))
}
