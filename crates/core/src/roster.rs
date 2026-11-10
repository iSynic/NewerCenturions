use crate::{Phase, RuleError, Session, atomic_write};
pub use centurions_content::IndexSlot;
use centurions_content::records::{
    CharacterIdentityMut, InventorySlots, ItemDefinition, LearnedScansMut,
};
use centurions_content::{Content, Warrior};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The most rows the warrior index holds.
const INDEX_SLOTS: usize = 500;

/// The warriors the player keeps between missions, stored apart from the supplied ones: the
/// warrior files, and the warrior index that lists them in the roster list's order.
#[derive(Clone, Serialize, Deserialize)]
pub struct Roster {
    schema_version: u32,
    content_identity: String,
    /// The kept warrior files.
    pub warriors: Vec<Warrior>,
    /// The warrior index, at most 500 rows.
    #[serde(default)]
    pub index: Vec<IndexSlot>,
    #[serde(skip)]
    path: Option<PathBuf>,
}
impl Roster {
    /// A roster of the warriors supplied with the game, listed by the game's own warrior index
    /// (`index`, read from the installation), not yet saved anywhere.
    pub fn supplied(content: &Content, index: &[IndexSlot]) -> Self {
        Self {
            schema_version: 2,
            content_identity: content.identity.clone(),
            warriors: content.warriors.clone(),
            index: index.iter().take(INDEX_SLOTS).cloned().collect(),
            path: None,
        }
    }
    /// Opens the roster file, or the supplied warriors listed by `supplied` when none exists yet;
    /// a roster over the size limit or written for other content is refused. A roster kept before
    /// the warrior index lists its warriors in their kept order, and takes the new form only when
    /// it is next saved.
    pub fn open(path: &Path, content: &Content, supplied: &[IndexSlot]) -> Result<Self, RuleError> {
        let mut roster = match std::fs::metadata(path) {
            Ok(metadata) => {
                if metadata.len() > 32 * 1024 * 1024 {
                    return Err(RuleError::Invalid(
                        "warrior roster exceeds size limit".into(),
                    ));
                }
                serde_json::from_slice::<Self>(&std::fs::read(path)?)?
            }
            // Linux reports a file parent as NotADirectory where Windows reports
            // NotFound; either way there is no roster yet.
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
                ) =>
            {
                Self::supplied(content, supplied)
            }
            Err(error) => return Err(error.into()),
        };
        if roster.schema_version == 1 && roster.index.is_empty() {
            roster.schema_version = 2;
            roster.list_every_warrior();
        }
        if roster.schema_version != 2 || roster.content_identity != content.identity {
            return Err(RuleError::Invalid("incompatible warrior roster".into()));
        }
        roster.validate()?;
        roster.path = Some(path.to_path_buf());
        Ok(roster)
    }
    /// Lists every kept warrior in the index, in kept order, as far as the index reaches.
    fn list_every_warrior(&mut self) {
        self.index = self
            .warriors
            .iter()
            .take(INDEX_SLOTS)
            .map(|warrior| IndexSlot {
                name: warrior.name.clone(),
                level: warrior.record().level().unwrap_or(0),
            })
            .collect();
    }
    /// Checks that the roster holds at most 1024 warriors, each valid and none repeated, and an
    /// index of at most 500 rows.
    fn validate(&self) -> Result<(), RuleError> {
        if self.warriors.len() > 1024 || self.index.len() > INDEX_SLOTS {
            return Err(RuleError::Invalid(
                "warrior roster capacity exceeded".into(),
            ));
        }
        for (i, warrior) in self.warriors.iter().enumerate() {
            validate_warrior(warrior)?;
            if self.warriors[..i]
                .iter()
                .any(|w| w.original_sha256 == warrior.original_sha256)
            {
                return Err(RuleError::Invalid(
                    "duplicate warrior identity in roster".into(),
                ));
            }
        }
        Ok(())
    }
    /// The kept warrior of this name; names compare without regard to case, as warrior files do
    /// on the original's disks.
    fn file(&self, name: &str) -> Option<&Warrior> {
        let wanted = name.to_lowercase();
        self.warriors
            .iter()
            .find(|warrior| warrior.name.to_lowercase() == wanted)
    }
    /// Whether a warrior of this name is already kept.
    ///
    /// Original: Name
    pub fn holds_name(&self, name: &str) -> bool {
        self.file(name).is_some()
    }
    /// The kept warrior the index lists in `row`; none for a blank row or a name no warrior has.
    pub fn listed(&self, row: usize) -> Option<&Warrior> {
        self.index
            .get(row)
            .filter(|slot| !slot.name.is_empty())
            .and_then(|slot| self.file(&slot.name))
    }
    /// Takes a name out of the index: its first row is blanked and keeps its level word. With
    /// `erase` the warrior of that name is erased too.
    ///
    /// Original: minus
    pub fn minus(&mut self, name: &str, erase: bool) -> Result<(), RuleError> {
        let mut next = self.clone();
        next.blank_row(name, erase);
        self.commit(next)
    }
    /// Lists warriors again: each in turn takes the first blank row of the index with its level.
    /// A full index leaves a warrior unlisted.
    ///
    /// Original: plus
    pub fn plus(&mut self, warriors: &[Warrior]) -> Result<(), RuleError> {
        let mut next = self.clone();
        for warrior in warriors {
            next.fill_row(&warrior.name, warrior.record().level()?);
        }
        self.commit(next)
    }
    /// Saves warriors to the roster: each in turn is taken out of the index and listed again in
    /// the first blank row with its level, and its warrior file is written, replacing a kept
    /// warrior of the same name.
    ///
    /// Original: savecharacter
    pub fn save_characters(&mut self, warriors: &[Warrior]) -> Result<(), RuleError> {
        let mut next = self.clone();
        for warrior in warriors {
            validate_warrior(warrior)?;
            next.blank_row(&warrior.name, false);
            next.fill_row(&warrior.name, warrior.record().level()?);
            let wanted = warrior.name.to_lowercase();
            match next.warriors.iter().position(|kept| {
                kept.original_sha256 == warrior.original_sha256
                    || kept.name.to_lowercase() == wanted
            }) {
                Some(index) => next.warriors[index] = warrior.clone(),
                None => next.warriors.push(warrior.clone()),
            }
        }
        self.commit(next)
    }
    /// Lists a new warrior's name once its level is chosen, before any warrior is saved under it:
    /// the name is taken out of the index and written to the first blank row with `level_word`.
    ///
    /// Original: Startlevel
    pub fn start_level(&mut self, name: &str, level_word: i16) -> Result<(), RuleError> {
        let mut next = self.clone();
        next.blank_row(name, false);
        next.fill_row(name, level_word);
        self.commit(next)
    }
    /// Blanks the first index row of `name`; with `erase` the warrior of that name goes too. Rows
    /// compare names exactly.
    ///
    /// Original: minus
    fn blank_row(&mut self, name: &str, erase: bool) {
        if erase {
            let wanted = name.to_lowercase();
            self.warriors
                .retain(|warrior| warrior.name.to_lowercase() != wanted);
        }
        if let Some(slot) = self.index.iter_mut().find(|slot| slot.name == name) {
            slot.name.clear();
        }
    }
    /// Writes `name` and `level` to the first blank row, or to a new row while the index has room.
    ///
    /// Original: plus
    fn fill_row(&mut self, name: &str, level: i16) {
        let slot = IndexSlot {
            name: name.to_owned(),
            level,
        };
        if let Some(blank) = self.index.iter_mut().find(|slot| slot.name.is_empty()) {
            *blank = slot;
        } else if self.index.len() < INDEX_SLOTS {
            self.index.push(slot);
        }
    }
    /// Checks the changed roster, writes it to its file and keeps it; nothing changes if either
    /// fails.
    fn commit(&mut self, next: Self) -> Result<(), RuleError> {
        next.validate()?;
        if let Some(path) = &next.path {
            atomic_write(path, &serde_json::to_vec_pretty(&next)?)?;
        }
        *self = next;
        Ok(())
    }
}
/// A newly created warrior as saved to the roster before it joins a squad: it is checked with
/// the new warrior's checksums and keeps its own scan shortcuts.
///
/// Original: savecharacter
pub fn new_warrior_record(warrior: &Warrior) -> Result<Warrior, RuleError> {
    let mut record = warrior.clone();
    CharacterIdentityMut(&mut record.bytes).refresh_new_warrior_checksums()?;
    Ok(record)
}
/// The warriors as saved to the roster: temporary mission items are dropped and
/// the saved copy is checked.
///
/// Original: savecharacter
pub fn mission_records(state: &Session, content: &Content) -> Result<Vec<Warrior>, RuleError> {
    state
        .warriors
        .iter()
        .enumerate()
        .map(|(member, warrior)| saved_record(state, content, warrior, member))
        .collect()
}

