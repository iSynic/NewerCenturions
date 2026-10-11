use crate::{RandomState, RuleError};
use centurions_content::{Warrior, mission::MAP_CELLS, mission::OpeningMission, span, word};

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
    let extra = span(extra, 0, 10)?;
    let map = word(extra, 0)?;
    if map < 0 {
        return Err(RuleError::Invalid(
            "tile mutation map is unavailable".into(),
        ));
    }
    let index = OpeningMission::index(i32::from(word(extra, 2)?), i32::from(word(extra, 4)?))?;
    let layer = if word(extra, 8)? == 2 {
        TileLayer::Object
    } else {
        TileLayer::Floor
    };
    let mut value = word(extra, 6)?;
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventReplacement {
    pub map: i16,
    pub slot: usize,
    pub script: i16,
}

pub fn event_replacement(extra: &[u8]) -> Result<EventReplacement, RuleError> {
    let extra = span(extra, 0, 10)?;
    let map = word(extra, 0)?;
    if map < 0 {
        return Err(RuleError::Unsupported {
            action: "story.replace_event",
            dependency: "replacement of NPC or encounter menu branches",
        });
    }
    let slot = word(extra, 2)?;
    if !(0..100).contains(&slot) {
        return Err(RuleError::Invalid(
            "event replacement slot is unavailable".into(),
        ));
    }
    Ok(EventReplacement {
        map,
        slot: slot as usize,
        script: word(extra, 4)?,
    })
}

pub fn replace_event_script(
    headers: &[u8],
    slot: usize,
    replacement: &[u8],
) -> Result<Vec<u8>, RuleError> {
    let mut result = span(headers, 0, 4200)?.to_vec();
    if slot >= 100 {
        return Err(RuleError::Invalid(
            "event replacement slot is unavailable".into(),
        ));
    }
    let replacement = span(replacement, 0, 42)?;
    result[slot * 42 + 8..slot * 42 + 41].copy_from_slice(&replacement[8..41]);
    Ok(result)
}

pub fn set_event_chances(headers: &[u8], extra: &[u8]) -> Result<Vec<u8>, RuleError> {
    let extra = span(extra, 0, 10)?;
    let mut slots = Vec::new();
    let single = word(extra, 2)?;
    if single != 0 {
        slots.push(single);
    }
    let start = word(extra, 6)?;
    let end = word(extra, 8)?;
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
    let mut result = span(headers, 0, 4200)?.to_vec();
    let chance = word(extra, 4)? as u8;
    for slot in slots {
        result[slot as usize * 42 + 7] = chance;
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
        sum = sum
            .wrapping_add(word(&warrior.bytes, 0xfc)?.wrapping_add(word(&warrior.bytes, 0xb6a)?));
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
