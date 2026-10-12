use crate::RuleError;
use centurions_content::ITEM_BYTES;

pub const SPELL_BYTES: usize = 58;
pub const ITEM_TABLE_ADDRESSES: [u32; 5] = [0x4537fe, 0x46225e, 0x45ad2e, 0x46978e, 0x470cbe];

/// Tables are in native ID-family order: 0, 200, 400, 600, 800.
pub struct ItemTables<'a> {
    pub families: [&'a [u8]; 5],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordWrite {
    pub address: u32,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordLoad<const N: usize> {
    pub bytes: [u8; N],
    pub source_address: Option<u32>,
    pub writes: Vec<RecordWrite>,
    pub native_return: i32,
}

pub fn loaditem(
    tables: &ItemTables<'_>,
    current: &[u8; ITEM_BYTES],
    id: i16,
) -> Result<RecordLoad<ITEM_BYTES>, RuleError> {
    load_item_record(tables, current, id, 0x45363c, true)
}

pub fn loaditemcompare(
    tables: &ItemTables<'_>,
    current: &[u8; ITEM_BYTES],
    id: i16,
) -> Result<RecordLoad<ITEM_BYTES>, RuleError> {
    load_item_record(tables, current, id, 0x4781ee, true)
}

pub fn loadweapon(
    tables: &ItemTables<'_>,
    current: &[u8; ITEM_BYTES],
    id: i16,
) -> Result<RecordLoad<ITEM_BYTES>, RuleError> {
    load_item_record(tables, current, id, 0x453768, true)
}

pub fn loadammo(
    tables: &ItemTables<'_>,
    current: &[u8; ITEM_BYTES],
    id: i16,
) -> Result<RecordLoad<ITEM_BYTES>, RuleError> {
    load_item_record(tables, current, id, 0x4536d2, true)
}

pub fn loaditemnotype(
    tables: &ItemTables<'_>,
    current: &[u8; ITEM_BYTES],
    id: i16,
) -> Result<RecordLoad<ITEM_BYTES>, RuleError> {
    load_item_record(tables, current, id, 0x45363c, false)
}

fn load_item_record(
    tables: &ItemTables<'_>,
    current: &[u8; ITEM_BYTES],
    id: i16,
    destination: u32,
    normalize_type: bool,
) -> Result<RecordLoad<ITEM_BYTES>, RuleError> {
    let id = id.wrapping_abs();
    let family = id / 200;
    let mut result = RecordLoad {
        bytes: *current,
        source_address: None,
        writes: Vec::new(),
        native_return: 0,
    };
    // The switch has no default copy. Its common type-normalization suffix still runs.
    if let Ok(family) = usize::try_from(family)
        && let Some(table) = tables.families.get(family)
    {
        let offset = (id % 200) as usize * ITEM_BYTES;
        result = copy_record(table, offset, ITEM_TABLE_ADDRESSES[family], destination)?;
    }
    if normalize_type {
        let value = i16::from_be_bytes([result.bytes[26], result.bytes[27]]);
        let bytes = value.wrapping_abs().to_be_bytes();
        result.bytes[26..28].copy_from_slice(&bytes);
        result.writes.push(RecordWrite {
            address: destination + 26,
            bytes: bytes.to_vec(),
        });
    }
    Ok(result)
}

/// CODE 1 0x1314fc: signed BYTE arguments, sign extension, then unsigned WORD products.
pub fn loadspell(
    table: &[u8],
    school: i8,
    level: i8,
    index: i8,
) -> Result<RecordLoad<SPELL_BYTES>, RuleError> {
    let offset = spell_offset([i16::from(school), i16::from(level), i16::from(index)]);
    copy_record(table, offset, 0x443788, 0x44374e)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpellLoad {
    pub coordinates: [i16; 3],
    pub record: RecordLoad<SPELL_BYTES>,
}

/// CODE 1 0x13145c reads a WORD descriptor, despite the decompiler's int signature.
pub fn loadspell2(table: &[u8], descriptor: i16) -> Result<SpellLoad, RuleError> {
    let school = descriptor / 1000;
    let remainder = descriptor.wrapping_sub(school.wrapping_mul(1000));
    let level = remainder / 100;
    let index = remainder.wrapping_sub(level.wrapping_mul(100));
    let coordinates = [
        school.wrapping_sub(1),
        level.wrapping_sub(1),
        index.wrapping_sub(1),
    ];
    let mut record = copy_record(table, spell_offset(coordinates), 0x443788, 0x44374e)?;
    let mut writes = Vec::with_capacity(record.writes.len() + 3);
    for (index, value) in coordinates.iter().enumerate() {
        writes.push(RecordWrite {
            address: 0x4435c6 + index as u32 * 2,
            bytes: value.to_be_bytes().to_vec(),
        });
    }
    // The native quotient WORD is stored before being decremented in place.
    writes.insert(
        0,
        RecordWrite {
            address: 0x4435c6,
            bytes: school.to_be_bytes().to_vec(),
        },
    );
    writes.insert(
        2,
        RecordWrite {
            address: 0x4435c8,
            bytes: level.to_be_bytes().to_vec(),
        },
    );
    writes.append(&mut record.writes);
    record.writes = writes;
    Ok(SpellLoad {
        coordinates,
        record,
    })
}

fn spell_offset([school, level, index]: [i16; 3]) -> usize {
    usize::from(school as u16) * 2842
        + usize::from(level as u16) * 406
        + usize::from(index as u16) * SPELL_BYTES
}

fn copy_record<const N: usize>(
    table: &[u8],
    offset: usize,
    base: u32,
    destination: u32,
) -> Result<RecordLoad<N>, RuleError> {
    let source = offset
        .checked_add(N)
        .and_then(|end| table.get(offset..end))
        .ok_or_else(|| unavailable("record.load", "source record outside supplied table"))?;
    let bytes: [u8; N] = source.try_into().unwrap();
    let mut writes = Vec::with_capacity(N / 4 + 1);
    for (index, chunk) in source.as_chunks::<4>().0.iter().enumerate() {
        writes.push(RecordWrite {
            address: destination + index as u32 * 4,
            bytes: chunk.to_vec(),
        });
    }
    if !N.is_multiple_of(4) {
        writes.push(RecordWrite {
            address: destination + (N / 4 * 4) as u32,
            bytes: source[N / 4 * 4..].to_vec(),
        });
    }
    Ok(RecordLoad {
        bytes,
        source_address: Some(base + offset as u32),
        writes,
        native_return: 0,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetWork {
    pub count: i16,
    pub targets: [[i16; 3]; 7],
    pub mask: [i16; 110],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClearedTargets {
    pub state: TargetWork,
    pub writes: Vec<RecordWrite>,
}

/// CODE 1 0x13822a has no resource service calls or random draws.
pub fn cleartarget() -> ClearedTargets {
    let mut writes = Vec::with_capacity(132);
    writes.push(RecordWrite {
        address: 0x44721a,
        bytes: 0_i16.to_be_bytes().to_vec(),
    });
    for target in 0..7 {
        for field in [4, 2, 0] {
            writes.push(RecordWrite {
                address: 0x443722 + target * 6 + field,
                bytes: (-1_i16).to_be_bytes().to_vec(),
            });
        }
    }
    for index in 0..110 {
        writes.push(RecordWrite {
            address: 0x4434e0 + index * 2,
            bytes: 0_i16.to_be_bytes().to_vec(),
        });
    }
    ClearedTargets {
        state: TargetWork {
            count: 0,
            targets: [[-1; 3]; 7],
            mask: [0; 110],
        },
        writes,
    }
}

fn unavailable(action: &'static str, dependency: &'static str) -> RuleError {
    RuleError::Unsupported { action, dependency }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_families_copy_all_bytes_before_normalizing_only_the_destination_type() {
        let families: Vec<Vec<u8>> = (0..5)
            .map(|family| {
                let mut bytes = vec![family + 10; 200 * ITEM_BYTES];
                let offset = 199 * ITEM_BYTES;
                bytes[offset + 26..offset + 28].copy_from_slice(&(-123_i16).to_be_bytes());
                bytes
            })
            .collect();
        let tables = ItemTables {
            families: std::array::from_fn(|i| families[i].as_slice()),
        };
        let current = [0x5a; ITEM_BYTES];
        for family in 0..5 {
            let id = (family * 200 + 199) as i16;
            let before = families[family].clone();
            let raw = loaditemnotype(&tables, &current, -id).unwrap();
            assert_eq!(raw.bytes.as_slice(), &before[199 * ITEM_BYTES..]);
            assert_eq!(raw.writes.len(), 38);
            for result in [
                loaditem(&tables, &current, id).unwrap(),
                loaditemcompare(&tables, &current, -id).unwrap(),
                loadweapon(&tables, &current, id).unwrap(),
                loadammo(&tables, &current, id).unwrap(),
            ] {
                let mut expected = raw.bytes;
                expected[26..28].copy_from_slice(&123_i16.to_be_bytes());
                assert_eq!(result.bytes, expected);
                assert_eq!(
                    result.source_address,
                    Some(ITEM_TABLE_ADDRESSES[family] + 199 * 150)
                );
                assert_eq!(result.writes.len(), 39);
                assert_eq!(result.writes[37].bytes.len(), 2);
                assert_eq!(result.writes[38].address, result.writes[0].address + 26);
                assert_eq!(result.native_return, 0);
            }
            assert_eq!(families[family], before);
        }
    }

    #[test]
    fn missing_item_switch_branch_retains_work_and_bad_reads_are_atomic() {
        let tables = ItemTables { families: [&[]; 5] };
        let mut current = [0xa5; ITEM_BYTES];
        current[26..28].copy_from_slice(&(-7_i16).to_be_bytes());
        let untouched = loaditemnotype(&tables, &current, 1000).unwrap();
        assert_eq!(untouched.bytes, current);
        assert!(untouched.writes.is_empty());
        assert_eq!(untouched.source_address, None);
        let normalized = loaditem(&tables, &current, -32767).unwrap();
        let mut expected = current;
        expected[26..28].copy_from_slice(&7_i16.to_be_bytes());
        assert_eq!(normalized.bytes, expected);
        assert_eq!(normalized.writes.len(), 1);
        assert!(loaditem(&tables, &current, 0).is_err());
        assert_eq!(
            loaditem(&tables, &current, i16::MIN).unwrap().bytes,
            expected
        );
        current[26..28].copy_from_slice(&i16::MIN.to_be_bytes());
        let minimum_type = loaditem(&tables, &current, 1000).unwrap();
        assert_eq!(minimum_type.bytes, current);
        assert_eq!(minimum_type.writes[0].bytes, vec![128, 0]);
    }

    #[test]
    fn spell_byte_and_word_inputs_preserve_native_flat_table_addressing_and_write_order() {
        let table: Vec<u8> = (0..11368).map(|i| (i * 17 % 251) as u8).collect();
        let raw = loadspell(&table, 1, 1, 5).unwrap();
        assert_eq!(raw.bytes.as_slice(), &table[3538..3596]);
        assert_eq!(raw.source_address, Some(0x443788 + 3538));
        assert_eq!(raw.writes.len(), 15);
        assert_eq!(raw.writes[14].address, 0x443786);
        let encoded = loadspell2(&table, 2206).unwrap();
        assert_eq!(encoded.coordinates, [1, 1, 5]);
        assert_eq!(encoded.record.bytes, raw.bytes);
        let stores: Vec<(u32, i16)> = encoded.record.writes[..5]
            .iter()
            .map(|w| {
                (
                    w.address,
                    i16::from_be_bytes(w.bytes.as_slice().try_into().unwrap()),
                )
            })
            .collect();
        assert_eq!(
            stores,
            vec![
                (0x4435c6, 2),
                (0x4435c6, 1),
                (0x4435c8, 2),
                (0x4435c8, 1),
                (0x4435ca, 5)
            ]
        );
        // No seven-column range guard exists: index 7 aliases the next level's first record.
        assert_eq!(
            loadspell(&table, 0, 0, 7).unwrap().bytes,
            loadspell(&table, 0, 1, 0).unwrap().bytes
        );
        assert_eq!(
            loadspell2(&table, 1108).unwrap().record.bytes,
            loadspell(&table, 0, 0, 7).unwrap().bytes
        );
        for coordinates in [[-1, 0, 0], [0, -1, 0], [0, 0, -1]] {
            assert!(loadspell(&table, coordinates[0], coordinates[1], coordinates[2]).is_err());
        }
        for descriptor in [0, -1, i16::MIN, 1100, 1001] {
            assert!(loadspell2(&table, descriptor).is_err());
        }
        assert!(loadspell(&table[..3595], 1, 1, 5).is_err());
    }

    #[test]
    fn target_clear_has_the_native_132_word_stores_in_order() {
        let result = cleartarget();
        assert_eq!(
            result.state,
            TargetWork {
                count: 0,
                targets: [[-1; 3]; 7],
                mask: [0; 110]
            }
        );
        assert_eq!(result.writes.len(), 132);
        assert_eq!(
            result.writes[0],
            RecordWrite {
                address: 0x44721a,
                bytes: vec![0, 0]
            }
        );
        for target in 0..7 {
            for (ordinal, field) in [4, 2, 0].into_iter().enumerate() {
                assert_eq!(
                    result.writes[1 + target * 3 + ordinal],
                    RecordWrite {
                        address: 0x443722 + target as u32 * 6 + field,
                        bytes: vec![255, 255]
                    }
                );
            }
        }
        for (index, write) in result.writes[22..].iter().enumerate() {
            assert_eq!(write.address, 0x4434e0 + index as u32 * 2);
            assert_eq!(write.bytes, vec![0, 0]);
        }
    }
}
