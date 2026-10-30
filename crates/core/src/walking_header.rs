use crate::{RandomState, RuleError};
use centurions_content::mission::long;

/// A walking event the party has triggered: its slot in the header table and its record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WalkingHeader {
    pub slot: usize,
    pub record: Vec<u8>,
}

/// A walking event's matching header and its row, recorded before the chance
/// test, so a failed chance still reports the row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeaderLookup {
    pub matched: Option<usize>,
    pub header: Option<WalkingHeader>,
}

/// Looks for a walking event at a square. The first matching header's slot is reported even when
/// its chance fails. An event runs only with a chance above zero and a roll of 100 not above it.
pub fn lookup(
    headers: &[u8],
    map: i32,
    x: i32,
    y: i32,
    random: &mut RandomState,
) -> Result<HeaderLookup, RuleError> {
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
        let passed = chance > 0 && random.bound(100) <= chance;
        return Ok(HeaderLookup {
            matched: Some(slot),
            header: passed.then(|| WalkingHeader {
                slot,
                record: record.to_vec(),
            }),
        });
    }
    Ok(HeaderLookup {
        matched: None,
        header: None,
    })
}
