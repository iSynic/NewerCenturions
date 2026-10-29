use crate::{RandomState, RuleError};
use centurions_content::records::{EnemyRecord, EnemyRecordMut};
use centurions_content::span;

pub const ALLY_BYTES: usize = 288;
pub const ALLY_CAPACITY: usize = 20;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllyAddition {
    pub records: Vec<u8>,
    pub count: usize,
    pub warning: Option<i16>,
}

pub fn add_ally(
    records: &[u8],
    count: usize,
    definition: &[u8],
    difficulty: i16,
    random: &mut RandomState,
) -> Result<AllyAddition, RuleError> {
    let mut records = span(records, 0, ALLY_BYTES * ALLY_CAPACITY)?.to_vec();
    if count >= ALLY_CAPACITY {
        normalize_menu(&mut records, count);
        return Ok(AllyAddition {
            records,
            count,
            warning: Some(108),
        });
    }
    let definition = span(definition, 0, ALLY_BYTES)?;
    crate::combat::monster_weapons::validate(definition)?;
    let record = &mut records[count * ALLY_BYTES..(count + 1) * ALLY_BYTES];
    record.copy_from_slice(definition);
    let mut stamina = EnemyRecord::from_bytes(record).base_stamina()?;
    for _ in 0..EnemyRecord::from_bytes(record).level()?.max(0) {
        stamina = stamina.wrapping_add(random.bound(8));
    }
    EnemyRecordMut::from_bytes(record).set_stamina(stamina);
    EnemyRecordMut::from_bytes(record).set_maximum_stamina(stamina);
    EnemyRecordMut::from_bytes(record).set_faction(0);
    EnemyRecordMut::from_bytes(record).set_movement_direction(0, 0);
    let psi = EnemyRecord::from_bytes(record).psi_points()?;
    EnemyRecordMut::from_bytes(record).set_maximum_psi(psi);
    // The ally caller does not apply the battle initializer's immunity cap.
    let immunity = EnemyRecord::from_bytes(record)
        .magic_immunity()?
        .wrapping_add(difficulty.wrapping_mul(3));
    EnemyRecordMut::from_bytes(record).set_magic_immunity(immunity);
    let armor = EnemyRecord::from_bytes(record)
        .armor()?
        .wrapping_add(difficulty.wrapping_mul(-2));
    EnemyRecordMut::from_bytes(record).set_armor(armor);
    let speed = EnemyRecord::from_bytes(record)
        .speed()?
        .wrapping_add(difficulty);
    EnemyRecordMut::from_bytes(record).set_speed(speed);
    EnemyRecordMut::from_bytes(record).adjust_elemental_saves(difficulty.wrapping_mul(7));
    crate::combat::monster_weapons::initialize(
        record,
        random,
        crate::combat::monster_weapons::MatchOrder::First,
    )?;
    normalize_menu(&mut records, count + 1);
    Ok(AllyAddition {
        records,
        count: count + 1,
        warning: None,
    })
}

pub fn clear_stage_allies(records: &[u8]) -> Result<Vec<u8>, RuleError> {
    let mut records = span(records, 0, ALLY_BYTES * ALLY_CAPACITY)?.to_vec();
    for index in 0..ALLY_CAPACITY {
        EnemyRecordMut::from_bytes(&mut records[index * ALLY_BYTES..]).clear_definition_id();
    }
    Ok(records)
}

fn normalize_menu(records: &mut [u8], count: usize) {
    for record in records
        .as_chunks_mut::<ALLY_BYTES>()
        .0
        .iter_mut()
        .take(count)
    {
        EnemyRecordMut::from_bytes(record).set_target_body(0);
        EnemyRecordMut::from_bytes(record).clear_damage_marker();
    }
}
