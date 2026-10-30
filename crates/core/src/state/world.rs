use crate::{Presentation, RuleError, ScriptState};
use serde::{Deserialize, Serialize};

/// Map event header bytes at the start of a map record.
pub(crate) const EVENT_HEADER_BYTES: usize = 4200;
/// Region state bytes after the event headers.
pub(crate) const REGION_BYTES: usize = 644;
/// Land map cells, 90 by 90: a visibility byte, a floor and object value and an event each.
pub(crate) const LAND_CELLS: usize = 8100;

/// Where the party is on a land and what it sees: map and square, the day and time, fatigue,
/// facing, the running script, the presentation, and the land's current event headers, regions,
/// floor, objects and events.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Exploration {
    /// The map the party is on.
    pub map: i32,
    /// The party's column.
    pub x: i32,
    /// The party's row.
    pub y: i32,
    /// The day of the game.
    pub day: i16,
    /// The hour of the day.
    pub hour: i16,
    /// The minute of the hour.
    pub minute: i16,
    /// The party's fatigue.
    pub fatigue: i16,
    /// The direction the party faces.
    pub facing: u8,
    /// The last step the party took, as a change of column and row.
    #[serde(default)]
    pub last_move: [i8; 2],
    /// The land script that is running, if any.
    pub script: Option<ScriptState>,
    /// What the player is looking at, if anything.
    pub presentation: Option<Presentation>,
    /// The event headers of the map as they now stand.
    pub event_headers: Vec<u8>,
    /// The calendar schedule of the scenario level.
    #[serde(default)]
    pub schedule: Vec<u8>,
    /// The map's regions as they now stand.
    #[serde(default)]
    pub region_state: Vec<u8>,
    /// Which squares the party has seen.
    #[serde(default)]
    pub visibility: Vec<u8>,
    /// The floor tile of each square.
    #[serde(default)]
    pub floor: Vec<i16>,
    /// The object tile of each square.
    #[serde(default)]
    pub objects: Vec<i16>,
    /// The event of each square.
    #[serde(default)]
    pub events: Vec<i32>,
}

/// The records of one land that change as the party plays: event headers, regions, what has been
/// seen, floor, objects and events.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapState {
    /// The map's event headers.
    pub event_headers: Vec<u8>,
    /// The map's regions.
    pub regions: Vec<u8>,
    /// Which squares the party has seen.
    pub visibility: Vec<u8>,
    /// The floor tile of each square.
    pub floor: Vec<i16>,
    /// The object tile of each square.
    pub objects: Vec<i16>,
    /// The event of each square.
    pub events: Vec<i32>,
}

/// The saved state of a scenario level the party has left: its maps, schedule and menus.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LevelState {
    /// The saved state of each map visited on the level.
    pub maps: std::collections::BTreeMap<i32, MapState>,
    /// The level's calendar schedule.
    pub schedule: Vec<u8>,
    /// The level's encounter menus.
    pub menus: Vec<u8>,
    /// The level's typed encounters.
    #[serde(default)]
    pub typed_encounters: Vec<u8>,
}

impl MapState {
    /// Reads a land's changing records from their stored layout.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, RuleError> {
        let bytes = centurions_content::span(bytes, 0, 77744)?;
        Ok(Self {
            event_headers: bytes[..EVENT_HEADER_BYTES].to_vec(),
            regions: bytes[EVENT_HEADER_BYTES..EVENT_HEADER_BYTES + REGION_BYTES].to_vec(),
            visibility: bytes[4844..12944].to_vec(),
            floor: bytes[12944..29144]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|v| i16::from_be_bytes(*v))
                .collect(),
            objects: bytes[29144..45344]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|v| i16::from_be_bytes(*v))
                .collect(),
            events: bytes[45344..77744]
                .as_chunks::<4>()
                .0
                .iter()
                .map(|v| i32::from_be_bytes(*v))
                .collect(),
        })
    }
    /// Writes a land's changing records in their stored layout; refused unless each has its full
    /// size.
    pub fn to_bytes(&self) -> Result<Vec<u8>, RuleError> {
        if self.event_headers.len() != EVENT_HEADER_BYTES
            || self.regions.len() != REGION_BYTES
            || self.visibility.len() != LAND_CELLS
            || self.floor.len() != LAND_CELLS
            || self.objects.len() != LAND_CELLS
            || self.events.len() != LAND_CELLS
        {
            return Err(RuleError::Invalid("invalid native map records".into()));
        }
        let mut bytes = self.event_headers.clone();
        bytes.extend(&self.regions);
        bytes.extend(&self.visibility);
        bytes.extend(self.floor.iter().flat_map(|v| v.to_be_bytes()));
        bytes.extend(self.objects.iter().flat_map(|v| v.to_be_bytes()));
        bytes.extend(self.events.iter().flat_map(|v| v.to_be_bytes()));
        Ok(bytes)
    }
    /// The land's records as the scenario authored them, with nothing seen yet.
    pub fn authored(map: &centurions_content::mission::OpeningMission) -> Self {
        Self {
            event_headers: map.header.clone(),
            regions: map.regions.clone(),
            visibility: vec![0; LAND_CELLS],
            floor: map.floor.clone(),
            objects: map.objects.clone(),
            events: map.events.clone(),
        }
    }
    /// Checks that a land's records match the sizes of the authored land.
    pub(crate) fn validate(
        &self,
        map: &centurions_content::mission::OpeningMission,
    ) -> Result<(), RuleError> {
        if self.event_headers.len() != map.header.len()
            || self.regions.len() != map.regions.len()
            || self.visibility.len() != LAND_CELLS
            || self.floor.len() != map.floor.len()
            || self.objects.len() != map.objects.len()
            || self.events.len() != map.events.len()
        {
            return Err(RuleError::Invalid("invalid persistent map records".into()));
        }
        Ok(())
    }
}

impl Exploration {
    /// The party on a land as a new mission finds it: at the land's starting square on day 1 at
    /// noon, with fatigue 4.
    pub(crate) fn authored(index: i32, map: &centurions_content::mission::OpeningMission) -> Self {
        Self {
            map: index,
            x: map.start.0,
            y: map.start.1,
            day: 1,
            hour: 12,
            minute: 0,
            facing: 0,
            last_move: [0; 2],
            script: None,
            presentation: None,
            fatigue: 4,
            event_headers: map.header.clone(),
            schedule: map.schedule.clone(),
            region_state: map.regions.clone(),
            visibility: vec![0; LAND_CELLS],
            floor: map.floor.clone(),
            objects: map.objects.clone(),
            events: map.events.clone(),
        }
    }

    /// The records of the land the party is on.
    pub fn map_state(&self) -> MapState {
        MapState {
            event_headers: self.event_headers.clone(),
            regions: self.region_state.clone(),
            visibility: self.visibility.clone(),
            floor: self.floor.clone(),
            objects: self.objects.clone(),
            events: self.events.clone(),
        }
    }
    /// Makes a land the one the party is on, with the records given.
    pub fn restore_map(&mut self, map: i32, records: MapState) {
        self.map = map;
        self.event_headers = records.event_headers;
        self.region_state = records.regions;
        self.visibility = records.visibility;
        self.floor = records.floor;
        self.objects = records.objects;
        self.events = records.events;
    }
}
