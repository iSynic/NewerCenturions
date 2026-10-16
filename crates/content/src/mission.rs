use crate::{ContentError, span, word};
use std::{ops::Deref, sync::Arc};

pub const MAP_SIDE: usize = 90;
pub const MAP_CELLS: usize = MAP_SIDE * MAP_SIDE;
const SCENARIO: &str = "TheNewCenturions/Scenarios/The Thunder of Guns/0/";

#[derive(Clone, Debug)]
pub struct OpeningMission {
    pub name: String,
    pub start: (i32, i32),
    pub header: Vec<u8>,
    pub regions: Vec<u8>,
    pub floor: Vec<i16>,
    pub objects: Vec<i16>,
    pub events: Vec<i32>,
    pub tile_properties: Vec<u8>,
    pub terrain: u8,
    pub default_tile: i16,
    pub minutes_per_tick: i16,
    pub shared: Arc<StoryTables>,
}

#[derive(Clone, Debug, Default)]
pub struct StoryTables {
    pub areas: Vec<u8>,
    pub item_cache: Vec<u8>,
    pub races: Vec<u8>,
    pub profiles: Vec<u8>,
    pub door_scripts: Vec<u8>,
    pub dialogue: Vec<u8>,
    pub treasure: Vec<u8>,
    pub shops: Vec<u8>,
    pub prologue: String,
    pub item_descriptions: Vec<u8>,
    pub stock_item_descriptions: Vec<u8>,
    pub scan_descriptions: Vec<u8>,
    pub schedule: Vec<u8>,
    pub scheduled_scripts: Vec<u8>,
    pub menus: Vec<u8>,
    pub typed_encounters: Vec<u8>,
    pub movies: std::collections::BTreeMap<i16, String>,
    pub combat: Option<crate::combat::CombatContent>,
    pub creature_descriptions: Vec<u8>,
    pub archive_order: Option<Vec<usize>>,
    pub map_notes: Vec<u8>,
    pub note_labels: Vec<String>,
    pub unknown_note_labels: Vec<String>,
}

impl Deref for OpeningMission {
    type Target = StoryTables;
    fn deref(&self) -> &Self::Target {
        &self.shared
    }
}

#[derive(Clone, Debug)]
pub struct MissionStage {
    pub record: Vec<u8>,
    pub name: String,
    pub start: (i32, i32, i32),
    pub start_script: i16,
}

#[derive(Clone, Debug)]
pub struct ScenarioContent {
    pub maps: Vec<OpeningMission>,
    pub stages: Vec<MissionStage>,
    pub name: String,
    pub level: u8,
    pub opening_map: i32,
    pub items: Vec<Vec<u8>>,
    pub catalog: Arc<std::collections::BTreeMap<(String, u8), ScenarioContent>>,
}

impl ScenarioContent {
    pub fn single(map: OpeningMission) -> Self {
        Self {
            maps: vec![map],
            stages: vec![],
            name: "The Thunder of Guns".into(),
            level: 0,
            opening_map: 0,
            items: vec![],
            catalog: Default::default(),
        }
    }

    pub fn read(
        mut read: impl FnMut(&str) -> Result<Vec<u8>, ContentError>,
    ) -> Result<Self, ContentError> {
        Self::read_level(&mut read, "The Thunder of Guns", 0)
    }

