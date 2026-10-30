pub use crate::bytes::long;
use crate::tile_sets::{self, TileSet};
use crate::{ContentError, span, word};
use std::{
    ops::Deref,
    sync::{Arc, OnceLock},
};

/// Cells along one side of a map.
pub const MAP_SIDE: usize = 90;
/// Cells in a map.
pub const MAP_CELLS: usize = MAP_SIDE * MAP_SIDE;
/// The scenario directory the loaders read, before a level is substituted.
pub(crate) const SCENARIO: &str = "TheNewCenturions/Scenarios/The Thunder of Guns/0/";

/// One map of a scenario: its terrain, objects and events, with the tables the
/// whole scenario level shares.
#[derive(Clone, Debug)]
pub struct OpeningMission {
    /// The mission's name, from the first record of `Data Mission`.
    pub name: String,
    /// Where the party enters the opening map, from `Data CS`.
    pub start: (i32, i32),
    /// The map's 4200-byte record from `Data DD`, kept as stored.
    pub header: Vec<u8>,
    /// The map's 644-byte region record from `Data RD`; it names the map's tile set.
    pub regions: Vec<u8>,
    /// The tile on each cell of the map's floor layer, `MAP_CELLS` of them.
    pub floor: Vec<i16>,
    /// The tile on each cell of the object layer.
    pub objects: Vec<i16>,
    /// The event code on each cell; it says what stepping on the cell does.
    pub events: Vec<i32>,
    /// The tile set's stats per tile: sound, time, solidity and shore.
    pub map_stats: Vec<u8>,
    /// The terrain number of the tile set the map uses.
    pub terrain: u8,
    /// The tile set's default tile.
    pub default_tile: i16,
    /// Game minutes one step takes on this tile set.
    pub minutes_per_tick: i16,
    /// The tables every map of the scenario level uses.
    pub shared: Arc<StoryTables>,
}

/// The scenario level's data files, held as stored and read through the record
/// views.
#[derive(Clone, Debug, Default)]
pub struct StoryTables {
    /// `Data AD`: the spell area shapes, 49 bytes each.
    pub areas: Vec<u8>,
    /// The tutorial rectangle table, loaded once at startup and never changed.
    pub tutorial_rectangles: Option<Vec<u8>>,
    /// The first 200 records of the stock item table, `Data ID`.
    pub item_cache: Vec<u8>,
    /// `Data Race`: the race records.
    pub races: Vec<u8>,
    /// `Data MOS`: the profession and level-up tables.
    pub profiles: Vec<u8>,
    /// `Data ED3`: the story scripts, 42 bytes each, that doors and events run.
    pub door_scripts: Vec<u8>,
    /// `Data SD2`: the scenario's dialogue, 256 bytes a message.
    pub dialogue: Vec<u8>,
    /// `Data TD`: the treasure records.
    pub treasure: Vec<u8>,
    /// `Data SD`: the shop records.
    pub shops: Vec<u8>,
    /// The text shown as the game opens: `TEXT 1000` of the scenario.
    pub prologue: String,
    /// `Data NI Des`: descriptions of the scenario's own items.
    pub item_descriptions: Vec<u8>,
    /// `Data ID DES`: descriptions of the stock items.
    pub stock_item_descriptions: Vec<u8>,
    /// `Data Scan DES`: one description for each scan of each school.
    pub scan_descriptions: Vec<u8>,
    /// `Data TD3`: the timed events, 40 bytes each.
    pub schedule: Vec<u8>,
    /// `Data ED3` as the scripts the timed events run.
    pub scheduled_scripts: Vec<u8>,
    /// `Data ED`: the menu encounters a story offers.
    pub menus: Vec<u8>,
    /// `Data ED2`: the typed encounter records.
    pub typed_encounters: Vec<u8>,
    /// The scenario's `TEXT` resources by id, the movies a story script plays.
    pub movies: std::collections::BTreeMap<i16, String>,
    /// The battle, monster, encounter and attack tables.
    pub combat: Option<crate::combat::CombatContent>,
    /// `Data DES`: the creature descriptions, 256 bytes each.
    pub creature_descriptions: Vec<u8>,
    /// The scenario's Data MENU: 251 big-endian word record numbers, one-based.
    pub archive_table: Option<Vec<u16>>,
    /// `Data MD2`: the map notes, 340 bytes each.
    pub map_notes: Vec<u8>,
    /// The labels of the note list, from `STR# -102` of the scenario.
    pub note_labels: Vec<String>,
    /// The labels the note list shows before the party has identified a note, from
    /// `STR# -101` of the scenario.
    pub unknown_note_labels: Vec<String>,
}

impl Deref for OpeningMission {
    type Target = StoryTables;
    fn deref(&self) -> &Self::Target {
        &self.shared
    }
}

/// One stage of the mission, a 464-byte record of `Data Mission`.
#[derive(Clone, Debug)]
pub struct MissionStage {
    /// The record as stored.
    pub record: Vec<u8>,
    /// The stage's name; an empty name means the stage does not exist.
    pub name: String,
    /// Where the stage starts: x, y and the number of the map.
    pub start: (i32, i32, i32),
    /// The story script that runs as the stage begins; zero runs none.
    pub start_script: i16,
}

