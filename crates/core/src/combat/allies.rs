use crate::combat::{CombatField, EnemySetup, footprint};
use crate::story::allies::{ALLY_BYTES, ALLY_CAPACITY};
use crate::{Event, RandomState, RuleError};
use centurions_content::combat::MONSTER_BYTES;
use centurions_content::combat::MonsterRecord;
use centurions_content::mission::OpeningMission;
use centurions_content::records::{Condition, EnemyRecord, EnemyRecordMut};

/// Places the allies carried over from the last battle around the first warrior, each on the first
/// passable cell found searching outward ring by ring. The map changes only if every ally is
/// placed.
pub fn place_allies(
    menu: &[u8],
    count: usize,
    map: &mut CombatField,
    mission: &OpeningMission,
    first: (i16, i16),
    bounds: (i16, i16, i16, i16),
    _random: &mut RandomState,
) -> Result<EnemySetup, RuleError> {
    if menu.len() != ALLY_CAPACITY * ALLY_BYTES
        || count > ALLY_CAPACITY
        || map.tiles.len() != crate::combat::MAP_CELLS
    {
        return Err(RuleError::Invalid("invalid ally placement input".into()));
    }
    let mut staged = map.clone();
    let mut result = EnemySetup {
        records: vec![],
        mon_pos: vec![],
    };
    for index in 0..count {
        let mut bytes = menu[index * ALLY_BYTES..(index + 1) * ALLY_BYTES].to_vec();
        EnemyRecordMut::from_bytes(&mut bytes).set_target(-1);
        EnemyRecordMut::from_bytes(&mut bytes).set_target_scan_excluded(false);
        EnemyRecordMut::from_bytes(&mut bytes).set_reaction_ready(false);
        let mut found = None;
        let mut attempted = [false; 8100];
        'search: for ring in 1..=180_i16 {
            for dy in -ring..ring {
                for dx in -ring..ring {
                    let position = (
                        first.0.wrapping_add(dx.wrapping_mul(2)),
                        first.1.wrapping_add(dy.wrapping_mul(2)),
                    );
                    if position.1 < bounds.0
                        || position.0 < bounds.1
                        || position.1 >= bounds.2
                        || position.0 >= bounds.3
                    {
                        continue;
                    }
                    // Rejected ally cells leave the map and RNG unchanged.
                    let cell = crate::combat::records::grid_index(position, staged.map_shift)?;
                    if std::mem::replace(&mut attempted[cell], true) {
                        continue;
                    }
                    let candidate = footprint::place(
                        &staged,
                        &MonsterRecord {
                            bytes: bytes.clone(),
                        },
                        position,
                        index as i16 + 10,
                    )?;
                    if footprint::placement_passable(&candidate, mission)? {
                        found = Some(candidate);
                        break 'search;
                    }
                }
            }
        }
        let placed =
            found.ok_or_else(|| RuleError::Invalid("no free ally placement cell".into()))?;
        staged = placed.map;
        result.records.push(placed.monster);
        result.mon_pos.push(placed.position);
    }
    *map = staged;
    Ok(result)
}

/// The ally menu after the battle, with the survivors who stay and the sounds played while they are
/// chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllySynchronization {
    pub menu: Vec<u8>,
    pub count: usize,
    pub modal: bool,
    pub sounds: Vec<i16>,
}

/// The survivor dialog: which surviving friendly monsters may stay, which are ticked, and how many
/// the player may tick.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SurvivorSelection {
    pub eligible: Vec<usize>,
    pub selected: Vec<bool>,
    pub limit: usize,
}

impl SurvivorSelection {
    /// Checks that the list is a valid dialog: surviving friendly monsters only, no repeats, and no
    /// more ticks than the limit.
    pub fn validate(&self, records: &[MonsterRecord]) -> Result<(), RuleError> {
        if self.eligible.is_empty()
            || self.eligible.len() > 32
            || self.selected.len() != self.eligible.len()
            || !(4..=18).contains(&self.limit)
            || self.selected.iter().filter(|selected| **selected).count() > self.limit
            || self.eligible.iter().enumerate().any(|(position, index)| {
                self.eligible[..position].contains(index)
                    || records.get(*index).is_none_or(|record| {
                        record.stamina().unwrap_or(0) <= 0
                            || record.record().faction().unwrap_or(1) != 0
                            || record.record().survivor_marker().unwrap_or(0) == 0
                    })
            })
        {
            return Err(RuleError::Invalid("invalid survivor selection".into()));
        }
        Ok(())
    }

