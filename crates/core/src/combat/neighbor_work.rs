//! Shared WORD work written by checkforenemy110e96, independent of caller locals.
use crate::{RuleError, combat::Combat};
use centurions_content::Warrior;
use serde::{Deserialize, Serialize};
mod retained;
mod scan;
pub(crate) use retained::scan as scan_retained;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NeighborWork {
    /// WORD4434c2, cleared on entry and set by a nonempty departure comparison.
    pub departure: i16,
    /// Three 110-WORD rows at442934, selected by native modes0,1,2.
    pub presence: [Vec<i16>; 3],
    /// WORDs4434c4..4434de; the next six WORDs alias the retained party mask.
    pub bodies: Vec<i16>,
}

impl Default for NeighborWork {
    fn default() -> Self {
        Self::startup()
    }
}

impl NeighborWork {
    pub(crate) fn startup() -> Self {
        Self {
            departure: 0,
            presence: std::array::from_fn(|_| vec![0; 110]),
            bodies: vec![0; 14],
        }
    }

    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        if self.bodies.len() != 14 || self.presence.iter().any(|row| row.len() != 110) {
            return Err("invalid retained neighbor work".into());
        }
        Ok(())
    }

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

pub(crate) struct NativeNeighbors<'a> {
    pub selected_warrior: usize,
    pub party_mask: &'a mut [i16; 6],
    /// Memory beside the map that checkforenemy and movemonster read past its edges.
    pub aliases: super::map_alias::MapAliases<'a>,
    /// Warrior table records past the squad, when the caller lends them.
    pub table: Option<&'a crate::state::WarriorTable>,
}

pub(crate) fn begin(combat: &mut Combat) {
    combat.neighbor_work.departure = 0;
}

pub(crate) fn require_roster_body(
    combat: &Combat,
    warriors: &[Warrior],
    body: i16,
) -> Result<(), RuleError> {
    if (0..warriors.len() as i16).contains(&body)
        || body >= 10 && ((body - 10) as usize) < combat.enemies.records.len()
    {
        Ok(())
    } else if body >= 10 {
        Err(unsupported("neighbor attack past the loaded NPC records"))
    } else if body == 9 {
        // attack2 and combatupdate2 index NPC record -1 for body 9, before 0x439d3c.
        Err(unsupported("neighbor attack body 9 before the NPC array"))
    } else if body >= 0 {
        Err(unsupported("neighbor attack in an inactive warrior record"))
    } else {
        Err(unsupported("neighbor attack before the warrior table"))
    }
}

/// One loop iteration. The caller owns its local count/index and entry writer.
pub(crate) fn scan(
    combat: &mut Combat,
    warriors: &[Warrior],
    mode: usize,
    direction: (i8, i8),
    context: &mut NativeNeighbors<'_>,
) -> Result<Vec<i16>, RuleError> {
    if combat
        .order
        .get(combat.cursor)
        .copied()
        .unwrap_or(combat.cursor as i16)
        < 0
    {
        return Ok(vec![]);
    }
    scan_with_size(combat, warriors, mode, direction, context, &mut 0)
}

pub(crate) fn scan_with_size(
    combat: &mut Combat,
    warriors: &[Warrior],
    mode: usize,
    direction: (i8, i8),
    context: &mut NativeNeighbors<'_>,
    size: &mut i8,
) -> Result<Vec<i16>, RuleError> {
    let actor = combat
        .order
        .get(combat.cursor)
        .copied()
        .unwrap_or(combat.cursor as i16);
    if !combat.victory_work.reaction_admitted() {
        return Ok(vec![]);
    }
    let mut work = std::mem::take(&mut combat.neighbor_work);
    let result = scan::run(
        &scan::Scene {
            warriors,
            map: &combat.map,
            aliases: context.aliases,
            table: context.table,
            actor,
            selected_warrior: context.selected_warrior,
            selected_enemy: combat.selected_enemy,
            warrior_position: &|index| {
                combat
                    .squad
                    .positions
                    .get(index)
                    .copied()
                    .ok_or_else(|| RuleError::from("selected warrior outside the squad"))
            },
            enemy: &|index| combat.story_record(index),
            enemy_position: &|index| combat.story_position(index),
        },
        &mut work,
        context.party_mask,
        mode,
        direction,
        size,
    );
    combat.neighbor_work = work;
    result
}

fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.neighbors",
        dependency,
    }
}
