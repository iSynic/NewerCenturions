//! Borrow current retained targeting inputs; the active prefix is not the caster.
use super::work;
use crate::RuleError;
use crate::combat::{
    self, TacticalMap, retained::NpcBacking, spells::SelectionView, tactical_work::TacticalWork,
};
use crate::state::WarriorTable;
use centurions_content::{
    Warrior, combat::MonsterRecord, mission::OpeningMission, records::WarriorRecord,
};

pub(super) struct RetainedSelection<'a> {
    backing: &'a NpcBacking,
    tactical: &'a TacticalWork,
    map: &'a TacticalMap,
    count: i16,
    table: &'a WarriorTable,
}

impl<'a> RetainedSelection<'a> {
    pub(super) fn new(
        backing: &'a NpcBacking,
        tactical: &'a TacticalWork,
        table: &'a WarriorTable,
    ) -> Result<Self, RuleError> {
        let count = backing.active_count;
        // Session validation holds the count WORD 0x43874a in 0..=100.
        if count > 100 {
            return Err("retained NPC count WORD 0x43874a above 100".into());
        }
        let map = &tactical.map;
        Ok(Self {
            backing,
            tactical,
            map,
            count,
            table,
        })
    }
}

impl SelectionView for RetainedSelection<'_> {
    fn npc(&self, index: usize) -> Result<&MonsterRecord, RuleError> {
        Ok(&self.backing.slot(index)?.record)
    }
    fn npc_count(&self) -> usize {
        self.count.max(0) as usize
    }
    fn npc_target_bound(&self) -> i16 {
        self.count.wrapping_add(10)
    }
    fn position(&self, body: usize) -> Result<(i16, i16), RuleError> {
        if body < 10 {
            self.tactical
                .positions
                .get(body)
                .copied()
                .ok_or_else(|| RuleError::from("NPC firearm party body outside the squad"))
        } else {
            let point = self.backing.slot(body - 10)?.position()?;
            Ok((
                (point.0.wrapping_sub(i32::from(self.map.shift.0) + 5)) as i16,
                (point.1.wrapping_sub(i32::from(self.map.shift.1) + 5)) as i16,
            ))
        }
    }
    fn faction(&self, warriors: &[Warrior], body: usize) -> Result<i16, RuleError> {
        if body < 10 {
            // Bodies past the squad read the table; 8..9 index the editor backup 0x44ecb0.
            match warriors.get(body) {
                Some(warrior) => Ok(warrior.record().faction()?),
                None => {
                    let bytes = self
                        .table
                        .bytes(body)
                        .ok_or("faction body past the table")?;
                    Ok(WarriorRecord::from_bytes(&bytes).faction()?)
                }
            }
        } else {
            Ok(self.npc(body - 10)?.record().faction()?)
        }
    }
    fn map_shift(&self) -> (i16, i16) {
        self.map.shift
    }
    fn visible(
        &self,
        mission: &OpeningMission,
        from: (i16, i16),
        to: (i16, i16),
    ) -> Result<bool, RuleError> {
        visible(self.map, mission, from, to)
    }
    fn best_choice(
        &self,
        _warriors: &[Warrior],
        _areas: &[u8],
        _shape: usize,
        _side: i16,
        _distances: &[i16; 112],
        _range: i16,
    ) -> Result<Option<usize>, RuleError> {
        Err(work::missing("retained NPC firearm area selection"))
    }
}

pub(super) fn visible(
    map: &TacticalMap,
    mission: &OpeningMission,
    from: (i16, i16),
    to: (i16, i16),
) -> Result<bool, RuleError> {
    // can_see reads only this current map, mission, coordinates and delay. The
    // unchanged numeric kernel receives no actors and this view is never installed.
    let view = combat::Combat::from_records(
        map.clone(),
        combat::SquadPlacement {
            positions: vec![],
            under_tiles: vec![],
        },
        combat::EnemySetup {
            records: vec![],
            positions: vec![],
        },
    );
    combat::lifecycle::can_see(&view, mission, from, to, 0)
}
