use crate::{RandomState, RuleError};
use centurions_content::{WARRIOR_BYTES, Warrior, span, word};

fn put_word(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

/// upgradewarrior 0x162bb2..0x1631e6: each cost iteration rounds through SINGLE.
pub fn upgrade_cost(
    warrior: &Warrior,
    selector: u16,
    profile: &[u8],
) -> Result<(i32, i32), RuleError> {
    let (cost, percent) = upgrade_price(warrior, selector, profile)?;
    Ok((cost.trunc() as i32, percent))
}

/// upgradewarrior 0x163200..0x1632f8 compares SINGLE-rounded points and truncates the debit.
pub fn upgrade_payment(
    warrior: &Warrior,
    selector: u16,
    profile: &[u8],
) -> Result<Option<i32>, RuleError> {
    let (cost, _) = upgrade_cost(warrior, selector, profile)?;
    let cost = cost as f32;
    let points = i32::from_be_bytes(span(&warrior.bytes, 0x9d2, 4)?.try_into().unwrap());
    let rounded = points as f32;
    Ok((rounded >= cost).then(|| (f64::from(points) - f64::from(cost)).trunc() as i32))
}

fn upgrade_price(
    warrior: &Warrior,
    selector: u16,
    profile: &[u8],
) -> Result<(f32, i32), RuleError> {
    const BASE: [i16; 33] = [
        100, 100, 100, 100, 100, 100, 100, 50, 200, 200, 150, 150, 150, 150, 150, 150, 175, 125,
        200, 175, 100, 100, 300, 250, 350, 400, 100, 100, 100, 200, 250, 300, 1000,
    ];
    let base = *BASE.get(usize::from(selector)).ok_or_else(|| {
        RuleError::Invalid("upgrade selector is outside the original table".into())
    })?;
    let offset = match selector {
        7 => Some(0xe4),
        16 => Some(0xf0),
        17 => Some(0xec),
        18 => Some(0xf8),
        19 => Some(0xfc),
        20 => Some(0xe0),
        21 => Some(0x108),
        22 => Some(0x106),
        23..=31 => Some(0x1c + usize::from(selector - 23) * 2),
        32 => Some(0xf4),
        _ => None,
    };
    let percent = if let Some(offset) = offset {
        word(profile, offset)?
    } else {
        10
    };
    if percent < 0 {
        return Err(RuleError::Unsupported {
            action: "character.upgrade_cost",
            dependency: "negative MOS upgrade percentages are not recovered",
        });
    }
    let count = word(&warrior.bytes, 0xace + usize::from(selector) * 2)?;
    let factor = (f64::from((100. + f64::from(percent)) as f32) / 100.) as f32;
    let mut cost = f32::from(base);
    for _ in 0..count.max(0) {
        cost = (f64::from(cost) * f64::from(factor)) as f32;
        if cost >= 50_000_000. {
            // Nonnegative percentages cannot fall back below the original final cap.
            return Ok((50_000_000., i32::from(percent)));
        }
    }
    Ok((cost, i32::from(percent)))
}

/// CODE 1 applyupgrade, 0x1634f8. The caller owns costs and eligibility.
pub fn apply_upgrade(
    warrior: &mut Warrior,
    selector: u16,
    increment: i16,
) -> Result<(), RuleError> {
    span(&warrior.bytes, 0, WARRIOR_BYTES)?;
    let bytes = &mut warrior.bytes;
    let target = match selector {
        0..=6 => Some(0x9e2 + selector as usize * 2),
        7 => Some(0x9d8),
        8..=15 => Some(0x112 + (selector as usize - 8) * 2),
        16..=19 => Some(4 + (selector as usize - 16) * 2),
        20 => Some(0xb42),
        21 => Some(0xb40),
        22 => Some(0x22),
        23..=31 => Some(0xf4 + (selector as usize - 23) * 2),
        32 => Some(0xc),
        _ => None,
    };
    if let Some(offset) = target {
        let delta = match selector {
            7 => increment.wrapping_mul(10),
            32 => 1,
            _ => increment,
        };
        let value = word(bytes, offset)?.wrapping_add(delta);
        put_word(bytes, offset, value);
        if selector == 0 {
            let value = word(bytes, 0x9d8)?.wrapping_add(100);
            put_word(bytes, 0x9d8, value);
        }
    }
    let stamina = word(bytes, 0xb42)?.wrapping_add(word(bytes, 0xb46)?);
    let psi = word(bytes, 0xb40)?.wrapping_add(word(bytes, 0xb44)?);
    put_word(bytes, 0x52, stamina);
    put_word(bytes, 0x5a, psi);
    Ok(())
}

/// CODE 1 strength, 0x138648: the two WORD work results, in native order.
pub fn strength_adjustments(attribute: i16) -> [i16; 2] {
    match attribute {
        ..=3 => [-20, 0],
        4 => [-15, -1],
        5 => [-10, 0],
        6 => [-5, 0],
        7..=15 => [0, 0],
        16 => [1, 0],
        17..=28 => [attribute - 15, (attribute - 13) / 4],
        29 => [14, 5],
        30..=33 => [attribute - 15, 4],
        34 => [19, 5],
        _ => [20, 5],
    }
}

/// CODE 1 agility, 0x1388aa: signed WORD averaging precedes the lookup.
pub fn agility_adjustment(warrior: &Warrior) -> Result<i16, RuleError> {
    let sum = word(&warrior.bytes, 0x9e8)?
        .wrapping_add(word(&warrior.bytes, 0x9ea)?)
        .wrapping_add(1);
    let mean = sum.wrapping_add(((sum as u16) >> 15) as i16) >> 1;
    Ok(match mean {
        3..=6 => mean - 8,
        7..=14 => 0,
        15..=34 => (mean - 13) / 2,
        _ => 11,
    })
}

/// CODE 1 savevsattr, 0x157c68: strict comparison after wrapped threshold.
pub fn save_against_attribute(
    warrior: &Warrior,
    selector: i16,
    random: &mut RandomState,
) -> Result<bool, RuleError> {
    let offset = match selector {
        0 => 0x9e2,
        1 => 0x9e4,
        2 => 0x9e6,
        3 => 0x9e8,
        4 => 0x9ec,
        5 => 0x9ee,
        _ => {
            return Err(RuleError::Unsupported {
                action: "character.attribute_save",
                dependency: "original selector leaves the attribute register uninitialized",
            });
        }
    };
    let threshold = word(&warrior.bytes, offset)?.wrapping_mul(4);
    Ok(random.bound(100) < threshold)
}

/// CODE 1 calcstats, 0x1636ec: derive statistics from the current record.
pub fn calculate_statistics(warrior: &mut Warrior) -> Result<(), RuleError> {
    span(&warrior.bytes, 0, WARRIOR_BYTES)?;
    let bytes = &mut warrior.bytes;
    let strength = word(bytes, 0x9e2)?;
    let intellect = word(bytes, 0x9e4)?;
    let agility = word(bytes, 0x9e6)?;
    let speed = word(bytes, 0x9e8)?;
    let accuracy = word(bytes, 0x9ea)?;
    let psi = word(bytes, 0x9ec)?;
    let perception = word(bytes, 0x9ee)?;
    for (offset, value) in [
        (0xb48, agility.wrapping_add(perception) / 5),
        (0xb4a, psi / 4),
        (0xb4c, psi / 4),
        (0xb4e, psi / 4),
        (0xb50, psi / 4),
        (0xb52, agility.wrapping_add(intellect) / 6),
        (0xb54, psi.wrapping_add(perception) / 6),
        (0xb56, psi / 4),
        (0xb58, accuracy.wrapping_add(agility) / 3),
        (0xb5a, accuracy.wrapping_add(strength) / 3),
        (0xb5c, accuracy.wrapping_add(speed) / 7),
        (0xb5e, accuracy.wrapping_add(speed) / 5),
        (0xb60, agility.wrapping_add(perception) / 7),
        (0xb46, psi / 2),
        (0xb7a, strength / 10),
    ] {
        put_word(bytes, offset, value);
    }
    if word(bytes, 0x36)? == 0 {
        put_word(bytes, 0xb40, 0);
        put_word(bytes, 0xb44, 0);
    } else {
        put_word(
            bytes,
            0xb44,
            (agility.wrapping_add(intellect) / 6).wrapping_add(perception / 3),
        );
    }
    // Disabled skills keep the prior record value; they are not reset to zero.
    for (skill, destination, value) in [
        (
            0xf4,
            0xb62,
            accuracy.wrapping_add(intellect).wrapping_add(strength) / 6,
        ),
        (0xf6, 0xb64, accuracy.wrapping_add(intellect) / 6),
        (0xf8, 0xb66, accuracy.wrapping_add(strength) / 15),
        (0xfa, 0xb68, agility.wrapping_add(intellect) / 10),
        (0xfc, 0xb6a, accuracy.wrapping_add(strength) / 10),
        (0xfe, 0xb6c, agility.wrapping_add(intellect) / 12),
        (0x100, 0xb6e, agility.wrapping_add(intellect) / 15),
        (0x102, 0xb70, strength / 3),
        (0x104, 0xb72, accuracy.wrapping_add(intellect) / 6),
    ] {
        if word(bytes, skill)? != 0 {
            put_word(bytes, destination, value);
        }
    }
    Ok(())
}
