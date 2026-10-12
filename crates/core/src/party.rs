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
