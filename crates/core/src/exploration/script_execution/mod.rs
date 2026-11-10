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
pub(super) use lands::enter_land;
mod map_mutation;
mod party;
mod presentation;

/// How a script parameter selects its EXTRA record.
#[derive(Clone, Copy)]
pub(super) enum ExtraIndex {
    /// The parameter read as an unsigned number: a negative parameter selects far past the table.
    Signed(i16),
    /// The parameter's magnitude.
    Magnitude(i16),
    /// An index the caller has already converted.
    Exact(usize),
}

/// Reads a script's extra record into the shared extra row and returns the row. A record at or
/// past the end of the table overwrites only the bytes it has, leaving the rest of the row as it
/// was. A land without an extra table is refused with `missing`, or a general message when none
/// is given.
///
/// Original: loadextracode
pub(super) fn load_extra(
    globals: &mut crate::Globals,
    map: &OpeningMission,
    index: ExtraIndex,
    missing: Option<&'static str>,
) -> Result<Vec<u8>, RuleError> {
    let tables = map
        .combat
        .as_ref()
        .ok_or(missing.unwrap_or("story extra records missing"))?;
    let id = match index {
        ExtraIndex::Signed(parameter) => parameter as u16,
        ExtraIndex::Magnitude(parameter) => parameter.unsigned_abs(),
        ExtraIndex::Exact(index) => index as u16,
    };
    Ok(read_extra(globals, &tables.encounters, id))
}

/// Reads EXTRA record `id` of `table` over the shared extra row and returns the row.
///
/// Original: loadextracode
pub(crate) fn read_extra(globals: &mut crate::Globals, table: &[u8], id: u16) -> Vec<u8> {
    let read = ExtraTable(table).read(id);
    globals.extra_record.resize(ExtraTable::RECORD_BYTES, 0);
    globals.extra_record[..read.len()].copy_from_slice(read);
    globals.extra_record.clone()
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
