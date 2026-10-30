use crate::RuleError;
use crate::combat::Combat;
use centurions_content::combat::MONSTER_BYTES;
use centurions_content::combat::MonsterRecord;
use serde::{Deserialize, Serialize};
pub(super) mod monster_pick;

/// The battle order and its cursor, which outlast the battle.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BattleOrder {
    pub order: Vec<i16>,
    #[serde(rename = "cursor")]
    pub index: usize,
}

impl BattleOrder {
    /// Before any battle the order is all zero with the cursor at the start.
    pub(crate) fn startup() -> Self {
        Self {
            order: vec![0; super::BODY_SLOTS],
            index: 0,
        }
    }

    /// Keeps the live battle's order and cursor.
    pub(crate) fn capture(combat: &Combat) -> Self {
        Self {
            order: combat.battle_order.clone(),
            index: combat.battle_order_index,
        }
    }

    /// The order holds the 110 body slots and the cursor fits a word.
    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        if self.order.len() != 110 || self.index > u16::MAX as usize {
            return Err("invalid retained initiative work".into());
        }
        Ok(())
    }

    /// The body at the cursor. A cursor past the order reads what follows it: the cursor itself,
    /// then the selected monster.
    pub(crate) fn current(&self, monster_up: usize) -> Result<i16, RuleError> {
        if let Some(body) = self.order.get(self.index) {
            return Ok(*body);
        }
        // Rows past the battle order read the cursor, the selected monster and the
        // dispatch mode that follow it.
        match self.index {
            110 => Ok(self.index as i16),
            111 => Ok(monster_up as i16),
            112 => Err(RuleError::Unsupported {
                action: "combat.initiative",
                dependency: "retained order row 112 reads the dispatch mode WORD",
            }),
            _ => Err(RuleError::Unsupported {
                action: "combat.initiative",
                dependency: "retained order row past the initiative work",
            }),
        }
    }
}

/// One of the 100 monster records, with where the monster stands.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MonsterSlot {
    pub record: MonsterRecord,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<(i32, i32)>,
}

impl MonsterSlot {
    /// Where the monster stands; unsupported for a slot no battle has placed.
    pub(crate) fn position(&self) -> Result<(i32, i32), RuleError> {
        self.position.ok_or(RuleError::Unsupported {
            action: "combat.retained_npc",
            dependency: "inactive NPC position absent from legacy context",
        })
    }
}

/// The 100 monster slots and their positions, which outlast the roster and nested
/// battles.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Monsters {
    /// The monster count; ordinary booty clears it, early returns do not.
    #[serde(default)]
    #[serde(rename = "active_count")]
    pub num_mon: i16,
    pub slots: Vec<Option<MonsterSlot>>,
    #[serde(default)]
    pub origin: (i32, i32),
}

impl Default for Monsters {
    /// A set of 100 empty monster slots.
    fn default() -> Self {
        Self {
            num_mon: 0,
            slots: vec![None; 100],
            origin: (0, 0),
        }
    }
}

impl Monsters {
    /// Copies a whole monster record into its slot, leaving the position.
    ///
    /// Original: buttonchoice
    pub(crate) fn write_record(
        &mut self,
        index: usize,
        record: MonsterRecord,
    ) -> Result<(), RuleError> {
        let slot = self
            .slots
            .get_mut(index)
            .ok_or("NPC copy outside record array")?;
        if let Some(slot) = slot {
            slot.record = record;
        } else {
            *slot = Some(MonsterSlot {
                record,
                position: None,
            });
        }
        Ok(())
    }

    /// The monster slot at `index`, for changing.
    pub(crate) fn slot_mut(&mut self, index: usize) -> Result<&mut MonsterSlot, RuleError> {
        self.slots
            .get_mut(index)
            .and_then(Option::as_mut)
            .ok_or(RuleError::Unsupported {
                action: "combat.retained_npc",
                dependency: "inactive NPC record absent from legacy context",
            })
    }
    /// The records of the monsters in the current count.
    pub(crate) fn active_records(&self) -> Result<Vec<MonsterRecord>, RuleError> {
        let count = self.num_mon;
        if count > 100 {
            return Err(RuleError::Unsupported {
                action: "combat.rewards",
                dependency: "retained NPC count outside record array",
            });
        }
        (0..count.max(0) as usize)
            .map(|index| self.slot(index).map(|slot| slot.record.clone()))
            .collect()
    }

