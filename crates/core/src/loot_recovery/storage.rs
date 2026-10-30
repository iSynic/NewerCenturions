use crate::RuleError;
use serde::{Deserialize, Serialize};

/// Item IDs set aside while the party's equipment is held; booty mode 10 reads
/// the first 180.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Storage {
    /// The stored item IDs.
    #[serde(rename = "displaced_items")]
    pub storage_items: Vec<i16>,
    /// The stored money, used when the save has no confiscated-equipment block.
    #[serde(rename = "held_money")]
    pub wealth_store: i32,
}

impl Default for Storage {
    /// A new game holds no money and no items.
    fn default() -> Self {
        Self::startup()
    }
}

impl Storage {
    /// Property held for the party at the start of a game: no items and no money.
    pub(crate) fn startup() -> Self {
        Self {
            storage_items: vec![0; 600],
            wealth_store: 0,
        }
    }
    /// Checks that the holding has all 600 item places.
    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        if self.storage_items.len() != 600 {
            return Err("invalid displaced-item work".into());
        }
        Ok(())
    }
    /// The first 180 places, which are the items offered back to the party.
    pub(crate) fn prefix(&self) -> Vec<i16> {
        self.storage_items[..180].to_vec()
    }
}
