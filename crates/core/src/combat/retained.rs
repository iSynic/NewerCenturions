use crate::RuleError;
use crate::combat::Combat;
use centurions_content::combat::MonsterRecord;
use serde::{Deserialize, Serialize};

/// WORDs4470b8..447192 and cursor447194 survive the combat-mode flag.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InitiativeWork {
    pub order: Vec<i16>,
    pub cursor: usize,
}

impl InitiativeWork {
    pub(crate) fn startup() -> Self {
        Self {
            order: vec![0; 110],
            cursor: 0,
        }
    }

    pub(crate) fn capture(combat: &Combat) -> Self {
        Self {
            order: combat.order.clone(),
            cursor: combat.cursor,
        }
    }

    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        if self.order.len() != 110 || self.cursor > u16::MAX as usize {
            return Err("invalid retained initiative work".into());
        }
        Ok(())
    }

    pub(crate) fn current(&self, selected_enemy: usize) -> Result<i16, RuleError> {
        if let Some(body) = self.order.get(self.cursor) {
            return Ok(*body);
        }
        // Rows past order 0x4470b8 read cursor 0x447194, selected NPC 0x447196, then
        // dispatch mode 0x447198.
        match self.cursor {
            110 => Ok(self.cursor as i16),
            111 => Ok(selected_enemy as i16),
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NpcSlot {
    pub record: MonsterRecord,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<(i32, i32)>,
}

impl NpcSlot {
    pub(crate) fn position(&self) -> Result<(i32, i32), RuleError> {
        self.position.ok_or(RuleError::Unsupported {
            action: "combat.retained_npc",
            dependency: "inactive NPC position absent from legacy context",
        })
    }
}

/// Native record/position arrays outlive the active roster and child battles.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NpcBacking {
    /// Signed WORD43874a; ordinary booty clears it, early/storage returns do not.
    #[serde(default)]
    pub active_count: i16,
    pub slots: Vec<Option<NpcSlot>>,
    #[serde(default)]
    pub origin: (i32, i32),
}

impl Default for NpcBacking {
    fn default() -> Self {
        Self {
            active_count: 0,
            slots: vec![None; 100],
            origin: (0, 0),
        }
    }
}

impl NpcBacking {
    /// buttonchoice10ad copies the whole NPC record without its coordinate WORDs.
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
            *slot = Some(NpcSlot {
                record,
                position: None,
            });
        }
        Ok(())
    }

    pub(crate) fn slot_mut(&mut self, index: usize) -> Result<&mut NpcSlot, RuleError> {
        self.slots
            .get_mut(index)
            .and_then(Option::as_mut)
            .ok_or(RuleError::Unsupported {
                action: "combat.retained_npc",
                dependency: "inactive NPC record absent from legacy context",
            })
    }
    pub(crate) fn active_records(&self) -> Result<Vec<MonsterRecord>, RuleError> {
        let count = self.active_count;
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

    pub(crate) fn store_records(&mut self, records: Vec<MonsterRecord>) {
        for (slot, record) in self.slots.iter_mut().zip(records) {
            slot.as_mut().unwrap().record = record;
        }
    }

    pub(crate) fn startup() -> Self {
        // Independent startup bytes: records439d3c, positions446f0a and origin442780.
        Self {
            active_count: 0,
            origin: (0, 0),
            slots: vec![
                Some(NpcSlot {
                    record: MonsterRecord {
                        bytes: vec![0; 288]
                    },
                    position: Some((0, 0)),
                });
                100
            ],
        }
    }

    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        // Every writer of the count WORD 0x43874a stores a roster length of at most
        // 100, a summon's count + 1 below 100 or zero. Native cleanup controls also
        // start from negative counts, which the cleanup readers handle.
        if self.slots.len() != 100
            || self.active_count > 100
            || self
                .slots
                .iter()
                .flatten()
                .any(|slot| slot.record.bytes.len() != 288)
        {
            return Err(RuleError::Invalid("invalid retained NPC backing".into()));
        }
        Ok(())
    }

    pub(crate) fn capture(&mut self, combat: &Combat) {
        self.active_count = combat.enemies.records.len() as i16;
        self.origin = (
            i32::from(combat.map.shift.0) + 5,
            i32::from(combat.map.shift.1) + 5,
        );
        self.slots.clone_from(&combat.retained_npcs.slots);
        for (index, (record, position)) in combat
            .enemies
            .records
            .iter()
            .zip(&combat.enemies.positions)
            .take(100)
            .enumerate()
        {
            self.slots[index] = Some(NpcSlot {
                record: record.clone(),
                position: Some((
                    i32::from(position.0) + i32::from(combat.map.shift.0) + 5,
                    i32::from(position.1) + i32::from(combat.map.shift.1) + 5,
                )),
            });
        }
    }

    /// combatsetup resets the origin without rewriting inactive coordinate WORDs.
    pub(crate) fn rebase_origin(&mut self, shift: (i16, i16)) {
        if self.slots.iter().all(Option::is_none) {
            return;
        }
        let origin = (i32::from(shift.0) + 5, i32::from(shift.1) + 5);
        for slot in self.slots.iter_mut().flatten() {
            if let Some(position) = slot.position.as_mut() {
                let x = position.0.wrapping_sub(self.origin.0) as i16;
                let y = position.1.wrapping_sub(self.origin.1) as i16;
                *position = (i32::from(x) + origin.0, i32::from(y) + origin.1);
            }
        }
        self.origin = origin;
    }

    pub(crate) fn slot(&self, index: usize) -> Result<&NpcSlot, RuleError> {
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
    pub fn survivor_record(&self, index: usize) -> Option<&MonsterRecord> {
        if let Some(combat) = &self.combat {
            combat.enemies.records.get(index)
        } else {
            self.native_work
                .npc_backing
                .slots
                .get(index)?
                .as_ref()
                .map(|slot| &slot.record)
        }
    }

    pub(crate) fn retain_npcs(&mut self) {
        self.retain_monster_work();
        if let Some(combat) = &self.combat {
            self.native_work
                .spell_target_tail
                .clone_from(&combat.target_tail);
            self.native_work.target_work = combat.target_work;
            self.native_work.round_work.capture(combat);
            self.native_work.turn_round = Some(combat.round);
            self.native_work.tactical_work = super::tactical_work::TacticalWork::capture(
                combat,
                &self.native_work.tactical_work,
            );
            self.native_work.victory_work = combat.victory_work;
            self.native_work.neighbor_work = combat.neighbor_work.clone();
            self.native_work.npc_backing.capture(combat);
            self.native_work.retained_death_queue = Some(combat.death_queue.clone());
            self.native_work.initiative_work = InitiativeWork::capture(combat);
        }
    }
}

