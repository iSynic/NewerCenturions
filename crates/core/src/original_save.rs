//! Reads an original saved game, and projects a session into the original
//! Data I1 layout, without writing either file.
use crate::{LevelState, MapState, RandomState, RuleError, Session};
use centurions_content::{Content, InventoryRecord, Warrior, span, ui};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

/// The parts of an original saved game that a session keeps: the information record, each level's
/// shop records by level number, and each level's map rows beyond the scenario's maps.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImportRecords {
    pub information: Vec<u8>,
    pub scenario_records: BTreeMap<u8, Vec<u8>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub unmapped_maps: BTreeMap<u8, Vec<u8>>,
}
impl ImportRecords {
    /// The scenario-duration word stored in the information record.
    pub(crate) fn scenario_duration_tail(&self) -> Result<i16, RuleError> {
        Information::read(&self.information)?.word(0x44263e)
    }

    /// Checks that the information record has its full length and that its location checksum
    /// matches. Shop records must come to at most 20 levels of at most 1000 whole 3002-byte rows,
    /// and map rows to at most 20 levels of at most 20 whole 77744-byte rows.
    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        if self.information.len() != INFO_BYTES {
            return Err("invalid native information length".into());
        }
        let value = |address| -> Result<i32, RuleError> {
            let mut cursor = 0;
            let mut result = None;
            for &(start, size) in FIELDS {
                if (address == 0 && start == 0)
                    || (start != 0 && address >= start && address + 4 <= start + size as u32)
                {
                    let offset = if start == 0 {
                        0
                    } else {
                        (address - start) as usize
                    };
                    result = Some(i32::from_be_bytes(
                        self.information[cursor + offset..cursor + offset + 4]
                            .try_into()
                            .unwrap(),
                    ));
                }
                cursor += size;
            }
            result.ok_or_else(|| RuleError::Invalid("native location field missing".into()))
        };
        let checksum = [0x442774, 0x44277c, 0x442778, 0x442768, 0x442764]
            .into_iter()
            .try_fold(0_i32, |sum, address| {
                Ok::<_, RuleError>(sum.wrapping_add(value(address)?))
            })?;
        if value(0)? != checksum {
            return Err("native save location checksum mismatch".into());
        }
        if self.scenario_records.len() > 20
            || self
                .scenario_records
                .values()
                .any(|records| records.len() > 1000 * 3002 || !records.len().is_multiple_of(3002))
        {
            return Err("invalid native encounter records".into());
        }
        if self.unmapped_maps.len() > 20
            || self
                .unmapped_maps
                .values()
                .any(|rows| rows.len() > 20 * 77744 || !rows.len().is_multiple_of(77744))
        {
            return Err("invalid unmapped native map records".into());
        }
        Ok(())
    }
}

/// Size of the information record in a saved game.
const INFO_BYTES: usize = 61730;
/// The fields of the information record as address and length, in the order a saved game stores
/// them. The entry with address 0 holds the location checksum.
const FIELDS: &[(u32, usize)] = &[
    (0x435502, 2),
    (0x442562, 20),
    (0x442576, 10),
    (0x442580, 438),
    (0x442736, 24),
    (0x442742, 29),
    (0x44275c, 56),
    (0x442790, 8),
    (0x448aa0, 18828),
    (0x4427c6, 18),
    (0x440edc, 5760),
    (0x43788a, 480),
    (0x42e2ec, 13200),
    (0x438acc, 40),
    (0, 4),
    (0x435500, 2),
    (0x40857a, 2),
    (0x408578, 2),
    (0x402f42, 1),
    (0x4354fe, 2),
    (0x44255e, 4),
    (0x42dbe4, 800),
    (0x4285f4, 22000),
];