    /// Puts each record back in its slot.
    pub(crate) fn store_records(&mut self, records: Vec<MonsterRecord>) {
        for (slot, record) in self.slots.iter_mut().zip(records) {
            slot.as_mut().unwrap().record = record;
        }
    }

    /// Before any battle every slot holds a zeroed monster at the origin.
    pub(crate) fn startup() -> Self {
        // Before any battle the records, positions and origin are zero.
        Self {
            num_mon: 0,
            origin: (0, 0),
            slots: vec![
                Some(MonsterSlot {
                    record: MonsterRecord {
                        bytes: vec![0; MONSTER_BYTES]
                    },
                    position: Some((0, 0)),
                });
                100
            ],
        }
    }

    /// The slots are the 100 monster records and the count does not pass them.
    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        // The monster count is never above 100; cleanup also handles a negative
        // count.
        if self.slots.len() != 100
            || self.num_mon > 100
            || self
                .slots
                .iter()
                .flatten()
                .any(|slot| slot.record.bytes.len() != MONSTER_BYTES)
        {
            return Err(RuleError::Invalid("invalid retained NPC backing".into()));
        }
        Ok(())
    }

    /// Keeps the battle's monsters, their positions and the map origin, so they outlast it.
    pub(crate) fn capture(&mut self, combat: &Combat) {
        self.num_mon = combat.roster.records.len() as i16;
        self.origin = crate::combat::records::grid_point((0, 0), combat.field.map_shift);
        self.slots.clone_from(&combat.monsters.slots);
        for (index, (record, position)) in combat
            .roster
            .records
            .iter()
            .zip(&combat.roster.mon_pos)
            .take(100)
            .enumerate()
        {
            self.slots[index] = Some(MonsterSlot {
                record: record.clone(),
                position: Some(crate::combat::records::grid_point(
                    *position,
                    combat.field.map_shift,
                )),
            });
        }
    }

    /// Battle setup moves the origin without touching unused positions.
    ///
    /// Original: combatsetup
    pub(crate) fn rebase_origin(&mut self, shift: (i16, i16)) {
        if self.slots.iter().all(Option::is_none) {
            return;
        }
        let origin = crate::combat::records::grid_point((0, 0), shift);
        for slot in self.slots.iter_mut().flatten() {
            if let Some(position) = slot.position.as_mut() {
                let x = position.0.wrapping_sub(self.origin.0) as i16;
                let y = position.1.wrapping_sub(self.origin.1) as i16;
                *position = (i32::from(x) + origin.0, i32::from(y) + origin.1);
            }
        }
        self.origin = origin;
    }

    /// The monster slot at `index`; unsupported when no monster was ever there.
    pub(crate) fn slot(&self, index: usize) -> Result<&MonsterSlot, RuleError> {
        self.slots
            .get(index)
            .and_then(Option::as_ref)
            .ok_or(RuleError::Unsupported {
                action: "combat.retained_npc",
                dependency: "inactive NPC record absent from legacy context",
            })
    }
}

impl crate::Session {
    /// The record of the monster left standing at `index`: the live battle's, or the one kept after
    /// it ended.
    pub fn survivor_record(&self, index: usize) -> Option<&MonsterRecord> {
        if let Some(combat) = &self.combat {
            combat.roster.records.get(index)
        } else {
            self.globals
                .monsters
                .slots
                .get(index)?
                .as_ref()
                .map(|slot| &slot.record)
        }
    }

