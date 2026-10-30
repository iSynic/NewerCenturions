use crate::{RuleError, story, story_allies};
use centurions_content::Warrior;

/// Party ammunition stock entries.
pub(crate) const AMMUNITION_SLOTS: usize = 200;
/// Party buff bytes.
pub(crate) const PARTY_CONDITION_BYTES: usize = 20;
/// Mission note bytes.
pub(crate) const NOTE_BYTES: usize = 19;
/// Story flag bytes; flag byte 0 starts as 0xff.
pub(crate) const FLAG_BYTES: usize = 128;
/// One warrior's Psi selection memory row, copied from the warrior's scan shortcuts.
pub(crate) const PSI_MEMORY_ROW_BYTES: usize = 80;
/// Psi selection memory: six warrior rows.
pub(crate) const DEFINE_SPELLS_BYTES: usize = 6 * PSI_MEMORY_ROW_BYTES;
/// Inventory records in each of the crate and booty stores.
pub(crate) const STORED_ITEM_RECORDS: usize = 1000;

/// The party starts with no ammunition in any slot.
pub(crate) fn empty_ammunition() -> Vec<i32> {
    vec![0; AMMUNITION_SLOTS]
}

/// The party starts with no condition in any of the party-wide buffs.
pub(crate) fn empty_party_buffs() -> Vec<u8> {
    vec![0; PARTY_CONDITION_BYTES]
}

/// The six members' psi selections as they stand in their records, which is what the spell menu
/// remembers at the start.
pub(crate) fn initial_psi_memory(warriors: &[Warrior]) -> Result<Vec<u8>, RuleError> {
    if warriors.len() > 6 {
        return Err(RuleError::Invalid(
            "psi memory exceeds party capacity".into(),
        ));
    }
    let mut memory = vec![0; DEFINE_SPELLS_BYTES];
    for (index, warrior) in warriors.iter().enumerate() {
        memory[index * PSI_MEMORY_ROW_BYTES..(index + 1) * PSI_MEMORY_ROW_BYTES]
            .copy_from_slice(warrior.record().psi_selection_memory()?);
    }
    Ok(memory)
}
/// A new game starts with no allies.
pub(crate) fn empty_allies() -> Vec<u8> {
    vec![0; story_allies::ALLY_BYTES * story_allies::ALLY_CAPACITY]
}

/// A new game starts with blank notes.
pub(crate) fn empty_notes() -> Vec<u8> {
    vec![0; NOTE_BYTES]
}
/// Whether the notes are all blank.
pub(crate) fn no_notes(notes: &[u8]) -> bool {
    notes.iter().all(|n| *n == 0)
}
/// Whether a level is zero, so it can be omitted from a save.
pub(crate) fn zero_level(level: &u8) -> bool {
    *level == 0
}
/// Whether a word is zero, so it can be omitted from a save.
pub(crate) fn zero_word(value: &i16) -> bool {
    *value == 0
}

/// The random library starts from seed 1.
pub(crate) fn initial_library_seed() -> u32 {
    1
}

/// The story flags of a new game: all eight flags of the first byte set, the rest clear.
pub(crate) fn initial_flags() -> Vec<u8> {
    let mut flags = vec![0; FLAG_BYTES];
    flags[0] = 0xff;
    flags
}

/// Whether the difficulty is the normal setting, so it can be omitted from a save.
pub(crate) fn normal_difficulty(level: &i16) -> bool {
    *level == 0
}

/// Whether a menu is as long as the scenario authored it, or truncated to a whole number of
/// records.
pub(crate) fn menu_length(actual: usize, authored: usize) -> bool {
    actual == authored || actual == authored / story::MENU_RECORD_BYTES * story::MENU_RECORD_BYTES
}
