use crate::combat::monster_weapons;
use crate::combat::monster_weapons::MatchOrder;
use crate::{RandomState, RuleError};
use centurions_content::records::{EnemyRecord, EnemyRecordMut};
use centurions_content::span;

/// Size of one ally record.
pub const ALLY_BYTES: usize = 288;
/// The most allies the party can hold over.
pub const ALLY_CAPACITY: usize = 20;

/// The ally table after an addition: its records, how many allies there are, and the warning raised
/// when there was no room.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllyAddition {
    pub records: Vec<u8>,
    pub count: usize,
    pub warning: Option<i16>,
}

/// Adds a monster to the party as an ally: its stamina is rolled from its hit dice, and difficulty
/// adjusts its psi resistance, armor, agility and elemental saves. A full table warns instead.
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
    monster_weapons::validate(definition)?;
    let record = &mut records[count * ALLY_BYTES..(count + 1) * ALLY_BYTES];
    record.copy_from_slice(definition);
    let mut stamina = EnemyRecord::from_bytes(record).stamina_bonus()?;
    for _ in 0..EnemyRecord::from_bytes(record).hit_dice()?.max(0) {
        stamina = stamina.wrapping_add(random.bound(8));
    }
    EnemyRecordMut::from_bytes(record).set_stamina(stamina);
    EnemyRecordMut::from_bytes(record).set_stamina_max(stamina);
    EnemyRecordMut::from_bytes(record).set_faction(0);
    EnemyRecordMut::from_bytes(record).set_movement_direction(0, 0);
    let psi = EnemyRecord::from_bytes(record).psi_points()?;
    EnemyRecordMut::from_bytes(record).set_psi_points_max(psi);
    // Allies skip the immunity cap a battle applies when it starts.
    let immunity = EnemyRecord::from_bytes(record)
        .psi_resistance()?
        .wrapping_add(difficulty.wrapping_mul(3));
    EnemyRecordMut::from_bytes(record).set_psi_resistance(immunity);
    let armor = EnemyRecord::from_bytes(record)
        .ac()?
        .wrapping_add(difficulty.wrapping_mul(-2));
    EnemyRecordMut::from_bytes(record).set_armor(armor);
    let speed = EnemyRecord::from_bytes(record)
        .agility()?
        .wrapping_add(difficulty);
    EnemyRecordMut::from_bytes(record).set_agility(speed);
    EnemyRecordMut::from_bytes(record).adjust_elemental_saves(difficulty.wrapping_mul(7));
    monster_weapons::initialize(record, random, MatchOrder::First)?;
    normalize_menu(&mut records, count + 1);
    Ok(AllyAddition {
        records,
        count: count + 1,
        warning: None,
    })
}

/// Clears every ally when a stage begins.
pub fn clear_stage_allies(records: &[u8]) -> Result<Vec<u8>, RuleError> {
    let mut records = span(records, 0, ALLY_BYTES * ALLY_CAPACITY)?.to_vec();
    for index in 0..ALLY_CAPACITY {
        EnemyRecordMut::from_bytes(&mut records[index * ALLY_BYTES..]).clear_monster_id();
    }
    Ok(records)
}

/// Clears the target and the been-attacked mark of each of the first `count` allies.
fn normalize_menu(records: &mut [u8], count: usize) {
    for record in records
        .as_chunks_mut::<ALLY_BYTES>()
        .0
        .iter_mut()
        .take(count)
    {
        EnemyRecordMut::from_bytes(record).set_target(0);
        EnemyRecordMut::from_bytes(record).clear_been_attacked();
    }
}
