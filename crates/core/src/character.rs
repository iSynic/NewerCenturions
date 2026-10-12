use crate::{RandomState, RuleError};
use centurions_content::{WARRIOR_BYTES, Warrior, span, word};

fn put_word(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
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

#[cfg(test)]
mod tests {
    use super::*;

    fn warrior(attributes: [i16; 7]) -> Warrior {
        let mut bytes = vec![0; WARRIOR_BYTES];
        for (index, attribute) in attributes.into_iter().enumerate() {
            put_word(&mut bytes, 0x9e2 + index * 2, attribute);
        }
        Warrior {
            name: "Synthetic".into(),
            original_sha256: String::new(),
            bytes,
        }
    }

    #[test]
    fn derived_statistics_preserve_disabled_skills_and_other_record_bytes() {
        let mut value = warrior([30, 36, 42, 48, 54, 60, 66]);
        put_word(&mut value.bytes, 0x36, 1);
        put_word(&mut value.bytes, 0xf4, 1);
        put_word(&mut value.bytes, 0xb64, 99);
        value.bytes[0xa21..0xa25].copy_from_slice(b"Test");
        let original = value.bytes.clone();
        calculate_statistics(&mut value).unwrap();
        for (offset, expected) in [
            (0xb44, 35),
            (0xb48, 21),
            (0xb52, 13),
            (0xb5c, 14),
            (0xb62, 20),
            (0xb64, 99),
            (0xb7a, 3),
        ] {
            assert_eq!(word(&value.bytes, offset).unwrap(), expected);
        }
        assert_eq!(value.bytes[..0xb40], original[..0xb40]);
        assert_eq!(value.bytes[0xb7c..], original[0xb7c..]);
    }

    #[test]
    fn signed_attributes_wrap_before_division_and_school_zero_clears_only_psi_fields() {
        let mut value = warrior([30_000, 20_000, 32_000, -7, -6, -7, 32_000]);
        put_word(&mut value.bytes, 0xb40, 99);
        put_word(&mut value.bytes, 0xb44, 99);
        calculate_statistics(&mut value).unwrap();
        assert_eq!(word(&value.bytes, 0xb48).unwrap(), -307);
        assert_eq!(word(&value.bytes, 0xb4a).unwrap(), -1);
        assert_eq!(word(&value.bytes, 0xb46).unwrap(), -3);
        assert_eq!(word(&value.bytes, 0xb40).unwrap(), 0);
        assert_eq!(word(&value.bytes, 0xb44).unwrap(), 0);
    }

    #[test]
    fn truncated_records_fail_before_any_writes() {
        let mut value = warrior([1; 7]);
        value.bytes.truncate(WARRIOR_BYTES - 1);
        let before = value.bytes.clone();
        assert!(calculate_statistics(&mut value).is_err());
        assert_eq!(value.bytes, before);
    }

    #[test]
    fn enabled_skill_destinations_keep_word_wrapping_before_signed_division() {
        let mut value = warrior([20_000; 7]);
        for offset in (0xf4..=0x104).step_by(2) {
            put_word(&mut value.bytes, offset, 1);
        }
        calculate_statistics(&mut value).unwrap();
        for (offset, expected) in [
            (0xb62, -922),
            (0xb64, -4256),
            (0xb66, -1702),
            (0xb68, -2553),
            (0xb6a, -2553),
            (0xb6c, -2128),
            (0xb6e, -1702),
            (0xb70, 6666),
            (0xb72, -4256),
        ] {
            assert_eq!(word(&value.bytes, offset).unwrap(), expected);
        }
    }

    #[test]
    fn adjustment_extremes_keep_the_native_exception_and_wrapped_average() {
        assert_eq!(strength_adjustments(29), [14, 5]);
        assert_eq!(strength_adjustments(30), [15, 4]);
        assert_eq!(strength_adjustments(i16::MIN), [-20, 0]);
        assert_eq!(strength_adjustments(i16::MAX), [20, 5]);
        let value = warrior([0, 0, 0, i16::MAX, i16::MAX, 0, 0]);
        assert_eq!(agility_adjustment(&value).unwrap(), 11);
    }

    #[test]
    fn attribute_save_consumes_one_draw_and_invalid_selectors_consume_none() {
        let value = warrior([25; 7]);
        let mut random = RandomState::new(12);
        let mut control = random.clone();
        let expected = control.bound(100) < 100;
        assert_eq!(
            save_against_attribute(&value, 0, &mut random).unwrap(),
            expected
        );
        assert_eq!(random, control);
        assert!(save_against_attribute(&value, 6, &mut random).is_err());
        assert_eq!(random, control);
        let wrapped = warrior([8192; 7]);
        assert!(!save_against_attribute(&wrapped, 0, &mut random).unwrap());
        assert_eq!(random.draws, control.draws + 1);
    }

    #[test]
    fn upgrade_word_offsets_keep_adjacent_fields_and_recalculate_maxima() {
        for (selector, offset) in [(16, 4), (17, 6), (18, 8), (19, 10), (22, 0x22)] {
            let mut value = warrior([0; 7]);
            value.bytes[..0x24].fill(0x55);
            put_word(&mut value.bytes, offset, i16::MAX);
            put_word(&mut value.bytes, 0xb42, 100);
            put_word(&mut value.bytes, 0xb46, 20);
            put_word(&mut value.bytes, 0xb40, -20);
            put_word(&mut value.bytes, 0xb44, 10);
            let before = value.bytes.clone();
            apply_upgrade(&mut value, selector, 1).unwrap();
            assert_eq!(word(&value.bytes, offset).unwrap(), i16::MIN);
            assert_eq!(&value.bytes[..offset], &before[..offset]);
            assert_eq!(&value.bytes[offset + 2..0x24], &before[offset + 2..0x24]);
            assert_eq!(word(&value.bytes, 0x52).unwrap(), 120);
            assert_eq!(word(&value.bytes, 0x5a).unwrap(), -10);
        }
    }

    #[test]
    fn upgrade_special_cases_preserve_fixed_increments_and_unknown_selector_suffix() {
        let mut value = warrior([i16::MAX; 7]);
        apply_upgrade(&mut value, 0, 1).unwrap();
        assert_eq!(word(&value.bytes, 0x9e2).unwrap(), i16::MIN);
        assert_eq!(word(&value.bytes, 0x9d8).unwrap(), 100);
        apply_upgrade(&mut value, 7, 3277).unwrap();
        assert_eq!(word(&value.bytes, 0x9d8).unwrap(), -32666);
        apply_upgrade(&mut value, 32, -100).unwrap();
        assert_eq!(word(&value.bytes, 0xc).unwrap(), 1);
        put_word(&mut value.bytes, 0x52, 100);
        let before = value.bytes.clone();
        apply_upgrade(&mut value, u16::MAX, 5).unwrap();
        assert_eq!(word(&value.bytes, 0x52).unwrap(), 0);
        assert_eq!(&value.bytes[..0x52], &before[..0x52]);
        assert_eq!(&value.bytes[0x54..], &before[0x54..]);
    }
}
