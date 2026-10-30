use crate::RuleError;
use crate::story::allies::{ALLY_BYTES, ALLY_CAPACITY};
use centurions_content::records::{EnemyRecord, EnemyRecordMut};
use centurions_content::span;

/// The ally table after removal: its records and how many allies remain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllyRemoval {
    pub records: Vec<u8>,
    pub count: usize,
}

/// Removes each ally whose monster number is `class`, then clears the remaining allies' targets.
/// Only the record after the removed one is copied down, and only below slot five.
pub fn remove_class(records: &[u8], count: usize, class: i16) -> Result<AllyRemoval, RuleError> {
    if count > ALLY_CAPACITY {
        return Err(RuleError::Unsupported {
            action: "story.remove_allies",
            dependency: "ally count exceeds twenty records",
        });
    }
    let mut records = span(records, 0, ALLY_CAPACITY * ALLY_BYTES)?.to_vec();
    let mut count = count;
    let mut index = 0;
    while index < count {
        if i16::from(EnemyRecord::from_bytes(&records[index * ALLY_BYTES..]).monster_id()) == class
        {
            count -= 1;
            // Only the next record is copied down, repeatedly, below slot five: an
            // original quirk kept as is.
            for _ in index..5 {
                records.copy_within(
                    (index + 1) * ALLY_BYTES..(index + 2) * ALLY_BYTES,
                    index * ALLY_BYTES,
                );
            }
        }
        index += 1;
    }
    for record in records
        .as_chunks_mut::<ALLY_BYTES>()
        .0
        .iter_mut()
        .take(count)
    {
        EnemyRecordMut::from_bytes(record).set_target(0);
        EnemyRecordMut::from_bytes(record).clear_been_attacked();
    }
    Ok(AllyRemoval { records, count })
}
