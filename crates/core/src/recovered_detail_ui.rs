//! Item detail and combat odds plans from the complete CODE 1 instruction bodies.

use crate::RuleError;
use crate::recovered_attack::{self, AttackEffect, AttackTables, AttackWork, DamageFrame};
use crate::recovered_drawing::{self, DrawingContext, DrawingEffect, DrawingServices};
use crate::recovered_economy::ArithmeticQualification;
use crate::recovered_files::{
    self, FileBuffer, FileContext, FileEffect, FileObservation, FileReply, FileRequest,
};
use crate::recovered_geometry::BodyRect;
use crate::recovered_records::{self, ITEM_TABLE_ADDRESSES, ItemTables, RecordWrite, SPELL_BYTES};
use crate::recovered_rendering::RenderingMemory;
use crate::recovered_text::{self, NumericServices, PascalString, TextContext, TextEffect};
use centurions_content::{ITEM_BYTES, WARRIOR_BYTES};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DetailRequest {
    GetPort {
        destination: u32,
    },
    GetForeColor {
        destination: u32,
    },
    GetBackColor {
        destination: u32,
    },
    InsetRect {
        pointer: u32,
        horizontal: i16,
        vertical: i16,
    },
    /// Pointers are in source push order: source first, destination second.
    Sane {
        opcode: u16,
        source: Option<u32>,
        destination: u32,
    },
    File(FileRequest),
    ReadFile {
        handle: u32,
        size: u32,
        count: u32,
        destination: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DetailReply {
    Writes(Vec<RecordWrite>),
    File(FileReply),
}

pub trait DetailServices: DrawingServices + NumericServices {
    /// Supply the external operation's actual outputs without executing the plan.
    fn observe(
        &mut self,
        request: &DetailRequest,
        memory: &RenderingMemory,
    ) -> Result<DetailReply, RuleError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DetailEffect {
    Write(RecordWrite),
    Drawing(DrawingEffect),
    Observation {
        request: DetailRequest,
        reply: DetailReply,
    },
    TextMode(i16),
    TextFaceSlot(u16),
    RgbForeColor {
        pointer: u32,
        bytes: [u8; 6],
    },
    RgbBackColor {
        pointer: u32,
        bytes: [u8; 6],
    },
    NumberToString {
        value: i32,
        destination: u32,
        result: PascalString,
    },
    IndexedString {
        resource_id: i16,
        index: i16,
        destination: u32,
        result: PascalString,
    },
    TextBox {
        pointer: u32,
        length: i32,
        bytes: Vec<u8>,
        rectangle_pointer: u32,
        rectangle: BodyRect,
        alignment: i16,
    },
    File(FileEffect),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetailPlan {
    pub memory: RenderingMemory,
    pub effects: Vec<DetailEffect>,
    /// Both native entries are void; D0 is not defined by the return boundary.
    pub native_return: Option<i32>,
    /// Present only when the existing calcdamage adapter is reached.
    pub arithmetic_qualification: Option<ArithmeticQualification>,
}

fn plan(memory: &RenderingMemory) -> DetailPlan {
    DetailPlan {
        memory: memory.clone(),
        effects: vec![],
        native_return: None,
        arithmetic_qualification: None,
    }
}
fn read(memory: &RenderingMemory, address: u32, length: usize) -> Result<Vec<u8>, RuleError> {
    (0..length)
        .map(|i| memory.byte(address.wrapping_add(i as u32)))
        .collect()
}
fn sw(memory: &RenderingMemory, address: u32) -> Result<i16, RuleError> {
    Ok(memory.word(address)? as i16)
}
fn sb(memory: &RenderingMemory, address: u32) -> Result<i16, RuleError> {
    Ok(i16::from(memory.byte(address)? as i8))
}
fn write(result: &mut DetailPlan, address: u32, bytes: &[u8]) -> Result<(), RuleError> {
    read(&result.memory, address, bytes.len())?;
    result.memory.insert_region(address, bytes);
    result.effects.push(DetailEffect::Write(RecordWrite {
        address,
        bytes: bytes.to_vec(),
    }));
    Ok(())
}
fn word(result: &mut DetailPlan, address: u32, value: i16) -> Result<(), RuleError> {
    write(result, address, &value.to_be_bytes())
}
fn long(result: &mut DetailPlan, address: u32, value: u32) -> Result<(), RuleError> {
    write(result, address, &value.to_be_bytes())
}
fn add_word(result: &mut DetailPlan, address: u32, value: i16) -> Result<(), RuleError> {
    let value = sw(&result.memory, address)?.wrapping_add(value);
    word(result, address, value)
}
fn draw(result: &mut DetailPlan, effect: DrawingEffect) {
    result.effects.push(DetailEffect::Drawing(effect));
}
fn move_to(result: &mut DetailPlan, horizontal: i16, vertical: i16) {
    draw(
        result,
        DrawingEffect::MoveTo {
            horizontal,
            vertical,
        },
    );
}
fn pascal(memory: &RenderingMemory, address: u32) -> Result<PascalString, RuleError> {
    let length = usize::from(memory.byte(address)?);
    PascalString::from_raw(&read(memory, address, length + 1)?)
}
fn literal(result: &mut DetailPlan, address: u32) -> Result<(), RuleError> {
    let text = pascal(&result.memory, address)?;
    draw(result, DrawingEffect::DrawString(text));
    Ok(())
}
fn c_string(memory: &RenderingMemory, address: u32) -> Result<Vec<u8>, RuleError> {
    let mut bytes = vec![];
    for i in 0..=memory.bytes.len() {
        let byte = memory.byte(address.wrapping_add(i as u32))?;
        bytes.push(byte);
        if byte == 0 {
            return Ok(bytes);
        }
    }
    Err(RuleError::Invalid(
        "unterminated detail C string within supplied memory".into(),
    ))
}
fn copy_c(result: &mut DetailPlan, source: u32, destination: u32) -> Result<(), RuleError> {
    // libc overlap is not defined: keep the existing separate-buffer adapter limit.
    let bytes = c_string(&result.memory, source)?;
    write(result, destination, &bytes)
}
fn append_c(result: &mut DetailPlan, source: u32, destination: u32) -> Result<(), RuleError> {
    let length = c_string(&result.memory, destination)?.len() - 1;
    copy_c(result, source, destination.wrapping_add(length as u32))
}
fn pascal_to_c(result: &mut DetailPlan, address: u32) -> Result<(), RuleError> {
    let text = pascal(&result.memory, address)?;
    for (i, byte) in text.payload().iter().enumerate() {
        write(result, address.wrapping_add(i as u32), &[*byte])?;
    }
    write(
        result,
        address.wrapping_add(text.payload().len() as u32),
        &[0],
    )
}
fn c_to_pascal(result: &mut DetailPlan, address: u32) -> Result<(), RuleError> {
    let bytes = c_string(&result.memory, address)?;
    let length = bytes.len() - 1;
    let length_byte = u8::try_from(length).map_err(|_| RuleError::Unsupported {
        action: "detail.c_to_pascal",
        dependency: "the imported C-to-Pascal conversion is represented only through 255 bytes",
    })?;
    for i in (0..length).rev() {
        write(result, address.wrapping_add(i as u32 + 1), &bytes[i..i + 1])?;
    }
    write(result, address, &[length_byte])
}
fn observe(
    result: &mut DetailPlan,
    request: DetailRequest,
    services: &mut (impl DetailServices + ?Sized),
) -> Result<DetailReply, RuleError> {
    let reply = services.observe(&request, &result.memory)?;
    result.effects.push(DetailEffect::Observation {
        request,
        reply: reply.clone(),
    });
    Ok(reply)
}
fn observed_writes(
    result: &mut DetailPlan,
    request: DetailRequest,
    services: &mut (impl DetailServices + ?Sized),
) -> Result<(), RuleError> {
    match observe(result, request, services)? {
        DetailReply::Writes(writes) => {
            for effect in writes {
                write(result, effect.address, &effect.bytes)?;
            }
            Ok(())
        }
        _ => Err(RuleError::Invalid(
            "missing detail Toolbox output writes".into(),
        )),
    }
}
fn sane(
    result: &mut DetailPlan,
    opcode: u16,
    source: Option<u32>,
    destination: u32,
    services: &mut (impl DetailServices + ?Sized),
) -> Result<(), RuleError> {
    observed_writes(
        result,
        DetailRequest::Sane {
            opcode,
            source,
            destination,
        },
        services,
    )
}
fn indexed(
    result: &mut DetailPlan,
    resource_id: i16,
    index: i16,
    destination: u32,
    services: &mut (impl DetailServices + ?Sized),
) -> Result<(), RuleError> {
    let text = services.indexed_string(resource_id, index)?;
    result.effects.push(DetailEffect::IndexedString {
        resource_id,
        index,
        destination,
        result: text.clone(),
    });
    write(result, destination, text.raw())
}
fn number(
    result: &mut DetailPlan,
    value: i16,
    kind: u8,
    services: &mut (impl DetailServices + ?Sized),
) -> Result<(), RuleError> {
    let mut context = TextContext::default();
    let effects = match kind {
        0 => recovered_text::string(&mut context, value, services)?,
        1 => recovered_text::string_no_zero_no_plus(&mut context, value, services)?,
        2 => recovered_text::string_no_zero(&mut context, value, services)?,
        _ => return Err(RuleError::Invalid("unknown detail numeric child".into())),
    };
    for effect in effects {
        match effect {
            TextEffect::NumberToString(value) => {
                result.effects.push(DetailEffect::NumberToString {
                    value,
                    destination: 0x479ed8,
                    result: context.scratch.clone(),
                });
                write(result, 0x479ed8, context.scratch.raw())?;
            }
            TextEffect::DrawString(text) => draw(result, DrawingEffect::DrawString(text)),
            _ => {
                return Err(RuleError::Invalid(
                    "unexpected detail numeric child effect".into(),
                ));
            }
        }
    }
    Ok(())
}
fn convert_number(
    result: &mut DetailPlan,
    value: i32,
    destination: u32,
    services: &mut (impl DetailServices + ?Sized),
) -> Result<(), RuleError> {
    let text = services.number_to_string(value)?;
    result.effects.push(DetailEffect::NumberToString {
        value,
        destination,
        result: text.clone(),
    });
    write(result, destination, text.raw())
}
fn load_item(result: &mut DetailPlan, id: i16, ammo: bool) -> Result<(), RuleError> {
    let address = if ammo { 0x4536d2 } else { 0x45363c };
    let current: [u8; ITEM_BYTES] = read(&result.memory, address, ITEM_BYTES)?
        .try_into()
        .expect("item record");
    let mut backing: [Vec<u8>; 5] = std::array::from_fn(|_| vec![]);
    let absolute = id.wrapping_abs();
    if let Ok(family) = usize::try_from(absolute / 200)
        && family < 5
    {
        let offset = (absolute % 200) as usize * ITEM_BYTES;
        let bytes = read(
            &result.memory,
            ITEM_TABLE_ADDRESSES[family].wrapping_add(offset as u32),
            ITEM_BYTES,
        )?;
        backing[family].resize(offset + ITEM_BYTES, 0);
        backing[family][offset..].copy_from_slice(&bytes);
    }
    let tables = ItemTables {
        families: std::array::from_fn(|i| backing[i].as_slice()),
    };
    let child = if ammo {
        recovered_records::loadammo(&tables, &current, id)?
    } else {
        recovered_records::loaditem(&tables, &current, id)?
    };
    for effect in child.writes {
        write(result, effect.address, &effect.bytes)?;
    }
    Ok(())
}
fn load_spell(result: &mut DetailPlan, descriptor: i16) -> Result<(), RuleError> {
    let school = (descriptor / 1000).wrapping_sub(1);
    let rest = descriptor.wrapping_sub((descriptor / 1000).wrapping_mul(1000));
    let level = (rest / 100).wrapping_sub(1);
    let index = rest
        .wrapping_sub((rest / 100).wrapping_mul(100))
        .wrapping_sub(1);
    let offset = usize::from(school as u16) * 2842
        + usize::from(level as u16) * 406
        + usize::from(index as u16) * SPELL_BYTES;
    let bytes = read(
        &result.memory,
        0x443788_u32.wrapping_add(offset as u32),
        SPELL_BYTES,
    )?;
    let mut table = vec![0; offset + SPELL_BYTES];
    table[offset..].copy_from_slice(&bytes);
    let child = recovered_records::loadspell2(&table, descriptor)?;
    for effect in child.record.writes {
        write(result, effect.address, &effect.bytes)?;
    }
    Ok(())
}
fn icon_context(rect: BodyRect) -> DrawingContext {
    DrawingContext {
        rect,
        banner_rect: rect,
        source_rect: rect,
        source_row: 0,
        source_column: 0,
        icon_handle: 0,
        item_work: [0; ITEM_BYTES],
        text_work: PascalString::default(),
        window: 0,
        item_window: 0,
        half_width_input: 0,
        active_member: 0,
        omit_banner: 0,
        show_item_background: 0,
        text_in_other_port: 0,
        erase_pattern: 0,
        default_font: 0,
        selection: 0,
        map_port: 0,
        text_port: 0,
        secondary_port: 0,
        backing_port: 0,
        tile_port: 0,
        depth: 0,
        old_world: 0,
        old_device: 0,
        buildings: vec![],
    }
}
fn icon(
    result: &mut DetailPlan,
    id: i16,
    rectangle: BodyRect,
    services: &mut (impl DetailServices + ?Sized),
) -> Result<(), RuleError> {
    let child =
        recovered_drawing::ploticon3(&mut icon_context(rectangle), id, rectangle, services)?;
    for effect in child.effects {
        match effect {
            DrawingEffect::Write(effect) => write(result, effect.address, &effect.bytes)?,
            effect => draw(result, effect),
        }
    }
    Ok(())
}
fn picture(
    result: &mut DetailPlan,
    id: i16,
    rectangle: BodyRect,
    services: &mut (impl DetailServices + ?Sized),
) -> Result<(), RuleError> {
    for effect in recovered_drawing::pict(id, rectangle, services)? {
        draw(result, effect);
    }
    Ok(())
}
fn text_box(
    result: &mut DetailPlan,
    pointer: u32,
    length: i32,
    rectangle_pointer: u32,
) -> Result<(), RuleError> {
    let rectangle = result.memory.rect(rectangle_pointer)?;
    result.effects.push(DetailEffect::TextBox {
        pointer,
        length,
        bytes: read(
            &result.memory,
            pointer,
            usize::try_from(length)
                .map_err(|_| RuleError::Invalid("negative detail text length".into()))?,
        )?,
        rectangle_pointer,
        rectangle,
        alignment: 0,
    });
    Ok(())
}
fn description_box(result: &mut DetailPlan, text: u32, rectangle: u32) -> Result<(), RuleError> {
    let length = (c_string(&result.memory, text)?.len() - 1) as i16;
    word(result, 0x4434b6, length)?;
    c_to_pascal(result, text)?;
    let length = i32::from(sw(&result.memory, 0x4434b6)?);
    text_box(result, text.wrapping_add(1), length, rectangle)
}
fn file(
    result: &mut DetailPlan,
    request: FileRequest,
    services: &mut (impl DetailServices + ?Sized),
) -> Result<FileReply, RuleError> {
    match observe(result, DetailRequest::File(request), services)? {
        DetailReply::File(reply) => Ok(reply),
        _ => Err(RuleError::Invalid("missing detail file observation".into())),
    }
}
fn filename(
    result: &mut DetailPlan,
    capacity: usize,
    services: &mut (impl DetailServices + ?Sized),
) -> Result<(), RuleError> {
    let variant = result.memory.byte(0x44274c)?;
    let mut context = FileContext {
        scenario_directory: c_string(&result.memory, 0x44260c)?,
        variant,
        filename: read(&result.memory, 0x4389a4, capacity)?,
        ..FileContext::default()
    };
    let name = c_string(&result.memory, 0x40299e)?;
    let converted = services.number_to_string(i32::from(variant))?;
    let child = recovered_files::get_filename(
        &mut context,
        &name,
        &[FileObservation {
            request: FileRequest::NumberToString(i32::from(variant)),
            reply: FileReply::Pascal(converted),
        }],
    )?;
    for effect in child.effects {
        if let FileEffect::Write {
            destination: FileBuffer::Filename,
            offset,
            bytes,
        } = &effect
        {
            write(result, 0x4389a4_u32.wrapping_add(*offset as u32), bytes)?;
        } else {
            result.effects.push(DetailEffect::File(effect));
        }
    }
    Ok(())
}
fn discarded_file(
    result: &mut DetailPlan,
    request: FileRequest,
    services: &mut (impl DetailServices + ?Sized),
) -> Result<(), RuleError> {
    match file(result, request, services)? {
        FileReply::Ignored | FileReply::Integer(_) | FileReply::Error(_) => Ok(()),
        _ => Err(RuleError::Invalid(
            "unexpected discarded detail stdio result".into(),
        )),
    }
}

/// quickinfo, 0x132b0c. The four raw WORD arguments begin at A6+8.
pub fn quick_info(
    memory: &RenderingMemory,
    frame: u32,
    incoming_a2: u32,
    filename_capacity: usize,
    services: &mut (impl DetailServices + ?Sized),
) -> Result<DetailPlan, RuleError> {
    let mut result = plan(memory);
    let member = result.memory.word(frame.wrapping_add(8))?;
    let slot = result.memory.word(frame.wrapping_add(10))?;
    let at = |offset: u32| frame.wrapping_sub(offset);
    observed_writes(
        &mut result,
        DetailRequest::GetPort {
            destination: at(0x12c),
        },
        services,
    )?;
    observed_writes(
        &mut result,
        DetailRequest::GetForeColor {
            destination: at(0x122),
        },
        services,
    )?;
    observed_writes(
        &mut result,
        DetailRequest::GetBackColor {
            destination: at(0x128),
        },
        services,
    )?;
    let descriptor = sw(&result.memory, frame.wrapping_add(12))?;
    word(&mut result, at(0x114), descriptor)?;
    word(&mut result, at(0x118), 0)?;
    word(&mut result, at(0x112), 0)?;
    if (slot as i16) < 0 && (member as i16) > -1 {
        return Ok(result);
    }
    let port_address = match sw(&result.memory, frame.wrapping_add(14))? {
        1 => 0x438788,
        2 => 0x438790,
        _ => 0x438798,
    };
    let port = result.memory.long(port_address)?;
    draw(&mut result, DrawingEffect::SetPort(port));
    let pattern = result.memory.long(0x4358ce)?;
    draw(&mut result, DrawingEffect::BackPixelPattern(pattern));
    draw(&mut result, DrawingEffect::TextSize(12));
    draw(&mut result, DrawingEffect::ForeColor(69));
    result.effects.push(DetailEffect::TextMode(1));
    result.effects.push(DetailEffect::TextFaceSlot(0));
    let font = sw(&result.memory, 0x402f3c)?;
    draw(&mut result, DrawingEffect::TextFont(font));
    word(&mut result, at(0x108), 460)?;
    word(&mut result, at(0x106), 0)?;
    word(&mut result, at(0x102), 800)?;
    word(&mut result, at(0x104), 580)?;
    let rectangle = result.memory.rect(at(0x108))?;
    picture(&mut result, 226, rectangle, services)?;
    let mut has_ammunition = false;
    let pack = if member == 0xffff {
        0x41fc74_u32.wrapping_add(u32::from(slot) * 22)
    } else {
        0x448aa0_u32
            .wrapping_add(u32::from(member) * WARRIOR_BYTES as u32)
            .wrapping_add(u32::from(slot) * 22)
            .wrapping_add(0x132)
    };
    if member == 0xffff {
        let id = sw(&result.memory, frame.wrapping_add(12))?;
        load_item(&mut result, id, false)?;
        load_item(&mut result, 0, true)?;
        let value = sw(&result.memory, 0x453650)?;
        word(&mut result, at(0x112), value)?;
        let value = sw(&result.memory, 0x453654)?;
        word(&mut result, at(0x116), value)?;
        let value = sb(&result.memory, 0x45368c)?.wrapping_add(sw(&result.memory, 0x453698)?);
        word(&mut result, at(0x118), value)?;
        if result.memory.byte(0x442752)? != 0 {
            merge_pack(&mut result, pack, false)?;
        }
    } else {
        let id = sw(&result.memory, pack)?;
        load_item(&mut result, id, false)?;
        if result.memory.word(pack.wrapping_add(4))? != 0 {
            let id = sw(&result.memory, pack.wrapping_add(4))?;
            load_item(&mut result, id, true)?;
            has_ammunition = true;
        } else {
            load_item(&mut result, 0, true)?;
        }
        merge_pack(&mut result, pack, true)?;
        let value =
            sb(&result.memory, pack.wrapping_add(21))?.wrapping_add(sw(&result.memory, 0x453698)?);
        word(&mut result, at(0x118), value)?;
        let value = sw(&result.memory, pack.wrapping_add(2))?;
        word(&mut result, at(0x112), value)?;
        let value = sw(&result.memory, pack.wrapping_add(6))?;
        word(&mut result, at(0x116), value)?;
    }
    for (vertical, address, byte, kind) in [
        (479, 0x4536c6, false, 1),
        (498, 0x45369a, false, 1),
        (517, 0x453642, true, 2),
        (536, 0x453694, false, 1),
        (555, at(0x112), false, 1),
        (574, 0x453698, false, 1),
    ] {
        move_to(&mut result, 450, vertical);
        let value = if byte {
            sb(&result.memory, address)?
        } else {
            sw(&result.memory, address)?
        };
        number(&mut result, value, kind, services)?;
    }
    let multiplier_input = if (member as i16) > -1 {
        i32::from(sb(&result.memory, pack.wrapping_add(18))?) as u32
    } else if (slot as i16) > -1 {
        i32::from(sb(
            &result.memory,
            0x41fc74_u32
                .wrapping_add(u32::from(slot) * 22)
                .wrapping_add(18),
        )?) as u32
    } else {
        incoming_a2
    };
    long(&mut result, at(0x13a), multiplier_input.wrapping_add(100))?;
    sane(&mut result, 0x280e, Some(at(0x13a)), at(0x13a), services)?;
    sane(&mut result, 0x1010, Some(at(0x13a)), at(0x13a), services)?;
    let value = result.memory.long(at(0x13a))?;
    long(&mut result, at(0x11c), value)?;
    long(&mut result, at(0x130), 0x42c80000)?;
    sane(&mut result, 0x100e, Some(at(0x11c)), at(0x13a), services)?;
    sane(&mut result, 0x1006, Some(at(0x130)), at(0x13a), services)?;
    sane(&mut result, 0x1010, Some(at(0x13a)), at(0x11c), services)?;
    let value = sw(&result.memory, 0x45367a)?.wrapping_add(sw(&result.memory, 0x453664)?);
    word(&mut result, 0x4434b6, value)?;
    if has_ammunition {
        let value = sw(&result.memory, 0x453710)?.wrapping_add(sw(&result.memory, 0x4536fa)?);
        add_word(&mut result, 0x4434b6, value)?;
    }
    if result.memory.word(0x453652)? != 0 {
        let value = sw(&result.memory, 0x4434b6)?.wrapping_mul(sw(&result.memory, 0x453652)?);
        word(&mut result, 0x4434b6, value)?;
    }
    let mut damage = sw(&result.memory, 0x4434b6)?;
    for (ammo, item) in [
        (0x45375e, 0x4536c8),
        (0x453712, 0x45367c),
        (0x453714, 0x45367e),
        (0x453716, 0x453680),
        (0x453718, 0x453682),
        (0x45371a, 0x453684),
        (0x45371c, 0x453686),
    ] {
        let value = sw(&result.memory, ammo)?
            .wrapping_add(sw(&result.memory, item)?)
            .wrapping_mul(sw(&result.memory, 0x453652)?);
        damage = damage.wrapping_add(value);
    }
    word(&mut result, at(0x13a), damage)?;
    sane(&mut result, 0x200e, Some(at(0x13a)), at(0x13a), services)?;
    sane(&mut result, 0x1004, Some(at(0x11c)), at(0x13a), services)?;
    sane(&mut result, 0x16, None, at(0x13a), services)?;
    sane(&mut result, 0x2810, Some(at(0x13a)), at(0x13a), services)?;
    damage = sw(&result.memory, at(0x138))?;
    draw(&mut result, DrawingEffect::ForeColor(205));
    move_to(&mut result, 652, 479);
    let value = damage.wrapping_mul(sw(&result.memory, 0x453688)?);
    number(&mut result, value, 1, services)?;
    draw(&mut result, DrawingEffect::ForeColor(69));
    move_to(&mut result, 580, 479);
    number(&mut result, damage, 1, services)?;
    for (vertical, address, kind) in [
        (498, 0x453678, 2),
        (517, 0x453652, 1),
        (536, at(0x116), 1),
        (555, 0x453688, 1),
    ] {
        move_to(&mut result, 580, vertical);
        let value = sw(&result.memory, address)?;
        number(&mut result, value, kind, services)?;
    }
    move_to(&mut result, 580, 574);
    let value = sb(&result.memory, 0x45368e)?.wrapping_add(sb(&result.memory, 0x45368d)?);
    number(&mut result, value, 2, services)?;
    word(&mut result, at(0x110), 498)?;
    word(&mut result, at(0x10e), 638)?;
    word(&mut result, at(0x10a), 787)?;
    word(&mut result, at(0x10c), 573)?;
    draw(&mut result, DrawingEffect::TextSize(10));
    draw(&mut result, DrawingEffect::TextFont(10));
    draw(&mut result, DrawingEffect::ForeColor(273));
    let text = at(0x100);
    if sw(&result.memory, 0x453666)? > 19 {
        let index = sw(&result.memory, 0x453666)?.wrapping_sub(19);
        indexed(&mut result, 136, index, text, services)?;
        pascal_to_c(&mut result, text)?;
        description_box(&mut result, text, at(0x110))?;
    }
    if sw(&result.memory, 0x453666)? == -10 {
        copy_c(&mut result, 0x402a3a, text)?;
        let index = sw(&result.memory, 0x453668)?.wrapping_sub(19);
        indexed(&mut result, 133, index, 0x479ed8, services)?;
        pascal_to_c(&mut result, 0x479ed8)?;
        append_c(&mut result, 0x479ed8, text)?;
        description_box(&mut result, text, at(0x110))?;
    }
    if sw(&result.memory, 0x453668)? > 1100 && sw(&result.memory, 0x453668)? < 4000 {
        let descriptor = sw(&result.memory, 0x453668)?;
        load_spell(&mut result, descriptor)?;
        copy_c(&mut result, 0x402a30, text)?;
        let resource = sw(&result.memory, 0x4435c6)?
            .wrapping_add(1)
            .wrapping_mul(1000)
            .wrapping_add(sw(&result.memory, 0x4435c8)?);
        let index = sw(&result.memory, 0x4435ca)?.wrapping_add(1);
        indexed(&mut result, resource, index, 0x479ed8, services)?;
        pascal_to_c(&mut result, 0x479ed8)?;
        append_c(&mut result, 0x479ed8, text)?;
        description_box(&mut result, text, at(0x110))?;
    }
    if sw(&result.memory, 0x453666)? == 122 {
        let source = match sw(&result.memory, 0x453668)? {
            1 => Some(0x402a12),
            2 => Some(0x4029f2),
            3 => Some(0x4029d2),
            4 => Some(0x4029b2),
            _ => None,
        };
        if let Some(source) = source {
            copy_c(&mut result, source, text)?;
        }
        description_box(&mut result, text, at(0x110))?;
    }
    for address in [0x45366a, 0x45366c] {
        if sw(&result.memory, address)? > 0 && sw(&result.memory, address)? < 16 {
            copy_c(&mut result, 0x402d2d, text)?;
            if sw(&result.memory, 0x45366e)? > 0 {
                copy_c(&mut result, 0x4029b0, text)?;
            }
            let value = i32::from(sw(&result.memory, 0x45366e)?);
            convert_number(&mut result, value, 0x479ed8, services)?;
            pascal_to_c(&mut result, 0x479ed8)?;
            append_c(&mut result, 0x479ed8, text)?;
            append_c(&mut result, 0x4029aa, text)?;
            let index = sw(&result.memory, address)?;
            indexed(&mut result, 1201, index, 0x479ed8, services)?;
            pascal_to_c(&mut result, 0x479ed8)?;
            append_c(&mut result, 0x479ed8, text)?;
            description_box(&mut result, text, at(0x110))?;
        }
    }
    draw(&mut result, DrawingEffect::ForeColor(69));
    let name_address = if sw(&result.memory, 0x4536a2)? > 799 {
        add_word(&mut result, at(0x114), -800)?;
        filename(&mut result, filename_capacity, services)?;
        0x4389a4
    } else {
        0x402986
    };
    let mode = c_string(&result.memory, 0x402f24)?;
    let name = c_string(&result.memory, name_address)?;
    let handle = match file(&mut result, FileRequest::Open { name, mode }, services)? {
        FileReply::Handle(handle) => handle,
        _ => return Err(RuleError::Invalid("missing detail fopen handle".into())),
    };
    long(&mut result, 0x4434a6, handle)?;
    let offset = (i32::from(sw(&result.memory, at(0x114))?) as u32).wrapping_shl(8);
    let handle = result.memory.long(0x4434a6)?;
    discarded_file(
        &mut result,
        FileRequest::Seek {
            handle,
            offset,
            origin: 0,
        },
        services,
    )?;
    let handle = result.memory.long(0x4434a6)?;
    match observe(
        &mut result,
        DetailRequest::ReadFile {
            handle,
            size: 256,
            count: 1,
            destination: 0x479ed8,
        },
        services,
    )? {
        DetailReply::File(FileReply::Read { bytes_written, .. }) if bytes_written.len() <= 256 => {
            write(&mut result, 0x479ed8, &bytes_written)?
        }
        _ => {
            return Err(RuleError::Invalid(
                "missing bounded detail fread output".into(),
            ));
        }
    }
    let handle = result.memory.long(0x4434a6)?;
    discarded_file(&mut result, FileRequest::Close(handle), services)?;
    word(&mut result, at(0x110), 473)?;
    word(&mut result, at(0x10e), 13)?;
    word(&mut result, at(0x10a), 350)?;
    word(&mut result, at(0x10c), 534)?;
    let pattern = result.memory.long(0x4358ce)?;
    draw(&mut result, DrawingEffect::BackPixelPattern(pattern));
    draw(&mut result, DrawingEffect::ForeColor(69));
    text_box(&mut result, 0x479ed9, 256, at(0x110))?;
    move_to(&mut result, 168, 551);
    if result.memory.word(0x45365c)? != 0 {
        if has_ammunition {
            literal(&mut result, 0x402976)?;
            let selection = sw(&result.memory, 0x453738)?;
            let base = crate::rules::selection_base(selection);
            word(&mut result, 0x4471e8, base)?;
            let index = sw(&result.memory, 0x453738)?
                .wrapping_sub(sw(&result.memory, 0x4471e8)?)
                .wrapping_add(1);
            indexed(&mut result, base.wrapping_add(1), index, 0x479ed8, services)?;
        } else {
            literal(&mut result, 0x402964)?;
            let index = sw(&result.memory, 0x45365c)?;
            indexed(&mut result, 142, index, 0x479ed8, services)?;
        }
    } else if result.memory.word(0x45365e)? != 0 {
        literal(&mut result, 0x40294e)?;
        let index = sw(&result.memory, 0x45365e)?;
        indexed(&mut result, 142, index, 0x479ed8, services)?;
    } else if result.memory.byte(0x453693)? != 0 {
        literal(&mut result, 0x402938)?;
        let index = sb(&result.memory, 0x453693)?.wrapping_add(1);
        indexed(&mut result, -11000, index, 0x479ed8, services)?;
    } else {
        literal(&mut result, 0x402922)?;
        let index = sw(&result.memory, 0x453656)?.wrapping_add(1);
        indexed(&mut result, -11000, index, 0x479ed8, services)?;
    }
    move_to(&mut result, 168, 571);
    literal(&mut result, 0x479ed8)?;
    word(&mut result, at(0x108), 542)?;
    word(&mut result, at(0x106), 15)?;
    let value = sw(&result.memory, at(0x106))?.wrapping_add(32);
    word(&mut result, at(0x102), value)?;
    let value = sw(&result.memory, at(0x108))?.wrapping_add(32);
    word(&mut result, at(0x104), value)?;
    let rectangle = result.memory.rect(at(0x108))?;
    let id = sw(&result.memory, 0x453646)?;
    icon(&mut result, id, rectangle, services)?;
    add_word(&mut result, at(0x106), 50)?;
    add_word(&mut result, at(0x102), 50)?;
    if has_ammunition {
        let rectangle = result.memory.rect(at(0x108))?;
        let id = sw(&result.memory, 0x4536dc)?;
        icon(&mut result, id, rectangle, services)?;
    }
    add_word(&mut result, at(0x106), 50)?;
    add_word(&mut result, at(0x102), 50)?;
    let mut animation = sw(&result.memory, 0x453676)?;
    if has_ammunition && result.memory.word(0x45370c)? != 0 {
        animation = sw(&result.memory, 0x45370c)?;
    }
    if animation == 0 && result.memory.word(0x453674)? != 0 {
        animation = 1;
    }
    if animation != 0 {
        let rectangle = result.memory.rect(at(0x108))?;
        icon(
            &mut result,
            animation.wrapping_add(9999),
            rectangle,
            services,
        )?;
    }
    let port = result.memory.long(at(0x12c))?;
    draw(&mut result, DrawingEffect::SetPort(port));
    let pointer = at(0x128);
    let bytes = read(&result.memory, pointer, 6)?.try_into().expect("RGB");
    result
        .effects
        .push(DetailEffect::RgbBackColor { pointer, bytes });
    let pointer = at(0x122);
    let bytes = read(&result.memory, pointer, 6)?.try_into().expect("RGB");
    result
        .effects
        .push(DetailEffect::RgbForeColor { pointer, bytes });
    Ok(result)
}

fn merge_pack(result: &mut DetailPlan, pack: u32, equipped: bool) -> Result<(), RuleError> {
    let value = sb(&result.memory, pack.wrapping_add(13))?;
    word(result, 0x4536c6, value)?;
    let value = sb(&result.memory, pack.wrapping_add(14))?;
    add_word(result, 0x453694, value)?;
    let value = sb(&result.memory, pack.wrapping_add(15))?
        .wrapping_add(sb(&result.memory, 0x453723)?) as u8;
    write(result, 0x45368d, &[value])?;
    let value = result.memory.byte(pack.wrapping_add(11))?;
    write(result, 0x45368e, &[value])?;
    for (offset, destination) in [(16, 0x45363f), (17, 0x453640)] {
        let value = result.memory.byte(pack.wrapping_add(offset))?;
        let value = if equipped {
            value.wrapping_add(result.memory.byte(destination)?)
        } else {
            value
        };
        write(result, destination, &[value])?;
    }
    let value = sb(&result.memory, pack.wrapping_add(18))?
        .wrapping_add(sb(&result.memory, 0x453727)?) as u8;
    write(result, 0x453691, &[value])?;
    let value = sb(&result.memory, pack.wrapping_add(19))?;
    add_word(result, 0x45369a, value)?;
    let value = sb(&result.memory, pack.wrapping_add(20))?
        .wrapping_add(sb(&result.memory, 0x453721)?)
        .wrapping_add(sw(&result.memory, 0x45370e)?);
    add_word(result, 0x453678, value)
}

fn active_address(result: &DetailPlan, offset: u32) -> Result<u32, RuleError> {
    Ok(0x448aa0_u32
        .wrapping_add(u32::from(result.memory.word(0x4435d2)?) * WARRIOR_BYTES as u32)
        .wrapping_add(offset))
}
fn active_word(result: &DetailPlan, offset: u32) -> Result<i16, RuleError> {
    sw(&result.memory, active_address(result, offset)?)
}
fn active_byte(result: &DetailPlan, offset: u32) -> Result<u8, RuleError> {
    result.memory.byte(active_address(result, offset)?)
}
fn active_signed_byte(result: &DetailPlan, offset: u32) -> Result<i16, RuleError> {
    sb(&result.memory, active_address(result, offset)?)
}

fn calculated_damage(
    result: &mut DetailPlan,
    mode: i16,
    damage_frame: DamageFrame,
) -> Result<i16, RuleError> {
    let member = result.memory.word(0x4435d2)?;
    let offset = usize::from(member) * WARRIOR_BYTES;
    let warrior = read(
        &result.memory,
        0x448aa0_u32.wrapping_add(offset as u32),
        WARRIOR_BYTES,
    )?;
    let mut warrior_memory = vec![0; offset + WARRIOR_BYTES];
    warrior_memory[offset..].copy_from_slice(&warrior);
    let mut backing: [Vec<u8>; 5] = std::array::from_fn(|_| vec![]);
    let equipment = if mode == 0 { 0x12c } else { 0x12e };
    let id = i16::from_be_bytes([warrior[equipment], warrior[equipment + 1]]);
    let ammunition = i16::from_be_bytes([warrior[0x126], warrior[0x127]]);
    let ids = if id != 0 && mode == 0 && ammunition != 0 {
        vec![id, ammunition]
    } else if id != 0 {
        vec![id]
    } else {
        vec![]
    };
    for id in ids {
        let absolute = id.wrapping_abs();
        if let Ok(family) = usize::try_from(absolute / 200)
            && family < 5
        {
            let offset = (absolute % 200) as usize * ITEM_BYTES;
            let bytes = read(
                &result.memory,
                ITEM_TABLE_ADDRESSES[family].wrapping_add(offset as u32),
                ITEM_BYTES,
            )?;
            backing[family].resize(backing[family].len().max(offset + ITEM_BYTES), 0);
            backing[family][offset..offset + ITEM_BYTES].copy_from_slice(&bytes);
        }
    }
    // calcdamage does not read the weapon, packet, coordinates or attack count.
    let current = AttackWork {
        item: read(&result.memory, 0x45363c, ITEM_BYTES)?
            .try_into()
            .expect("item"),
        ammunition: read(&result.memory, 0x4536d2, ITEM_BYTES)?
            .try_into()
            .expect("ammunition"),
        weapon: [0; ITEM_BYTES],
        packet: [0; SPELL_BYTES],
        spell_coordinates: [0; 3],
        attack_count: 0,
        damage_total: sw(&result.memory, 0x41f714)?,
    };
    let tables = AttackTables {
        warrior_memory: &warrior_memory,
        items: ItemTables {
            families: std::array::from_fn(|i| backing[i].as_slice()),
        },
        spells: &[],
    };
    let child = recovered_attack::calcdamage(&current, &tables, member, mode, 1, damage_frame)?;
    result.arithmetic_qualification = Some(child.qualification);
    for effect in child.effects {
        match effect {
            AttackEffect::Write(effect) => write(result, effect.address, &effect.bytes)?,
            AttackEffect::LocationLog(_) => {
                return Err(RuleError::Invalid(
                    "unexpected calcdamage location log".into(),
                ));
            }
        }
    }
    Ok(child.native_return)
}

fn color_icon_global(
    result: &mut DetailPlan,
    id: i16,
    rectangle: u32,
    services: &mut (impl DetailServices + ?Sized),
) -> Result<(), RuleError> {
    draw(result, DrawingEffect::GetColorIcon(id));
    let handle = services.color_icon(id)?;
    long(result, 0x439598, handle)?;
    if result.memory.long(0x439598)? != 0 {
        let handle = result.memory.long(0x439598)?;
        let rect = result.memory.rect(rectangle)?;
        draw(result, DrawingEffect::PlotColorIcon { handle, rect });
        let handle = result.memory.long(0x439598)?;
        draw(result, DrawingEffect::DisposeColorIcon(handle));
    }
    Ok(())
}

/// updateodds, 0x161068. The target is the raw unsigned WORD at A6+8.
pub fn update_odds(
    memory: &RenderingMemory,
    frame: u32,
    damage_frame: DamageFrame,
    services: &mut (impl DetailServices + ?Sized),
) -> Result<DetailPlan, RuleError> {
    let mut result = plan(memory);
    let target = result.memory.word(frame.wrapping_add(8))?;
    let at = |offset: u32| frame.wrapping_sub(offset);
    word(&mut result, at(0xe), 0)?;
    word(&mut result, at(6), 480)?;
    word(&mut result, at(2), 800)?;
    word(&mut result, at(8), 300)?;
    let bottom = sw(&result.memory, at(8))?.wrapping_add(108);
    word(&mut result, at(4), bottom)?;
    let rectangle = result.memory.rect(at(8))?;
    if (target as i16) < 0 {
        picture(&mut result, 224, rectangle, services)?;
        return Ok(result);
    }
    picture(&mut result, 201, rectangle, services)?;
    word(&mut result, at(6), 487)?;
    let value = sw(&result.memory, at(6))?.wrapping_add(64);
    word(&mut result, at(2), value)?;
    word(&mut result, at(8), 309)?;
    let value = sw(&result.memory, at(8))?.wrapping_add(64);
    word(&mut result, at(4), value)?;
    let npc = 0x439d3c_u32.wrapping_add(u32::from(target) * 288);
    match sw(&result.memory, npc.wrapping_add(16))? {
        0 => observed_writes(
            &mut result,
            DetailRequest::InsetRect {
                pointer: at(8),
                horizontal: 16,
                vertical: 16,
            },
            services,
        )?,
        1 => observed_writes(
            &mut result,
            DetailRequest::InsetRect {
                pointer: at(8),
                horizontal: 16,
                vertical: 0,
            },
            services,
        )?,
        2 => observed_writes(
            &mut result,
            DetailRequest::InsetRect {
                pointer: at(8),
                horizontal: 0,
                vertical: 16,
            },
            services,
        )?,
        _ => (),
    }
    let id = sw(&result.memory, npc.wrapping_add(0x50))?;
    word(&mut result, 0x4434b6, id)?;
    if sw(&result.memory, npc.wrapping_add(16))? != -3
        && result.memory.byte(npc.wrapping_add(0x107))? == 1
    {
        add_word(&mut result, 0x4434b6, 1000)?;
    }
    let id = sw(&result.memory, 0x4434b6)?;
    color_icon_global(&mut result, id, at(8), services)?;
    word(&mut result, at(6), 502)?;
    let value = sw(&result.memory, at(6))?.wrapping_add(32);
    word(&mut result, at(2), value)?;
    word(&mut result, at(8), 375)?;
    let value = sw(&result.memory, at(8))?.wrapping_add(32);
    word(&mut result, at(4), value)?;
    long(&mut result, 0x439598, 0)?;
    let id = if active_byte(&result, 0x9dd)? != 0 {
        let id = active_word(&result, 0xe4)?;
        load_item(&mut result, id, false)?;
        sw(&result.memory, 0x453646)?
    } else if active_word(&result, 0xca)? != 0 {
        let id = active_word(&result, 0xca)?;
        load_item(&mut result, id, false)?;
        sw(&result.memory, 0x453646)?
    } else {
        7002
    };
    color_icon_global(&mut result, id, at(8), services)?;
    let mut chance;
    if active_byte(&result, 0x9dd)? == 0 {
        chance = 33_i16
            .wrapping_add(active_word(&result, 6)?)
            .wrapping_add(active_word(&result, 0xb5a)?)
            .wrapping_add(active_signed_byte(&result, 0x157)?);
        if active_word(&result, 0x62)? != 0 {
            chance = chance.wrapping_sub(active_word(&result, 0x62)?.wrapping_abs());
        }
        for (offset, penalty) in [(0x6a, 15), (0x94, 15), (0x98, 10)] {
            if active_word(&result, offset)? != 0 {
                chance = chance.wrapping_sub(penalty);
            }
        }
        if active_word(&result, 0xa6)? != 0 {
            chance = chance.wrapping_sub(active_word(&result, 0xa6)?.wrapping_abs());
        }
        for index in 0..8 {
            if result.memory.byte(npc.wrapping_add(0xa8 + index))? != 0 {
                chance = chance.wrapping_add(active_word(&result, 0xae + index * 2)?);
                let adjustment = active_word(&result, 0xae + index * 2)? / 5;
                add_word(&mut result, 0x4434b8, adjustment)?;
            }
        }
        chance = chance.wrapping_sub(sw(&result.memory, npc.wrapping_add(8))?);
        chance = chance.wrapping_sub(
            sb(&result.memory, npc.wrapping_add(0xb9))?
                .wrapping_abs()
                .wrapping_mul(2),
        );
        if result.memory.byte(npc.wrapping_add(0xb4))? != 0 {
            chance =
                chance.wrapping_add(sb(&result.memory, npc.wrapping_add(0xb4))?.wrapping_abs());
        }
        for (offset, bonus) in [(0xb8, 15), (0xcd, 15), (0xcf, 10)] {
            if result.memory.byte(npc.wrapping_add(offset))? != 0 {
                chance = chance.wrapping_add(bonus);
            }
        }
        if result.memory.byte(npc.wrapping_add(0xd7))? != 0 {
            chance =
                chance.wrapping_add(sb(&result.memory, npc.wrapping_add(0xd7))?.wrapping_abs());
        }
        if result.memory.byte(npc.wrapping_add(0xd8))? != 0 {
            chance =
                chance.wrapping_sub(sb(&result.memory, npc.wrapping_add(0xd8))?.wrapping_abs());
        }
        if chance < 5 {
            chance = 5;
        }
    } else if result.memory.word(0x45375a)? != 0 || result.memory.word(0x4537f0)? != 0 {
        chance = 999;
    } else {
        chance = 33_i16
            .wrapping_add(active_word(&result, 4)?)
            .wrapping_add(active_word(&result, 0xb58)?)
            .wrapping_add(active_signed_byte(&result, 0x141)?);
        if result.memory.byte(npc.wrapping_add(0xb3))? != 0 {
            chance = 100;
        } else {
            chance = chance.wrapping_sub(sw(&result.memory, npc.wrapping_add(0x62))?);
            chance = chance.wrapping_sub(sw(&result.memory, npc.wrapping_add(4))? / 6);
            chance = chance.wrapping_sub(sw(&result.memory, npc)?.wrapping_mul(3));
            chance = chance.wrapping_sub(
                sb(&result.memory, npc.wrapping_add(0x104))?.wrapping_mul(sw(&result.memory, npc)?),
            );
            match sw(&result.memory, npc.wrapping_add(16))?.wrapping_abs() {
                0 => chance = chance.wrapping_sub(5),
                1 | 2 => chance = chance.wrapping_add(5),
                3 => chance = chance.wrapping_add(10),
                _ => (),
            }
            let maximum = 90_i16.wrapping_sub(sw(&result.memory, 0x442576)?.wrapping_mul(5));
            let minimum = 15_i16.wrapping_sub(sw(&result.memory, 0x442576)?.wrapping_mul(5));
            chance = crate::rules::pin(chance, minimum, maximum);
        }
    }
    draw(&mut result, DrawingEffect::TextSize(12));
    draw(&mut result, DrawingEffect::ForeColor(30));
    let font = sw(&result.memory, 0x402f3c)?;
    draw(&mut result, DrawingEffect::TextFont(font));
    result.effects.push(DetailEffect::TextFaceSlot(0));
    move_to(&mut result, 685, 337);
    let maximum = match sw(&result.memory, 0x442576)? {
        -2 => Some(100),
        -1 => Some(95),
        0 => Some(90),
        1 => Some(85),
        2 => Some(80),
        _ => None,
    };
    if let Some(maximum) = maximum
        && chance > maximum
    {
        chance = maximum;
    }
    number(&mut result, chance, 0, services)?;
    let (total, mut average);
    if active_byte(&result, 0x9dd)? == 0 {
        let attacks = active_word(&result, 0x9e)?.wrapping_add(1);
        word(&mut result, at(0xe), attacks)?;
        total = calculated_damage(&mut result, 1, damage_frame)?;
        average = total / 2;
        let resistance = sw(&result.memory, npc.wrapping_add(8))?;
        word(&mut result, at(0x26), resistance)?;
        damage_resistance(&mut result, frame, 0x26, 0x18, average, services)?;
        average = sw(&result.memory, at(0x16))?;
    } else {
        total = calculated_damage(&mut result, 0, damage_frame)?;
        let attacks = sw(&result.memory, 0x453688)?;
        word(&mut result, at(0xe), attacks)?;
        average = total.wrapping_sub(sw(&result.memory, at(0xe))?) / 2;
        average = average.wrapping_add(sw(&result.memory, at(0xe))?);
        let resistance = sw(&result.memory, npc.wrapping_add(8))?;
        word(&mut result, at(0x18), resistance)?;
        damage_resistance(&mut result, frame, 0x18, 0x26, average, services)?;
        average = sw(&result.memory, at(0x16))?;
        let resistance = sw(&result.memory, npc.wrapping_add(0x56))?;
        word(&mut result, at(0x18), resistance)?;
        damage_resistance(&mut result, frame, 0x18, 0x26, average, services)?;
        average = sw(&result.memory, at(0x16))?;
    }
    move_to(&mut result, 650, 375);
    let attacks = sw(&result.memory, at(0xe))?;
    number(&mut result, attacks, 0, services)?;
    move_to(&mut result, 730, 375);
    number(&mut result, total, 0, services)?;
    move_to(&mut result, 650, 394);
    number(&mut result, average, 0, services)?;
    Ok(result)
}

fn damage_resistance(
    result: &mut DetailPlan,
    frame: u32,
    work_offset: u32,
    other_offset: u32,
    average: i16,
    services: &mut (impl DetailServices + ?Sized),
) -> Result<(), RuleError> {
    let work = frame.wrapping_sub(work_offset);
    let other = frame.wrapping_sub(other_offset);
    let single = frame.wrapping_sub(0xc);
    let constant = frame.wrapping_sub(0x1c);
    sane(result, 0x200e, Some(work), work, services)?;
    sane(result, 0x1010, Some(work), work, services)?;
    let value = result.memory.long(work)?;
    long(result, single, value)?;
    long(result, constant, 0x42c80000)?;
    sane(result, 0x100e, Some(single), work, services)?;
    sane(result, 0x1006, Some(constant), work, services)?;
    sane(result, 0x1010, Some(work), single, services)?;
    long(result, work, 0x3fff8000)?;
    long(result, work.wrapping_add(4), 0)?;
    word(result, work.wrapping_add(8), 0)?;
    sane(result, 0x1002, Some(single), work, services)?;
    sane(result, 0x1010, Some(work), constant, services)?;
    let value = result.memory.long(constant)?;
    long(result, single, value)?;
    word(result, work, average)?;
    sane(result, 0x200e, Some(work), work, services)?;
    sane(result, 0x1010, Some(work), work, services)?;
    sane(result, 0x100e, Some(single), other, services)?;
    sane(result, 0x1004, Some(work), other, services)?;
    sane(result, 0x1010, Some(other), single, services)?;
    long(result, constant, 0x3f000000)?;
    let output = frame.wrapping_sub(0x18);
    sane(result, 0x100e, Some(single), output, services)?;
    sane(result, 0x1000, Some(constant), output, services)?;
    sane(result, 0x1010, Some(output), single, services)?;
    sane(result, 0x100e, Some(single), output, services)?;
    sane(result, 0x16, None, output, services)?;
    sane(result, 0x2810, Some(output), output, services)
}
