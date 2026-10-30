use crate::{RandomState, RuleError};
use centurions_content::{
    combat::MonsterRecord,
    records::{EnemyRecord, EnemyRecordMut},
};

/// Scales a monster for the difficulty: psi resistance rises by three per step unless it is already
/// 99 or more, elemental saves rise and armor falls by the given amounts per step, and agility
/// rises by one per step.
fn adjust(bytes: &mut [u8], difficulty: i16, resistance: i16, armor: i16) -> Result<(), RuleError> {
    let immunity = EnemyRecord::from_bytes(bytes).psi_resistance()?;
    if immunity < 99 {
        EnemyRecordMut::from_bytes(bytes)
            .set_psi_resistance(immunity.wrapping_add(difficulty.wrapping_mul(3)));
    }
    EnemyRecordMut::from_bytes(bytes).adjust_elemental_saves(difficulty.wrapping_mul(resistance));
    let armor = EnemyRecord::from_bytes(bytes)
        .armor_rating()?
        .wrapping_sub(difficulty.wrapping_mul(armor));
    EnemyRecordMut::from_bytes(bytes).set_armor_rating(armor);
    let speed = EnemyRecord::from_bytes(bytes)
        .agility()?
        .wrapping_add(difficulty);
    EnemyRecordMut::from_bytes(bytes).set_agility(speed);
    Ok(())
}

/// Adjusts a placed monster for the difficulty, after its own random draws.
///
/// Original: combatsetup
pub(crate) fn adjust_battle(bytes: &mut [u8], difficulty: i16) -> Result<(), RuleError> {
    adjust(bytes, difficulty, 10, 3)?;
    let factor = 1.0 + f64::from(difficulty) * 0.4;
    let psi = (f64::from(EnemyRecord::from_bytes(bytes).psi_points_max()?) * factor).trunc() as i32
        as i16;
    let hp = ((f64::from(EnemyRecord::from_bytes(bytes).stamina()?) * factor).trunc() as i32
        as i16)
        .max(1);
    let mut record = EnemyRecordMut::from_bytes(bytes);
    record.set_psi_points_max(psi);
    record.set_psi_points(psi);
    record.set_stamina(hp);
    record.set_stamina_max(hp);
    Ok(())
}

/// Builds a summoned monster: stamina dice first, then armor, speed and psi.
///
/// Original: buildmonster
pub fn summon(
    mut monster: MonsterRecord,
    faction: i16,
    difficulty: i16,
    random: &mut RandomState,
) -> Result<MonsterRecord, RuleError> {
    let bytes = &mut monster.bytes;
    let level = EnemyRecord::from_bytes(bytes).hit_dice()?;
    super::monster_weapons::validate(bytes)?;
    let mut hp = EnemyRecord::from_bytes(bytes).stamina_bonus()?;
    for _ in 0..level {
        hp = hp.wrapping_add(random.bound(8));
    }
    if faction > -1 {
        EnemyRecordMut::from_bytes(bytes).set_faction(faction);
    }
    EnemyRecordMut::from_bytes(bytes).set_movement_direction(0, 0);
    let mut armor = EnemyRecord::from_bytes(bytes).armor_rating()?;
    if armor != 0 {
        armor = armor.wrapping_add(random.bound(3).wrapping_sub(2));
    }
    EnemyRecordMut::from_bytes(bytes).set_armor_rating(armor.max(0));
    let speed = EnemyRecord::from_bytes(bytes)
        .agility()?
        .wrapping_add(random.bound(3).wrapping_sub(2))
        .max(1);
    EnemyRecordMut::from_bytes(bytes).set_agility(speed);
    let psi = EnemyRecord::from_bytes(bytes).psi_points()?;
    let spread = psi / 10;
    let psi = psi.wrapping_add(random.range(spread.wrapping_neg(), spread));
    adjust(bytes, difficulty, 7, 2)?;
    let factor = (1.0 + f64::from(difficulty) * 0.33) as f32;
    let scaled =
        |value: i16| ((f64::from(value as f32) * f64::from(factor)) as f32).trunc() as i32 as i16;
    let mut record = EnemyRecordMut::from_bytes(bytes);
    record.set_psi_points_max(scaled(psi));
    record.set_psi_points(scaled(psi));
    record.set_stamina(scaled(hp).max(1));
    record.set_stamina_max(scaled(hp).max(1));
    super::monster_weapons::initialize(bytes, random, super::monster_weapons::MatchOrder::First)?;
    Ok(monster)
}