    /// Ticks or unticks one survivor row. Ticking past the limit warns and leaves the row as it
    /// was; unticking a survivor who must stay unticks the row and then warns.
    pub fn toggle(
        &mut self,
        row: usize,
        records: &[MonsterRecord],
        random: &mut RandomState,
    ) -> Result<Vec<Event>, RuleError> {
        self.validate(records)?;
        if row >= 32 {
            return Err(RuleError::Invalid("survivor row unavailable".into()));
        }
        let mut events = vec![];
        let warning = if row >= self.eligible.len()
            || (!self.selected[row]
                && self.selected.iter().filter(|selected| **selected).count() >= self.limit)
        {
            Some(0x404002)
        } else {
            self.selected[row] = !self.selected[row];
            events.push(Event::Sound { resource_id: 141 });
            (!self.selected[row] && records[self.eligible[row]].record().survivor_marker()? == -1)
                .then_some(0x403fb0)
        };
        if let Some(address) = warning {
            events.extend([
                Event::Feedback(crate::feedback::GameFeedback::DialogText {
                    dialog: 173,
                    item: 41,
                    address,
                    wait: 80,
                }),
                Event::Sound { resource_id: 6000 },
                Event::Delay { ticks: 80 },
                Event::Sound {
                    resource_id: 29999_i16.wrapping_add(random.bound(4)),
                },
            ]);
        }
        Ok(events)
    }
}

/// Readies the monsters that outlast the battle: a captured machine is revived on the party's side,
/// and the fight conditions are cleared. Returns the dialog of surviving friends, highest stamina
/// first with the ones who must stay moved ahead. The ones who must stay are ticked, then the first
/// ten rows are ticked up to the limit. None when nobody qualifies.
pub fn prepare_survivors(
    records: &mut [MonsterRecord],
) -> Result<Option<SurvivorSelection>, RuleError> {
    if records.len() > 100
        || records
            .iter()
            .any(|record| record.bytes.len() != MONSTER_BYTES)
    {
        return Err(RuleError::Invalid("invalid survivor roster".into()));
    }
    let mut eligible = vec![];
    for (index, record) in records.iter_mut().enumerate() {
        let bytes = &mut record.bytes;
        if EnemyRecord::from_bytes(bytes).mechanical_state() == 255
            && EnemyRecord::from_bytes(bytes).capture_flag() == 1
        {
            EnemyRecordMut::from_bytes(bytes).set_mechanical_state(1);
            EnemyRecordMut::from_bytes(bytes).set_faction(0);
        }
        for condition in [
            Condition::IncreasedDefense,
            Condition::Confused,
            Condition::ScanBlocked,
            Condition::Entangled,
            Condition::Helpless,
            Condition::Retreating,
        ] {
            EnemyRecordMut::from_bytes(bytes).set_condition(condition, 0);
        }
        EnemyRecordMut::from_bytes(bytes).clear_been_attacked();
        if EnemyRecord::from_bytes(bytes).survivor_marker()? < 0 {
            EnemyRecordMut::from_bytes(bytes).set_faction(0);
        }
        if EnemyRecord::from_bytes(bytes).stamina()? > 0
            && EnemyRecord::from_bytes(bytes).faction()? == 0
            && EnemyRecord::from_bytes(bytes).survivor_marker()? != 0
        {
            eligible.push(index);
        }
    }
    if eligible.is_empty() {
        return Ok(None);
    }
    for _ in 0..eligible.len() - 1 {
        for position in 0..eligible.len() - 1 {
            let left = &records[eligible[position]].bytes;
            let right = &records[eligible[position + 1]].bytes;
            if EnemyRecord::from_bytes(left).stamina()?
                < EnemyRecord::from_bytes(right).stamina()?
                || EnemyRecord::from_bytes(right).survivor_marker()? == -1
            {
                eligible.swap(position, position + 1);
            }
        }
    }
    eligible.truncate(32);
    let mut selected = vec![false; eligible.len()];
    let mut selected_count = 0;
    for (position, &index) in eligible.iter().enumerate() {
        if records[index].record().survivor_marker()? == -1 && selected_count < 18 {
            selected[position] = true;
            selected_count += 1;
        }
    }
    let limit = (4 + selected_count).min(18);
    for selected in selected.iter_mut().take(10) {
        if !*selected && selected_count < limit {
            *selected = true;
            selected_count += 1;
        }
    }
    Ok(Some(SurvivorSelection {
        eligible,
        selected,
        limit,
    }))
}

