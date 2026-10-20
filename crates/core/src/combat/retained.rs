use crate::RuleError;
use crate::combat::Combat;
use centurions_content::combat::MonsterRecord;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NpcSlot {
    pub record: MonsterRecord,
    pub position: (i32, i32),
}

/// Native record/position arrays outlive the active roster and child battles.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NpcBacking {
    pub slots: Vec<Option<NpcSlot>>,
    #[serde(default)]
    pub origin: (i32, i32),
}

impl Default for NpcBacking {
    fn default() -> Self {
        Self {
            slots: vec![None; 100],
            origin: (0, 0),
        }
    }
}

impl NpcBacking {
    pub(crate) fn startup() -> Self {
        // Independent startup bytes: records439d3c, positions446f0a and origin442780.
        Self {
            origin: (0, 0),
            slots: vec![
                Some(NpcSlot {
                    record: MonsterRecord {
                        bytes: vec![0; 288]
                    },
                    position: (0, 0),
                });
                100
            ],
        }
    }

    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        if self.slots.len() != 100
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
                position: (
                    i32::from(position.0) + i32::from(combat.map.shift.0) + 5,
                    i32::from(position.1) + i32::from(combat.map.shift.1) + 5,
                ),
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
            let x = slot.position.0.wrapping_sub(self.origin.0) as i16;
            let y = slot.position.1.wrapping_sub(self.origin.1) as i16;
            slot.position = (i32::from(x) + origin.0, i32::from(y) + origin.1);
        }
        self.origin = origin;
    }

    fn slot(&self, index: usize) -> Result<&NpcSlot, RuleError> {
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
    pub(crate) fn retain_npcs(&mut self) {
        if let Some(combat) = &self.combat {
            self.npc_backing.capture(combat);
            self.retained_death_queue = Some(combat.death_queue.clone());
        }
    }
}

impl Combat {
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
            let position = self.retained_npcs.slot(index)?.position;
            Ok((
                position.0.wrapping_sub(i32::from(self.map.shift.0) + 5) as i16,
                position.1.wrapping_sub(i32::from(self.map.shift.1) + 5) as i16,
            ))
        }
    }
}
