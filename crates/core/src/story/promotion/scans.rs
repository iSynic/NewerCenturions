//! Scan-selection rules. The saved draft remains in promotion::Progress;
//! this module owns budget, learned choices and shortcut normalization.
use crate::RuleError;
use centurions_content::{
    Warrior,
    records::{LearnedScans, LearnedScansMut, ProfessionProfile, SpellInfoRecord, WarriorRecord},
};
use serde::{Deserialize, Serialize};

/// The cost of a scan of each tier in scan points.
pub const SCAN_COSTS: [i16; 7] = [5, 7, 10, 15, 25, 40, 60];

/// The scan selection draft, with signed 16-bit costs.
///
/// Original: scanselect
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanSelection {
    pub warrior: Warrior,
    pub tier: usize,
    pub remaining: i16,
    highest_tier: i16,
}

impl ScanSelection {
    /// Begins choosing scans for a warrior who can perform them.
    pub fn new(warrior: &Warrior, profiles: &[u8]) -> Result<Self, RuleError> {
        if warrior.record().max_spells_attacks()? == 0
            || !(1..=4).contains(&warrior.record().psi_school()?)
        {
            return Err(RuleError::Unsupported {
                action: "editor.scans",
                dependency: "warrior cannot perform scans",
            });
        }
        let profile = warrior.record().profession()?;
        if profile < 1 {
            return Err(RuleError::Invalid("scan MOS profile is unavailable".into()));
        }
        let highest_tier = ProfessionProfile::from_table(profiles, profile)?.highest_scan_tier()?;
        let mut warrior = warrior.clone();
        let remaining = normalize_scans(&mut warrior)?;
        Ok(Self {
            warrior,
            tier: 0,
            remaining,
            highest_tier,
        })
    }

    /// Picks the tier of scan the player is choosing from; it cannot be above what the warrior's
    /// profession allows.
    pub fn select_tier(&mut self, tier: usize) -> Result<(), RuleError> {
        if tier >= 7 || tier as i16 + 1 > self.highest_tier {
            return Err(RuleError::Unsupported {
                action: "editor.scans",
                dependency: "selected tier exceeds the warrior's MOS scan limit",
            });
        }
        self.tier = tier;
        Ok(())
    }

    /// Selects or drops a scan of the chosen tier, within the warrior's points. Returns the sound
    /// to play, or nothing when a scan cannot be afforded.
    pub fn toggle(&mut self, column: usize) -> Result<Option<i16>, RuleError> {
        if column >= 7 {
            return Err(RuleError::Invalid(
                "scan column outside original matrix".into(),
            ));
        }
        let cost = SCAN_COSTS[self.tier];
        let bytes = &mut self.warrior.bytes;
        let selected = LearnedScans(bytes).selected(self.tier, column);
        if !selected && self.remaining < cost {
            return Ok(None);
        }
        let count = LearnedScans(bytes).tier_count(self.tier)?;
        LearnedScansMut(bytes).set_selected(self.tier, column, !selected);
        self.remaining = self
            .remaining
            .wrapping_add(if selected { cost } else { -cost });
        LearnedScansMut(bytes)
            .set_tier_count(self.tier, count.wrapping_add(if selected { 1 } else { -1 }));
        Ok(Some(if selected { 144 } else { 145 }))
    }

    /// Finishes the choice and returns the warrior with the scans learned.
    pub fn finish(mut self, attacks: &[u8]) -> Result<Warrior, RuleError> {
        finish_scans(&mut self.warrior, attacks)?;
        Ok(self.warrior)
    }
}

/// Keeps the scans the warrior already has, within the points the level allows.
pub(super) fn keep_spells(warrior: &mut Warrior, attacks: &[u8]) -> Result<(), RuleError> {
    normalize_scans(warrior)?;
    finish_scans(warrior, attacks)
}

/// Drops learned scans, from the first tier on, that the warrior's points no longer cover, and
/// returns the points left.
fn normalize_scans(warrior: &mut Warrior) -> Result<i16, RuleError> {
    let bytes = &mut warrior.bytes;
    let level = WarriorRecord::from_bytes(bytes).level()?;
    let mut available = level.wrapping_mul(level).wrapping_add(16);
    for (row, cost) in SCAN_COSTS.into_iter().enumerate() {
        for column in 0..7 {
            if LearnedScans(bytes).selected(row, column) {
                let next = available.wrapping_sub(cost);
                if next > -1 {
                    available = next;
                } else {
                    LearnedScansMut(bytes).set_selected(row, column, false);
                }
            }
        }
    }
    Ok(available)
}

/// Marks whether a selected scan heals and clears each shortcut that names a scan not selected.
fn finish_scans(warrior: &mut Warrior, attacks: &[u8]) -> Result<(), RuleError> {
    let bytes = &mut warrior.bytes;
    LearnedScansMut(bytes).set_healing_available(false);
    let school = WarriorRecord::from_bytes(bytes).psi_school()? - 1;
    if !(0..4).contains(&school) {
        return Err(RuleError::Unsupported {
            action: "story.promotion",
            dependency: "spell school outside loaded bands",
        });
    }
    for row in 0..7 {
        for column in 0..7 {
            if LearnedScans(bytes).selected(row, column) {
                let spell =
                    SpellInfoRecord::scan_entry(attacks, school as usize, row * 7 + column)?;
                if spell.effect()? == 57 {
                    LearnedScansMut(bytes).set_healing_available(true);
                }
            }
        }
    }
    for index in 0..10 {
        let (row, column) = LearnedScans(bytes).shortcut(index)?;
        if !(0..7).contains(&row) || !(0..7).contains(&column) {
            return Err(RuleError::Unsupported {
                action: "story.promotion",
                dependency: "spell shortcut outside original selection matrix",
            });
        }
        if !LearnedScans(bytes).selected(row as usize, column as usize) {
            LearnedScansMut(bytes).clear_shortcut(index);
        }
    }
    Ok(())
}
