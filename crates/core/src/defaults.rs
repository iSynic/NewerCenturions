use crate::{RuleError, story, story_allies};
use centurions_content::Warrior;

pub(crate) fn empty_ammunition() -> Vec<i32> {
    vec![0; 200]
}

pub(crate) fn empty_party_buffs() -> Vec<u8> {
    vec![0; 20]
}

pub(crate) fn initial_psi_memory(warriors: &[Warrior]) -> Result<Vec<u8>, RuleError> {
    if warriors.len() > 6 {
        return Err(RuleError::Invalid(
            "psi memory exceeds party capacity".into(),
        ));
    }
    let mut memory = vec![0; 480];
    for (index, warrior) in warriors.iter().enumerate() {
        memory[index * 80..(index + 1) * 80].copy_from_slice(centurions_content::span(
            &warrior.bytes,
            0xa76,
            80,
        )?);
    }
    Ok(memory)
}
pub(crate) fn empty_allies() -> Vec<u8> {
    vec![0; story_allies::ALLY_BYTES * story_allies::ALLY_CAPACITY]
}

pub(crate) fn empty_notes() -> Vec<u8> {
    vec![0; 19]
}
pub(crate) fn no_notes(notes: &[u8]) -> bool {
    notes.iter().all(|n| *n == 0)
}
pub(crate) fn zero_level(level: &u8) -> bool {
    *level == 0
}
pub(crate) fn zero_word(value: &i16) -> bool {
    *value == 0
}

pub(crate) fn initial_library_seed() -> u32 {
    1
}

pub(crate) fn initial_flags() -> Vec<u8> {
    let mut flags = vec![0; 128];
    flags[0] = 0xff;
    flags
}

pub(crate) fn normal_difficulty(level: &i16) -> bool {
    *level == 0
}

pub(crate) fn menu_length(actual: usize, authored: usize) -> bool {
    actual == authored || actual == authored / story::MENU_RECORD_BYTES * story::MENU_RECORD_BYTES
}
