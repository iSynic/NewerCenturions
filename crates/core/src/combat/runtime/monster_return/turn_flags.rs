//! A monster's turn flags, shared with attacks and getup, even across nested
//! battles.
use serde::{Deserialize, Serialize};

/// What the current monster turn has tried: the spell attempts so far (`nospell`) and whether it
/// has attacked (`didattack`). They are shared with attacks and the start of a turn, even across
/// nested battles.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonsterTurnFlags {
    #[serde(rename = "attempts")]
    pub no_spell: i16,
    #[serde(rename = "inhibit")]
    pub did_attack: i16,
}

impl MonsterTurnFlags {
    /// The flags at the start of the game: nothing tried.
    pub(crate) fn startup() -> Self {
        Self {
            no_spell: 0,
            did_attack: 0,
        }
    }
    /// Clears both flags as a new turn is set up.
    ///
    /// Original: getup
    pub(crate) fn getup(&mut self) {
        self.no_spell = 0;
        self.did_attack = 0;
    }
    /// The spell attempts so far this turn.
    pub(crate) fn no_spell(&self) -> i16 {
        self.no_spell
    }
    /// Whether the monster has attacked this turn.
    pub(crate) fn did_attack(&self) -> bool {
        self.did_attack != 0
    }
    /// Counts more spell attempts.
    pub(crate) fn add_no_spell(&mut self, amount: i16) {
        self.no_spell = self.no_spell.wrapping_add(amount);
    }
}
