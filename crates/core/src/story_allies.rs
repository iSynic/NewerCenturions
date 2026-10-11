use crate::{RandomState, RuleError};
use centurions_content::{span, word};

pub const ALLY_BYTES: usize = 288;
pub const ALLY_CAPACITY: usize = 20;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllyAddition {
    pub records: Vec<u8>,
    pub count: usize,
    pub warning: Option<i16>,
}

pub fn add_ally(
    records: &[u8],
    count: usize,
    definition: &[u8],
    difficulty: i16,
    random: &mut RandomState,
) -> Result<AllyAddition, RuleError> {
    let mut records = span(records, 0, ALLY_BYTES * ALLY_CAPACITY)?.to_vec();
    if count >= ALLY_CAPACITY {
        normalize_menu(&mut records, count);
        return Ok(AllyAddition {
            records,
            count,
            warning: Some(108),
        });
    }
    let definition = span(definition, 0, ALLY_BYTES)?;
    if word(definition, 0x4e)? < 0 {
        return Err(RuleError::Unsupported {
            action: "story.add_ally",
            dependency: "random NPC weapon initialization",
        });
    }
    let record = &mut records[count * ALLY_BYTES..(count + 1) * ALLY_BYTES];
    record.copy_from_slice(definition);
    let mut stamina = word(record, 2)?;
    for _ in 0..word(record, 0)?.max(0) {
        stamina = stamina.wrapping_add(random.bound(8));
    }
    for offset in [0x110, 0x112] {
        record[offset..offset + 2].copy_from_slice(&stamina.to_be_bytes());
    }
    record[0xe..0x10].fill(0);
    record[0x107] = 0;
    record[0x108] = 0;
    let speed = word(record, 0x10e)?;
    record[0x52..0x54].copy_from_slice(&speed.to_be_bytes());
    for (offset, addition) in [
        (0xa, difficulty.wrapping_mul(3)),
        (8, difficulty.wrapping_mul(-2)),
        (4, difficulty),
    ] {
        let value = word(record, offset)?.wrapping_add(addition);
        record[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }
    for byte in &mut record[0x8b..0x91] {
        *byte = byte.wrapping_add(difficulty.wrapping_mul(7) as u8);
    }
    normalize_menu(&mut records, count + 1);
    Ok(AllyAddition {
        records,
        count: count + 1,
        warning: None,
    })
}

pub fn clear_stage_allies(records: &[u8]) -> Result<Vec<u8>, RuleError> {
    let mut records = span(records, 0, ALLY_BYTES * ALLY_CAPACITY)?.to_vec();
    for index in 0..ALLY_CAPACITY {
        records[index * ALLY_BYTES + 0x103] = 0;
    }
    Ok(records)
}

fn normalize_menu(records: &mut [u8], count: usize) {
    for record in records
        .as_chunks_mut::<ALLY_BYTES>()
        .0
        .iter_mut()
        .take(count)
    {
        record[0x10c..0x10e].fill(0);
        record[0x105] = 0;
    }
}
