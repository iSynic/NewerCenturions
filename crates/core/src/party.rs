use crate::RuleError;
use centurions_content::{WARRIOR_BYTES, Warrior, span};

/// Pools the party's credits: each member's unsigned carried credits are added to
/// a wrapping 32-bit pool.
///
/// Original: pool
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
        *pool = pool.wrapping_add(i32::from(warrior.record().carried_credits()? as u16));
        warrior.record_mut().set_carried_credits(0);
    }
    Ok(())
}

/// Shares the pool equally; the remainder is lost.
///
/// Original: share
pub fn share_credits(warriors: &mut [Warrior], pool: &mut i32) -> Result<i16, RuleError> {
    let count = i32::try_from(warriors.len())
        .ok()
        .filter(|count| *count > 0 && *count <= 128)
        .ok_or_else(|| RuleError::Invalid("credit sharing requires 1..128 warriors".into()))?;
    pool_credits(warriors, pool)?;
    let each = (*pool / count) as i16;
    *pool = 0;
    for warrior in warriors {
        warrior.record_mut().set_carried_credits(each);
    }
    Ok(each)
}

/// Counts the charges of every matching pack, in record order.
///
/// Original: checkforcharge
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
                total = total.wrapping_add(item.charge()?);
            }
        }
    }
    Ok(total)
}
