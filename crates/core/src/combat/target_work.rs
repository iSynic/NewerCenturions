//! Target-list WORDs outside the 110-word mask: the first triple's cell WORDs
//! 443722/443724, the triple count WORD4435c0 and cast's loop index WORD442bd4.
//! Live combat owns them until teardown copies them back, like the mask tail.
use super::spells::SpellTarget;
use crate::RuleError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetWork {
    /// WORD443722/443724: the first triple's first and second WORDs.
    pub cell: (i16, i16),
    /// WORD4435c0.
    pub count: i16,
    /// WORD442bd4.
    pub index: i16,
}

impl TargetWork {
    pub fn is_startup(&self) -> bool {
        *self == Self::default()
    }

    /// cleartarget13822a writes -1 to every triple WORD.
    pub(crate) fn clear(&mut self) {
        self.cell = (-1, -1);
    }

    /// A target-list writer's first triple; the cell WORDs keep their value
    /// when the list stays empty.
    pub(crate) fn write(&mut self, targets: &[SpellTarget]) {
        if let Some(target) = targets.first() {
            self.cell = (target.first, target.second);
        }
    }

    /// resolvespell's summon arm with WORD43874a above 99 (1516f0..151706):
    /// cleartarget, a zero count and loop index 6, so cast's next increment
    /// passes every target count.
    pub(crate) fn refuse_summon(&mut self) {
        self.clear();
        self.count = 0;
        self.index = 6;
    }

    /// Whether cast's loop (10e658..10e664) ends after the triple at `index`.
    pub(crate) fn loop_ended(&self, index: usize) -> bool {
        self.index != index as i16
    }

    /// The absolute cell the movement arm (151418) reads: triple 0's WORD443722/443724.
    /// Outside the 90x90 map the arms index map WORD column*90+row: cleartarget's
    /// (-1,-1) reaches far past the map, and a body triple (body,-1) stores the body
    /// at an off-map position (docs/memory-alias-audit.md).
    pub(crate) fn arm_cell(&self) -> Result<(i16, i16), RuleError> {
        in_map(self.cell)
    }

    /// The absolute cell spelllist's summon arm (15be9c..15bf02) reads: the triple at
    /// cast's loop index WORD442bd4. `current` is that triple inside a cast loop. Outside
    /// one (collide, newland17, outdoor casts) the index is cast's last value and
    /// triples 1..6 hold cleartarget's -1; index 7 and above reach the defeated WORD
    /// and the attack packet.
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

    /// spelltargets 15d13c..15d1b6 after its own resolvespell writes -1 over
    /// triple[WORD442bd4] and triple[`slot`], its first argument: cast passes its
    /// loop index, collide and newland17 pass 0.
    pub(crate) fn resolved(&mut self, slot: i16) {
        if self.index == 0 || slot == 0 {
            self.clear();
        }
    }

    /// A combatchoice selection writer's triple (11b318..11b414, 11b3f2..11b41a,
    /// 11b164..11b1b8): the count WORD advances; triple 0 is the cell owner.
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

    /// cast's loop index for the triple about to resolve, with the list count.
    pub(crate) fn enter(&mut self, index: usize, count: usize) {
        self.index = index as i16;
        self.count = count as i16;
    }

    /// cast's last increment (10e658) and its closing cleartarget (10e678).
    pub(crate) fn leave(&mut self) {
        self.index = self.index.wrapping_add(1);
        self.clear();
    }
}

fn in_map(cell: (i16, i16)) -> Result<(i16, i16), RuleError> {
    if (0..90).contains(&cell.0) && (0..90).contains(&cell.1) {
        return Ok(cell);
    }
    Err(RuleError::Unsupported {
        action: "combat.spell.arm",
        dependency: "movement or summon arm at a target cell outside the 90x90 map",
    })
}
