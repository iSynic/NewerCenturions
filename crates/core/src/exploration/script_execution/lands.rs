//! Script operations that edit or enter another land, move the party, and set
//! land light and game time.
use super::super::abandon_script;
use super::{ExtraIndex, Operation, ScriptStep, load_extra};
use crate::{Event, MapState, Presentation, RuleError, Session};
use centurions_content::Content;
use centurions_content::mission::OpeningMission;
use centurions_content::records::{
    BoatCampingExtra, ExtraTable, LandLightExtra, RegionTable, RegionTableMut, ShiftPartyExtra,
    StoryRecord, StoryRecordMut, TravelExtra,
};
use centurions_content::word;

/// Runs the script operations that change a land's tiles, light, regions or game time, move or
/// shift the party, test boat or camping, and take it to another land.
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
    state.globals.extra_record = extra.to_vec();
    match operation {
        // Writes the target land's tile set and light level. Only land 0 loads
        // as the target, a deliberate original quirk (see
        // docs/source-notes/rules.md). The closing load reloads the current
        // land's tile properties (Session::land).
        Operation::ChangeLandTiles => {
            if word(extra, 4)? != 0 {
                return Err(super::super::unsupported(
                    "story.opcode",
                    "land tile set target",
                ));
            }
            let terrain = word(extra, 0)? as u8;
            // Tile sets 0 and 10-14 quit the game and custom sets start at 15; only
            // the closing load of land 0 itself loads the new set here.
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
            // The map, x and y are kept as signed bytes.
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
            state.globals.extra_record = sizes.to_vec();
            let regions = land_regions(state, content, i32::from(word(extra, 0)?))?;
            let chance = RegionTable(regions)
                .encounter_chance(region)?
                .wrapping_add(delta);
            let mut table = RegionTableMut(regions);
            table.set_encounter_chance(region, chance);
            // The second extra holds left, right, top and bottom.
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

/// Story moves of the party within or between lands, and land light and time.
pub(super) fn travel(
    operation: Operation,
    parameter: i16,
    state: &mut Session,
    content: &Content,
    map: &OpeningMission,
    effects: &mut Vec<Event>,
) -> Result<ScriptStep, RuleError> {
    let position = state.exploration.as_mut().unwrap();
    let script = position.script.as_mut().unwrap();
    match operation {
        // Travel, and teleport-activate, which then starts any event at the
        // destination.
        Operation::Travel | Operation::TeleportActivate => {
            let extra = load_extra(&mut state.globals, map, ExtraIndex::Signed(parameter), None)?;
            let destination = TravelExtra(extra);
            let target = destination.map(position.map)?;
            let x = destination.x(position.x)?;
            let y = destination.y(position.y)?;
            OpeningMission::index(x, y)?;
            // Only a nonnegative target replaces the script's position.
            let script = position.script.as_mut().unwrap();
            let (a4, a3) = script.registers;
            script.registers = (
                destination.x(i32::from(a4))? as i16,
                destination.y(i32::from(a3))? as i16,
            );
            enter_land(state, content, target)?;
            let position = state.exploration.as_mut().unwrap();
            position.x = x;
            position.y = y;
            let script = position.script.as_mut().unwrap();
            StoryRecordMut(&mut script.record).relocate(position.map, x, y);
            effects.push(Event::Moved { x, y });
            if destination.sound()? != 0 {
                effects.push(Event::Sound {
                    resource_id: destination.sound()?,
                });
            }
            let paused = destination.dialogue()? != 0;
            if paused {
                position.presentation = Some(Presentation::Dialogue {
                    id: destination.dialogue()?,
                    text: map.dialogue(destination.dialogue()?)?,
                });
                effects.push(Event::StoryPresented);
            }
            if operation == Operation::TeleportActivate && !activate_destination(state)? {
                effects.extend(super::super::abandon_script(state, content)?);
                return Ok(ScriptStep::Pause);
            }
            if paused {
                return Ok(ScriptStep::Pause);
            }
        }
        Operation::ShiftParty => {
            let extra = load_extra(
                &mut state.globals,
                map,
                ExtraIndex::Signed(parameter),
                Some("story extras unavailable"),
            )?;
            let extra = ShiftPartyExtra(extra);
            let random = extra.random()?;
            let mut shift = |amount: i16| {
                if !random {
                    amount
                } else if state.random.bound(100) < 50 {
                    state.random.range(1, amount)
                } else {
                    state.random.range(1, amount).wrapping_neg()
                }
            };
            let (dx, dy) = (shift(extra.dx()?), shift(extra.dy()?));
            let position = state.exploration.as_mut().unwrap();
            let (x, y) = (position.x + i32::from(dx), position.y + i32::from(dy));
            // The game writes outside the 90-by-90 land without a check; that is
            // refused here.
            OpeningMission::index(x, y)?;
            position.x = x;
            position.y = y;
            let map_id = position.map;
            let script = position.script.as_mut().unwrap();
            centurions_content::records::StoryRecordMut(&mut script.record).relocate(map_id, x, y);
            effects.push(Event::Moved { x, y });
        }
        Operation::BoatCampingBranch => {
            let extra = load_extra(
                &mut state.globals,
                map,
                ExtraIndex::Signed(parameter),
                Some("story extras unavailable"),
            )?;
            let extra = BoatCampingExtra(extra);
            let boat_condition = extra.boat_condition()?;
            let boat_failed = if matches!(boat_condition, 1 | 2) {
                let boat = state.globals.boat != 0;
                (boat_condition == 1 && !boat) || (boat_condition == 2 && boat)
            } else {
                false
            };
            let camping = state.field_controls.as_ref().is_some_and(|c| c.camping);
            let condition = extra.camping_condition()?;
            if boat_failed || (condition == 1 && !camping) || (condition == 2 && camping) {
                script.cursor = StoryRecord::ACTIONS as u8;
            }
            // The body ends only after the boat-camping branch's remaining writes.
            match extra.boat_change()? {
                1 => state.globals.boat = 1,
                2 => state.globals.boat = 0,
                _ => {}
            }
        }
        Operation::SetLandLight => {
            let extra = load_extra(
                &mut state.globals,
                map,
                ExtraIndex::Signed(parameter),
                Some("story extras unavailable"),
            )?;
            let extra = LandLightExtra(extra);
            let level = extra.level()?.wrapping_sub(1);
            let stored = RegionTable(&position.region_state).light_level()?;
            // The stored byte is zero-extended against the parameter word.
            if extra.abandon_if_unchanged()? && u16::from(stored) == level as u16 {
                effects.extend(abandon_script(state, content)?);
                return Ok(ScriptStep::Pause);
            }
            RegionTableMut(&mut position.region_state).set_light_level(level as u8);
        }
        // Sets (1) or offsets (2) the day, hour and minute, with no -1 keep,
        // carrying a day and an hour once each. Its clock tick skips the hourly
        // work; only an hour of exactly 24 starts the next day and its time
        // encounters.
        Operation::AlterGameTime => {
            let extra = load_extra(
                &mut state.globals,
                map,
                ExtraIndex::Signed(parameter),
                Some("story extras unavailable"),
            )?;
            (position.day, position.hour, position.minute) =
                crate::story::clock::alter((position.day, position.hour, position.minute), extra)?;
            let depth = |state: &Session| {
                std::iter::successors(state.zero_time.as_ref(), |f| f.parent.as_deref()).count()
            };
            let before = depth(state);
            effects.extend(crate::time::zero_tick(state, content)?);
            // Only a new calendar frame suspends this script.
            if depth(state) > before {
                return Ok(ScriptStep::Pause);
            }
        }
        _ => unreachable!("opcode routed to wrong story adapter"),
    }
    Ok(ScriptStep::Continue)
}

/// The loaded land's region table, or a saved land's, taken from the scenario on
/// first use.
///
/// Original: loadland
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

/// After a teleport, a destination floor value of magnitude 1000 or more starts
/// the walking event at the party from its first action, after the trigger roll.
/// False where the script returns early.
///
/// Original: newland
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
        state.globals.header_row = row as i16;
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

/// Makes another land the one the party is in, saving the land it leaves; nothing happens when it
/// is the current land.
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
