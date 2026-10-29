use crate::{RandomState, RuleError};
use centurions_content::records::{
    EventChanceExtra, EventHeaderTable, EventHeaderTableMut, EventReplacementExtra, StoryRecord,
    TileMutationExtra,
};
use centurions_content::{Warrior, mission::MAP_CELLS, mission::OpeningMission, span};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileLayer {
    Floor,
    Object,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TileMutation {
    pub map: i16,
    pub index: usize,
    pub layer: TileLayer,
    pub value: i16,
}

pub fn tile_mutation(extra: &[u8]) -> Result<TileMutation, RuleError> {
    let extra = TileMutationExtra(span(extra, 0, TileMutationExtra::BYTES)?);
    let map = extra.map()?;
    if map < 0 {
        return Err(RuleError::Invalid(
            "tile mutation map is unavailable".into(),
        ));
    }
    let index = OpeningMission::index(i32::from(extra.x()?), i32::from(extra.y()?))?;
    let layer = if extra.layer()? == 2 {
        TileLayer::Object
    } else {
        TileLayer::Floor
    };
    let mut value = extra.tile()?;
    if layer == TileLayer::Object && value > 22999 {
        value = value.wrapping_sub(23000);
    }
    Ok(TileMutation {
        map,
        index,
        layer,
        value,
    })
}

pub fn apply_tile(layer: &[i16], mutation: &TileMutation) -> Result<Vec<i16>, RuleError> {
    if layer.len() != MAP_CELLS || mutation.index >= MAP_CELLS {
        return Err(RuleError::Invalid(
            "tile mutation layer is unavailable".into(),
        ));
    }
    let mut result = layer.to_vec();
    result[mutation.index] = mutation.value;
    Ok(result)
}

/// newland -12 at 143730 writes WORDs from objects40c500 into the following
/// LONG event layer410448. Authored extra618 includes column90; do not clip it.
pub(crate) fn clear_object_rectangle(
    objects: &mut [i16],
    events: &mut [i32],
    first: (i32, i32),
    last: (i32, i32),
) -> Result<(), RuleError> {
    if objects.len() != MAP_CELLS || events.len() != MAP_CELLS {
        return Err("object rectangle layers unavailable".into());
    }
    if first.0 > last.0 || first.1 > last.1 {
        return Ok(());
    }
    // D3 is unsigned for MULU; D6 is a signed WORD index. ADDQ.W at the
    // signed maximum would wrap and keep the original inclusive loop running.
    let start = i64::from(first.0) * 90 + i64::from(first.1);
    let end = i64::from(last.0) * 90 + i64::from(last.1);
    if first.0 < 0
        || first.1 < i32::from(i16::MIN)
        || last.0 >= i32::from(i16::MAX)
        || last.1 >= i32::from(i16::MAX)
        || start < 0
        || end >= (MAP_CELLS * 3) as i64
    {
        return Err(RuleError::Unsupported {
            action: "story.clear_objects",
            dependency: "rectangle writes outside object and event layers or wraps a WORD loop",
        });
    }
    for x in first.0..=last.0 {
        for y in first.1..=last.1 {
            let index = (x * 90 + y) as usize;
            if index < MAP_CELLS {
                objects[index] = 0;
            } else {
                let word = index - MAP_CELLS;
                let mask = if word.is_multiple_of(2) {
                    0x0000_ffff
                } else {
                    0xffff_0000
                };
                events[word / 2] = (events[word / 2] as u32 & mask) as i32;
            }
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventReplacement {
    pub map: i16,
    pub slot: usize,
    pub script: i16,
}

pub fn event_replacement(extra: &[u8]) -> Result<EventReplacement, RuleError> {
    let extra = EventReplacementExtra(span(extra, 0, EventReplacementExtra::BYTES)?);
    let map = extra.map()?;
    if map < 0 {
        return Err(RuleError::Unsupported {
            action: "story.replace_event",
            dependency: "replacement of NPC or encounter menu branches",
        });
    }
    let slot = extra.slot()?;
    if !(0..100).contains(&slot) {
        return Err(RuleError::Invalid(
            "event replacement slot is unavailable".into(),
        ));
    }
    Ok(EventReplacement {
        map,
        slot: slot as usize,
        script: extra.script()?,
    })
}

pub fn replace_event_script(
    headers: &[u8],
    slot: usize,
    replacement: &[u8],
) -> Result<Vec<u8>, RuleError> {
    let mut result = span(headers, 0, EventHeaderTable::BYTES)?.to_vec();
    if slot >= 100 {
        return Err(RuleError::Invalid(
            "event replacement slot is unavailable".into(),
        ));
    }
    let replacement = span(replacement, 0, StoryRecord::BYTES)?;
    EventHeaderTableMut(&mut result).replace_script(slot, replacement);
    Ok(result)
}

pub fn set_event_chances(headers: &[u8], extra: &[u8]) -> Result<Vec<u8>, RuleError> {
    let extra = EventChanceExtra(span(extra, 0, EventChanceExtra::BYTES)?);
    let mut slots = Vec::new();
    let single = extra.slot()?;
    if single != 0 {
        slots.push(single);
    }
    let start = extra.first_slot()?;
    let end = extra.last_slot()?;
    if start != 0 && start <= end {
        if start < 0 || end >= 100 {
            return Err(RuleError::Invalid(
                "event chance range is unavailable".into(),
            ));
        }
        slots.extend(start..=end);
    }
    if slots.iter().any(|slot| !(0..100).contains(slot)) {
        return Err(RuleError::Invalid(
            "event chance slot is unavailable".into(),
        ));
    }
    let mut result = span(headers, 0, EventHeaderTable::BYTES)?.to_vec();
    let chance = extra.chance()? as u8;
    let mut table = EventHeaderTableMut(&mut result);
    for slot in slots {
        table.set_trigger_chance(slot as usize, chance);
    }
    Ok(result)
}

pub fn secret_search_skill(warriors: &[Warrior]) -> Result<i16, RuleError> {
    if warriors.is_empty() || warriors.len() > 6 {
        return Err(RuleError::Invalid(
            "secret search squad is unavailable".into(),
        ));
    }
    let mut sum = 0_i16;
    for warrior in warriors {
        let record = warrior.record();
        sum = sum.wrapping_add(
            record
                .special_skill(4)?
                .wrapping_add(record.special_skill_bonus(4)?),
        );
    }
    Ok(sum / warriors.len() as i16)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecretDiscovery {
    pub index: usize,
    pub ordinal: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecretSearch {
    pub floor: Vec<i16>,
    pub discoveries: Vec<SecretDiscovery>,
    pub center_revealed: bool,
}

pub fn discover_secrets(
    floor: &[i16],
    position: (i16, i16),
    skill: i16,
    random: &mut RandomState,
) -> Result<SecretSearch, RuleError> {
    if floor.len() != MAP_CELLS {
        return Err(RuleError::Invalid(
            "secret search layer is unavailable".into(),
        ));
    }
    let (px, py) = position;
    OpeningMission::index(i32::from(px), i32::from(py))?;
    let mut result = SecretSearch {
        floor: floor.to_vec(),
        discoveries: Vec::new(),
        center_revealed: false,
    };
    let mut ordinal = 0;
    for y in py - 1..=py + 1 {
        for x in px - 1..=px + 1 {
            if !(0..90).contains(&x) || !(0..90).contains(&y) {
                continue;
            }
            ordinal += 1;
            let index = OpeningMission::index(i32::from(x), i32::from(y))?;
            let tile = result.floor[index];
            if tile.wrapping_abs() <= 2999 || random.bound(100) > skill {
                continue;
            }
            result.floor[index] = if tile > 0 {
                tile.wrapping_sub(1000)
            } else {
                tile.wrapping_add(1000)
            };
            result.discoveries.push(SecretDiscovery { index, ordinal });
            if x == px && y == py {
                result.center_revealed = true;
                return Ok(result);
            }
        }
    }
    Ok(result)
}
