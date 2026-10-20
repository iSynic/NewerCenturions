use crate::{Presentation, RuleError, ScriptState};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Exploration {
    pub map: i32,
    pub x: i32,
    pub y: i32,
    pub day: i16,
    pub hour: i16,
    pub minute: i16,
    pub fatigue: i16,
    pub facing: u8,
    #[serde(default)]
    pub last_move: [i8; 2],
    pub script: Option<ScriptState>,
    pub presentation: Option<Presentation>,
    pub event_headers: Vec<u8>,
    #[serde(default)]
    pub schedule: Vec<u8>,
    #[serde(default)]
    pub region_state: Vec<u8>,
    #[serde(default)]
    pub visibility: Vec<u8>,
    #[serde(default)]
    pub floor: Vec<i16>,
    #[serde(default)]
    pub objects: Vec<i16>,
    #[serde(default)]
    pub events: Vec<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapState {
    pub event_headers: Vec<u8>,
    pub regions: Vec<u8>,
    pub visibility: Vec<u8>,
    pub floor: Vec<i16>,
    pub objects: Vec<i16>,
    pub events: Vec<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LevelState {
    pub maps: std::collections::BTreeMap<i32, MapState>,
    pub schedule: Vec<u8>,
    pub menus: Vec<u8>,
    #[serde(default)]
    pub typed_encounters: Vec<u8>,
}

impl MapState {
    pub fn from_native(bytes: &[u8]) -> Result<Self, RuleError> {
        let bytes = centurions_content::span(bytes, 0, 77744)?;
        Ok(Self {
            event_headers: bytes[..4200].to_vec(),
            regions: bytes[4200..4844].to_vec(),
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
    pub fn native_bytes(&self) -> Result<Vec<u8>, RuleError> {
        if self.event_headers.len() != 4200
            || self.regions.len() != 644
            || self.visibility.len() != 8100
            || self.floor.len() != 8100
            || self.objects.len() != 8100
            || self.events.len() != 8100
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
    pub fn authored(map: &centurions_content::mission::OpeningMission) -> Self {
        Self {
            event_headers: map.header.clone(),
            regions: map.regions.clone(),
            visibility: vec![0; 8100],
            floor: map.floor.clone(),
            objects: map.objects.clone(),
            events: map.events.clone(),
        }
    }
    pub(crate) fn validate(
        &self,
        map: &centurions_content::mission::OpeningMission,
    ) -> Result<(), RuleError> {
        if self.event_headers.len() != map.header.len()
            || self.regions.len() != map.regions.len()
            || self.visibility.len() != 8100
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
