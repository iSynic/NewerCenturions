//! Story operations that edit or enter another land through saveland/loadland.
use super::{Operation, ScriptStep};
use crate::{Event, MapState, RuleError, Session};
use centurions_content::Content;
use centurions_content::mission::OpeningMission;
use centurions_content::records::{ExtraTable, RegionTable, RegionTableMut, StoryRecordMut};
use centurions_content::word;

pub(super) fn execute(
    operation: Operation,
    parameter: i16,
    state: &mut Session,
    content: &Content,
    map: &OpeningMission,
    effects: &mut Vec<Event>,
) -> Result<ScriptStep, RuleError> {
    let extras = &map
        .combat
        .as_ref()
        .ok_or("story extras unavailable")?
        .encounters;
    let extra = ExtraTable(extras).record(parameter as usize)?;
    state.native_work.extra_record = extra.to_vec();
    match operation {
        // newland57 writes the target land's tile set and light level. It pushes the
        // target as a word to loadland, which reads a long, so only land 0 loads. The
        // closing loadland reloads the current land's tile properties (Session::land).
        Operation::ChangeLandTiles => {
            if word(extra, 4)? != 0 {
                return Err(super::super::unsupported(
                    "story.opcode",
                    "land tile set target",
                ));
            }
            let terrain = word(extra, 0)? as u8;
            // loadpixmap quits on sets 0 and 10-14 and names custom files from 15; only
            // the closing loadland of land 0 itself loads the new set here.
            if state.exploration.as_ref().unwrap().map == 0
                && content
                    .land_at(state.mission.as_deref(), state.scenario_level, 0, terrain)
                    .is_none()
            {
                return Err(super::super::unsupported(
                    "story.opcode",
                    "land tile set without an installed BD file quits the original",
                ));
            }
            let mut regions = RegionTableMut(land_regions(state, content, 0)?);
            regions.set_terrain(terrain);
            regions.set_light_level(word(extra, 2)? as u8);
        }
        Operation::PartyPosition => match word(extra, 0)? {
            // newland70 keeps the map, x and y as signed bytes 442738/44270c/44270d.
            1 => {
                let position = state.exploration.as_ref().unwrap();
                state.saved_position = [position.map as u8, position.x as u8, position.y as u8];
            }
            2 => {
                let [map, x, y] = state.saved_position.map(|byte| i32::from(byte as i8));
                OpeningMission::index(x, y)?;
                enter_land(state, content, map)?;
                let position = state.exploration.as_mut().unwrap();
                (position.x, position.y) = (x, y);
                let script = position.script.as_mut().unwrap();
                StoryRecordMut(&mut script.record).relocate(map, x, y);
                effects.push(Event::Moved { x, y });
            }
            _ => {}
        },
        Operation::ResizeRegion => {
            let region = word(extra, 2)?;
            if !(0..RegionTable::REGIONS as i16).contains(&region) {
                return Err(super::super::unsupported(
                    "story.opcode",
                    "random rectangle outside the land",
                ));
            }
            let region = region as usize;
            let (delta, mode) = (word(extra, 6)?, word(extra, 8)?);
            let sizes = ExtraTable(extras).record(parameter as usize + 1)?;
            state.native_work.extra_record = sizes.to_vec();
            let regions = land_regions(state, content, i32::from(word(extra, 0)?))?;
            let chance = RegionTable(regions)
                .encounter_chance(region)?
                .wrapping_add(delta);
            let mut table = RegionTableMut(regions);
            table.set_encounter_chance(region, chance);
            // newland92's second extra holds left, right, top and bottom.
            let (left, right) = (word(sizes, 0)?, word(sizes, 2)?);
            let (top, bottom) = (word(sizes, 4)?, word(sizes, 6)?);
            // Bound fields are first y, first x, last y, last x.
            let changes = match mode {
                0 => Some(([top, left, bottom, right], false)),
                1 => Some(([right, left, right, left], true)),
                2 => Some(([top, left, bottom, right], true)),
                _ => None,
            };
            if let Some((values, offset)) = changes {
                for (field, value) in values.into_iter().enumerate() {
                    let base = if offset {
                        table.bound(region, field)?
                    } else {
                        0
                    };
                    table.set_bound(region, field, base.wrapping_add(value));
                }
            }
        }
        _ => unreachable!("land operation routing"),
    }
    Ok(ScriptStep::Continue)
}

/// The loaded land's live region table, or a saved land's, authored on first use
/// as loadland would.
fn land_regions<'a>(
    state: &'a mut Session,
    content: &Content,
    target: i32,
) -> Result<&'a mut Vec<u8>, RuleError> {
    let position = state.exploration.as_mut().unwrap();
    if target == position.map {
        return Ok(&mut position.region_state);
    }
    let authored = content
        .map_at(state.mission.as_deref(), state.scenario_level, target)
        .ok_or_else(|| RuleError::Invalid("story land unavailable".into()))?;
    Ok(&mut state
        .visited_maps
        .entry(target)
        .or_insert_with(|| MapState::authored(authored))
        .regions)
}

/// loadland for a story move: the current land is saved and the target restored.
/// The script stays active across it; only its coordinates change.
/// newland20's tail: a destination floor word of magnitude 1000 or more starts the
/// walking event keyed at the party from its first action, after the trigger roll.
/// False where the original returns early.
pub(super) fn activate_destination(state: &mut Session) -> Result<bool, RuleError> {
    let position = state.exploration.as_mut().unwrap();
    let tile = position.floor[OpeningMission::index(position.x, position.y)?];
    if tile.wrapping_abs() < 1000 {
        return Ok(true);
    }
    let found = crate::walking_header::lookup(
        &position.event_headers,
        position.map,
        position.x,
        position.y,
        &mut state.random,
    )?;
    if let Some(row) = found.matched {
        state.native_work.header_row = row as i16;
    }
    let position = state.exploration.as_mut().unwrap();
    let Some(header) = found.header else {
        return Ok(false);
    };
    let origin = crate::ScriptOrigin {
        map: position.map,
        x: position.x,
        y: position.y,
        level: state.scenario_level,
    };
    let script = position.script.as_mut().unwrap();
    script.record = header.record;
    script.cursor = 0;
    script.map_slot = header.slot;
    script.origin = Some(origin);
    script.chained = true;
    Ok(true)
}

pub(super) fn enter_land(
    state: &mut Session,
    content: &Content,
    target: i32,
) -> Result<(), RuleError> {
    if target == state.exploration.as_ref().unwrap().map {
        return Ok(());
    }
    state.check_land_tile_set(content, state.scenario_level, target)?;
    let position = state.exploration.as_mut().unwrap();
    let authored = content
        .map_at(state.mission.as_deref(), state.scenario_level, target)
        .ok_or_else(|| RuleError::Invalid("travel target map unavailable".into()))?;
    let records = state
        .visited_maps
        .remove(&target)
        .unwrap_or_else(|| MapState::authored(authored));
    state
        .visited_maps
        .insert(position.map, position.map_state());
    position.restore_map(target, records);
    state.record_battle_land();
    Ok(())
}