    /// Keeps what outlasts the battle: the monsters, the battle order, the tracks, the target
    /// picks, the kill counts, the neighbors and the scripts still to run.
    pub(crate) fn store_monsters(&mut self) {
        self.store_monster_pick();
        if let Some(combat) = &self.combat {
            self.globals.track_tail.clone_from(&combat.track_tail);
            self.globals.target_picks = combat.target_picks;
            self.globals.round_scripts.capture(combat);
            self.globals.turn_round = Some(combat.combat_round);
            self.globals.battlefield =
                super::battlefield::Battlefield::capture(combat, &self.globals.battlefield);
            self.globals.kill_counts = combat.kill_counts;
            self.globals.neighbors = combat.neighbors.clone();
            self.globals.monsters.capture(combat);
            self.globals.outer_todo_queue = Some(combat.todo_queue.clone());
            self.globals.battle_order = BattleOrder::capture(combat);
        }
    }
}

impl Combat {
    /// The position a scripted spell reads for its target. spelltargets treats the
    /// value as a party position even when a script supplied a map coordinate, so
    /// large values read the battle order and what follows it.
    ///
    /// Original: spelltargets
    pub(crate) fn script_target_position(
        &self,
        first: i16,
        small_field: bool,
        party_positions: &[(i16, i16); 6],
    ) -> Result<(i16, i16), RuleError> {
        if let Ok(index) = usize::try_from(first)
            && let Some(point) = self.squad.pos.get(index)
        {
            return Ok(*point);
        }
        // A battle's squad holds only the warriors it entered with; the later party
        // slots keep whatever was last there.
        if let Ok(index @ 0..6) = usize::try_from(first) {
            return Ok(party_positions[index]);
        }
        if first == 72 {
            // These read the screen layout values that follow the table, which
            // depend only on the field-size preference.
            return Ok(if small_field { (320, 377) } else { (480, 485) });
        }
        // The monster positions run straight into the party positions.
        if (-100..0).contains(&first) {
            return self.story_position((first + 100) as usize);
        }
        if first < 0 {
            return Err(RuleError::Unsupported {
                action: "story.spell",
                dependency: "script target position before the NPC position array",
            });
        }
        if first < 8 {
            return Err(RuleError::Unsupported {
                action: "story.spell",
                dependency: "script target position in a presentation-shared party slot",
            });
        }
        if first > 63 {
            return Err(RuleError::Unsupported {
                action: "story.spell",
                dependency: "script target position past the initiative work",
            });
        }
        let index = (first as usize * 4 - 30) / 2;
        let read = |index| match index {
            0..110 => self.battle_order.get(index).copied(),
            110 => Some(self.battle_order_index as i16),
            111 => Some(self.monster_up as i16),
            112 => Some(0),
            _ => None,
        };
        Ok((
            read(index).ok_or("script target position alias outside initiative")?,
            read(index + 1).ok_or("script target position alias outside initiative")?,
        ))
    }

    /// The monster record a story script means by `index`: a live monster, or else the one kept in
    /// its slot.
    pub(crate) fn story_record(&self, index: usize) -> Result<&MonsterRecord, RuleError> {
        if let Some(record) = self.roster.records.get(index) {
            Ok(record)
        } else {
            Ok(&self.monsters.slot(index)?.record)
        }
    }

    /// The monster record a story script changes at `index`: a live monster, or else the one kept
    /// in its slot.
    pub(crate) fn story_record_mut(
        &mut self,
        index: usize,
    ) -> Result<&mut MonsterRecord, RuleError> {
        if index < self.roster.records.len() {
            Ok(&mut self.roster.records[index])
        } else {
            self.monsters
                .slots
                .get_mut(index)
                .and_then(Option::as_mut)
                .map(|slot| &mut slot.record)
                .ok_or(RuleError::Unsupported {
                    action: "combat.retained_npc",
                    dependency: "inactive NPC record absent from legacy context",
                })
        }
    }

    /// Where the monster a story script means by `index` stands, live or kept.
    pub(crate) fn story_position(&self, index: usize) -> Result<(i16, i16), RuleError> {
        if let Some(position) = self.roster.mon_pos.get(index) {
            Ok(*position)
        } else {
            let position = self.monsters.slot(index)?.position()?;
            Ok(crate::combat::records::grid_position(
                position,
                self.field.map_shift,
            ))
        }
    }
}
