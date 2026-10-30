//! Map reads and writes past the edge of the 90 × 90 battle map.
//!
//! The game indexes the map as column × 90 + row without clipping, so a cell off
//! the map lands in what lies beside it: the tutorial rectangles and round values
//! before it, and the land's objects and events after it (see
//! docs/source-notes/combat.md).
use super::CombatField;
use crate::RuleError;
use centurions_content::mission::{MissionStage, OpeningMission};
use centurions_content::records::{SpellMapAliases, TutorialRectangles};

/// The number of cells on the battle map.
const CELLS: i32 = 8100;
/// The end of the land's objects, which follow the map.
const OBJECTS_END: i32 = 2 * CELLS;
/// The end of the land's events, which follow the objects.
const EVENTS_END: i32 = 4 * CELLS;

/// The owner of one linear map index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cell {
    Tile(usize),
    /// The word just before the map, index -1.
    Edge,
    /// A word the session holds read-only, or not at all.
    Other,
    /// The land's objects, indexes 8100..16200.
    Object(usize),
    /// The land's events, indexes 16200..32400: the high or low half of one.
    Event {
        cell: usize,
        high: bool,
    },
}

/// Finds what a map index lands on, inside the map or beside it.
pub(crate) fn locate(index: i32) -> Cell {
    match index {
        0..CELLS => Cell::Tile(index as usize),
        -1 => Cell::Edge,
        CELLS..OBJECTS_END => Cell::Object((index - CELLS) as usize),
        OBJECTS_END..EVENTS_END => {
            let offset = index - OBJECTS_END;
            Cell::Event {
                cell: (offset / 2) as usize,
                high: offset % 2 == 0,
            }
        }
        _ => Cell::Other,
    }
}

/// Read-only owners for indexes past the map: the tutorial rectangles and the land arrays.
#[derive(Clone, Copy, Default)]
pub struct MapAliases<'a> {
    tutorial: Option<&'a [u8]>,
    objects: &'a [i16],
    events: &'a [i32],
    stages: &'a [MissionStage],
}

impl<'a> MapAliases<'a> {
    /// The tutorial rectangles only; ToolBoxInit loads them once.
    pub fn of_mission(mission: &'a OpeningMission) -> Self {
        Self {
            tutorial: mission.tutorial_rectangles.as_deref(),
            ..Self::default()
        }
    }

    /// The current land's objects and events, as the session holds them.
    pub fn with_land(self, objects: &'a [i16], events: &'a [i32]) -> Self {
        Self {
            objects,
            events,
            ..self
        }
    }

    /// The aliases with the tutorial rectangles of the mission.
    pub fn with_tutorial(self, mission: &'a OpeningMission) -> Self {
        Self {
            tutorial: mission.tutorial_rectangles.as_deref(),
            ..self
        }
    }

    /// The aliases with the stages of the mission.
    pub fn with_stages(self, stages: &'a [MissionStage]) -> Self {
        Self { stages, ..self }
    }

    /// The value at `index`, or None where the session does not hold it.
    pub fn read(self, map: &CombatField, index: i32) -> Result<Option<i16>, RuleError> {
        match locate(index) {
            Cell::Tile(cell) => return Ok(map.tiles.get(cell).copied()),
            Cell::Edge => return Ok(Some(map.edge_word)),
            _ => {}
        }
        if let Some(rectangles) = self.tutorial
            && let Some(word) = TutorialRectangles(rectangles).tactical_alias(index)?
        {
            return Ok(Some(word));
        }
        Ok(SpellMapAliases::new(self.objects, self.events, self.stages).tactical_word(index)?)
    }
}

/// Owned land arrays for work that runs on copies of the session.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Land {
    pub objects: Vec<i16>,
    pub events: Vec<i32>,
}

/// The tutorial rectangles and, when a caller copied them, the land arrays.
pub(crate) fn land_aliases<'a>(
    mission: &'a OpeningMission,
    land: Option<&'a Land>,
) -> MapAliases<'a> {
    let aliases = MapAliases::of_mission(mission);
    match land {
        Some(land) => aliases.with_land(&land.objects, &land.events),
        None => aliases,
    }
}

/// Writes `value` at `index` into the map, the word before it or the land arrays.
/// Returns false, writing nothing, where the session cannot write.
pub(crate) fn write(
    map: &mut CombatField,
    land: Option<(&mut [i16], &mut [i32])>,
    index: i32,
    value: i16,
) -> bool {
    match locate(index) {
        Cell::Tile(cell) => map.tiles.get_mut(cell).map(|tile| *tile = value).is_some(),
        Cell::Edge => {
            map.edge_word = value;
            true
        }
        Cell::Object(cell) => land
            .and_then(|(objects, _)| objects.get_mut(cell))
            .map(|object| *object = value)
            .is_some(),
        Cell::Event { cell, high } => land
            .and_then(|(_, events)| events.get_mut(cell))
            .map(|event| {
                let bits = *event as u32;
                *event = if high {
                    (bits & 0xffff | u32::from(value as u16) << 16) as i32
                } else {
                    (bits & 0xffff_0000 | u32::from(value as u16)) as i32
                };
            })
            .is_some(),
        Cell::Other => false,
    }
}

/// The session's tutorial rectangles and current land arrays, borrowed field by field.
macro_rules! session_map_aliases {
    ($state:expr) => {
        $crate::combat::map_alias::session_map_aliases!(
            $state,
            aliases $crate::combat::map_alias::MapAliases::default()
        )
    };
    ($state:expr, content $content:expr) => {
        match $state.land($content) {
            Some(mission) => $crate::combat::map_alias::session_map_aliases!($state, mission),
            None => $crate::combat::map_alias::session_map_aliases!(
                $state,
                aliases $crate::combat::map_alias::MapAliases::default()
            ),
        }
    };
    ($state:expr, $mission:expr) => {
        $crate::combat::map_alias::session_map_aliases!(
            $state,
            aliases $crate::combat::map_alias::MapAliases::of_mission($mission)
        )
    };
    ($state:expr, aliases $aliases:expr) => {
        $aliases.with_land(
            $state
                .exploration
                .as_ref()
                .map_or(&[][..], |position| &position.objects[..]),
            $state
                .exploration
                .as_ref()
                .map_or(&[][..], |position| &position.events[..]),
        )
    };
}
pub(crate) use session_map_aliases;