/// The information record of a saved game: its field bytes by address, and the stored checksum.
struct Information {
    memory: BTreeMap<u32, u8>,
    checksum: i32,
}
impl Information {
    /// Reads the information record of a saved game and checks its length and checksum.
    fn read(bytes: &[u8]) -> Result<Self, RuleError> {
        if bytes.len() != INFO_BYTES {
            return Err("invalid native Data I1 length".into());
        }
        let mut memory = BTreeMap::new();
        let mut cursor = 0;
        let mut checksum = 0;
        for &(address, size) in FIELDS {
            let data = span(bytes, cursor, size)?;
            if address == 0 {
                checksum = i32::from_be_bytes(data.try_into().unwrap());
            } else {
                memory.extend(
                    data.iter()
                        .enumerate()
                        .map(|(i, byte)| (address + i as u32, *byte)),
                );
            }
            cursor += size;
        }
        let result = Self { memory, checksum };
        let actual = [0x442774, 0x44277c, 0x442778, 0x442768, 0x442764]
            .into_iter()
            .try_fold(0_i32, |sum, address| {
                Ok::<_, RuleError>(sum.wrapping_add(result.long(address)?))
            })?;
        if actual != result.checksum {
            return Err("native save location checksum mismatch".into());
        }
        Ok(result)
    }
    /// The bytes at an address range, which must lie within stored fields.
    fn bytes(&self, at: u32, length: usize) -> Result<Vec<u8>, RuleError> {
        (0..length)
            .map(|i| {
                self.memory
                    .get(&(at + i as u32))
                    .copied()
                    .ok_or_else(|| RuleError::Invalid("native save field missing".into()))
            })
            .collect()
    }
    /// The big-endian two-byte signed value at an address.
    fn word(&self, at: u32) -> Result<i16, RuleError> {
        Ok(i16::from_be_bytes(self.bytes(at, 2)?.try_into().unwrap()))
    }
    /// The big-endian four-byte signed value at an address.
    fn long(&self, at: u32) -> Result<i32, RuleError> {
        Ok(i32::from_be_bytes(self.bytes(at, 4)?.try_into().unwrap()))
    }
    /// The byte at an address.
    fn byte(&self, at: u32) -> Result<u8, RuleError> {
        Ok(self.bytes(at, 1)?[0])
    }
}

/// Reads a saved game file, refusing one longer than the limit.
fn read(path: &Path, maximum: usize) -> Result<Vec<u8>, RuleError> {
    if std::fs::metadata(path)?.len() > maximum as u64 {
        return Err("native save file exceeds limit".into());
    }
    Ok(centurions_content::installation::data_file(path)?)
}

/// Reads one level's saved map, schedule, menu, typed-encounter and shop tables from a save
/// directory, checking each against the scenario's own tables. Returns the level state, the shop
/// records and the map rows beyond the scenario's maps.
fn level(
    path: &Path,
    content: &Content,
    name: &str,
    level: u8,
) -> Result<(LevelState, Vec<u8>, Vec<u8>), RuleError> {
    let context = content.context(Some(name), level)?;
    let scenario = context
        .scenario
        .as_ref()
        .ok_or("native scenario unavailable")?;
    let table = read(&path.join("Data A1"), 20 * 77744 + 65536)?;
    if table.is_empty() || !table.len().is_multiple_of(77744) || table.len() / 77744 > 20 {
        return Err("invalid native saved map table".into());
    }
    let maps = table
        .as_chunks::<77744>()
        .0
        .iter()
        .take(scenario.maps.len())
        .enumerate()
        .map(|(i, row)| Ok((i as i32, MapState::from_bytes(row)?)))
        .collect::<Result<_, RuleError>>()?;
    let schedule = read(&path.join("Data TD3"), 151 * 40 + 65536)?;
    let menus = read(&path.join("Data F1"), 1000 * 426 + 65536)?;
    let typed_encounters = read(&path.join("Data G1"), 1000 * 516 + 65536)?;
    let shops = read(&path.join("Data E1"), 1000 * 3002 + 65536)?;
    if schedule.len() != scenario.maps[0].schedule.len()
        || menus.len() != scenario.maps[0].menus.len() / 426 * 426
        || typed_encounters.len() != scenario.maps[0].typed_encounters.len() / 516 * 516
        || !shops.len().is_multiple_of(3002)
        || shops.len() != scenario.maps[0].shops.len()
    {
        return Err("invalid native saved scenario tables".into());
    }
    Ok((
        LevelState {
            maps,
            schedule,
            menus,
            typed_encounters,
        },
        shops,
        table[table.len().min(scenario.maps.len() * 77744)..].to_vec(),
    ))
}

