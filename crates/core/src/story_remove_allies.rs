use crate::RuleError;
use crate::story_allies::{ALLY_BYTES, ALLY_CAPACITY};
use centurions_content::records::{EnemyRecord, EnemyRecordMut};
use centurions_content::span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllyRemoval {
    pub records: Vec<u8>,
    pub count: usize,
}

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
        if i16::from(EnemyRecord::from_bytes(&records[index * ALLY_BYTES..]).definition_id())
            == class
        {
            count -= 1;
            // The original repeatedly copies only the following record below slot five.
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
        EnemyRecordMut::from_bytes(record).set_target_body(0);
        EnemyRecordMut::from_bytes(record).clear_damage_marker();
    }
    Ok(AllyRemoval { records, count })
}