impl Combat {
    /// spelltargets15cb0e reads a party-position index even when newland17
    /// supplied a map coordinate. The table continues into the turn order,
    /// cursor, selected NPC and encounter word cleared at combat entry.
    pub(crate) fn script_target_position(
        &self,
        first: i16,
        small_field: bool,
        party_positions: &[(i16, i16); 6],
    ) -> Result<(i16, i16), RuleError> {
        if let Ok(index) = usize::try_from(first)
            && let Some(point) = self.squad.positions.get(index)
        {
            return Ok(*point);
        }
        // A battle's squad holds only the warriors it entered with; combatsetup leaves
        // the later party slots as the last writer left them.
        if let Ok(index @ 0..6) = usize::try_from(first) {
            return Ok(party_positions[index]);
        }
        if first == 72 {
            // WORD4471ba is ToolBoxInit's viewport right; WORD4471bc is
            // combatupdate's final statistics rectangle top. Both original
            // preference22 layouts are fixed, independent of host window size.
            return Ok(if small_field { (320, 377) } else { (480, 485) });
        }
        // The NPC positions 0x446f0a end where the party positions 0x44709a begin.
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
            0..110 => self.order.get(index).copied(),
            110 => Some(self.cursor as i16),
            111 => Some(self.selected_enemy as i16),
            112 => Some(0),
            _ => None,
        };
        Ok((
            read(index).ok_or("script target position alias outside initiative")?,
            read(index + 1).ok_or("script target position alias outside initiative")?,
        ))
    }

    pub(crate) fn story_record(&self, index: usize) -> Result<&MonsterRecord, RuleError> {
        if let Some(record) = self.enemies.records.get(index) {
            Ok(record)
        } else {
            Ok(&self.retained_npcs.slot(index)?.record)
        }
    }

    pub(crate) fn story_record_mut(
        &mut self,
        index: usize,
    ) -> Result<&mut MonsterRecord, RuleError> {
        if index < self.enemies.records.len() {
            Ok(&mut self.enemies.records[index])
        } else {
            self.retained_npcs
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

    pub(crate) fn story_position(&self, index: usize) -> Result<(i16, i16), RuleError> {
        if let Some(position) = self.enemies.positions.get(index) {
            Ok(*position)
        } else {
            let position = self.retained_npcs.slot(index)?.position()?;
            Ok((
                position.0.wrapping_sub(i32::from(self.map.shift.0) + 5) as i16,
                position.1.wrapping_sub(i32::from(self.map.shift.1) + 5) as i16,
            ))
        }
    }
}
