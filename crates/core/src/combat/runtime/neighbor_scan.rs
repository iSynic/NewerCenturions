//! What checkforenemy records about a body's neighbours.
use crate::combat::map_alias::MapAliases;
use crate::{RuleError, combat::Combat};
use centurions_content::Warrior;
use serde::{Deserialize, Serialize};
mod out_of_combat;
mod scan;
pub(crate) use out_of_combat::scan as scan_out_of_combat;

/// What the neighbor scan remembers: whether a body left an enemy's side, who stood beside whom for
/// each of the three scan modes, and the neighbors found.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Neighbors {
    /// Set when a body leaves an enemy's side.
    pub departure: i16,
    /// Who stands beside whom, one row for each of the three scan modes.
    pub presence: [Vec<i16>; 3],
    /// The neighbours found; the list runs into the party mask (see
    /// docs/source-notes/combat.md).
    pub bodies: Vec<i16>,
}

impl Default for Neighbors {
    /// The startup state of the scan: nobody beside anybody.
    fn default() -> Self {
        Self::startup()
    }
}

impl Neighbors {
    /// The scan's state at the start of the game: no departure, empty rows for the three modes and
    /// a list of fourteen neighbors.
    pub(crate) fn startup() -> Self {
        Self {
            departure: 0,
            presence: std::array::from_fn(|_| vec![0; 110]),
            bodies: vec![0; 14],
        }
    }

    /// Refuses a saved scan whose neighbor list or rows have the wrong size.
    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        if self.bodies.len() != 14 || self.presence.iter().any(|row| row.len() != 110) {
            return Err("invalid retained neighbor work".into());
        }
        Ok(())
    }

    /// The neighbor at `index`; the list runs on past its own fourteen entries into the party's
    /// target mask.
    pub(crate) fn body(&self, index: usize, mask: &[i16; 6]) -> Result<i16, RuleError> {
        if let Some(word) = self.bodies.get(index) {
            Ok(*word)
        } else {
            index
                .checked_sub(14)
                .and_then(|index| mask.get(index))
                .copied()
                .ok_or_else(|| unsupported("neighbor list outside represented target-mask alias"))
        }
    }

    /// Writes the neighbor at `index`, in the list or beyond it in the party's target mask.
    fn write_body(
        &mut self,
        index: usize,
        body: i16,
        mask: &mut [i16; 6],
    ) -> Result<(), RuleError> {
        if let Some(word) = self.bodies.get_mut(index) {
            *word = body;
        } else if let Some(word) = index.checked_sub(14).and_then(|i| mask.get_mut(i)) {
            *word = body;
        } else {
            return Err(unsupported(
                "neighbor list outside represented target-mask alias",
            ));
        }
        Ok(())
    }
}

/// What a scan borrows besides the battle: the selected warrior, the party's target mask, the
/// memory beside the map that the scan may read past its edge, and the warrior table past the
/// squad.
pub(crate) struct NeighborScan<'a> {
    pub char_select_new: usize,
    pub party_track: &'a mut [i16; 6],
    /// Memory beside the map that checkforenemy and movemonster read past its edges.
    pub aliases: MapAliases<'a>,
    /// Warrior table records past the squad, when the caller lends them.
    pub table: Option<&'a crate::state::WarriorTable>,
}

/// Clears the departure mark before a new scan.
pub(crate) fn begin(combat: &mut Combat) {
    combat.neighbors.departure = 0;
}

/// Refuses a body that no loaded record stands for: a warrior outside the squad or a monster past
/// the loaded ones.
pub(crate) fn require_roster_body(
    combat: &Combat,
    warriors: &[Warrior],
    body: i16,
) -> Result<(), RuleError> {
    if (0..warriors.len() as i16).contains(&body)
        || body >= 10 && ((body - 10) as usize) < combat.roster.records.len()
    {
        Ok(())
    } else if body >= 10 {
        Err(unsupported("neighbor attack past the loaded NPC records"))
    } else if body == 9 {
        // Body 9 would read the monster record before the first.
        Err(unsupported("neighbor attack body 9 before the NPC array"))
    } else if body >= 0 {
        Err(unsupported("neighbor attack in an inactive warrior record"))
    } else {
        Err(unsupported("neighbor attack before the warrior table"))
    }
}

/// Scans the neighbors of the acting body in `mode`; nothing is scanned while the current battle
/// order entry is negative.
///
/// Original: checkforenemy
pub(crate) fn scan(
    combat: &mut Combat,
    warriors: &[Warrior],
    mode: usize,
    direction: (i8, i8),
    context: &mut NeighborScan<'_>,
) -> Result<Vec<i16>, RuleError> {
    if combat
        .battle_order
        .get(combat.battle_order_index)
        .copied()
        .unwrap_or(combat.battle_order_index as i16)
        < 0
    {
        return Ok(vec![]);
    }
    scan_with_size(combat, warriors, mode, direction, context, &mut 0)
}

/// Scans the neighbors of the acting body as `scan` does, and reports the footprint size of the
/// body for the caller.
///
/// Original: checkforenemy
pub(crate) fn scan_with_size(
    combat: &mut Combat,
    warriors: &[Warrior],
    mode: usize,
    direction: (i8, i8),
    context: &mut NeighborScan<'_>,
    size: &mut i8,
) -> Result<Vec<i16>, RuleError> {
    let actor = combat
        .battle_order
        .get(combat.battle_order_index)
        .copied()
        .unwrap_or(combat.battle_order_index as i16);
    if !combat.kill_counts.reaction_admitted() {
        return Ok(vec![]);
    }
    let mut neighbors = std::mem::take(&mut combat.neighbors);
    let result = scan::run(
        &scan::Scene {
            warriors,
            map: &combat.field,
            aliases: context.aliases,
            table: context.table,
            actor,
            char_select_new: context.char_select_new,
            monster_up: combat.monster_up,
            warrior_position: &|index| {
                combat
                    .squad
                    .pos
                    .get(index)
                    .copied()
                    .ok_or_else(|| RuleError::from("selected warrior outside the squad"))
            },
            monster: &|index| combat.story_record(index),
            enemy_position: &|index| combat.story_position(index),
        },
        &mut neighbors,
        context.party_track,
        mode,
        direction,
        size,
    );
    combat.neighbors = neighbors;
    result
}

/// The refusal for a scan that depends on an unrecovered part of the game.
fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.neighbors",
        dependency,
    }
}
