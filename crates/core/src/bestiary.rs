//! The bestiary menu and the monster record it leaves behind.
//!
//! The game reads the scenario's Data MENU table, then reads each listed
//! monster from Data MD in table order. Its loop runs well past the end of the
//! 251-entry table, over the battle grid, its own count and the values that
//! follow, so those values are read as monster numbers too: an original quirk
//! kept on purpose (see docs/bestiary-work.md). Without a cached table it scans
//! and sorts the bestiary, writes the cache and starts again. The current
//! monster record is whatever the last successful read left.
use crate::Session;
use centurions_content::{Content, word};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Number of monster numbers in the scenario's menu table.
const TABLE_WORDS: usize = 251;
/// Number of table entries the menu loop reads, which runs past the table's end (see the module
/// notes).
const LOOP_WORDS: usize = 999;
/// Size of one monster record in the scenario's Data MD file.
const RECORD_BYTES: usize = 288;
/// The menu count, read as table entry 937.
const COUNT_WORD: usize = 937;
/// Monster slot 0, read from table entry 971 on.
const NPC_WORD: usize = 971;

/// The scenario the bestiary was last built for.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BestiarySelector {
    /// Never built; scenario selectors start at five.
    Cold,
    Scenario(String),
}

/// Bestiary menu state the original save does not record. The reconstruction
/// keeps it with its own saves, as if the game had kept running.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BestiaryMenu {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<BestiarySelector>,
    /// The part of the table Data MENU holds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub table: Option<Vec<u16>>,
    /// No battle has been set up yet, so the battle grid still holds its
    /// startup values.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub battle_record_cold: bool,
    /// The Data MENU files the game writes beside the scenario, by level
    /// directory. The installation is never written; `None` is unknown content.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub caches: BTreeMap<String, Option<Vec<u16>>>,
}

impl BestiaryMenu {
    /// The bestiary menu as a new game starts it: cold selector, an empty table and no cached
    /// scenario tables.
    pub(crate) fn startup() -> Self {
        Self {
            selector: Some(BestiarySelector::Cold),
            table: Some(vec![0; TABLE_WORDS]),
            battle_record_cold: true,
            caches: BTreeMap::new(),
        }
    }

    /// Whether the menu is still in its unset state, so it can be left out of a save.
    pub(crate) fn is_unknown(&self) -> bool {
        *self == Self::default()
    }

    /// Checks that the saved table and every cached table have the full menu length.
    pub(crate) fn validate(&self) -> Result<(), crate::RuleError> {
        if std::iter::once(&self.table)
            .chain(self.caches.values())
            .flatten()
            .any(|table| table.len() != TABLE_WORDS)
        {
            return Err("invalid retained bestiary table".into());
        }
        Ok(())
    }
}

/// The scenario data the bestiary and the earlier game-file reset read.
struct Inputs<'a> {
    selector: &'a str,
    directory: String,
    shipped: Option<&'a [u16]>,
    records: &'a [u8],
    /// The last Data TD3 record resetgamefiles copied.
    schedule: &'a [u8],
}

impl Session {
    /// Builds the bestiary menu for a new game, after the game files are reset.
    ///
    /// Original: setupnewgame, updatemonstermenu
    pub(crate) fn update_monster_menu(&mut self, content: &Content) {
        let template = content.scenario.as_ref().and_then(|scenario| {
            let map = self.land(content)?;
            // resetgamefiles copies variants 0..2; the last one stays behind.
            let schedule = (0..3)
                .filter_map(|level| scenario.catalog.get(&(scenario.name.clone(), level)))
                .filter_map(|level| level.maps.first())
                .map(|map| map.schedule.as_slice())
                .next_back()
                .unwrap_or(&map.schedule);
            let inputs = Inputs {
                selector: &scenario.name,
                directory: format!("{}/{}", scenario.name, scenario.level),
                shipped: map.archive_table.as_deref(),
                records: &map.combat.as_ref()?.monsters,
                schedule: schedule.get(schedule.len().checked_sub(40)?..)?,
            };
            let slot = self.globals.monsters.slots.first()?.as_ref();
            let monster = slot.map(|slot| slot.record.bytes.as_slice());
            let previous = self.globals.monster_pick.known();
            Some(update(
                &mut self.globals.bestiary,
                &inputs,
                monster,
                previous,
            ))
        });
        if template.is_none() {
            self.globals.bestiary = BestiaryMenu::default();
        }
        match template.flatten() {
            Some(record) => self.globals.monster_pick.set(&record),
            None => self.globals.monster_pick.invalidate(),
        }
    }
}

/// Brings the monster menu up to date for the scenario being played, reusing the table already read
/// for its directory and keeping the previous monster record when the scenario has not changed.
///
/// Original: updatemonstermenu
fn update(
    menu: &mut BestiaryMenu,
    inputs: &Inputs<'_>,
    monster: Option<&[u8]>,
    previous: Option<Vec<u8>>,
) -> Option<Vec<u8>> {
    let selector = menu
        .selector
        .replace(BestiarySelector::Scenario(inputs.selector.into()));
    let cache = match menu.caches.get(&inputs.directory) {
        Some(written) => Some(written.clone()),
        None => inputs.shipped.map(|table| Some(table.to_vec())),
    };
    // The cache is read before the selector comparison, even on the early return.
    if let Some(contents) = &cache {
        menu.table.clone_from(contents);
    }
    match selector {
        Some(BestiarySelector::Scenario(name)) if name == inputs.selector => return previous,
        Some(_) => {}
        None => {
            menu.table = None;
            if cache.is_none() {
                menu.caches.insert(inputs.directory.clone(), None);
            }
            return None;
        }
    }
    let mut buffer = previous;
    let mut type_words = [0; 2];
    if cache.is_none() {
        let scanned = scan(menu.table.as_deref(), inputs.records, &mut buffer);
        menu.table.clone_from(&scanned);
        menu.caches.insert(inputs.directory.clone(), scanned);
        // Writing the cache gives the file the type "scen", which the next pass
        // then reads as monster numbers.
        type_words = [0x7363, 0x656e];
    }
    let table = menu.table.clone()?;
    let tail = tail(
        menu.battle_record_cold,
        type_words,
        inputs.schedule,
        monster?,
    )?;
    read_in_order(&table, &tail, inputs.records, buffer)
}

