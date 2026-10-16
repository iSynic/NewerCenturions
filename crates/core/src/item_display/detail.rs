use super::*;
use crate::{
    recovered_detail_ui::{DetailPlan, DetailReply, DetailRequest, DetailServices, quick_info},
    recovered_files::{FileReply, FileRequest},
    recovered_records::{ITEM_TABLE_ADDRESSES, RecordWrite},
};
use centurions_content::{mission::OpeningMission, span};

/// Original quickinfo with a bounded, read-only description stream and finite
/// SANE arithmetic. This adapts Toolbox services; it does not execute Apple's ROM.
pub fn detail(
    content: &Content,
    mission: &OpeningMission,
    item: &InventoryRecord,
    stock: bool,
    native: &NativeText,
    strings: &BTreeMap<i16, Vec<u8>>,
) -> Result<DetailPlan, RuleError> {
    const FRAME: u32 = 0x700200;
    let mut memory = RenderingMemory::default();
    for (address, length) in [
        (0x4358ce, 4),
        (0x438788, 24),
        (0x4389a4, 256),
        (0x439598, 4),
        (0x44260c, 256),
        (0x44274c, 8),
        (0x4434a6, 32),
        (0x4435c6, 16),
        (0x44374e, 58),
        (0x4471e8, 32),
        (0x45363c, 300),
        (0x479ed8, 512),
        (0x700000, 1024),
    ] {
        memory.insert_region(address, &vec![0; length]);
    }
    memory.insert_region(0x402900, native.bytes(0x402900, 0x640)?);
    memory.insert_region(FRAME + 8, &(if stock { -1_i16 } else { 0 }).to_be_bytes());
    memory.insert_region(FRAME + 10, &0_i16.to_be_bytes());
    memory.insert_region(FRAME + 12, &item.id().to_be_bytes());
    memory.insert_region(FRAME + 14, &0_i16.to_be_bytes());
    memory.insert_region(if stock { 0x41fc74 } else { 0x448bd2 }, &item.0);
    for id in [0, item.id(), word(&item.0, 4)?] {
        let index = usize::from(id.unsigned_abs());
        let definition = content.items.get(index).ok_or("detail item unavailable")?;
        memory.insert_region(
            ITEM_TABLE_ADDRESSES[index / 200] + (index % 200 * 150) as u32,
            definition,
        );
    }
    let definition = content
        .items
        .get(item.id().unsigned_abs() as usize)
        .ok_or("detail item unavailable")?;
    let spell = word(definition, 44)?;
    if (1101..4000).contains(&spell) {
        let attacks = &mission
            .combat
            .as_ref()
            .ok_or("detail spell table unavailable")?
            .attacks;
        let packet = crate::combat::spells::load_packet(attacks, spell)?;
        let offset = (spell / 1000 - 1) as usize * 2842
            + (spell % 1000 / 100 - 1) as usize * 406
            + (spell % 100 - 1) as usize * 58;
        memory.insert_region(0x443788 + offset as u32, &packet);
    }
    let id = usize::from(item.id().unsigned_abs());
    let description = if id < 800 {
        span(&mission.stock_item_descriptions, id * 256, 256)?
    } else {
        span(&mission.item_descriptions, (id - 800) * 256, 256)?
    };
    quick_info(
        &memory,
        FRAME,
        0,
        256,
        &mut Services {
            strings,
            description,
        },
    )
}

struct Services<'a> {
    strings: &'a BTreeMap<i16, Vec<u8>>,
    description: &'a [u8],
}

impl NumericServices for Services<'_> {
    fn number_to_string(&mut self, value: i32) -> Result<PascalString, RuleError> {
        PascalString::from_payload(value.to_string().as_bytes())
    }
    fn string_to_number(&mut self, _: &PascalString) -> Result<i32, RuleError> {
        unavailable()
    }
}
impl DrawingServices for Services<'_> {
    fn indexed_string(&mut self, resource: i16, index: i16) -> Result<PascalString, RuleError> {
        super::Services {
            strings: self.strings,
        }
        .indexed_string(resource, index)
    }
    fn color_icon(&mut self, id: i16) -> Result<u32, RuleError> {
        Ok(u32::from(id as u16))
    }
    fn picture(&mut self, id: i16) -> Result<PictureObservation, RuleError> {
        Ok(PictureObservation {
            handle: u32::from(id as u16),
            bounds: None,
        })
    }
    fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
        unavailable()
    }
    fn new_world(&mut self, _: i16, _: BodyRect) -> Result<(i16, BuildingWorld), RuleError> {
        unavailable()
    }
}
impl DetailServices for Services<'_> {
    fn observe(
        &mut self,
        request: &DetailRequest,
        memory: &RenderingMemory,
    ) -> Result<DetailReply, RuleError> {
        let write = |address, bytes| DetailReply::Writes(vec![RecordWrite { address, bytes }]);
        Ok(match request {
            DetailRequest::GetPort { destination } => {
                write(*destination, 0_u32.to_be_bytes().to_vec())
            }
            DetailRequest::GetForeColor { destination }
            | DetailRequest::GetBackColor { destination } => write(*destination, vec![0; 6]),
            DetailRequest::Sane {
                opcode,
                source,
                destination,
            } => write(
                *destination,
                arithmetic(memory, *opcode, *source, *destination)?,
            ),
            DetailRequest::File(FileRequest::Open { .. }) => {
                DetailReply::File(FileReply::Handle(1))
            }
            DetailRequest::File(
                FileRequest::Seek {
                    handle: 1,
                    origin: 0,
                    ..
                }
                | FileRequest::Close(1),
            ) => DetailReply::File(FileReply::Ignored),
            DetailRequest::ReadFile {
                handle: 1,
                size: 256,
                count: 1,
                ..
            } => DetailReply::File(FileReply::Read {
                bytes_written: self.description.to_vec(),
                elements_read: 1,
            }),
            _ => return unavailable(),
        })
    }
}

