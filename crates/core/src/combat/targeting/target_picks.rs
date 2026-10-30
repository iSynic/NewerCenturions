//! The target list's cell, count and cast index. The battle holds them until it
//! ends and copies them back.
use crate::RuleError;
use crate::combat::spells::SpellTarget;
use serde::{Deserialize, Serialize};

/// The targets picked for a cast: the first target's cell, how many are picked, and which is being
/// resolved.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetPicks {
    /// The first target's cell.
    pub cell: (i16, i16),
    /// Targets picked.
    pub count: i16,
    /// The target cast is resolving.
    pub index: i16,
}

impl TargetPicks {
    /// Whether the picks are still in their starting state: cell (0, 0), none picked, index 0.
    pub fn is_startup(&self) -> bool {
        *self == Self::default()
    }

    /// Clears every target to -1.
    ///
    /// Original: cleartarget
    pub(crate) fn clear(&mut self) {
        self.cell = (-1, -1);
    }

    /// Records the first target; the cell keeps its value when the list is empty.
    pub(crate) fn write(&mut self, targets: &[SpellTarget]) {
        if let Some(target) = targets.first() {
            self.cell = (target.first, target.second);
        }
    }

    /// A full roster refuses a summon: the targets clear and the index jumps to 6,
    /// so cast's loop ends.
    ///
    /// Original: resolvespell
    pub(crate) fn refuse_summon(&mut self) {
        self.clear();
        self.count = 0;
        self.index = 6;
    }

    /// Whether cast's loop ends after the target at `index`.
    pub(crate) fn loop_ended(&self, index: usize) -> bool {
        self.index != index as i16
    }

    /// The cell a moving spell reads: the first target's. Off the map the read lands
    /// beside it: a cleared target reaches far past the map, and a body target holds
    /// an off-map position (docs/memory-alias-audit.md).
    pub(crate) fn arm_cell(&self) -> Result<(i16, i16), RuleError> {
        in_map(self.cell)
    }

    /// The cell a summon reads: the target at cast's index. `current` is that target
    /// inside a cast. Outside one (a field collision, a script or an outdoor cast)
    /// the index is cast's last value and later targets are cleared; index 7 and up
    /// read past the list (see docs/source-notes/combat.md).
    pub(crate) fn summon_cell(&self, current: Option<(i16, i16)>) -> Result<(i16, i16), RuleError> {
        match self.index {
            0 => in_map(self.cell),
            1..=6 => in_map(current.unwrap_or((-1, -1))),
            _ => Err(RuleError::Unsupported {
                action: "combat.spell.arm",
                dependency: "summon arm reads a target triple past the seven-triple table",
            }),
        }
    }

    /// Clears the resolved target and the target at `slot`: cast passes its index,
    /// a field collision or a script passes 0.
    ///
    /// Original: spelltargets
    pub(crate) fn resolved(&mut self, slot: i16) {
        if self.index == 0 || slot == 0 {
            self.clear();
        }
    }

    /// Adds a picked target; the first target owns the cell.
    ///
    /// Original: combatchoice
    pub(crate) fn select(&mut self, first: Option<i16>, second: Option<i16>) {
        if self.count == 0 {
            if let Some(first) = first {
                self.cell.0 = first;
            }
            if let Some(second) = second {
                self.cell.1 = second;
            }
        }
        self.count = self.count.wrapping_add(1);
    }

    /// Starts resolving the target at `index` of `count`.
    pub(crate) fn enter(&mut self, index: usize, count: usize) {
        self.index = index as i16;
        self.count = count as i16;
    }

    /// Ends the cast: the index steps past the last target and the targets clear.
    ///
    /// Original: cast
    pub(crate) fn leave(&mut self) {
        self.index = self.index.wrapping_add(1);
        self.clear();
    }
}

/// The cell, if it lies on the 90 by 90 map.
fn in_map(cell: (i16, i16)) -> Result<(i16, i16), RuleError> {
    if (0..90).contains(&cell.0) && (0..90).contains(&cell.1) {
        return Ok(cell);
    }
    Err(RuleError::Unsupported {
        action: "combat.spell.arm",
        dependency: "movement or summon arm at a target cell outside the 90x90 map",
    })
}
