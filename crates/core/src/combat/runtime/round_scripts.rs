//! Round scripts and counters, which outlast the battle and nested battles.
use crate::combat::Combat;
use serde::{Deserialize, Serialize};

/// The round count and the five round scripts of the battle, which outlast the battle and nested
/// battles, with the slot being run and the rounds without an attack.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoundScripts {
    /// The signed round count and five round scripts from the battle record.
    pub counter: Option<i8>,
    pub slots: Option<[i16; 5]>,
    /// The round slot getup is running, shared with the round scripts.
    pub slot: Option<i16>,
    /// Set while a round script runs.
    pub processing: Option<u8>,
    /// Rounds without an attack; it counts up even on the round that ends the
    /// battle for inactivity.
    #[serde(rename = "inactive_rounds")]
    pub blank_round: Option<i16>,
}

impl RoundScripts {
    /// Whether the round scripts were never captured from a battle.
    pub(crate) fn is_unknown(&self) -> bool {
        *self == Self::default()
    }

    /// Keeps the battle's round count, scripts, quiet rounds and pending slot.
    pub(crate) fn capture(&mut self, combat: &Combat) {
        self.counter = Some(combat.round_macros.counter);
        self.slots = Some(combat.round_macros.slots);
        self.blank_round = Some(combat.blank_round);
        if let Some(program) = &combat.round_macros.pending {
            self.slot = Some(i16::from(program.slot));
        }
    }
}