fn extended(memory: &RenderingMemory, pointer: u32) -> Result<f64, RuleError> {
    let exponent = memory.word(pointer)?;
    let significand =
        (u64::from(memory.long(pointer + 2)?) << 32) | u64::from(memory.long(pointer + 6)?);
    let value = if exponent & 0x7fff == 0 && significand == 0 {
        0.
    } else {
        let fraction = (significand >> 1) as f64 * 2. + (significand & 1) as f64;
        fraction
            * 2_f64.powi(i32::from(exponent & 0x7fff) - 16383 - 63)
            * if exponent & 0x8000 == 0 { 1. } else { -1. }
    };
    if !value.is_finite() {
        return Err("nonfinite item display arithmetic".into());
    }
    Ok(value)
}
fn encode_extended(value: f64) -> Result<Vec<u8>, RuleError> {
    if !value.is_finite() {
        return Err("nonfinite item display arithmetic".into());
    }
    if value == 0. {
        return Ok(vec![0; 10]);
    }
    let bits = value.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32 - 1023;
    if exponent == -1023 {
        return Err("subnormal item display arithmetic".into());
    }
    let mut bytes = ((if value < 0. { 0x8000 } else { 0 }) | (exponent + 16383) as u16)
        .to_be_bytes()
        .to_vec();
    bytes.extend((((bits & 0x000f_ffff_ffff_ffff) | 0x0010_0000_0000_0000) << 11).to_be_bytes());
    Ok(bytes)
}
pub(crate) fn arithmetic(
    memory: &RenderingMemory,
    opcode: u16,
    source: Option<u32>,
    destination: u32,
) -> Result<Vec<u8>, RuleError> {
    let operation = opcode & 0xff;
    let format = opcode & 0xff00;
    if operation == 0x1c && format == 0x1000 {
        let source = source.ok_or("arithmetic classification operand missing")?;
        let value = f32::from_bits(memory.long(source)?);
        if !value.is_finite() {
            return Err("nonfinite arithmetic classification".into());
        }
        // Apple Numerics Manual, MC68000 Table20-2: signed classes4/5/6.
        let class: i16 = if value == 0. {
            4
        } else if value.is_normal() {
            5
        } else {
            6
        };
        return Ok((if value.is_sign_negative() {
            -class
        } else {
            class
        })
        .to_be_bytes()
        .to_vec());
    }
    if operation == 0x16 {
        return encode_extended(extended(memory, destination)?.trunc());
    }
    let source = source.ok_or("item display arithmetic operand missing")?;
    let value = if operation == 0x10 {
        extended(memory, source)?
    } else {
        match format {
            0 => extended(memory, source)?,
            0x1000 => f64::from(f32::from_bits(memory.long(source)?)),
            0x2000 => f64::from(memory.word(source)? as i16),
            0x2800 => f64::from(memory.long(source)? as i32),
            _ => return unavailable(),
        }
    };
    if operation == 0x10 {
        return match format {
            0x1000 => Ok((value as f32).to_be_bytes().to_vec()),
            0x2800 if (i32::MIN as f64..=i32::MAX as f64).contains(&value.round_ties_even()) => {
                Ok((value.round_ties_even() as i32).to_be_bytes().to_vec())
            }
            _ => unavailable(),
        };
    }
    if operation == 0x0e {
        return encode_extended(value);
    }
    let current = extended(memory, destination)?;
    encode_extended(match operation {
        0 => current + value,
        4 => current * value,
        6 => current / value,
        _ => return unavailable(),
    })
}
