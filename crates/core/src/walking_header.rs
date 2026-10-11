use crate::{RandomState, RuleError};
use centurions_content::mission::long;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WalkingHeader {
    pub slot: usize,
    pub record: Vec<u8>,
}

pub fn lookup(
    headers: &[u8],
    map: i32,
    x: i32,
    y: i32,
    random: &mut RandomState,
) -> Result<Option<WalkingHeader>, RuleError> {
    if headers.len() != 4200 {
        return Err(RuleError::Invalid(
            "walking header table must contain 100 records".into(),
        ));
    }
    let key = y
        .wrapping_mul(100)
        .wrapping_add(x)
        .wrapping_add(map.wrapping_mul(10_000));
    for (slot, record) in headers.as_chunks::<42>().0.iter().enumerate() {
        if long(record, 0)? != key {
            continue;
        }
        let chance = i16::from(record[7] as i8);
        if chance <= 0 || random.bound(100) > chance {
            return Ok(None);
        }
        return Ok(Some(WalkingHeader {
            slot,
            record: record.to_vec(),
        }));
    }
    Ok(None)
}
