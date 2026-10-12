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
