//! The hostile and defeated counts that decide a battle; neither is recounted
//! from the roster.
use serde::{Deserialize, Serialize};

/// The tally that decides a battle: the hostile monsters and the monsters defeated. Neither is
/// recounted from the field.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct KillCounts {
    /// Hostile monsters; battle setup resets it before creating them.
    #[serde(rename = "total")]
    pub num_enemy: i16,
    /// Monsters defeated; getbattleorder resets it apart from the total.
    #[serde(rename = "defeated")]
    pub kill_mon: i16,
}

impl KillCounts {
    /// The tally at the start of the game: no monsters and none defeated.
    pub(crate) fn startup() -> Self {
        Self::default()
    }

    /// Changes the number of hostile monsters.
    pub(crate) fn add_num_enemy(&mut self, delta: i16) {
        self.num_enemy = self.num_enemy.wrapping_add(delta);
    }

    /// Changes the number of monsters defeated.
    pub(crate) fn add_kill_mon(&mut self, delta: i16) {
        self.kill_mon = self.kill_mon.wrapping_add(delta);
    }

    /// Counts every hostile monster as defeated.
    pub(crate) fn finish(&mut self) {
        self.kill_mon = self.num_enemy;
    }

    /// Whether hostile monsters remain undefeated.
    pub(crate) fn has_remaining(self) -> bool {
        self.num_enemy > self.kill_mon
    }

    /// Whether neighbours react: the counts differ, even when more are defeated
    /// than the total.
    ///
    /// Original: checkforenemy
    pub(crate) fn reaction_admitted(self) -> bool {
        self.num_enemy != self.kill_mon
    }

    /// Whether monsters may still move: the defeated count, taken from the hostile
    /// count with wrapping, leaves some.
    pub(crate) fn movement_admitted(self) -> bool {
        self.num_enemy.wrapping_sub(self.kill_mon) > 0
    }
}
