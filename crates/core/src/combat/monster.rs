use crate::{RandomState, RuleError};
use centurions_content::{
    combat::MonsterRecord,
    records::{EnemyRecord, EnemyRecordMut},
};

fn adjust(bytes: &mut [u8], difficulty: i16, resistance: i16, armor: i16) -> Result<(), RuleError> {
    let immunity = EnemyRecord::from_bytes(bytes).magic_immunity()?;
    if immunity < 99 {
        EnemyRecordMut::from_bytes(bytes)
            .set_magic_immunity(immunity.wrapping_add(difficulty.wrapping_mul(3)));
    }
    EnemyRecordMut::from_bytes(bytes).adjust_elemental_saves(difficulty.wrapping_mul(resistance));
    let armor = EnemyRecord::from_bytes(bytes)
        .armor()?
        .wrapping_sub(difficulty.wrapping_mul(armor));
    EnemyRecordMut::from_bytes(bytes).set_armor(armor);
    let speed = EnemyRecord::from_bytes(bytes)
        .speed()?
        .wrapping_add(difficulty);
    EnemyRecordMut::from_bytes(bytes).set_speed(speed);
    Ok(())
}

/// combatsetup 0x11e70e adjusts placed enemies after their initialization draws.
pub(crate) fn adjust_battle(bytes: &mut [u8], difficulty: i16) -> Result<(), RuleError> {
    adjust(bytes, difficulty, 10, 3)?;
    let factor = 1.0 + f64::from(difficulty) * 0.4;
    let psi =
        (f64::from(EnemyRecord::from_bytes(bytes).maximum_psi()?) * factor).trunc() as i32 as i16;
    let hp = ((f64::from(EnemyRecord::from_bytes(bytes).stamina()?) * factor).trunc() as i32
        as i16)
        .max(1);
    let mut record = EnemyRecordMut::from_bytes(bytes);
    record.set_maximum_psi(psi);
    record.set_psi_points(psi);
    record.set_stamina(hp);
    record.set_maximum_stamina(hp);
    Ok(())
}

/// buildmonster 0x109210 consumes HP dice before armor, speed and psi draws.
pub fn summon(
    mut monster: MonsterRecord,
    faction: i16,
    difficulty: i16,
    random: &mut RandomState,
) -> Result<MonsterRecord, RuleError> {
    let bytes = &mut monster.bytes;
    let level = EnemyRecord::from_bytes(bytes).level()?;
    super::monster_weapons::validate(bytes)?;
    let mut hp = EnemyRecord::from_bytes(bytes).base_stamina()?;
    for _ in 0..level {
        hp = hp.wrapping_add(random.bound(8));
    }
    if faction > -1 {
        EnemyRecordMut::from_bytes(bytes).set_faction(faction);
    }
    EnemyRecordMut::from_bytes(bytes).set_movement_direction(0, 0);
    let mut armor = EnemyRecord::from_bytes(bytes).armor()?;
    if armor != 0 {
        armor = armor.wrapping_add(random.bound(3).wrapping_sub(2));
    }
    EnemyRecordMut::from_bytes(bytes).set_armor(armor.max(0));
    let speed = EnemyRecord::from_bytes(bytes)
        .speed()?
        .wrapping_add(random.bound(3).wrapping_sub(2))
        .max(1);
    EnemyRecordMut::from_bytes(bytes).set_speed(speed);
    let psi = EnemyRecord::from_bytes(bytes).psi_points()?;
    let spread = psi / 10;
    let psi = psi.wrapping_add(random.range(spread.wrapping_neg(), spread));
    adjust(bytes, difficulty, 7, 2)?;
    let factor = (1.0 + f64::from(difficulty) * 0.33) as f32;
    let scaled =
        |value: i16| ((f64::from(value as f32) * f64::from(factor)) as f32).trunc() as i32 as i16;
    let mut record = EnemyRecordMut::from_bytes(bytes);
    record.set_maximum_psi(scaled(psi));
    record.set_psi_points(scaled(psi));
    record.set_stamina(scaled(hp).max(1));
    record.set_maximum_stamina(scaled(hp).max(1));
    super::monster_weapons::initialize(bytes, random, super::monster_weapons::MatchOrder::First)?;
    Ok(monster)
}
