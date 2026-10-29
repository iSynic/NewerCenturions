//! Literal signed victory WORDs; neither value is reconstructed from a roster.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VictoryWork {
    /// WORD4435c4: combatsetup resets this before creating hostile NPCs.
    pub total: i16,
    /// WORD44374c: getbattleorder resets this independently of the total.
    pub defeated: i16,
}

impl VictoryWork {
    pub(crate) fn startup() -> Self {
        Self::default()
    }

    pub(crate) fn add_total(&mut self, delta: i16) {
        self.total = self.total.wrapping_add(delta);
    }

    pub(crate) fn add_defeated(&mut self, delta: i16) {
        self.defeated = self.defeated.wrapping_add(delta);
    }

    pub(crate) fn finish(&mut self) {
        self.defeated = self.total;
    }

    pub(crate) fn has_remaining(self) -> bool {
        self.total > self.defeated
    }

    /// checkforenemy uses inequality, including defeated values above total.
    pub(crate) fn reaction_admitted(self) -> bool {
        self.total != self.defeated
    }

    /// combat/getup compare the signed WORDs; NPC movement subtracts them.
    pub(crate) fn movement_admitted(self) -> bool {
        self.total.wrapping_sub(self.defeated) > 0
    }
}