    pub fn read_level(
        mut read: impl FnMut(&str) -> Result<Vec<u8>, ContentError>,
        name: &str,
        level: u8,
    ) -> Result<Self, ContentError> {
        let directory = format!("TheNewCenturions/Scenarios/{name}/{level}/");
        let mut selected = |path: &str| read(&path.replace(SCENARIO, &directory));
        let read = &mut selected;
        let opening = OpeningMission::read(&mut *read)?;
        let land = read(&format!("{SCENARIO}Data LD"))?;
        let headers = read(&format!("{SCENARIO}Data DD"))?;
        let regions = read(&format!("{SCENARIO}Data RD"))?;
        if !land.len().is_multiple_of(MAP_CELLS * 8)
            || land.is_empty()
            || land.len() > 20 * MAP_CELLS * 8
        {
            return Err(ContentError::Invalid("invalid scenario map table".into()));
        }
        let mut maps = Vec::new();
        for index in 0..land.len() / (MAP_CELLS * 8) {
            let region = span(&regions, index * 644, 644)?.to_vec();
            let terrain = region[0x208];
            let filename = match terrain {
                1 => "Data P BD",
                2 => "Data Swamp BD",
                3 => "Data Snow BD",
                4 => "Data Sub BD",
                5 => "Data Desert BD",
                6 => "Data Lava BD",
                7 => "Data Castle BD",
                8 => "Data Techno BD",
                9 => "Data Biologic BD",
                _ => return Err(ContentError::Invalid(format!("unknown terrain {terrain}"))),
            };
            let properties = read(&format!("TheNewCenturions/Data Files/{filename}"))?;
            let raw = span(&land, index * MAP_CELLS * 8, MAP_CELLS * 8)?;
            maps.push(OpeningMission {
                name: opening.name.clone(),
                start: opening.start,
                header: span(&headers, index * 4200, 4200)?.to_vec(),
                regions: region,
                floor: span(raw, 0, MAP_CELLS * 2)?
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|v| i16::from_be_bytes(*v))
                    .collect(),
                objects: span(raw, MAP_CELLS * 2, MAP_CELLS * 2)?
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|v| i16::from_be_bytes(*v))
                    .collect(),
                events: span(raw, MAP_CELLS * 4, MAP_CELLS * 4)?
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|v| i32::from_be_bytes(*v))
                    .collect(),
                tile_properties: span(&properties, 64, 12000)?.to_vec(),
                terrain,
                default_tile: word(&properties, 0)?,
                minutes_per_tick: if word(&properties, 2)? == 0 { 5 } else { 1 },
                shared: opening.shared.clone(),
            });
        }
        let mission = read(&format!("{SCENARIO}Data Mission"))?;
        let mut stages = Vec::new();
        for raw in mission.as_chunks::<464>().0 {
            let name = encoding_rs::MACINTOSH
                .decode(span(raw, 7, usize::from(raw[6]))?)
                .0
                .into_owned();
            let start = (
                i32::from(word(raw, 0)?),
                i32::from(word(raw, 2)?),
                i32::from(word(raw, 4)?),
            );
            if !name.is_empty() && (start.2 < 0 || start.2 as usize >= maps.len()) {
                return Err(ContentError::Invalid(
                    "stage start map is unavailable".into(),
                ));
            }
            stages.push(MissionStage {
                record: raw.to_vec(),
                name,
                start,
                start_script: word(raw, 262)?,
            });
        }
        let setup = read(&format!("{SCENARIO}Data CS"))?;
        let opening_map = long(&setup, 8)?;
        if opening_map < 0 || opening_map as usize >= maps.len() {
            return Err(ContentError::Invalid("opening map unavailable".into()));
        }
        let items = span(
            &read(&format!("{SCENARIO}Data NI"))?,
            0,
            200 * crate::ITEM_BYTES,
        )?
        .as_chunks::<{ crate::ITEM_BYTES }>()
        .0
        .iter()
        .map(|row| row.to_vec())
        .collect();
        Ok(Self {
            maps,
            stages,
            name: name.into(),
            level,
            opening_map,
            items,
            catalog: Default::default(),
        })
    }

    pub fn selection(&self, name: &str, level: u8) -> Option<&Self> {
        if self.name == name && self.level == level {
            Some(self)
        } else {
            self.catalog.get(&(name.to_string(), level))
        }
    }
}

pub fn long(bytes: &[u8], offset: usize) -> Result<i32, ContentError> {
    Ok(i32::from_be_bytes(
        span(bytes, offset, 4)?.try_into().unwrap(),
    ))
}

