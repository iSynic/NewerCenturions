//! Party-wide values that equipment changes. Every attribute selector from 16
//! upward names a party-wide value: wearing subtracts the absolute amount and
//! removal sets it to zero.
use crate::{RuleError, Session};

/// One ordered party-wide write made by wear or removal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlobalWrite {
    Subtract { selector: i16, amount: i16 },
    Clear { selector: i16 },
}

/// The difficulty (selector 40).
const DIFFICULTY: u32 = 0x442576;
/// The scenario-directory value that effect 110 also uses (selector 140).
const DURATION_TAIL: u32 = 0x44263e;

/// The location of the party-wide value an attribute selector of 16 or more names.
fn address(selector: i16) -> u32 {
    (0x442562_i64 + 2 * (i64::from(selector) - 30)) as u32
}

/// Callers that cannot carry Session state keep the writes atomic and explicit.
pub(crate) fn require_none(writes: &[GlobalWrite]) -> Result<(), RuleError> {
    if writes.is_empty() {
        Ok(())
    } else {
        Err(super::record_rules::unsupported(
            "item writes party-wide WORDs from a caller without the session",
        ))
    }
}

/// Selectors from 16 reach the party durations and other Information values,
/// other game values, and the warrior table, which has an owner but no adapter
/// here.
fn unrecorded(word: u32) -> RuleError {
    if (0x448aa0..0x44ecb0).contains(&word) {
        super::record_rules::unsupported("equipment global WORD in the warrior table")
    } else if (0x442562..0x442640).contains(&word) {
        super::record_rules::unsupported(
            "equipment global WORD in an unowned Information WORD with no recorded value",
        )
    } else {
        super::record_rules::unsupported(
            "equipment global WORD in unowned globals with no recorded value",
        )
    }
}

impl Session {
    /// Applies the writes in order, or none of them when a prior value is unknown.
    ///
    /// Difficulty, the effect 110 value, the party durations and the story flags
    /// are kept directly. Any other value is known only if equipment wrote it;
    /// wearing against an unknown value is diagnosed instead of assuming zero.
    pub(crate) fn apply_equipment_globals(
        &mut self,
        writes: &[GlobalWrite],
    ) -> Result<(), RuleError> {
        let (buffs, flags) = (self.party_condition.clone(), self.flags.clone());
        let result = self.apply_equipment_writes(writes);
        if result.is_err() {
            (self.party_condition, self.flags) = (buffs, flags);
        }
        result
    }

    /// Applies the party-wide values an item changed in wearing or removal, in order: wearing
    /// subtracts an amount from the value and removal sets it to zero. The difficulty is copied
    /// into a running battle too.
    fn apply_equipment_writes(&mut self, writes: &[GlobalWrite]) -> Result<(), RuleError> {
        let mut difficulty = self.difficulty;
        let mut tail = self.scenario_duration_tail;
        let mut words: std::collections::BTreeMap<u32, i16> =
            self.globals.equipment_globals.iter().copied().collect();
        for write in writes {
            let (selector, amount) = match *write {
                GlobalWrite::Subtract { selector, amount } => (selector, Some(amount)),
                GlobalWrite::Clear { selector } => (selector, None),
            };
            let word = address(selector);
            let owned = word != DIFFICULTY && self.information_word(i64::from(word)).is_some();
            let current = match word {
                DIFFICULTY => Some(difficulty),
                DURATION_TAIL => tail,
                _ if owned => self.information_word(i64::from(word)),
                _ => words.get(&word).copied(),
            };
            let value = match amount {
                None => 0,
                Some(amount) => current
                    .ok_or_else(|| unrecorded(word))?
                    .wrapping_sub(amount),
            };
            match word {
                DIFFICULTY => difficulty = value,
                DURATION_TAIL => tail = Some(value),
                _ if owned => {
                    self.set_information_word(i64::from(word), value);
                }
                _ => {
                    words.insert(word, value);
                }
            }
        }
        self.difficulty = difficulty;
        // The battle keeps its own copy of the value.
        if let Some(combat) = self.combat.as_mut() {
            combat.difficulty = difficulty;
        }
        self.scenario_duration_tail = tail;
        self.globals.equipment_globals = words.into_iter().collect();
        Ok(())
    }
}
