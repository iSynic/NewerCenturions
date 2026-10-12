use crate::RuleError;
use centurions_content::{WARRIOR_BYTES, Warrior, span, word};

/// CODE 1 pool, 0x1366a8: unsigned carried WORDs added to a wrapping LONG pool.
pub fn pool_credits(warriors: &mut [Warrior], pool: &mut i32) -> Result<(), RuleError> {
    if warriors.len() > 128 {
        return Err(RuleError::Invalid(
            "party exceeds signed BYTE index capacity".into(),
        ));
    }
    for warrior in warriors.iter() {
        span(&warrior.bytes, 0, WARRIOR_BYTES)?;
    }
    for warrior in warriors {
        *pool = pool.wrapping_add(i32::from(word(&warrior.bytes, 0x9da)? as u16));
        warrior.bytes[0x9da..0x9dc].fill(0);
    }
    Ok(())
}

/// CODE 1 share, 0x13684a: signed LONG division; remainder is discarded.
pub fn share_credits(warriors: &mut [Warrior], pool: &mut i32) -> Result<i16, RuleError> {
    let count = i32::try_from(warriors.len())
        .ok()
        .filter(|count| *count > 0 && *count <= 128)
        .ok_or_else(|| RuleError::Invalid("credit sharing requires 1..128 warriors".into()))?;
    pool_credits(warriors, pool)?;
    let each = (*pool / count) as i16;
    *pool = 0;
    for warrior in warriors {
        warrior.bytes[0x9da..0x9dc].copy_from_slice(&each.to_be_bytes());
    }
    Ok(each)
}

/// CODE 1 checkforcharge, 0x11279a: include every matching pack in record order.
pub fn item_charges(warriors: &[Warrior], item_id: i16) -> Result<i16, RuleError> {
    if warriors.len() > 128 {
        return Err(RuleError::Invalid(
            "party exceeds signed BYTE index capacity".into(),
        ));
    }
    let mut total = 0_i16;
    for warrior in warriors {
        for item in warrior.inventory()? {
            if item.id() == item_id {
                total = total.wrapping_add(word(&item.0, 2)?);
            }
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn warrior(credits: u16) -> Warrior {
        let mut bytes = vec![0; WARRIOR_BYTES];
        bytes[0x9da..0x9dc].copy_from_slice(&credits.to_be_bytes());
        Warrior {
            name: "Synthetic".into(),
            original_sha256: String::new(),
            bytes,
        }
    }

    #[test]
    fn pooling_reads_unsigned_words_and_sharing_discards_remainder() {
        let mut warriors = vec![warrior(65535), warrior(5), warrior(6)];
        let mut pool = 7;
        pool_credits(&mut warriors, &mut pool).unwrap();
        assert_eq!(pool, 65553);
        assert!(warriors.iter().all(|w| word(&w.bytes, 0x9da).unwrap() == 0));
        assert_eq!(share_credits(&mut warriors, &mut pool).unwrap(), 21851);
        assert_eq!(pool, 0);
        pool = 2;
        assert_eq!(share_credits(&mut warriors, &mut pool).unwrap(), 21851);
        assert_eq!(pool, 0);
    }

    #[test]
    fn pool_overflow_then_signed_division_retains_low_word() {
        let mut warriors = vec![warrior(1), warrior(0)];
        let mut pool = i32::MAX;
        assert_eq!(share_credits(&mut warriors, &mut pool).unwrap(), 0);
        assert_eq!(pool, 0);
        pool = -7;
        assert_eq!(share_credits(&mut warriors, &mut pool).unwrap(), -3);
    }

    #[test]
    fn malformed_record_and_empty_sharing_fail_before_mutation() {
        let mut warriors = vec![warrior(10), warrior(20)];
        warriors[1].bytes.pop();
        let before = warriors.clone();
        let mut pool = 30;
        assert!(pool_credits(&mut warriors, &mut pool).is_err());
        assert_eq!(warriors, before);
        assert_eq!(pool, 30);
        let mut oversized = vec![warrior(1); 129];
        let before = oversized.clone();
        assert!(pool_credits(&mut oversized, &mut pool).is_err());
        assert!(item_charges(&oversized, 1).is_err());
        assert_eq!(oversized, before);
        assert_eq!(pool, 30);
        assert!(share_credits(&mut [], &mut pool).is_err());
        assert_eq!(pool, 30);
    }

    #[test]
    fn charge_query_includes_duplicates_zero_and_negative_quantities_with_word_wrap() {
        let mut warriors = vec![warrior(0), warrior(0)];
        for (index, quantities) in [[32767_i16, 1, -2], [10, 0, 3]].iter().enumerate() {
            warriors[index].bytes[0x1a..0x1c].copy_from_slice(&3_i16.to_be_bytes());
            for (slot, quantity) in quantities.iter().enumerate() {
                let offset = 0x132 + slot * 22;
                let id = if slot == 2 { 99_i16 } else { 7 };
                warriors[index].bytes[offset..offset + 2].copy_from_slice(&id.to_be_bytes());
                warriors[index].bytes[offset + 2..offset + 4]
                    .copy_from_slice(&quantity.to_be_bytes());
            }
        }
        assert_eq!(item_charges(&warriors, 7).unwrap(), -32758);
        assert_eq!(item_charges(&warriors, 99).unwrap(), 1);
        assert_eq!(item_charges(&warriors, 1).unwrap(), 0);
    }
}
