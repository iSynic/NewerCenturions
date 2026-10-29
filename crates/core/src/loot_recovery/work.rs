use crate::RuleError;
use serde::{Deserialize, Serialize};

/// Item WORDs at425264, stride22, before crate4285f4. Only180 are read by booty10.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryWork {
    pub displaced_items: Vec<i16>,
    /// Held44278c when the older confiscated-equipment block is absent.
    pub held_money: i32,
}

impl Default for RecoveryWork {
    fn default() -> Self {
        Self::startup()
    }
}

impl RecoveryWork {
    pub(crate) fn startup() -> Self {
        Self {
            displaced_items: vec![0; 600],
            held_money: 0,
        }
    }
    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        if self.displaced_items.len() != 600 {
            return Err("invalid displaced-item work".into());
        }
        Ok(())
    }
    pub(crate) fn prefix(&self) -> Vec<i16> {
        self.displaced_items[..180].to_vec()
    }
}