impl OpeningMission {
    pub fn read(
        mut read: impl FnMut(&str) -> Result<Vec<u8>, ContentError>,
    ) -> Result<Self, ContentError> {
        let setup = read(&format!("{SCENARIO}Data CS"))?;
        let start = (long(&setup, 12)?, long(&setup, 16)?);
        if !(0..90).contains(&start.0) || !(0..90).contains(&start.1) {
            return Err(ContentError::Invalid(
                "opening coordinates outside map".into(),
            ));
        }
        let mission = read(&format!("{SCENARIO}Data Mission"))?;
        let name_bytes = span(&mission, 7, usize::from(span(&mission, 6, 1)?[0]))?;
        let name = encoding_rs::MACINTOSH.decode(name_bytes).0.into_owned();
        let header = span(&read(&format!("{SCENARIO}Data DD"))?, 0, 4200)?.to_vec();
        let regions = span(&read(&format!("{SCENARIO}Data RD"))?, 0, 644)?.to_vec();
        let terrain = regions[0x208];
        let data = read(&format!("{SCENARIO}Data LD"))?;
        let floor = span(&data, 0, MAP_CELLS * 2)?
            .as_chunks::<2>()
            .0
            .iter()
            .map(|v| i16::from_be_bytes(*v))
            .collect();
        let objects = span(&data, MAP_CELLS * 2, MAP_CELLS * 2)?
            .as_chunks::<2>()
            .0
            .iter()
            .map(|v| i16::from_be_bytes(*v))
            .collect();
        let events = span(&data, MAP_CELLS * 4, MAP_CELLS * 4)?
            .as_chunks::<4>()
            .0
            .iter()
            .map(|v| i32::from_be_bytes(*v))
            .collect();
        let filename = match terrain {
            1 => "Data P BD",
            2 => "Data Swamp BD",
            3 => "Data Snow BD",
            4 => "Data Sub BD",
            5 => "Data Desert BD",
            6 => "Data Lava BD",
            7 => "Data Castle BD",
            8 => "Data Techno BD",
            9 => "Data Biologic BD",
            _ => return Err(ContentError::Invalid(format!("unknown terrain {terrain}"))),
        };
        let properties = read(&format!("TheNewCenturions/Data Files/{filename}"))?;
        let schedule = read(&format!("{SCENARIO}Data TD3"))?;
        if schedule.is_empty() || !schedule.len().is_multiple_of(40) || schedule.len() > 151 * 40 {
            return Err(ContentError::Invalid(
                "invalid scheduled-event table".into(),
            ));
        }
        Ok(Self {
            name,
            start,
            header,
            regions,
            floor,
            objects,
            events,
            tile_properties: span(&properties, 64, 12000)?.to_vec(),
            terrain,
            default_tile: word(&properties, 0)?,
            minutes_per_tick: if word(&properties, 2)? == 0 { 5 } else { 1 },
            shared: Arc::new(StoryTables {
                areas: read("TheNewCenturions/Data Files/Data AD")?,
                item_cache: Vec::new(),
                races: read("TheNewCenturions/Data Files/Data Race")?,
                profiles: read("TheNewCenturions/Data Files/Data MOS")?,
                door_scripts: read(&format!("{SCENARIO}Data ED3"))?,
                dialogue: read(&format!("{SCENARIO}Data SD2"))?,
                treasure: read(&format!("{SCENARIO}Data TD"))?,
                shops: read(&format!("{SCENARIO}Data SD"))?,
                prologue: String::new(),
                item_descriptions: read(&format!("{SCENARIO}Data NI Des"))?,
                stock_item_descriptions: read("TheNewCenturions/Data Files/Data ID DES")?,
                scan_descriptions: read("TheNewCenturions/Data Files/Data Scan DES")?,
                schedule,
                scheduled_scripts: read(&format!("{SCENARIO}Data ED3"))?,
                menus: read(&format!("{SCENARIO}Data ED"))?,
                typed_encounters: read(&format!("{SCENARIO}Data ED2"))?,
                movies: Default::default(),
                combat: Some(crate::combat::CombatContent {
                    encounters: read(&format!("{SCENARIO}Data EDCD"))?,
                    battles: read(&format!("{SCENARIO}Data BD"))?,
                    monsters: read(&format!("{SCENARIO}Data MD"))?,
                    attacks: read("TheNewCenturions/Data Files/Data S")?,
                    race_moves: {
                        let races = read("TheNewCenturions/Data Files/Data Race")?;
                        (0..10)
                            .map(|index| span(&races, index * 408 + 197, 1).map(|v| v[0] as i8))
                            .collect::<Result<Vec<_>, _>>()?
                    },
                }),
                creature_descriptions: read(&format!("{SCENARIO}Data DES"))?,
                archive_order: None,
                map_notes: read(&format!("{SCENARIO}Data MD2"))?,
                note_labels: vec![],
                unknown_note_labels: vec![],
            }),
        })
    }

    pub fn index(x: i32, y: i32) -> Result<usize, ContentError> {
        if !(0..90).contains(&x) || !(0..90).contains(&y) {
            return Err(ContentError::Invalid(format!(
                "map coordinate {x},{y} outside 90x90"
            )));
        }
        Ok(x as usize * MAP_SIDE + y as usize)
    }
    pub fn tile_word(&self, tile: i16, field: usize) -> Result<i16, ContentError> {
        if !(0..300).contains(&tile) || field >= 40 || !field.is_multiple_of(2) {
            return Err(ContentError::Invalid(
                "tile property outside recovered table".into(),
            ));
        }
        word(&self.tile_properties, tile as usize * 40 + field)
    }
}

impl StoryTables {
    pub fn dialogue(&self, id: i16) -> Result<String, ContentError> {
        let record = span(&self.dialogue, usize::from(id.unsigned_abs()) * 256, 256)?;
        Ok(encoding_rs::MACINTOSH
            .decode(span(record, 1, usize::from(record[0]))?)
            .0
            .into_owned())
    }
    pub fn script(&self, id: i16) -> Result<[u8; 42], ContentError> {
        if id < 0 {
            return Err(ContentError::Invalid("negative door script".into()));
        }
        Ok(span(&self.door_scripts, id as usize * 42, 42)?
            .try_into()
            .unwrap())
    }
    pub fn item_description(&self, id: i16) -> Result<String, ContentError> {
        if !(0..1000).contains(&id) {
            return Err(ContentError::Invalid(
                "description outside scenario item band".into(),
            ));
        }
        let record = if id < 800 {
            span(&self.stock_item_descriptions, id as usize * 256, 256)?
        } else {
            span(&self.item_descriptions, (id - 800) as usize * 256, 256)?
        };
        Ok(encoding_rs::MACINTOSH
            .decode(span(record, 1, usize::from(record[0]))?)
            .0
            .into_owned())
    }
    pub fn scan_description(
        &self,
        school: i16,
        tier: usize,
        column: usize,
    ) -> Result<String, ContentError> {
        if !(1..=4).contains(&school) || tier >= 7 || column >= 7 {
            return Err(ContentError::Invalid(
                "scan description outside original matrix".into(),
            ));
        }
        let record = span(
            &self.scan_descriptions,
            ((school as usize - 1) * 49 + tier * 7 + column) * 256,
            256,
        )?;
        Ok(encoding_rs::MACINTOSH
            .decode(span(record, 1, usize::from(record[0]))?)
            .0
            .replace('\r', "\n"))
    }
}
