//! Literal NPC-loop WORDs, shared with attacks and Getup across child battles.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NpcWork {
    pub attempts: i16,
    pub inhibit: i16,
}

impl NpcWork {
    pub(crate) fn startup() -> Self {
        Self {
            attempts: 0,
            inhibit: 0,
        }
    }
    pub(crate) fn getup(&mut self) {
        self.attempts = 0;
        self.inhibit = 0;
    }
    pub(crate) fn attempts(&self) -> i16 {
        self.attempts
    }
    pub(crate) fn inhibited(&self) -> bool {
        self.inhibit != 0
    }
    pub(crate) fn add_attempts(&mut self, amount: i16) {
        self.attempts = self.attempts.wrapping_add(amount);
    }
}
