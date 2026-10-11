use crate::RuleError;
use centurions_content::span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllyRemoval {
    pub records: Vec<u8>,
    pub count: usize,
}

pub fn remove_class(records: &[u8], count: usize, class: i16) -> Result<AllyRemoval, RuleError> {
    if count > 20 {
        return Err(RuleError::Unsupported {
            action: "story.remove_allies",
            dependency: "ally count exceeds twenty records",
        });
    }
    let mut records = span(records, 0, 20 * 288)?.to_vec();
    let mut count = count;
    let mut index = 0;
    while index < count {
        if i16::from(records[index * 288 + 0x103] as i8) == class {
            count -= 1;
            // The original repeatedly copies only the following record below slot five.
            for _ in index..5 {
                records.copy_within((index + 1) * 288..(index + 2) * 288, index * 288);
            }
        }
        index += 1;
    }
    for record in records.as_chunks_mut::<288>().0.iter_mut().take(count) {
        record[0x10c..0x10e].fill(0);
        record[0x105] = 0;
    }
    Ok(AllyRemoval { records, count })
}