/// One level of a scenario: its maps, mission stages and tables, with the other
/// scenarios and levels the installation holds.
#[derive(Clone, Debug)]
pub struct ScenarioContent {
    /// The level's maps, in map-number order.
    pub maps: Vec<OpeningMission>,
    /// The mission's stages.
    pub stages: Vec<MissionStage>,
    /// The scenario's name.
    pub name: String,
    /// The level number the party plays this scenario at.
    pub level: u8,
    /// The map a new game begins on.
    pub opening_map: i32,
    /// The scenario's own item definitions, the 200 of `Data NI`.
    pub items: Vec<Vec<u8>>,
    /// The first long of the level's file named after the scenario, shown as
    /// "Recommended Levels" when choosing a party. Absent when that file is.
    pub recommended_levels: Option<i32>,
    /// The 0x140-byte party restrictions "Data RI". When present, choosing a
    /// party offers View Restrictions.
    pub restrictions: Option<Vec<u8>>,
    /// Every scenario level the installation holds, by name and level.
    pub catalog: Arc<std::collections::BTreeMap<(String, u8), ScenarioContent>>,
    /// The installation's tile sets, shared by every level.
    pub tile_sets: Arc<Vec<TileSet>>,
    switched: SwitchedMaps,
}

/// Each map under another tile set, made when a land first switches to it. A clone
/// starts empty, since the clone's maps may then be edited.
#[derive(Debug)]
struct SwitchedMaps(Vec<[OnceLock<OpeningMission>; 20]>);

impl Clone for SwitchedMaps {
    fn clone(&self) -> Self {
        switch_slots(self.0.len())
    }
}

/// Makes empty room for the switched copies of `maps` maps.
fn switch_slots(maps: usize) -> SwitchedMaps {
    SwitchedMaps(
        (0..maps)
            .map(|_| std::array::from_fn(|_| OnceLock::new()))
            .collect(),
    )
}

impl ScenarioContent {
    /// Wraps one map as a one-map scenario.
    pub fn single(map: OpeningMission) -> Self {
        Self {
            maps: vec![map],
            stages: vec![],
            name: "The Thunder of Guns".into(),
            level: 0,
            opening_map: 0,
            items: vec![],
            recommended_levels: None,
            restrictions: None,
            catalog: Default::default(),
            tile_sets: Default::default(),
            switched: switch_slots(1),
        }
    }

    /// Reads the opening scenario, The Thunder of Guns, level 0.
    pub fn read(
        mut read: impl FnMut(&str) -> Result<Vec<u8>, ContentError>,
    ) -> Result<Self, ContentError> {
        let tile_sets = Arc::new(tile_sets::read_all(&mut read));
        Self::read_level(&mut read, "The Thunder of Guns", 0, tile_sets)
    }

    /// Reads one level of the scenario `name` from its directory, with the maps, mission
    /// stages, items and party tables it holds.
    pub fn read_level(
        mut read: impl FnMut(&str) -> Result<Vec<u8>, ContentError>,
        name: &str,
        level: u8,
        tile_sets: Arc<Vec<TileSet>>,
    ) -> Result<Self, ContentError> {
        let directory = format!("TheNewCenturions/Scenarios/{name}/{level}/");
        let mut selected = |path: &str| read(&path.replace(SCENARIO, &directory));
        let read = &mut selected;
        let opening = OpeningMission::read(&mut *read)?;
        let mut level_sets = tile_sets.as_ref().clone();
        if let Ok(custom) = tile_sets::read(read, 15) {
            level_sets.push(custom);
        }
        let tile_sets = Arc::new(level_sets);
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
            let set = match tile_sets.iter().find(|set| {
                set.terrain == terrain || (15..20).contains(&terrain) && set.terrain == 15
            }) {
                Some(set) => set.clone(),
                None => tile_sets::read(read, terrain)?,
            };
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
                map_stats: set.properties,
                terrain,
                default_tile: set.default_tile,
                minutes_per_tick: set.minutes_per_tick,
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
        let recommended_levels = read(&format!("{SCENARIO}{name}"))
            .ok()
            .and_then(|bytes| long(&bytes, 0).ok());
        let restrictions = read(&format!("{SCENARIO}Data RI")).ok().map(|mut bytes| {
            bytes.resize(crate::records::PartyRestrictions::BYTES, 0);
            bytes
        });
        Ok(Self {
            switched: switch_slots(maps.len()),
            maps,
            stages,
            name: name.into(),
            level,
            opening_map,
            items,
            recommended_levels,
            restrictions,
            catalog: Default::default(),
            tile_sets,
        })
    }

