//! Story operations that rewrite a land's regions, objects, tiles and event headers,
//! preserving original dispatch order.
use super::super::active_table;
use super::{ExtraIndex, Operation, ScriptStep, load_extra};
use crate::{MapState, RuleError, Session, story, travel};
use centurions_content::mission::OpeningMission;
use centurions_content::records::{
    EventReplacementExtra, MapMutationExtra, ObjectRectangleExtra, RegionChangeExtra,
    RegionTableMut,
};
use centurions_content::{Content, span};

/// Runs the script operations that rewrite the land: its regions, objects, tiles, event headers and
/// event chances.
pub(super) fn execute(
    operation: Operation,
    parameter: i16,
    state: &mut Session,
    content: &Content,
    map: &OpeningMission,
) -> Result<ScriptStep, RuleError> {
    let position = state.exploration.as_mut().unwrap();
    match operation {
        Operation::ChangeRegion => {
            let extra = load_extra(&mut state.globals, map, ExtraIndex::Signed(parameter), None)?;
            let change = RegionChangeExtra(extra);
            let target = change.map()?;
            let authored = content
                .map(target)
                .ok_or_else(|| RuleError::Invalid("region target map unavailable".into()))?;
            if !(0..20).contains(&change.region()?) {
                return Err(RuleError::Invalid("region index unavailable".into()));
            }
            let regions = if target == position.map {
                &mut position.region_state
            } else {
                &mut state
                    .visited_maps
                    .entry(target)
                    .or_insert_with(|| MapState::authored(authored))
                    .regions
            };
            let region = change.region()? as usize;
            let mut regions = RegionTableMut(regions);
            regions.set_encounter_chance(region, change.encounter_chance());
            let minimum = change.battle_minimum()?;
            if minimum >= 0 {
                regions.set_battle_minimum(region, minimum);
            }
            let maximum = change.battle_maximum()?;
            if maximum >= 0 {
                regions.set_battle_maximum(region, maximum);
            }
        }
        Operation::ClearObjects => {
            let extra = load_extra(
                &mut state.globals,
                map,
                ExtraIndex::Magnitude(parameter),
                None,
            )?;
            let rectangle = ObjectRectangleExtra(extra);
            let target = rectangle.map()?;
            let authored = content
                .map_at(state.mission.as_deref(), state.scenario_level, target)
                .ok_or("object clearing map unavailable")?;
            let first = rectangle.first()?;
            let last = rectangle.last()?;
            let (objects, events) = if target == position.map {
                (&mut position.objects, &mut position.events)
            } else {
                let records = state
                    .visited_maps
                    .entry(target)
                    .or_insert_with(|| MapState::authored(authored));
                (&mut records.objects, &mut records.events)
            };
            travel::clear_object_rectangle(objects, events, first, last)?;
        }
        Operation::ReplaceMapEvent | Operation::ChangeTiles | Operation::SetEventChances => {
            let extra = load_extra(&mut state.globals, map, ExtraIndex::Signed(parameter), None)?;
            let target = MapMutationExtra(extra).map()?;
            if operation == Operation::ReplaceMapEvent && target < 0 {
                let (saved, authored) = match target {
                    -1 => (&mut state.menus, &map.menus),
                    -2 => (&mut state.typed_encounters, &map.typed_encounters),
                    _ => {
                        return Err(RuleError::Invalid(
                            "encounter replacement table unavailable".into(),
                        ));
                    }
                };
                let table = active_table(saved, authored);
                *saved = story::replace_encounter_branch(
                    table,
                    extra,
                    &map.script(EventReplacementExtra(extra).script()?)?,
                )?;
                let offset = usize::from(EventReplacementExtra(extra).slot()? as u16)
                    * story::MENU_RECORD_BYTES;
                let header = story::encounter_header_bytes(target == -2);
                state
                    .globals
                    .encounter_headers
                    .replace_header(target == -2, span(saved, offset, header)?)?;
                return Ok(ScriptStep::Continue);
            }
            let authored = content
                .map(target)
                .ok_or_else(|| RuleError::Invalid("map mutation target unavailable".into()))?;
            if operation == Operation::ChangeTiles {
                let mutation = travel::tile_mutation(extra)?;
                if target == position.map {
                    let layer = match mutation.layer {
                        travel::TileLayer::Floor => &mut position.floor,
                        travel::TileLayer::Object => &mut position.objects,
                    };
                    *layer = travel::apply_tile(layer, &mutation)?;
                } else {
                    let records = state
                        .visited_maps
                        .entry(target)
                        .or_insert_with(|| MapState::authored(authored));
                    let layer = match mutation.layer {
                        travel::TileLayer::Floor => &mut records.floor,
                        travel::TileLayer::Object => &mut records.objects,
                    };
                    *layer = travel::apply_tile(layer, &mutation)?;
                }
            } else {
                let headers = if target == position.map {
                    &mut position.event_headers
                } else {
                    &mut state
                        .visited_maps
                        .entry(target)
                        .or_insert_with(|| MapState::authored(authored))
                        .event_headers
                };
                *headers = if operation == Operation::SetEventChances {
                    travel::set_event_chances(headers, extra)?
                } else {
                    let replacement = travel::event_replacement(extra)?;
                    travel::replace_event_script(
                        headers,
                        replacement.slot,
                        &map.script(replacement.script)?,
                    )?
                };
            }
        }
        _ => unreachable!("opcode routed to wrong story adapter"),
    }
    Ok(ScriptStep::Continue)
}
