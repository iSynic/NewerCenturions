use crate::{RandomState, RuleError};
use centurions_content::{combat::MonsterRecord, word};

fn put_word(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

fn adjust(bytes: &mut [u8], difficulty: i16, resistance: i16, armor: i16) -> Result<(), RuleError> {
    let aim = word(bytes, 0xa)?;
    if aim < 99 {
        put_word(bytes, 0xa, aim.wrapping_add(difficulty.wrapping_mul(3)));
    }
    for value in &mut bytes[0x8b..0x91] {
        *value = value.wrapping_add(difficulty.wrapping_mul(resistance) as u8);
    }
    let armor = word(bytes, 8)?.wrapping_sub(difficulty.wrapping_mul(armor));
    put_word(bytes, 8, armor);
    let speed = word(bytes, 4)?.wrapping_add(difficulty);
    put_word(bytes, 4, speed);
    Ok(())
}

/// combatsetup 0x11e70e adjusts placed enemies after their initialization draws.
pub(crate) fn adjust_battle(bytes: &mut [u8], difficulty: i16) -> Result<(), RuleError> {
    adjust(bytes, difficulty, 10, 3)?;
    let factor = 1.0 + f64::from(difficulty) * 0.4;
    let psi = (f64::from(word(bytes, 0x52)?) * factor).trunc() as i32 as i16;
    let hp = ((f64::from(word(bytes, 0x110)?) * factor).trunc() as i32 as i16).max(1);
    for offset in [0x52, 0x10e] {
        put_word(bytes, offset, psi);
    }
    for offset in [0x110, 0x112] {
        put_word(bytes, offset, hp);
    }
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
    let level = word(bytes, 0)?;
    if !(0..=127).contains(&level) || word(bytes, 0x4e)? < 0 {
        return Err(RuleError::Unsupported {
            action: "combat.summon",
            dependency: "summoned level or random weapon tables",
        });
    }
    let mut hp = word(bytes, 2)?;
    for _ in 0..level {
        hp = hp.wrapping_add(random.bound(8));
    }
    if faction > -1 {
        put_word(bytes, 0xe, faction);
    }
    bytes[0x107] = 0;
    bytes[0x108] = 0;
    let mut armor = word(bytes, 8)?;
    if armor != 0 {
        armor = armor.wrapping_add(random.bound(3).wrapping_sub(2));
    }
    put_word(bytes, 8, armor.max(0));
    let speed = word(bytes, 4)?
        .wrapping_add(random.bound(3).wrapping_sub(2))
        .max(1);
    put_word(bytes, 4, speed);
    let psi = word(bytes, 0x10e)?;
    let spread = psi / 10;
    let psi = psi.wrapping_add(random.range(spread.wrapping_neg(), spread));
    adjust(bytes, difficulty, 7, 2)?;
    let factor = (1.0 + f64::from(difficulty) * 0.33) as f32;
    let scaled =
        |value: i16| ((f64::from(value as f32) * f64::from(factor)) as f32).trunc() as i32 as i16;
    for offset in [0x52, 0x10e] {
        put_word(bytes, offset, scaled(psi));
    }
    for offset in [0x110, 0x112] {
        put_word(bytes, offset, scaled(hp).max(1));
    }
    Ok(monster)
}