/// The uncached pass: sequential reads, filters, then the menu-text bubble sort.
fn scan(table: Option<&[u16]>, records: &[u8], buffer: &mut Option<Vec<u8>>) -> Option<Vec<u16>> {
    let mut table = table?.to_vec();
    if !records.len().is_multiple_of(RECORD_BYTES) {
        return None;
    }
    let mut entries = Vec::new();
    for (index, record) in records.as_chunks::<RECORD_BYTES>().0.iter().enumerate() {
        *buffer = Some(record.to_vec());
        // count > 999 stops the scan; table writes past 250 reach the battle record.
        if entries.len() > 999 {
            break;
        }
        let kind = word(record, 0).ok()?;
        if kind != 0 && record[0xa7] == 0 {
            if kind == 0xff {
                break;
            }
            if entries.len() == TABLE_WORDS - 1 {
                return None;
            }
            entries.push((index as u16 + 1, menu_text(record)?));
        }
    }
    let count = entries.len();
    for _ in 1..count {
        for item in 1..count {
            if entries[item - 1].1 > entries[item].1 {
                entries.swap(item - 1, item);
            }
        }
    }
    for (slot, (index, _)) in table[1..].iter_mut().zip(entries) {
        *slot = index;
    }
    Some(table)
}

/// The C-copied name as AppendMenu or SetMenuItemText would store it. Names
/// that the copy, the metacharacters or the empty-item skip would change, and
/// high characters whose strcmp order is not recovered, leave the order unknown.
fn menu_text(record: &[u8]) -> Option<Vec<u8>> {
    let length = usize::from(record[0xda]);
    let text = record.get(0xdb..0xdb + length)?;
    let terminated = record[0xdb + length..].contains(&0);
    let plain = |byte: &u8| (0x20..0x80).contains(byte) && !b";^!</(".contains(byte);
    (length != 0 && terminated && text.iter().all(plain)).then(|| text.to_vec())
}

/// A word read past the end of the menu table: a stored value, or the menu's own count.
#[derive(Clone, Copy)]
enum TailWord {
    Value(u16),
    Count,
}

/// Table entries 251..998 as a new game leaves them. The Standard File
/// information there reads as no file opened, leaving the "NEWC" creator
/// (docs/bestiary-work.md).
fn tail(
    cold: bool,
    type_words: [u16; 2],
    schedule: &[u8],
    monster: &[u8],
) -> Option<Vec<TailWord>> {
    if !cold {
        return None;
    }
    let mut tail = vec![TailWord::Value(0); LOOP_WORDS - TABLE_WORDS];
    let mut put = |index: usize, value: u16| tail[index - TABLE_WORDS] = TailWord::Value(value);
    // A registered build leaves both startup flags clear.
    put(939, type_words[0]);
    put(940, type_words[1]);
    put(941, 0x4e45);
    put(942, 0x5743);
    // The startup screen rectangle.
    put(949, 0xe0);
    put(950, 0xe0);
    for (index, bytes) in schedule.as_chunks::<2>().0.iter().enumerate() {
        put(951 + index, u16::from_be_bytes(*bytes));
    }
    for (index, bytes) in monster
        .get(..2 * (LOOP_WORDS - NPC_WORD))?
        .as_chunks::<2>()
        .0
        .iter()
        .enumerate()
    {
        put(NPC_WORD + index, u16::from_be_bytes(*bytes));
    }
    tail[COUNT_WORD - TABLE_WORDS] = TailWord::Count;
    Some(tail)
}

/// The cached pass. Each nonzero entry is counted and then reread for the seek,
/// so the count entry seeks with its incremented value. A seek past the file
/// reads nothing and keeps the previous record.
fn read_in_order(
    table: &[u16],
    tail: &[TailWord],
    records: &[u8],
    mut buffer: Option<Vec<u8>>,
) -> Option<Vec<u8>> {
    let mut count = 0u16;
    let words = table
        .iter()
        .map(|&value| TailWord::Value(value))
        .chain(tail.iter().copied());
    for entry in words {
        let tested = match entry {
            TailWord::Value(value) => value,
            TailWord::Count => count,
        };
        if tested == 0 {
            continue;
        }
        count = count.wrapping_add(1);
        let value = match entry {
            TailWord::Value(value) => value,
            TailWord::Count => count,
        };
        // The record number is an unsigned 16-bit value.
        let start = usize::from(value.wrapping_sub(1)) * RECORD_BYTES;
        if let Some(record) = records.get(start..start + RECORD_BYTES) {
            buffer = Some(record.to_vec());
        } else if start < records.len() {
            return None;
        }
        let current = buffer.as_deref()?;
        if word(current, 0).ok()? == 0xff && current[0xa7] == 0 {
            break;
        }
    }
    buffer
}
