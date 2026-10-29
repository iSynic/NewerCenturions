//! updatemonstermenu 0x135bb2: the bestiary table and the MONSTER work it leaves.
//!
//! The routine reads Data MENU (502 bytes) into the WORD table at 4395a6, then
//! reads Data MD records through MONSTER work 440dbc in table order. Its loop
//! runs to WORD 998, past the 251-WORD table, over the battle record at
//! 43979c, its own count WORD 439cf8 and the startup globals up to NPC slot 0.
//! Without a cache it scans and sorts the bestiary, writes the cache and
//! restarts. MONSTER work is whatever record the last successful read left.
use crate::Session;
use centurions_content::{Content, word};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const TABLE_WORDS: usize = 251;
const LOOP_WORDS: usize = 999;
const RECORD_BYTES: usize = 288;
/// WORD 439cf8, the menu count, is table WORD 937.
const COUNT_WORD: usize = 937;
/// NPC slot 0 at 439d3c is table WORD 971.
const NPC_WORD: usize = 971;

/// WORD 4042c8: the scenario selector the bestiary was last built for.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BestiarySelector {
    /// The cold DATA zero; scenario selectors start at five.
    Cold,
    Scenario(String),
}

/// Process state of updatemonstermenu that native saves do not record. The
/// reconstruction keeps it with its own saves, as if one process continued.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BestiaryWork {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<BestiarySelector>,
    /// WORDs 4395a6..43979c, the part of the table Data MENU holds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub table: Option<Vec<u16>>,
    /// No combatsetup has replaced the cold battle record at 43979c yet.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub battle_record_cold: bool,
    /// Data MENU files the original writes beside the scenario, by level
    /// directory. The installation is never written; `None` is unknown content.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub caches: BTreeMap<String, Option<Vec<u16>>>,
}

impl BestiaryWork {
    pub(crate) fn startup() -> Self {
        Self {
            selector: Some(BestiarySelector::Cold),
            table: Some(vec![0; TABLE_WORDS]),
            battle_record_cold: true,
            caches: BTreeMap::new(),
        }
    }

    pub(crate) fn is_unknown(&self) -> bool {
        *self == Self::default()
    }

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

/// The scenario data updatemonstermenu and the earlier resetgamefiles read.
struct Inputs<'a> {
    selector: &'a str,
    directory: String,
    shipped: Option<&'a [u16]>,
    records: &'a [u8],
    /// The last Data TD3 record resetgamefiles copied through 439d14.
    schedule: &'a [u8],
}

impl Session {
    /// setupnewgame 1538ba calls updatemonstermenu after resetgamefiles.
    pub(crate) fn update_monster_menu(&mut self, content: &Content) {
        let template = content.scenario.as_ref().and_then(|scenario| {
            let map = self.land(content)?;
            // resetgamefiles copies variants 0..2; the last copied record remains.
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
            let slot = self.native_work.npc_backing.slots.first()?.as_ref();
            let npc = slot.map(|slot| slot.record.bytes.as_slice());
            let previous = self.native_work.monster_work.known();
            Some(update(
                &mut self.native_work.bestiary,
                &inputs,
                npc,
                previous,
            ))
        });
        if template.is_none() {
            self.native_work.bestiary = BestiaryWork::default();
        }
        match template.flatten() {
            Some(record) => self.native_work.monster_work.set(&record),
            None => self.native_work.monster_work.invalidate(),
        }
    }
}

fn update(
    work: &mut BestiaryWork,
    inputs: &Inputs<'_>,
    npc: Option<&[u8]>,
    previous: Option<Vec<u8>>,
) -> Option<Vec<u8>> {
    let selector = work
        .selector
        .replace(BestiarySelector::Scenario(inputs.selector.into()));
    let cache = match work.caches.get(&inputs.directory) {
        Some(written) => Some(written.clone()),
        None => inputs.shipped.map(|table| Some(table.to_vec())),
    };
    // The cache is read before the selector comparison, even on the early return.
    if let Some(contents) = &cache {
        work.table.clone_from(contents);
    }
    match selector {
        Some(BestiarySelector::Scenario(name)) if name == inputs.selector => return previous,
        Some(_) => {}
        None => {
            work.table = None;
            if cache.is_none() {
                work.caches.insert(inputs.directory.clone(), None);
            }
            return None;
        }
    }
    let mut buffer = previous;
    let mut type_words = [0; 2];
    if cache.is_none() {
        let scanned = scan(work.table.as_deref(), inputs.records, &mut buffer);
        work.table.clone_from(&scanned);
        work.caches.insert(inputs.directory.clone(), scanned);
        // setfileinfo copies the C type "scen" to 439cfc before the restart.
        type_words = [0x7363, 0x656e];
    }
    let table = work.table.clone()?;
    let tail = tail(work.battle_record_cold, type_words, inputs.schedule, npc?)?;
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

#[derive(Clone, Copy)]
enum TailWord {
    Value(u16),
    Count,
}

/// WORDs 251..998 as setupnewgame leaves them. Standard File metadata at
/// 439cfc is not reconstruction state; it reads as no file opened there,
/// leaving ToolBoxInit's "NEWC" at 439d00 (docs/bestiary-work.md).
fn tail(cold: bool, type_words: [u16; 2], schedule: &[u8], npc: &[u8]) -> Option<Vec<TailWord>> {
    if !cold {
        return None;
    }
    let mut tail = vec![TailWord::Value(0); LOOP_WORDS - TABLE_WORDS];
    let mut put = |index: usize, value: u16| tail[index - TABLE_WORDS] = TailWord::Value(value);
    // A registered build leaves both startup flag BYTEs at 439cfa clear.
    put(939, type_words[0]);
    put(940, type_words[1]);
    put(941, 0x4e45);
    put(942, 0x5743);
    // ToolBoxInit's rectangle at 439d0c.
    put(949, 0xe0);
    put(950, 0xe0);
    for (index, bytes) in schedule.as_chunks::<2>().0.iter().enumerate() {
        put(951 + index, u16::from_be_bytes(*bytes));
    }
    for (index, bytes) in npc
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

/// The cached pass. Each nonzero WORD counts, then 135ca6 rereads it for the
/// seek, so the count WORD itself seeks with its incremented value. A seek
/// past the file transfers nothing and keeps the previous record.
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
        // SUBQ.W then MULU.W: an unsigned WORD record number.
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