/// Settles who stays with the party after a battle. When there are survivors, one dialog sound is
/// drawn at random; then the survivors ticked by default are kept.
pub fn synchronize(
    menu: &[u8],
    count: usize,
    records: &mut [MonsterRecord],
    random: &mut RandomState,
) -> Result<AllySynchronization, RuleError> {
    if menu.len() != ALLY_CAPACITY * ALLY_BYTES || count > ALLY_CAPACITY {
        return Err(RuleError::Invalid(
            "invalid ally synchronization input".into(),
        ));
    }
    let selection = prepare_survivors(records)?;
    let sounds = if selection.is_some() {
        vec![29999_i16.wrapping_add(random.bound(4))]
    } else {
        vec![]
    };
    let mut result = commit_survivors(menu, count, records, selection.as_ref())?;
    result.sounds = sounds;
    Ok(result)
}

/// Moves the ticked survivors into the ally menu with their condition durations cleared, clears
/// their rewards on the monster record, and sets every monster's stamina to zero. Without a dialog
/// the existing allies are only reset.
pub fn commit_survivors(
    menu: &[u8],
    count: usize,
    records: &mut [MonsterRecord],
    selection: Option<&SurvivorSelection>,
) -> Result<AllySynchronization, RuleError> {
    if menu.len() != ALLY_CAPACITY * ALLY_BYTES
        || count > ALLY_CAPACITY
        || records.len() > 100
        || records
            .iter()
            .any(|record| record.bytes.len() != MONSTER_BYTES)
    {
        return Err(RuleError::Invalid(
            "invalid ally synchronization input".into(),
        ));
    }
    let mut menu = menu.to_vec();
    let Some(selection) = selection else {
        normalize_menu(&mut menu, count);
        return Ok(AllySynchronization {
            menu,
            count,
            modal: false,
            sounds: vec![],
        });
    };
    selection.validate(records)?;
    let (eligible, selected) = (selection.eligible.clone(), selection.selected.clone());
    let mut selected_count = selected.iter().filter(|selected| **selected).count();
    let count = selected_count;
    for (index, selected) in eligible.into_iter().zip(selected) {
        if !selected {
            continue;
        }
        selected_count -= 1;
        let bytes = &mut records[index].bytes;
        let destination = &mut menu[selected_count * ALLY_BYTES..(selected_count + 1) * ALLY_BYTES];
        destination.copy_from_slice(bytes);
        for condition in 0..4 {
            if EnemyRecord::from_bytes(destination).condition_duration(condition) > 0 {
                EnemyRecordMut::from_bytes(destination).set_condition_duration(condition, 0);
            }
        }
        let mut record = EnemyRecordMut::from_bytes(bytes);
        record.clear_survivor_rewards();
    }
    for record in records {
        record.record_mut().set_stamina(0);
    }
    normalize_menu(&mut menu, count);
    Ok(AllySynchronization {
        menu,
        count,
        modal: true,
        sounds: vec![],
    })
}

/// Gives every ally on the menu a clean start: no target and not yet attacked.
fn normalize_menu(menu: &mut [u8], count: usize) {
    for index in 0..count {
        let bytes = &mut menu[index * ALLY_BYTES..(index + 1) * ALLY_BYTES];
        EnemyRecordMut::from_bytes(bytes).set_target(0);
        EnemyRecordMut::from_bytes(bytes).clear_been_attacked();
    }
}