/// Imports an original saved game into a new session without changing the saved files.
pub fn import(
    path: &Path,
    content: &Content,
    random: &RandomState,
    library: (u32, u64),
) -> Result<Session, RuleError> {
    let path = if path.is_dir() {
        path.join("Data I1")
    } else {
        path.to_path_buf()
    };
    if path.file_name().is_none_or(|n| n != "Data I1") {
        return Err("select the original Data I1 save file".into());
    }
    let directory = path.parent().ok_or("native save directory missing")?;
    let bytes = read(&path, INFO_BYTES + 65536)?;
    let info = Information::read(&bytes)?;
    let name = ui::c_string(&info.bytes(0x44260c, 256)?);
    let name = name
        .strip_prefix(":Scenarios:")
        .and_then(|name| name.split(':').next())
        .filter(|name| !name.is_empty())
        .ok_or("native save scenario identity missing")?;
    let selected_level = info.byte(0x44255f)?;
    let context = content.context(Some(name), selected_level)?;
    let mut state = Session::new(&context);
    // Values the file omits keep their startup values, as after a load right
    // after launch.
    state.globals.boat = info.byte(0x442748)?;
    state.globals.boat_facing = info.byte(0x442740)? as i8;
    state.scenario_level = selected_level;
    let count = i16::from(info.byte(0x44273a)? as i8).wrapping_add(1);
    if !(1..=6).contains(&count) {
        return Err("invalid native party count".into());
    }
    state.warriors = (0..count)
        .map(|i| {
            let mut warrior = Warrior::read(&info.bytes(0x448aa0 + i as u32 * 3138, 3138)?)?;
            if let Some(original) = content
                .warriors
                .iter()
                .find(|original| original.name == warrior.name)
            {
                warrior.original_sha256 = original.original_sha256.clone();
            }
            Ok(warrior)
        })
        .collect::<Result<_, RuleError>>()?;
    // The load reads all six party records; those past the squad stay in the
    // warrior table.
    for body in count..6 {
        let bytes = info.bytes(0x448aa0 + body as u32 * 3138, 3138)?;
        state.globals.warrior_table.store(body as usize, bytes)?;
    }
    crate::apply(
        crate::Command::EnterMission { name: name.into() },
        &mut state,
        &context,
    )?;
    state.scenario_level = selected_level;
    state.difficulty = info.word(0x442576)?;
    state.random = random.clone();
    (state.library_random_seed, state.library_random_draws) = library;
    state.flags = info.bytes(0x44258c, 128)?;
    state.party_condition = info.bytes(0x442562, 20)?;
    let tail = info.word(0x44263e)?;
    state.scenario_duration_tail = (tail != 0).then_some(tail);
    state.holdover = info.bytes(0x440edc, 5760)?;
    state.held_over = usize::try_from(info.word(0x44257c)?)
        .map_err(|_| RuleError::Invalid("invalid native ally count".into()))?;
    state.globals.define_spells = info.bytes(0x43788a, 480)?;
    state.psi_disabled = info.byte(0x442749)?;
    state.energy_recovery_disabled = info.byte(0x44274a)?;
    state.enemy_spell_disabled = info.byte(0x44274b)?;
    // The load resets the selected warrior after reading the records.
    state.char_select_new = 0;
    state.party_currency = info.long(0x442788)?;
    state.mission_score = info.word(0x435502)?;
    state.mission_stage = info.word(0x435500)?;
    state.map_picture = info.word(0x40857a)?;
    state.globals.random_battles = info.bytes(0x402f42, 1)?[0];
    state.globals.coordinates_hidden = info.word(0x4354fe)?;
    state.notes = info.bytes(0x442722, 19)?;
    state.saved_position = [
        info.byte(0x442738)?,
        info.byte(0x44270c)?,
        info.byte(0x44270d)?,
    ];
    state.ammunition = info
        .bytes(0x42dbe4, 800)?
        .as_chunks::<4>()
        .0
        .iter()
        .map(|row| i32::from_be_bytes(*row))
        .collect();
    state.crate_items = info
        .bytes(0x4285f4, 22000)?
        .as_chunks::<22>()
        .0
        .iter()
        .map(|row| InventoryRecord(*row))
        .collect();
    let (mut selected, shops, unmapped) = level(directory, content, name, selected_level)?;
    let map = info.long(0x442774)?;
    let current = selected
        .maps
        .remove(&map)
        .ok_or("native active map missing")?;
    let position = state.exploration.as_mut().unwrap();
    position.restore_map(map, current);
    position.x = info.long(0x442764)?;
    position.y = info.long(0x442768)?;
    position.day = info.word(0x4427d4)?;
    position.hour = info.word(0x4427ca)?;
    position.minute = info.word(0x4427c8)?;
    position.fatigue = info.word(0x442578)?;
    position.last_move = [info.byte(0x442736)? as i8, info.byte(0x442737)? as i8];
    position.schedule = selected.schedule;
    state.menus = selected.menus;
    state.typed_encounters = selected.typed_encounters;
    state.visited_maps = selected.maps;
    state.imported = Some(ImportRecords {
        information: bytes,
        scenario_records: BTreeMap::from([(selected_level, shops)]),
        unmapped_maps: BTreeMap::from([(selected_level, unmapped)]),
    });
    if directory
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| n.parse::<u8>().ok())
        == Some(selected_level)
    {
        let game = directory.parent().ok_or("native game directory missing")?;
        for entry in std::fs::read_dir(game)? {
            let entry = entry?;
            let Some(level_number) = entry
                .file_name()
                .to_str()
                .and_then(|n| n.parse::<u8>().ok())
            else {
                continue;
            };
            if level_number != selected_level && entry.path().join("Data A1").is_file() {
                let (records, shops, unmapped) = level(&entry.path(), content, name, level_number)?;
                state.visited_levels.insert(level_number, records);
                state
                    .imported
                    .as_mut()
                    .unwrap()
                    .scenario_records
                    .insert(level_number, shops);
                state
                    .imported
                    .as_mut()
                    .unwrap()
                    .unmapped_maps
                    .insert(level_number, unmapped);
            }
        }
    }
    state.globals.encounter_headers = crate::story::EncounterHeaders::startup();
    for level in 0..3 {
        let (menus, typed) = if level == selected_level {
            (&state.menus, &state.typed_encounters)
        } else if let Some(records) = state.visited_levels.get(&level) {
            (&records.menus, &records.typed_encounters)
        } else {
            continue;
        };
        state
            .globals
            .encounter_headers
            .copy_file(menus, false, false)?;
        state
            .globals
            .encounter_headers
            .copy_file(typed, true, false)?;
    }
    // The load copies each Data E file through the shared shop record.
    for level in 0..3 {
        if let Some(records) = state
            .imported
            .as_ref()
            .unwrap()
            .scenario_records
            .get(&level)
        {
            state.globals.the_shop.copy_file(records);
        }
    }
    state.suspend_allies = info.word(0x408578)?;
    let held_items = info
        .bytes(0x42e2ec, 13200)?
        .as_chunks::<22>()
        .0
        .iter()
        .map(|row| InventoryRecord(*row))
        .collect::<Vec<_>>();
    let wealth_store = info.long(0x44278c)?;
    state.globals.storage.wealth_store = wealth_store;
    if wealth_store != 0 || held_items.iter().any(|item| item.id() != 0) {
        // The original load does not restore whether confiscation is active.
        state.confiscated = Some(crate::confiscation::ConfiscatedEquipment {
            active: false,
            money: wealth_store,
            items: held_items,
        });
    }
    state.shops = state.imported.as_ref().unwrap().scenario_records.clone();
    if info.byte(0x442753)? != 0 {
        let id = info.byte(0x44273b)?;
        let records = &state.imported.as_ref().unwrap().scenario_records[&selected_level];
        state.globals.the_shop.load(centurions_content::span(
            records,
            usize::from(id) * centurions_content::records::SHOP_BYTES,
            centurions_content::records::SHOP_BYTES,
        )?)?;
        state.shop_access = Some(crate::shop::Access {
            level: selected_level,
            id: info.byte(0x44273b)?,
            enhancement: 0,
            remaining: 0,
        });
    }
    state.validate(content)?;
    Ok(state)
}