    /// The map under the tile set its live region table names. A map keeps its authored
    /// set until a story switches it; None for a set the installation lacks.
    pub fn land(&self, index: i32, terrain: u8) -> Option<&OpeningMission> {
        let index = usize::try_from(index).ok()?;
        let map = self.maps.get(index)?;
        // The authored region table names the authored set.
        if map.terrain == terrain || map.regions.get(520) == Some(&terrain) {
            return Some(map);
        }
        let set = self.tile_sets.iter().find(|set| {
            set.terrain == terrain || (15..20).contains(&terrain) && set.terrain == 15
        })?;
        let slot = self.switched.0.get(index)?.get(usize::from(terrain))?;
        Some(slot.get_or_init(|| OpeningMission {
            map_stats: set.properties.clone(),
            terrain,
            default_tile: set.default_tile,
            minutes_per_tick: set.minutes_per_tick,
            ..map.clone()
        }))
    }

    /// The scenario level called `name` at `level`: this one, or one from the catalog.
    pub fn selection(&self, name: &str, level: u8) -> Option<&Self> {
        if self.name == name && self.level == level {
            Some(self)
        } else {
            self.catalog.get(&(name.to_string(), level))
        }
    }
}

impl OpeningMission {
    /// Reads the opening map and the scenario tables around it.
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
        let set = tile_sets::read(&mut read, terrain)?;
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
            map_stats: set.properties,
            terrain,
            default_tile: set.default_tile,
            minutes_per_tick: set.minutes_per_tick,
            shared: Arc::new(StoryTables {
                areas: read("TheNewCenturions/Data Files/Data AD")?,
                tutorial_rectangles: read("TheNewCenturions/Data Files/Tutorial")
                    .ok()
                    .and_then(|bytes| span(&bytes, 1000, 16000).ok().map(<[u8]>::to_vec)),
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
                archive_table: None,
                map_notes: read(&format!("{SCENARIO}Data MD2"))?,
                note_labels: vec![],
                unknown_note_labels: vec![],
            }),
        })
    }

    /// The position of the cell (`x`, `y`) in a map's layers; a coordinate off the 90 by
    /// 90 map is refused.
    pub fn index(x: i32, y: i32) -> Result<usize, ContentError> {
        if !(0..90).contains(&x) || !(0..90).contains(&y) {
            return Err(ContentError::Invalid(format!(
                "map coordinate {x},{y} outside 90x90"
            )));
        }
        Ok(x as usize * MAP_SIDE + y as usize)
    }
    /// In the game a tile past the loaded file reads bytes an earlier land left
    /// in the 16000-byte property table; that leftover tail is not represented.
    pub fn tile_word(&self, tile: i16, field: usize) -> Result<i16, ContentError> {
        if !(0..400).contains(&tile) || field >= 40 || !field.is_multiple_of(2) {
            return Err(ContentError::Invalid(
                "tile property outside recovered table".into(),
            ));
        }
        if (tile as usize + 1) * 40 > self.map_stats.len() {
            return Err(ContentError::Invalid(
                "tile property retained from an earlier land's tile set".into(),
            ));
        }
        word(&self.map_stats, tile as usize * 40 + field)
    }

    /// How the tile blocks movement.
    pub fn movement_blocker(&self, tile: i16) -> Result<i16, ContentError> {
        self.tile_word(tile, 4)
    }

    /// The battle-map movement property. Movement cost strips the battle-map
    /// terrain marker, then indexes the table unsigned; this keeps its own
    /// bounds and errors.
    pub fn tactical_movement_property(&self, tile: i16) -> Result<i16, ContentError> {
        let normalized = if tile > 999 {
            tile.wrapping_sub(1000)
        } else {
            tile
        };
        word(&self.map_stats, usize::from(normalized as u16) * 40 + 2)
    }

    /// The sound the party's step makes on the tile.
    pub fn movement_sound(&self, tile: i16) -> Result<i16, ContentError> {
        self.tile_word(tile, 0)
    }
}

impl StoryTables {
    /// Zero-based Data MD records in Data MENU order; zero words are skipped.
    pub fn archive_order(&self) -> Option<Vec<usize>> {
        self.archive_table.as_ref().map(|table| {
            table
                .iter()
                .filter(|&&index| index != 0)
                .map(|&index| usize::from(index) - 1)
                .collect()
        })
    }
    /// The text of message `id`.
    pub fn dialogue(&self, id: i16) -> Result<String, ContentError> {
        let record = span(&self.dialogue, usize::from(id.unsigned_abs()) * 256, 256)?;
        Ok(encoding_rs::MACINTOSH
            .decode(span(record, 1, usize::from(record[0]))?)
            .0
            .into_owned())
    }
    /// The 42-byte story script `id`.
    pub fn script(&self, id: i16) -> Result<[u8; 42], ContentError> {
        if id < 0 {
            return Err(ContentError::Invalid("negative door script".into()));
        }
        Ok(span(&self.door_scripts, id as usize * 42, 42)?
            .try_into()
            .unwrap())
    }
    /// The description of item `id`: stock items below 800, the scenario's own from 800.
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
    /// The description of the scan at row `tier` and column `column` of `school`'s table.
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
