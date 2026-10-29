//! Shared round globals survive the combat-mode flag and nested arenas.
use super::Combat;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoundWork {
    /// Signed BYTE44255c and five WORDs439cec loaded from the battle record.
    pub counter: Option<i8>,
    pub slots: Option<[i16; 5]>,
    /// WORD408576 is shared by Getup and newland126, not a saved local slot.
    pub slot: Option<i16>,
    /// BYTE438583 is written literally on entry/return, without a stack pop.
    pub processing: Option<u8>,
    /// WORD4427a6 increments even on the wrap that completes inactivity.
    pub inactive_rounds: Option<i16>,
}

impl RoundWork {
    pub(crate) fn is_unknown(&self) -> bool {
        *self == Self::default()
    }

    pub(crate) fn capture(&mut self, combat: &Combat) {
        self.counter = Some(combat.round_scripts.counter);
        self.slots = Some(combat.round_scripts.slots);
        self.inactive_rounds = Some(combat.inactive_rounds);
        if let Some(program) = &combat.round_scripts.pending {
            self.slot = Some(i16::from(program.slot));
        }
    }
}