/// Saving the squad looks at each warrior's pack from the last item to the first, so the first
/// item of the last warrior saved is left as the item being worked on. A mission item is dropped
/// from the saved copy, which loads it again with its type made positive; a worn one that cannot
/// be taken off (a shield battery while a power shield is worn) gives its warning and stays.
/// Returns the warnings in the order they are given.
///
/// Original: savecharacter
pub(crate) fn load_saved_items(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<crate::Event>, RuleError> {
    let mut events = vec![];
    for member in 0..state.warriors.len() {
        let mut saved = state.warriors[member].clone();
        let count = InventorySlots(&saved.bytes).count()?;
        for slot in (0..count.max(0) as usize).rev() {
            let id = InventorySlots(&saved.bytes).item_id(slot)?;
            state.load_item_keeping_type(content, id);
            let kind = match &state.globals.loaded_items.item {
                Some(item) => ItemDefinition(item).item_type()?,
                None => 0,
            };
            if kind < 0 || matches!(kind, 23..=25) {
                state.load_item(content, id)?;
                let mission = state
                    .exploration
                    .as_ref()
                    .and_then(|p| content.map(p.map))
                    .ok_or("mission roster profiles missing")?;
                let (_, globals) = crate::equipment::drop_loaded(
                    &mut saved,
                    slot,
                    id,
                    content,
                    mission,
                    &mut events,
                )?;
                // The roster copy has no Session to receive party-wide writes.
                crate::equipment::require_no_globals(&globals)?;
            }
        }
    }
    Ok(events)
}

/// The record saved for the squad member in slot `member`: its temporary items are dropped,
/// its checksums stamped and its scan shortcuts taken back from the slot's Psi memory.
///
/// Original: savecharacter
pub fn saved_record(
    state: &Session,
    content: &Content,
    warrior: &Warrior,
    member: usize,
) -> Result<Warrior, RuleError> {
    let mut record = warrior.clone();
    for slot in (0..record.inventory()?.len()).rev() {
        let id = record.inventory()?[slot].id();
        if temporary(content, id)? {
            let mission = state
                .exploration
                .as_ref()
                .and_then(|p| content.map(p.map))
                .ok_or("mission roster profiles missing")?;
            // The roster copy has no Session to receive party-wide writes.
            let globals = crate::equipment::drop(&mut record, slot, content, mission)?;
            crate::equipment::require_no_globals(&globals)?;
        }
    }
    CharacterIdentityMut(&mut record.bytes).refresh_save_checksums()?;
    let psi = state
        .globals
        .define_spells
        .get(member * 80..(member + 1) * 80)
        .ok_or("squad member outside the Psi memory")?;
    LearnedScansMut(&mut record.bytes).restore_selection_memory(psi);
    Ok(record)
}

/// Whether the warrior carries an item that is lost when it is saved to the roster.
///
/// Original: partyselect
pub fn carries_temporary_items(content: &Content, warrior: &Warrior) -> Result<bool, RuleError> {
    for item in warrior.inventory()? {
        if temporary(content, item.id())? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// A mission item, of a negative type or of types 23 to 25, does not leave the mission.
fn temporary(content: &Content, id: i16) -> Result<bool, RuleError> {
    let definition = content
        .items
        .get(usize::from(id.unsigned_abs()))
        .ok_or("roster item definition missing")?;
    let kind = ItemDefinition(definition).item_type()?;
    Ok(kind < 0 || matches!(kind, 23..=25))
}

/// Checks a warrior's record: its name matches the record, its original checksum is well formed and
/// it is of the current format, not an old-format character.
///
/// Original: bandaid
fn validate_warrior(warrior: &Warrior) -> Result<(), RuleError> {
    if Warrior::read(&warrior.bytes)?.name != warrior.name
        || warrior.original_sha256.len() != 64
        || !warrior
            .original_sha256
            .bytes()
            .all(|b| b.is_ascii_hexdigit())
    {
        return Err(RuleError::Invalid("invalid roster warrior".into()));
    }
    warrior.require_current_format()?;
    warrior.inventory()?;
    Ok(())
}
/// Checks that a squad has one to six distinct warriors.
pub(crate) fn validate_squad(warriors: &[Warrior]) -> Result<(), RuleError> {
    if !(1..=6).contains(&warriors.len()) {
        return Err(RuleError::Invalid(
            "squad requires one to six warriors".into(),
        ));
    }
    for (i, warrior) in warriors.iter().enumerate() {
        validate_warrior(warrior)?;
        if warriors[..i]
            .iter()
            .any(|w| w.original_sha256 == warrior.original_sha256)
        {
            return Err(RuleError::Invalid("duplicate squad warrior".into()));
        }
    }
    Ok(())
}
/// Refuses unless warriors may be managed now: exploring with nothing on screen and no script
/// running, and no medical choice pending.
pub(crate) fn admit_management(state: &Session) -> Result<(), RuleError> {
    if state.phase != Phase::Exploration
        || state.medical_continuation.is_some()
        || state
            .exploration
            .as_ref()
            .is_none_or(|p| p.presentation.is_some() || p.script.is_some())
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    Ok(())
}

/// Adds a warrior to the squad under the level's restrictions: warnings 0x86
/// for a level above the limit, 0x87 for a full squad, 0x88 for a banned race
/// and 0x89 for a banned M.O.S., checked in that order. `level` is the level the roster lists
/// for the warrior and `squad` the number of warriors already chosen.
///
/// Original: partyselect
pub fn admit_to_squad(
    restrictions: &[u8],
    warrior: &Warrior,
    level: i16,
    squad: usize,
) -> Result<(), RuleError> {
    let limits = centurions_content::records::PartyRestrictions(restrictions);
    let record = warrior.record();
    let refuse = |index| {
        Err(RuleError::Refused {
            feedback: vec![crate::feedback::GameFeedback::Warning { index }],
        })
    };
    let maximum = limits.maximum_level()?;
    if maximum != 0 && level > maximum {
        return refuse(0x86);
    }
    let warriors = limits.maximum_warriors()?;
    if warriors != 0 && squad as i32 >= i32::from(warriors) {
        return refuse(0x87);
    }
    let banned = |flag: Option<bool>| {
        flag.ok_or_else(|| RuleError::Invalid("restriction index outside Data RI".into()))
    };
    if banned(limits.race_banned(record.race()?))? {
        return refuse(0x88);
    }
    if banned(limits.mos_banned(record.mos()?))? {
        return refuse(0x89);
    }
    Ok(())
}
