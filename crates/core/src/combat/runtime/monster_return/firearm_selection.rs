//! What a monster firing after the battle can see and target.
use super::turn_state;
use crate::RuleError;
use crate::combat::{
    self, CombatField, battlefield::Battlefield, globals::Monsters, spells::SelectionView,
};
use crate::state::WarriorTable;
use centurions_content::{
    Warrior, combat::MonsterRecord, mission::OpeningMission, records::WarriorRecord,
};

/// What a monster firing after the battle can see and target: the monsters, the party's positions
/// and the map.
pub(super) struct OutOfCombatSelection<'a> {
    backing: &'a Monsters,
    tactical: &'a Battlefield,
    map: &'a CombatField,
    count: i16,
    table: &'a WarriorTable,
}

impl<'a> OutOfCombatSelection<'a> {
    /// Gathers the monsters, positions and map for a shot after the battle.
    pub(super) fn new(
        backing: &'a Monsters,
        tactical: &'a Battlefield,
        table: &'a WarriorTable,
    ) -> Result<Self, RuleError> {
        let count = backing.num_mon;
        // Session validation holds num_mon in 0..=100.
        if count > 100 {
            return Err("retained NPC count WORD 0x43874a above 100".into());
        }
        let map = &tactical.field;
        Ok(Self {
            backing,
            tactical,
            map,
            count,
            table,
        })
    }
}

impl SelectionView for OutOfCombatSelection<'_> {
    /// The monster in a slot.
    fn monster(&self, index: usize) -> Result<&MonsterRecord, RuleError> {
        Ok(&self.backing.slot(index)?.record)
    }
    /// How many monsters there are.
    fn monster_count(&self) -> usize {
        self.count.max(0) as usize
    }
    /// The upper bound of a target choice: the monsters, after the ten warrior bodies.
    fn monster_target_bound(&self) -> i16 {
        self.count.wrapping_add(10)
    }
    /// The cell a body stands in.
    fn position(&self, body: usize) -> Result<(i16, i16), RuleError> {
        if body < 10 {
            self.tactical
                .pos
                .get(body)
                .copied()
                .ok_or_else(|| RuleError::from("NPC firearm party body outside the squad"))
        } else {
            let point = self.backing.slot(body - 10)?.position()?;
            Ok(crate::combat::records::grid_position(
                point,
                self.map.map_shift,
            ))
        }
    }
    /// The side a body fights on; bodies past the squad are read from the warrior table.
    fn faction(&self, warriors: &[Warrior], body: usize) -> Result<i16, RuleError> {
        if body < 10 {
            // Bodies past the squad read the table; 8 and 9 are the editor backup.
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
            Ok(self.monster(body - 10)?.record().faction()?)
        }
    }
    /// The shift of the tactical map.
    fn map_shift(&self) -> (i16, i16) {
        self.map.map_shift
    }
    /// Whether one cell sees another.
    fn visible(
        &self,
        mission: &OpeningMission,
        from: (i16, i16),
        to: (i16, i16),
    ) -> Result<bool, RuleError> {
        visible(self.map, mission, from, to)
    }
    /// Refuses a choice among area targets, which a shot after the battle does not support.
    fn best_choice(
        &self,
        _warriors: &[Warrior],
        _areas: &[u8],
        _shape: usize,
        _side: i16,
        _distances: &[i16; 112],
        _range: i16,
    ) -> Result<Option<usize>, RuleError> {
        Err(turn_state::missing("retained NPC firearm area selection"))
    }
}

/// Whether one cell sees another on the map; only the map and the cells matter.
pub(super) fn visible(
    map: &CombatField,
    mission: &OpeningMission,
    from: (i16, i16),
    to: (i16, i16),
) -> Result<bool, RuleError> {
    // Sight needs only the map, mission, coordinates and delay; this view holds no
    // actors and is never installed.
    let view = combat::Combat::from_records(
        map.clone(),
        combat::SquadPlacement {
            pos: vec![],
            char_under: vec![],
        },
        combat::EnemySetup {
            records: vec![],
            mon_pos: vec![],
        },
    );
    combat::lifecycle::can_see(&view, mission, from, to, 0)
}
