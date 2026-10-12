//! CODE 1 item displays with recovered child bodies and addressed memory writes.

use crate::RuleError;
use crate::recovered_drawing::{self, DrawingContext, DrawingEffect, DrawingServices};
use crate::recovered_files::{
    self, FileBuffer, FileContext, FileEffect, FileObservation, FileReply, FileRequest,
};
use crate::recovered_geometry::BodyRect;
use crate::recovered_records::{self, ITEM_TABLE_ADDRESSES, ItemTables, RecordWrite};
use crate::recovered_rendering::RenderingMemory;
use crate::recovered_text::{self, NumericServices, PascalString, TextContext, TextEffect};
use centurions_content::{ITEM_BYTES, WARRIOR_BYTES};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ItemUiRequest {
    GetDialogItem {
        dialog: u32,
        item: i16,
        item_type: u32,
        handle: u32,
        rectangle: u32,
    },
    GetDialogItemText {
        handle: u32,
        destination: u32,
    },
    OffsetRect {
        pointer: u32,
        horizontal: i16,
        vertical: i16,
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
pub enum ItemUiReply {
    /// Actual ordered writes of the external Toolbox call, including aliases.
    Writes(Vec<RecordWrite>),
    Pascal(PascalString),
    File(FileReply),
}

/// All methods supply observations; they must not execute UI plans or host I/O.
pub trait ItemUiServices: DrawingServices + NumericServices {
    fn observe(
        &mut self,
        request: &ItemUiRequest,
        memory: &RenderingMemory,
    ) -> Result<ItemUiReply, RuleError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ItemUiEffect {
    Write(RecordWrite),
    Drawing(DrawingEffect),
    TextMode(i16),
    /// Preserve the complete source WORD slot rather than infer a byte ABI.
    TextFaceSlot(u16),
    Observation {
        request: ItemUiRequest,
        reply: ItemUiReply,
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
    SetDialogItemText {
        handle: u32,
        source: u32,
        text: PascalString,
    },
    File(FileEffect),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemUiPlan {
    pub memory: RenderingMemory,
    pub effects: Vec<ItemUiEffect>,
    /// printstat is native void and does not assign a defined D0 result.
    pub native_return: Option<i32>,
}

fn plan(memory: &RenderingMemory, native_return: Option<i32>) -> ItemUiPlan {
    ItemUiPlan {
        memory: memory.clone(),
        effects: vec![],
        native_return,
    }
}

fn read(memory: &RenderingMemory, address: u32, length: usize) -> Result<Vec<u8>, RuleError> {
    (0..length)
        .map(|index| memory.byte(address.wrapping_add(index as u32)))
        .collect()
}

fn signed_word(memory: &RenderingMemory, address: u32) -> Result<i16, RuleError> {
    Ok(memory.word(address)? as i16)
}

fn signed_byte(memory: &RenderingMemory, address: u32) -> Result<i16, RuleError> {
    Ok(i16::from(memory.byte(address)? as i8))
}

fn write(result: &mut ItemUiPlan, address: u32, bytes: &[u8]) -> Result<(), RuleError> {
    read(&result.memory, address, bytes.len())?;
    result.memory.insert_region(address, bytes);
    result.effects.push(ItemUiEffect::Write(RecordWrite {
        address,
        bytes: bytes.to_vec(),
    }));
    Ok(())
}

fn word(result: &mut ItemUiPlan, address: u32, value: i16) -> Result<(), RuleError> {
    write(result, address, &value.to_be_bytes())
}

fn draw(result: &mut ItemUiPlan, effect: DrawingEffect) {
    result.effects.push(ItemUiEffect::Drawing(effect));
}

fn move_to(result: &mut ItemUiPlan, horizontal: i16, vertical: i16) {
    draw(
        result,
        DrawingEffect::MoveTo {
            horizontal,
            vertical,
        },
    );
}

fn literal(result: &mut ItemUiPlan, address: u32) -> Result<(), RuleError> {
    let text = pascal(&result.memory, address)?;
    draw(result, DrawingEffect::DrawString(text));
    Ok(())
}

fn pascal(memory: &RenderingMemory, address: u32) -> Result<PascalString, RuleError> {
    let length = usize::from(memory.byte(address)?);
    PascalString::from_raw(&read(memory, address, length + 1)?)
}

fn c_string(memory: &RenderingMemory, address: u32) -> Result<Vec<u8>, RuleError> {
    let mut result = vec![];
    for index in 0..=memory.bytes.len() {
        let byte = memory.byte(address.wrapping_add(index as u32))?;
        result.push(byte);
        if byte == 0 {
            return Ok(result);
        }
    }
    Err(RuleError::Invalid(
        "unterminated item-UI C string within supplied memory".into(),
    ))
}

fn observe(
    result: &mut ItemUiPlan,
    request: ItemUiRequest,
    services: &mut impl ItemUiServices,
) -> Result<ItemUiReply, RuleError> {
    let reply = services.observe(&request, &result.memory)?;
    result.effects.push(ItemUiEffect::Observation {
        request,
        reply: reply.clone(),
    });
    Ok(reply)
}

fn observe_writes(
    result: &mut ItemUiPlan,
    request: ItemUiRequest,
    services: &mut impl ItemUiServices,
) -> Result<(), RuleError> {
    match observe(result, request, services)? {
        ItemUiReply::Writes(writes) => {
            for effect in writes {
                write(result, effect.address, &effect.bytes)?;
            }
            Ok(())
        }
        _ => Err(RuleError::Invalid(
            "missing item-UI Toolbox output writes".into(),
        )),
    }
}

fn copy_record(
    result: &mut ItemUiPlan,
    source: u32,
    destination: u32,
    length: usize,
) -> Result<(), RuleError> {
    for index in (0..length / 4 * 4).step_by(4) {
        let bytes = read(&result.memory, source.wrapping_add(index as u32), 4)?;
        write(result, destination.wrapping_add(index as u32), &bytes)?;
    }
    let rest = length % 4;
    if rest != 0 {
        let offset = (length / 4 * 4) as u32;
        let bytes = read(&result.memory, source.wrapping_add(offset), rest)?;
        write(result, destination.wrapping_add(offset), &bytes)?;
    }
    Ok(())
}

fn load_item(result: &mut ItemUiPlan, id: i16, ammunition: bool) -> Result<(), RuleError> {
    let destination = if ammunition { 0x4536d2 } else { 0x45363c };
    let current: [u8; ITEM_BYTES] = read(&result.memory, destination, ITEM_BYTES)?
        .try_into()
        .expect("item record");
    let absolute = id.wrapping_abs();
    let family = absolute / 200;
    let mut backing: [Vec<u8>; 5] = std::array::from_fn(|_| Vec::new());
    if let Ok(family) = usize::try_from(family)
        && family < backing.len()
    {
        let offset = (absolute % 200) as usize * ITEM_BYTES;
        let bytes = read(
            &result.memory,
            ITEM_TABLE_ADDRESSES[family].wrapping_add(offset as u32),
            ITEM_BYTES,
        )?;
        // The existing loader reads only this selected record, never the prefix.
        backing[family].resize(offset + ITEM_BYTES, 0);
        backing[family][offset..].copy_from_slice(&bytes);
    }
    let tables = ItemTables {
        families: [
            &backing[0],
            &backing[1],
            &backing[2],
            &backing[3],
            &backing[4],
        ],
    };
    let loaded = if ammunition {
        recovered_records::loadammo(&tables, &current, id)?
    } else {
        recovered_records::loaditem(&tables, &current, id)?
    };
    for effect in loaded.writes {
        write(result, effect.address, &effect.bytes)?;
    }
    Ok(())
}

fn apply_text(
    result: &mut ItemUiPlan,
    context: &TextContext,
    effects: Vec<TextEffect>,
) -> Result<(), RuleError> {
    for effect in effects {
        match effect {
            TextEffect::NumberToString(value) => {
                result.effects.push(ItemUiEffect::NumberToString {
                    value,
                    destination: 0x479ed8,
                    result: context.scratch.clone(),
                });
                write(result, 0x479ed8, context.scratch.raw())?;
            }
            TextEffect::DrawString(text) => draw(result, DrawingEffect::DrawString(text)),
            TextEffect::GetDialogItem { .. } => (),
            TextEffect::SetDialogItemText { text, .. } => {
                result.effects.push(ItemUiEffect::SetDialogItemText {
                    handle: result.memory.long(0x4434dc)?,
                    source: 0x479ed8,
                    text,
                });
            }
            _ => {
                return Err(RuleError::Invalid(
                    "unexpected recovered item-UI text child effect".into(),
                ));
            }
        }
    }
    Ok(())
}

fn number(
    result: &mut ItemUiPlan,
    value: i16,
    plus: bool,
    services: &mut impl ItemUiServices,
) -> Result<(), RuleError> {
    let mut context = TextContext::default();
    let effects = if plus {
        recovered_text::string_plus(&mut context, value, services)?
    } else {
        recovered_text::string(&mut context, value, services)?
    };
    apply_text(result, &context, effects)
}

fn dialog_item(
    result: &mut ItemUiPlan,
    item: i16,
    item_type: u32,
    handle: u32,
    rectangle: u32,
    services: &mut impl ItemUiServices,
) -> Result<(), RuleError> {
    let dialog = result.memory.long(0x4394b8)?;
    observe_writes(
        result,
        ItemUiRequest::GetDialogItem {
            dialog,
            item,
            item_type,
            handle,
            rectangle,
        },
        services,
    )
}

fn dialog_number(
    result: &mut ItemUiPlan,
    item: i16,
    value: i16,
    services: &mut impl ItemUiServices,
) -> Result<(), RuleError> {
    dialog_item(result, item, 0x4434b0, 0x4434dc, 0x4434ba, services)?;
    let mut context = TextContext::default();
    let effects = recovered_text::dialog_number(&mut context, item, value, services)?;
    apply_text(result, &context, effects)
}

fn dialog_string(
    result: &mut ItemUiPlan,
    item: i16,
    address: u32,
    services: &mut impl ItemUiServices,
) -> Result<(), RuleError> {
    dialog_item(result, item, 0x4434b0, 0x4434dc, 0x4434ba, services)?;
    let text = pascal(&result.memory, address)?;
    for effect in recovered_text::dialog_string(item, &text) {
        if let TextEffect::SetDialogItemText { text, .. } = effect {
            result.effects.push(ItemUiEffect::SetDialogItemText {
                handle: result.memory.long(0x4434dc)?,
                source: address,
                text,
            });
        }
    }
    Ok(())
}

fn indexed_string(
    result: &mut ItemUiPlan,
    resource_id: i16,
    index: i16,
    services: &mut impl ItemUiServices,
) -> Result<(), RuleError> {
    let text = services.indexed_string(resource_id, index)?;
    result.effects.push(ItemUiEffect::IndexedString {
        resource_id,
        index,
        destination: 0x479ed8,
        result: text.clone(),
    });
    write(result, 0x479ed8, text.raw())
}

fn icon_context(rect: BodyRect) -> DrawingContext {
    // ploticon3 reads none of these unrelated fields; its scratch starts by clearing.
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
    result: &mut ItemUiPlan,
    id: i16,
    rectangle: BodyRect,
    services: &mut impl ItemUiServices,
) -> Result<(), RuleError> {
    let mut context = icon_context(rectangle);
    let child = recovered_drawing::ploticon3(&mut context, id, rectangle, services)?;
    for effect in child.effects {
        match effect {
            DrawingEffect::Write(effect) => write(result, effect.address, &effect.bytes)?,
            effect => draw(result, effect),
        }
    }
    Ok(())
}

fn picture(
    result: &mut ItemUiPlan,
    id: i16,
    rectangle: BodyRect,
    services: &mut impl ItemUiServices,
) -> Result<(), RuleError> {
    for effect in recovered_drawing::pict(id, rectangle, services)? {
        draw(result, effect);
    }
    Ok(())
}

fn offset_rect(
    result: &mut ItemUiPlan,
    pointer: u32,
    horizontal: i16,
    vertical: i16,
    services: &mut impl ItemUiServices,
) -> Result<(), RuleError> {
    observe_writes(
        result,
        ItemUiRequest::OffsetRect {
            pointer,
            horizontal,
            vertical,
        },
        services,
    )
}

fn file(
    result: &mut ItemUiPlan,
    request: FileRequest,
    services: &mut impl ItemUiServices,
) -> Result<FileReply, RuleError> {
    match observe(result, ItemUiRequest::File(request), services)? {
        ItemUiReply::File(reply) => Ok(reply),
        _ => Err(RuleError::Invalid(
            "missing item-UI file service observation".into(),
        )),
    }
}

fn filename(
    result: &mut ItemUiPlan,
    name_address: u32,
    capacity: usize,
    services: &mut impl ItemUiServices,
) -> Result<(), RuleError> {
    let variant = result.memory.byte(0x44274c)?;
    let mut context = FileContext {
        scenario_directory: c_string(&result.memory, 0x44260c)?,
        variant,
        filename: read(&result.memory, 0x4389a4, capacity)?,
        ..FileContext::default()
    };
    let name = c_string(&result.memory, name_address)?;
    let converted = services.number_to_string(i32::from(variant))?;
    let observation = FileObservation {
        request: FileRequest::NumberToString(i32::from(variant)),
        reply: FileReply::Pascal(converted),
    };
    let child = recovered_files::get_filename(&mut context, &name, &[observation])?;
    for effect in child.effects {
        if let FileEffect::Write {
            destination: FileBuffer::Filename,
            offset,
            bytes,
        } = &effect
        {
            write(result, 0x4389a4_u32.wrapping_add(*offset as u32), bytes)?;
        } else {
            result.effects.push(ItemUiEffect::File(effect));
        }
    }
    Ok(())
}

fn argument(result: &ItemUiPlan, frame: u32, offset: u32) -> Result<i16, RuleError> {
    signed_word(&result.memory, frame.wrapping_add(offset))
}

fn quantity_position(result: &mut ItemUiPlan, frame: u32) -> Result<(), RuleError> {
    let horizontal = argument(result, frame, 16)?.wrapping_add(1);
    let vertical = argument(result, frame, 10)?.wrapping_add(8);
    move_to(result, horizontal, vertical);
    Ok(())
}

fn stat_position(result: &mut ItemUiPlan, frame: u32, offset: i16) -> Result<(), RuleError> {
    let horizontal = argument(result, frame, 12)?.wrapping_sub(1);
    let vertical = argument(result, frame, 14)?.wrapping_add(offset);
    move_to(result, horizontal, vertical);
    Ok(())
}

fn category_icon(kind: i16) -> i16 {
    match kind {
        8 | 9 => 0x83d,
        11 | 12 => 0x83e,
        14 => 0x841,
        15 => 0x835,
        16 | 18 => 0x837,
        17 | 21 => 0x836,
        19 => 0x834,
        23 => 0x83c,
        25 => 0x839,
        27 => 0x83a,
        29 => 0x83b,
        31 => 0x840,
        32 => 0x83f,
        _ => 0x838,
    }
}

fn category_label(kind: i16) -> Option<u32> {
    match kind {
        1 => Some(0x4033ec),
        2 => Some(0x4033e4),
        3 => Some(0x4033dc),
        4 => Some(0x4033d4),
        6 => Some(0x4033cc),
        7 => Some(0x4033c6),
        8 => Some(0x4033c0),
        9 => Some(0x4033ba),
        11 => Some(0x4033b0),
        12 => Some(0x4033a6),
        14 => Some(0x40339c),
        15 => Some(0x403394),
        16 => Some(0x40338a),
        17 => Some(0x403386),
        18 => Some(0x40337c),
        19 => Some(0x403372),
        21 => Some(0x40336a),
        23 => Some(0x403362),
        24 => Some(0x403356),
        25 => Some(0x40334e),
        27 => Some(0x40334a),
        29 => Some(0x403342),
        31 => Some(0x40333a),
        32 => Some(0x403334),
        _ => None,
    }
}

fn signed_pack_byte(result: &ItemUiPlan, pack: u32, offset: u32) -> Result<i16, RuleError> {
    signed_byte(&result.memory, pack.wrapping_add(offset))
}

fn numeric_suffix(
    result: &mut ItemUiPlan,
    value: i16,
    address: u32,
    services: &mut impl ItemUiServices,
) -> Result<(), RuleError> {
    if value != 0 {
        number(result, value, true, services)?;
        literal(result, address)?;
    }
    Ok(())
}

/// showp, 0x12dc58. The native six WORD arguments reside at A6+8 through A6+18.
pub fn show_pack(
    memory: &RenderingMemory,
    frame: u32,
    maximum_markers: usize,
    services: &mut impl ItemUiServices,
) -> Result<ItemUiPlan, RuleError> {
    let mut result = plan(memory, Some(0));
    let slot = result.memory.word(frame.wrapping_add(8))?;
    let markers = argument(&result, frame, 18)?;
    let pack = 0x41fc74_u32.wrapping_add(u32::from(slot) * 22);
    if result.memory.word(pack.wrapping_add(4))? != 0 {
        word(&mut result, 0x4434b6, 0)?;
        let kind = signed_word(&result.memory, 0x45365c)?;
        word(&mut result, 0x4434b6, category_icon(kind))?;
        if result.memory.word(0x4434b6)? != 0 {
            let left = argument(&result, frame, 16)?.wrapping_add(1);
            word(&mut result, 0x4471f4, left)?;
            let top = argument(&result, frame, 10)?.wrapping_add(14);
            word(&mut result, 0x4471f2, top)?;
            let right = signed_word(&result.memory, 0x4471f4)?.wrapping_add(16);
            word(&mut result, 0x4471f8, right)?;
            let bottom = signed_word(&result.memory, 0x4471f2)?.wrapping_add(16);
            word(&mut result, 0x4471f6, bottom)?;
            let rect = result.memory.rect(0x4471f2)?;
            let id = signed_word(&result.memory, 0x4434b6)?;
            icon(&mut result, id, rect, services)?;
            draw(&mut result, DrawingEffect::TextSize(10));
            draw(&mut result, DrawingEffect::ForeColor(33));
            quantity_position(&mut result, frame)?;
            let quantity = signed_word(&result.memory, pack.wrapping_add(2))?;
            number(&mut result, quantity, false, services)?;
        }
    } else if result.memory.word(0x45365e)? != 0 {
        draw(&mut result, DrawingEffect::TextSize(10));
        draw(&mut result, DrawingEffect::ForeColor(33));
        quantity_position(&mut result, frame)?;
        let quantity = signed_word(&result.memory, pack.wrapping_add(2))?;
        number(&mut result, quantity, false, services)?;
        let horizontal = argument(&result, frame, 12)?.wrapping_add(1);
        let vertical = argument(&result, frame, 14)?.wrapping_add(12);
        move_to(&mut result, horizontal, vertical);
        if let Some(address) = category_label(signed_word(&result.memory, 0x45365e)?) {
            literal(&mut result, address)?;
        }
    } else if result.memory.word(pack.wrapping_add(2))? != 0 {
        draw(&mut result, DrawingEffect::TextSize(10));
        draw(&mut result, DrawingEffect::ForeColor(33));
        quantity_position(&mut result, frame)?;
        let quantity = signed_word(&result.memory, pack.wrapping_add(2))?;
        number(&mut result, quantity, false, services)?;
    }

    draw(&mut result, DrawingEffect::TextSize(9));
    if result.memory.byte(0x453693)? != 0 {
        stat_position(&mut result, frame, 12)?;
        draw(&mut result, DrawingEffect::ForeColor(205));
        literal(&mut result, 0x40332c)?;
        stat_position(&mut result, frame, 24)?;
        let value = signed_byte(&result.memory, 0x45368e)?
            .wrapping_add(signed_byte(&result.memory, 0x45368d)?);
        if value != 0 {
            literal(&mut result, 0x403324)?;
        }
        let value = signed_byte(&result.memory, 0x453692)?
            .wrapping_add(signed_word(&result.memory, 0x45369a)?);
        if value != 0 {
            literal(&mut result, 0x40331c)?;
        }
        for (address, label) in [
            (0x453691, 0x403312),
            (0x453690, 0x403308),
            (0x45368f, 0x4032fe),
            (0x45368c, 0x4032f2),
            (0x45368b, 0x4032ec),
        ] {
            if result.memory.byte(address)? != 0 {
                literal(&mut result, label)?;
            }
        }
        let value = signed_byte(&result.memory, 0x45368a)?
            .wrapping_add(signed_word(&result.memory, 0x453694)?);
        if value != 0 {
            literal(&mut result, 0x4032e2)?;
        }
        for (address, label) in [(0x453696, 0x4032da), (0x4536d0, 0x4032d0)] {
            if result.memory.word(address)? != 0 {
                literal(&mut result, label)?;
            }
        }
        stat_position(&mut result, frame, 36)?;
        draw(&mut result, DrawingEffect::ForeColor(341));
        let mut total = 0_i16;
        for address in [
            0x453692, 0x45368d, 0x453691, 0x453690, 0x45368f, 0x45368c, 0x45368b, 0x45368a,
        ] {
            total = total.wrapping_add(signed_byte(&result.memory, address)?);
        }
        total = total.wrapping_add(signed_word(&result.memory, 0x453696)?);
        total = total.wrapping_add(signed_word(&result.memory, 0x4536d0)?);
        number(&mut result, total, true, services)?;
        if result.memory.word(0x4536d0)? != 0 {
            literal(&mut result, 0x4032ce)?;
        }
    } else {
        stat_position(&mut result, frame, 10)?;
        draw(&mut result, DrawingEffect::ForeColor(205));
        let value =
            signed_pack_byte(&result, pack, 11)?.wrapping_add(signed_pack_byte(&result, pack, 15)?);
        numeric_suffix(&mut result, value, 0x4032c8, services)?;
        stat_position(&mut result, frame, 20)?;
        draw(&mut result, DrawingEffect::ForeColor(409));
        let value = signed_pack_byte(&result, pack, 18)?;
        numeric_suffix(&mut result, value, 0x4032c2, services)?;
        stat_position(&mut result, frame, 30)?;
        let value = signed_pack_byte(&result, pack, 20)?;
        if value != 0 {
            draw(&mut result, DrawingEffect::ForeColor(341));
            numeric_suffix(&mut result, value, 0x4032bc, services)?;
        }
        stat_position(&mut result, frame, 20)?;
        draw(&mut result, DrawingEffect::ForeColor(33));
        let value = signed_pack_byte(&result, pack, 19)?
            .wrapping_add(signed_word(&result.memory, 0x45369a)?);
        numeric_suffix(&mut result, value, 0x4032b6, services)?;
        stat_position(&mut result, frame, 30)?;
        draw(&mut result, DrawingEffect::ForeColor(341));
        let value = signed_pack_byte(&result, pack, 14)?
            .wrapping_add(signed_word(&result.memory, 0x453694)?);
        numeric_suffix(&mut result, value, 0x4032b0, services)?;
        stat_position(&mut result, frame, 40)?;
        draw(&mut result, DrawingEffect::ForeColor(205));
        let value = signed_pack_byte(&result, pack, 16)?
            .wrapping_add(signed_byte(&result.memory, 0x45368f)?)
            .wrapping_add(signed_byte(&result.memory, 0x45363f)?);
        numeric_suffix(&mut result, value, 0x4032a6, services)?;
        stat_position(&mut result, frame, 50)?;
        draw(&mut result, DrawingEffect::ForeColor(409));
        let value = signed_pack_byte(&result, pack, 17)?
            .wrapping_add(signed_byte(&result.memory, 0x453690)?)
            .wrapping_add(signed_byte(&result.memory, 0x453640)?);
        numeric_suffix(&mut result, value, 0x40329c, services)?;
    }

    if markers != 0 {
        let left = argument(&result, frame, 12)?.wrapping_sub(2);
        word(&mut result, 0x4434bc, left)?;
        let right = signed_word(&result.memory, 0x4434bc)?.wrapping_add(8);
        word(&mut result, 0x4434c0, right)?;
        let top = argument(&result, frame, 10)?.wrapping_sub(10);
        word(&mut result, 0x4434ba, top)?;
        let bottom = signed_word(&result.memory, 0x4434ba)?.wrapping_add(8);
        word(&mut result, 0x4434be, bottom)?;
        let count = usize::from(markers.max(0) as u16);
        if count > maximum_markers {
            return Err(RuleError::Invalid(
                "showp marker count exceeds supplied processing budget".into(),
            ));
        }
        for index in 0..count {
            if [7, 14, 21, 28].contains(&index) {
                offset_rect(&mut result, 0x4434ba, -63, 8, services)?;
            }
            let rect = result.memory.rect(0x4434ba)?;
            icon(&mut result, 208, rect, services)?;
            offset_rect(&mut result, 0x4434ba, 9, 0, services)?;
        }
    }
    Ok(result)
}

/// updateshopwings, 0x133886. Full warrior copies occur before the display gate.
pub fn update_shop_wings(
    memory: &RenderingMemory,
    frame: u32,
    services: &mut impl ItemUiServices,
) -> Result<ItemUiPlan, RuleError> {
    let mut result = plan(memory, Some(0));
    let right = result.memory.word(frame.wrapping_add(10))?;
    let left = result.memory.word(frame.wrapping_add(8))?;
    let left_source = if left == 0xfffe {
        0x44721c
    } else {
        0x448aa0_u32.wrapping_add(u32::from(left) * WARRIOR_BYTES as u32)
    };
    let left_copy = frame.wrapping_sub(0x1884);
    copy_record(&mut result, left_source, left_copy, WARRIOR_BYTES)?;
    let right_copy = left_copy.wrapping_add(WARRIOR_BYTES as u32);
    if right != 0xffff {
        let source = if right == 0xfffe {
            0x447e5e
        } else {
            0x448aa0_u32.wrapping_add(u32::from(right) * WARRIOR_BYTES as u32)
        };
        copy_record(&mut result, source, right_copy, WARRIOR_BYTES)?;
    }
    if result.memory.word(0x439582)? == 0 {
        return Ok(result);
    }
    let rect_address = frame.wrapping_sub(0x188c);
    word(&mut result, rect_address, 387)?;
    let bottom = signed_word(&result.memory, rect_address)?.wrapping_add(73);
    word(&mut result, rect_address.wrapping_add(4), bottom)?;
    let font = signed_word(&result.memory, 0x402f3c)?;
    draw(&mut result, DrawingEffect::TextFont(font));
    result.effects.push(ItemUiEffect::TextFaceSlot(0));
    draw(&mut result, DrawingEffect::TextSize(12));
    draw(&mut result, DrawingEffect::ForeColor(69));
    word(&mut result, rect_address.wrapping_add(2), 0)?;
    word(&mut result, rect_address.wrapping_add(6), 80)?;
    let rect = result.memory.rect(rect_address)?;
    picture(&mut result, 227, rect, services)?;
    for (horizontal, vertical, offset) in [(50, 455, 26), (4, 424, 2518), (42, 424, 2520)] {
        move_to(&mut result, horizontal, vertical);
        let value = signed_word(&result.memory, left_copy.wrapping_add(offset))?;
        number(&mut result, value, false, services)?;
    }
    word(&mut result, rect_address.wrapping_add(2), 720)?;
    word(&mut result, rect_address.wrapping_add(6), 800)?;
    let rect = result.memory.rect(rect_address)?;
    picture(&mut result, 227, rect, services)?;
    if right != 0xffff {
        for (horizontal, vertical, offset) in [(768, 455, 26), (724, 424, 2518), (762, 424, 2520)] {
            move_to(&mut result, horizontal, vertical);
            let value = signed_word(&result.memory, right_copy.wrapping_add(offset))?;
            number(&mut result, value, false, services)?;
        }
    }
    Ok(result)
}

/// updateedititem, 0x134ae0. Member and slot use unsigned WORD multiplications.
pub fn update_edit_item(
    memory: &RenderingMemory,
    frame: u32,
    filename_capacity: usize,
    services: &mut impl ItemUiServices,
) -> Result<ItemUiPlan, RuleError> {
    let mut result = plan(memory, Some(0));
    draw(&mut result, DrawingEffect::ForeColor(69));
    let old_id = signed_word(&result.memory, 0x41fc48)?;
    load_item(&mut result, old_id, false)?;
    let slot = result.memory.word(frame.wrapping_add(10))?;
    let member = result.memory.word(frame.wrapping_add(8))?;
    let source = 0x448aa0_u32
        .wrapping_add(u32::from(member) * WARRIOR_BYTES as u32)
        .wrapping_add(u32::from(slot) * 22)
        .wrapping_add(0x132);
    copy_record(&mut result, source, 0x41fc48, 22)?;
    let id = signed_word(&result.memory, 0x41fc48)?;
    load_item(&mut result, id, false)?;
    for (item, address) in [
        (28, 0x41fc55),
        (29, 0x41fc57),
        (30, 0x41fc58),
        (31, 0x41fc53),
        (32, 0x41fc59),
        (33, 0x41fc5a),
        (34, 0x41fc5b),
        (35, 0x41fc56),
        (36, 0x41fc5d),
        (37, 0x41fc5c),
    ] {
        let value = signed_byte(&result.memory, address)?;
        dialog_number(&mut result, item, value, services)?;
    }
    for (item, address) in [(38, 0x41fc4a), (39, 0x41fc4e)] {
        let value = signed_word(&result.memory, address)?;
        dialog_number(&mut result, item, value, services)?;
    }
    let selection_id = signed_word(&result.memory, 0x41fc48)?;
    let selection_base = crate::rules::selection_base(selection_id);
    word(&mut result, 0x4471e8, selection_base)?;
    let resource_id = signed_word(&result.memory, 0x4471e8)?.wrapping_add(1);
    let index = signed_word(&result.memory, 0x41fc48)?
        .wrapping_sub(signed_word(&result.memory, 0x4471e8)?)
        .wrapping_add(1);
    indexed_string(&mut result, resource_id, index, services)?;
    dialog_string(&mut result, 66, 0x479ed8, services)?;
    let mut descriptor = signed_word(&result.memory, 0x4536a2)?;
    let name_address = if signed_word(&result.memory, 0x4536a2)? > 799 {
        descriptor = descriptor.wrapping_sub(800);
        filename(&mut result, 0x40299e, filename_capacity, services)?;
        0x4389a4
    } else {
        0x402986
    };
    let mode = c_string(&result.memory, 0x402f24)?;
    let name = c_string(&result.memory, name_address)?;
    let handle = match file(&mut result, FileRequest::Open { name, mode }, services)? {
        FileReply::Handle(handle) => handle,
        _ => {
            return Err(RuleError::Invalid(
                "missing item description fopen result".into(),
            ));
        }
    };
    write(&mut result, 0x4434a6, &handle.to_be_bytes())?;
    let offset = (i32::from(descriptor) as u32).wrapping_shl(8);
    let seek_handle = result.memory.long(0x4434a6)?;
    match file(
        &mut result,
        FileRequest::Seek {
            handle: seek_handle,
            offset,
            origin: 0,
        },
        services,
    )? {
        FileReply::Ignored | FileReply::Integer(_) | FileReply::Error(_) => (),
        _ => {
            return Err(RuleError::Invalid(
                "unexpected discarded item description fseek result".into(),
            ));
        }
    }
    let request = ItemUiRequest::ReadFile {
        handle: result.memory.long(0x4434a6)?,
        size: 256,
        count: 1,
        destination: 0x479ed8,
    };
    match observe(&mut result, request, services)? {
        ItemUiReply::File(FileReply::Read { bytes_written, .. }) if bytes_written.len() <= 256 => {
            write(&mut result, 0x479ed8, &bytes_written)?
        }
        _ => {
            return Err(RuleError::Invalid(
                "missing bounded item description fread output".into(),
            ));
        }
    }
    let handle = result.memory.long(0x4434a6)?;
    match file(&mut result, FileRequest::Close(handle), services)? {
        FileReply::Ignored | FileReply::Integer(_) | FileReply::Error(_) => (),
        _ => {
            return Err(RuleError::Invalid(
                "unexpected discarded item description fclose result".into(),
            ));
        }
    }
    dialog_string(&mut result, 65, 0x479ed8, services)?;
    dialog_item(&mut result, 3, 0x4434b0, 0x4434dc, 0x4434ba, services)?;
    let rect = result.memory.rect(0x4434ba)?;
    draw(&mut result, DrawingEffect::EraseRect(rect));
    if result.memory.byte(0x41fc51)? != 0 {
        let rect = result.memory.rect(0x4434ba)?;
        icon(&mut result, 176, rect, services)?;
    }
    dialog_item(&mut result, 64, 0x4434b0, 0x4434dc, 0x4434ba, services)?;
    let rect = result.memory.rect(0x4434ba)?;
    let id = signed_word(&result.memory, 0x453646)?;
    icon(&mut result, id, rect, services)?;
    Ok(result)
}

fn stat_text(
    result: &mut ItemUiPlan,
    color: i32,
    prefix: u32,
    value: i16,
    plus: bool,
    suffix: Option<u32>,
    services: &mut impl ItemUiServices,
) -> Result<(), RuleError> {
    draw(result, DrawingEffect::ForeColor(color));
    literal(result, prefix)?;
    number(result, value, plus, services)?;
    if let Some(suffix) = suffix {
        literal(result, suffix)?;
    }
    Ok(())
}

/// showitemstats, 0x157758. Ammunition modifies the existing item work in place.
pub fn show_item_stats(
    memory: &RenderingMemory,
    services: &mut impl ItemUiServices,
) -> Result<ItemUiPlan, RuleError> {
    let mut result = plan(memory, Some(0));
    let port = result.memory.long(0x3fe302)?;
    let font = signed_word(&result.memory, port.wrapping_add(0x44))?;
    let port = result.memory.long(0x3fe302)?;
    let face = result.memory.byte(port.wrapping_add(0x46))?;
    let port = result.memory.long(0x3fe302)?;
    let size = signed_word(&result.memory, port.wrapping_add(0x4a))?;
    let port = result.memory.long(0x3fe302)?;
    let mode = signed_word(&result.memory, port.wrapping_add(0x48))?;
    draw(&mut result, DrawingEffect::TextSize(10));
    draw(&mut result, DrawingEffect::TextFont(10));
    result.effects.push(ItemUiEffect::TextFaceSlot(0));
    if result.memory.word(0x41fc4c)? != 0 {
        let id = signed_word(&result.memory, 0x41fc4c)?;
        load_item(&mut result, id, true)?;
        let value = result
            .memory
            .byte(0x45368d)?
            .wrapping_add(result.memory.byte(0x453723)?);
        write(&mut result, 0x45368d, &[value])?;
        for (destination, source) in [
            (0x45367a, 0x453710),
            (0x453664, 0x4536fa),
            (0x4536c8, 0x45375e),
            (0x45367c, 0x453712),
            (0x45367e, 0x453714),
            (0x453680, 0x453716),
            (0x453682, 0x453718),
            (0x453684, 0x45371a),
            (0x453686, 0x45371c),
            (0x453678, 0x45370e),
        ] {
            let value = signed_word(&result.memory, destination)?
                .wrapping_add(signed_word(&result.memory, source)?);
            word(&mut result, destination, value)?;
        }
    }
    let mut damage =
        signed_word(&result.memory, 0x45367a)?.wrapping_add(signed_word(&result.memory, 0x453664)?);
    for address in [
        0x4536c8, 0x45367c, 0x45367e, 0x453680, 0x453682, 0x453684, 0x453686,
    ] {
        let value = signed_word(&result.memory, address)?;
        if value > 0 {
            damage = damage.wrapping_add(value);
        }
    }
    for address in [0x453688, 0x453652] {
        let multiplier = signed_word(&result.memory, address)?;
        if multiplier != 0 {
            damage = (i32::from(damage) * i32::from(multiplier)) as i16;
        }
    }
    let horizontal = signed_word(&result.memory, 0x4471f4)?.wrapping_add(42);
    let vertical = signed_word(&result.memory, 0x4471f2)?.wrapping_add(30);
    move_to(&mut result, horizontal, vertical);
    draw(&mut result, DrawingEffect::ForeColor(33));
    draw(&mut result, DrawingEffect::TextSize(10));
    result.effects.push(ItemUiEffect::TextMode(1));
    result.effects.push(ItemUiEffect::TextFaceSlot(0));
    if result.memory.byte(0x453693)? != 0 {
        draw(&mut result, DrawingEffect::ForeColor(409));
        literal(&mut result, 0x400f7c)?;
        let index = signed_byte(&result.memory, 0x453693)?.wrapping_add(1);
        indexed_string(&mut result, -11000, index, services)?;
        literal(&mut result, 0x479ed8)?;
        literal(&mut result, 0x400f78)?;
    }
    let value = signed_word(&result.memory, 0x453688)?;
    if value != 0 {
        stat_text(
            &mut result,
            409,
            0x400f74,
            value,
            false,
            Some(0x400f72),
            services,
        )?;
    }
    if damage != 0 {
        stat_text(&mut result, 205, 0x400f68, damage, false, None, services)?;
    }
    let value = signed_byte(&result.memory, 0x41fc5a)?;
    if value != 0 {
        stat_text(
            &mut result,
            409,
            0x400f64,
            value,
            false,
            Some(0x400f5e),
            services,
        )?;
    }
    let value = signed_word(&result.memory, 0x4536d0)?;
    if value != 0 {
        stat_text(
            &mut result,
            33,
            0x400f52,
            value,
            false,
            Some(0x400f4e),
            services,
        )?;
    }
    let value = signed_byte(&result.memory, 0x41fc57)?;
    if value != 0 {
        stat_text(
            &mut result,
            341,
            0x400f48,
            value,
            true,
            Some(0x400f44),
            services,
        )?;
    }
    for (first, second, label, color, plus) in [
        (0x45363f, 0x41fc58, 0x400f3c, 33, true),
        (0x453640, 0x41fc59, 0x400f34, 33, true),
    ] {
        let value =
            signed_byte(&result.memory, first)?.wrapping_add(signed_byte(&result.memory, second)?);
        if value != 0 {
            stat_text(
                &mut result,
                color,
                label,
                value,
                plus,
                Some(0x400f72),
                services,
            )?;
        }
    }
    let value =
        signed_byte(&result.memory, 0x41fc5c)?.wrapping_add(signed_word(&result.memory, 0x453678)?);
    if value != 0 {
        stat_text(
            &mut result,
            409,
            0x400f2e,
            value,
            true,
            Some(0x400f72),
            services,
        )?;
    }
    let value =
        signed_byte(&result.memory, 0x41fc5b)?.wrapping_add(signed_word(&result.memory, 0x45369a)?);
    if value > 0 {
        stat_text(
            &mut result,
            409,
            0x400f28,
            value,
            false,
            Some(0x400f72),
            services,
        )?;
    }
    if signed_word(&result.memory, 0x453694)? > 0 {
        let value = signed_byte(&result.memory, 0x41fc56)?
            .wrapping_add(signed_word(&result.memory, 0x453694)?);
        stat_text(
            &mut result,
            409,
            0x400f22,
            value,
            true,
            Some(0x400f72),
            services,
        )?;
    }
    if result.memory.byte(0x41fc5d)? != 0 || result.memory.word(0x453698)? != 0 {
        let value = signed_byte(&result.memory, 0x41fc5d)?
            .wrapping_add(signed_word(&result.memory, 0x453698)?);
        stat_text(
            &mut result,
            341,
            0x400f1c,
            value,
            true,
            Some(0x400f72),
            services,
        )?;
    }
    let mut negative = false;
    for address in [0x45367c, 0x45367e, 0x453680, 0x453682, 0x453684, 0x453686] {
        if signed_word(&result.memory, address)? < 0 {
            negative = true;
            break;
        }
    }
    if negative {
        draw(&mut result, DrawingEffect::ForeColor(341));
        literal(&mut result, 0x400f14)?;
        for (address, label) in [
            (0x4536c8, 0x400f10),
            (0x45367c, 0x400f0c),
            (0x45367e, 0x400f08),
            (0x453680, 0x400f04),
            (0x453682, 0x400f00),
            (0x453684, 0x400efc),
            (0x453686, 0x400ef8),
        ] {
            if signed_word(&result.memory, address)? < 0 {
                literal(&mut result, label)?;
            }
        }
    }
    draw(&mut result, DrawingEffect::ForeColor(33));
    draw(&mut result, DrawingEffect::TextFont(font));
    result
        .effects
        .push(ItemUiEffect::TextFaceSlot(u16::from(face)));
    draw(&mut result, DrawingEffect::TextSize(size));
    result.effects.push(ItemUiEffect::TextMode(mode));
    Ok(result)
}

fn copy_substring(
    result: &mut ItemUiPlan,
    source: u32,
    destination: u32,
    first: i16,
    last: i16,
) -> Result<(), RuleError> {
    let source_length = usize::from(result.memory.byte(source)?) + 1;
    let source_bytes = read(&result.memory, source, source_length)?;
    let count = if first <= last {
        usize::try_from(last - first + 1).expect("bounded Pascal range")
    } else {
        0
    };
    let mut destination_bytes = read(&result.memory, destination, count + 1)?;
    recovered_text::copy_part_of_string(&mut destination_bytes, &source_bytes, first, last)?;
    for index in 1..=count {
        write(
            result,
            destination.wrapping_add(index as u32),
            &destination_bytes[index..index + 1],
        )?;
    }
    write(result, destination, &destination_bytes[..1])
}

/// printstat, 0x146aa2. Each CR is included in its substring; final draw is unconditional.
pub fn print_stat(
    memory: &RenderingMemory,
    frame: u32,
    services: &mut impl ItemUiServices,
) -> Result<ItemUiPlan, RuleError> {
    let mut result = plan(memory, None);
    let mut vertical = argument(&result, frame, 10)?;
    let item = argument(&result, frame, 8)?;
    dialog_item(
        &mut result,
        item,
        frame.wrapping_sub(0x20a),
        frame.wrapping_sub(0x208),
        frame.wrapping_sub(0x212),
        services,
    )?;
    let handle = result.memory.long(frame.wrapping_sub(0x208))?;
    let source = frame.wrapping_sub(0x100);
    let destination = frame.wrapping_sub(0x200);
    let text = match observe(
        &mut result,
        ItemUiRequest::GetDialogItemText {
            handle,
            destination: source,
        },
        services,
    )? {
        ItemUiReply::Pascal(text) => text,
        _ => {
            return Err(RuleError::Invalid(
                "missing printstat GetIText Pascal output".into(),
            ));
        }
    };
    write(&mut result, source, text.raw())?;
    let length = i16::from(result.memory.byte(source)?);
    word(&mut result, frame.wrapping_sub(0x204), length)?;
    let mut first = 1_i16;
    let mut index = 1_i16;
    while index <= signed_word(&result.memory, frame.wrapping_sub(0x204))? {
        let byte = result.memory.byte(source.wrapping_add(index as u32))?;
        write(&mut result, frame.wrapping_sub(0x201), &[byte])?;
        if result.memory.byte(frame.wrapping_sub(0x201))? == 13 {
            copy_substring(&mut result, source, destination, first, index)?;
            first = index.wrapping_add(1);
            let horizontal = signed_word(&result.memory, 0x4434bc)?;
            move_to(&mut result, horizontal, vertical);
            literal(&mut result, destination)?;
            vertical = vertical.wrapping_add(15);
        }
        index = index.wrapping_add(1);
    }
    copy_substring(
        &mut result,
        source,
        destination,
        first,
        index.wrapping_sub(1),
    )?;
    let horizontal = signed_word(&result.memory, 0x4434bc)?;
    move_to(&mut result, horizontal, vertical);
    literal(&mut result, destination)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recovered_drawing::{BuildingWorld, PictureObservation};

    struct Services {
        numbers: Vec<i32>,
        icons: Vec<i16>,
        indexed: Vec<(i16, i16)>,
        requests: Vec<ItemUiRequest>,
        dialog_text: PascalString,
        short_read: Vec<u8>,
        fail_number: bool,
    }

    impl Default for Services {
        fn default() -> Self {
            Self {
                numbers: vec![],
                icons: vec![],
                indexed: vec![],
                requests: vec![],
                dialog_text: PascalString::default(),
                short_read: vec![3, b'X'],
                fail_number: false,
            }
        }
    }

    impl NumericServices for Services {
        fn number_to_string(&mut self, value: i32) -> Result<PascalString, RuleError> {
            if self.fail_number {
                return Err(RuleError::Unsupported {
                    action: "test.number",
                    dependency: "conversion observation absent",
                });
            }
            self.numbers.push(value);
            PascalString::from_payload(value.to_string().as_bytes())
        }
        fn string_to_number(&mut self, _: &PascalString) -> Result<i32, RuleError> {
            Err(RuleError::Invalid("unexpected string parse".into()))
        }
    }

    impl DrawingServices for Services {
        fn color_icon(&mut self, id: i16) -> Result<u32, RuleError> {
            self.icons.push(id);
            Ok(u32::from(id as u16) + 500)
        }
        fn indexed_string(
            &mut self,
            resource_id: i16,
            index: i16,
        ) -> Result<PascalString, RuleError> {
            self.indexed.push((resource_id, index));
            PascalString::from_payload(if resource_id == -11000 {
                b"Mode"
            } else {
                b"Blade"
            })
        }
        fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
            Err(RuleError::Invalid("unexpected world lookup".into()))
        }
        fn picture(&mut self, _: i16) -> Result<PictureObservation, RuleError> {
            Ok(PictureObservation {
                handle: 0,
                bounds: None,
            })
        }
        fn new_world(&mut self, _: i16, _: BodyRect) -> Result<(i16, BuildingWorld), RuleError> {
            Err(RuleError::Invalid("unexpected world allocation".into()))
        }
    }

    impl ItemUiServices for Services {
        fn observe(
            &mut self,
            request: &ItemUiRequest,
            memory: &RenderingMemory,
        ) -> Result<ItemUiReply, RuleError> {
            self.requests.push(request.clone());
            match request {
                ItemUiRequest::GetDialogItem {
                    item,
                    item_type,
                    handle,
                    rectangle,
                    ..
                } => Ok(ItemUiReply::Writes(vec![
                    RecordWrite {
                        address: *item_type,
                        bytes: 4_i16.to_be_bytes().to_vec(),
                    },
                    RecordWrite {
                        address: *handle,
                        bytes: (1000_u32 + u32::from(*item as u16)).to_be_bytes().to_vec(),
                    },
                    RecordWrite {
                        address: *rectangle,
                        bytes: rect_bytes(BodyRect {
                            top: 10,
                            left: 20,
                            bottom: 30,
                            right: 40,
                        })
                        .to_vec(),
                    },
                ])),
                ItemUiRequest::GetDialogItemText { .. } => {
                    Ok(ItemUiReply::Pascal(self.dialog_text.clone()))
                }
                ItemUiRequest::OffsetRect {
                    pointer,
                    horizontal,
                    vertical,
                } => {
                    let rect = memory.rect(*pointer)?;
                    let values = [
                        rect.top.wrapping_add(*vertical),
                        rect.left.wrapping_add(*horizontal),
                        rect.bottom.wrapping_add(*vertical),
                        rect.right.wrapping_add(*horizontal),
                    ];
                    Ok(ItemUiReply::Writes(
                        values
                            .into_iter()
                            .enumerate()
                            .map(|(index, value)| RecordWrite {
                                address: pointer.wrapping_add(index as u32 * 2),
                                bytes: value.to_be_bytes().to_vec(),
                            })
                            .collect(),
                    ))
                }
                ItemUiRequest::File(FileRequest::Open { .. }) => {
                    Ok(ItemUiReply::File(FileReply::Handle(77)))
                }
                ItemUiRequest::File(FileRequest::Seek { .. }) => {
                    Ok(ItemUiReply::File(FileReply::Integer(-1)))
                }
                ItemUiRequest::File(FileRequest::Close(_)) => {
                    Ok(ItemUiReply::File(FileReply::Error(-1)))
                }
                ItemUiRequest::ReadFile { .. } => Ok(ItemUiReply::File(FileReply::Read {
                    bytes_written: self.short_read.clone(),
                    elements_read: 0,
                })),
                _ => Err(RuleError::Invalid(
                    "unexpected fixture service request".into(),
                )),
            }
        }
    }

    fn rect_bytes(rect: BodyRect) -> [u8; 8] {
        let mut bytes = [0; 8];
        for (index, value) in [rect.top, rect.left, rect.bottom, rect.right]
            .into_iter()
            .enumerate()
        {
            bytes[index * 2..index * 2 + 2].copy_from_slice(&value.to_be_bytes());
        }
        bytes
    }

    fn put_word(memory: &mut RenderingMemory, address: u32, value: i16) {
        memory.insert_region(address, &value.to_be_bytes());
    }
    fn put_long(memory: &mut RenderingMemory, address: u32, value: u32) {
        memory.insert_region(address, &value.to_be_bytes());
    }
    fn put_pascal(memory: &mut RenderingMemory, address: u32, text: &[u8]) {
        memory.insert_region(address, PascalString::from_payload(text).unwrap().raw());
    }

    fn common() -> RenderingMemory {
        let mut memory = RenderingMemory::default();
        memory.insert_region(0x45363c, &[0; ITEM_BYTES * 2]);
        memory.insert_region(0x41fc48, &[0; 66]);
        memory.insert_region(0x479ed8, &[0xaa; 256]);
        memory.insert_region(0x4434a6, &[0; 64]);
        memory.insert_region(0x439598, &[0; 4]);
        memory.insert_region(0x4471e8, &[0; 18]);
        put_long(&mut memory, 0x4394b8, 0x11111111);
        memory
    }

    fn moves(plan: &ItemUiPlan) -> Vec<(i16, i16)> {
        plan.effects
            .iter()
            .filter_map(|effect| match effect {
                ItemUiEffect::Drawing(DrawingEffect::MoveTo {
                    horizontal,
                    vertical,
                }) => Some((*horizontal, *vertical)),
                _ => None,
            })
            .collect()
    }

    fn drawn(plan: &ItemUiPlan) -> Vec<Vec<u8>> {
        plan.effects
            .iter()
            .filter_map(|effect| match effect {
                ItemUiEffect::Drawing(DrawingEffect::DrawString(text)) => {
                    Some(text.payload().to_vec())
                }
                _ => None,
            })
            .collect()
    }

    #[test]
    fn pack_display_retains_category_markers_signed_modifiers_and_special_labels() {
        let frame = 0x800000;
        let mut memory = common();
        memory.insert_region(frame, &[0; 20]);
        for (offset, value) in [(8, 0), (10, 200), (12, 300), (14, 400), (16, 500), (18, 8)] {
            put_word(&mut memory, frame + offset, value);
        }
        put_word(&mut memory, 0x41fc76, -3);
        put_word(&mut memory, 0x41fc78, 1);
        put_word(&mut memory, 0x45365c, 25);
        memory.insert_region(0x41fc7f, &[0xfe]);
        memory.insert_region(0x41fc83, &[1]);
        put_pascal(&mut memory, 0x4032c8, b"X");
        let original = memory.clone();
        let mut service = Services::default();
        let result = show_pack(&memory, frame, 8, &mut service).unwrap();
        assert_eq!(memory, original);
        assert_eq!(service.numbers, [-3, -1]);
        assert_eq!(service.icons, [vec![0x839], vec![208; 8]].concat());
        assert_eq!(moves(&result)[0], (501, 208));
        let markers: Vec<_> = result
            .effects
            .iter()
            .filter_map(|effect| match effect {
                ItemUiEffect::Drawing(DrawingEffect::PlotColorIcon { handle, rect })
                    if *handle == 708 =>
                {
                    Some(*rect)
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            markers[0],
            BodyRect {
                top: 190,
                left: 298,
                bottom: 198,
                right: 306
            }
        );
        assert_eq!(
            markers[7],
            BodyRect {
                top: 198,
                left: 298,
                bottom: 206,
                right: 306
            }
        );
        assert_eq!(
            result.memory.rect(0x4434ba).unwrap(),
            BodyRect {
                top: 198,
                left: 307,
                bottom: 206,
                right: 315
            }
        );
        assert!(service.requests.iter().any(|request| matches!(
            request,
            ItemUiRequest::OffsetRect {
                horizontal: -63,
                vertical: 8,
                ..
            }
        )));
        assert_eq!(
            drawn(&result),
            [b"-3".to_vec(), b"-1".to_vec(), b"X".to_vec()]
        );
        assert!(show_pack(&memory, frame, 7, &mut Services::default()).is_err());
        assert_eq!(memory, original);

        put_word(&mut memory, frame + 18, -1);
        let mut service = Services::default();
        let negative = show_pack(&memory, frame, 0, &mut service).unwrap();
        assert_eq!(service.icons, [0x839]);
        assert_eq!(
            negative.memory.rect(0x4434ba).unwrap(),
            BodyRect {
                top: 190,
                left: 298,
                bottom: 198,
                right: 306
            }
        );

        put_word(&mut memory, frame + 18, 0);
        put_word(&mut memory, 0x41fc78, 0);
        put_word(&mut memory, 0x45365e, 24);
        memory.insert_region(0x453693, &[1]);
        for (address, value) in [
            (0x45368e, 1),
            (0x45368d, 0xff),
            (0x453692, 2),
            (0x453691, 0xfd),
            (0x453690, 4),
            (0x45368f, 5),
            (0x45368a, 0xfa),
        ] {
            memory.insert_region(address, &[value]);
        }
        put_word(&mut memory, 0x45369a, -2);
        put_word(&mut memory, 0x453694, 6);
        put_word(&mut memory, 0x453696, 7);
        put_word(&mut memory, 0x4536d0, 8);
        for (address, text) in [
            (0x403356, b"Type".as_slice()),
            (0x40332c, b"Special"),
            (0x403312, b"A"),
            (0x403308, b"B"),
            (0x4032fe, b"C"),
            (0x4032da, b"D"),
            (0x4032d0, b"E"),
            (0x4032ce, b"F"),
        ] {
            put_pascal(&mut memory, address, text);
        }
        let mut service = Services::default();
        let special = show_pack(&memory, frame, 0, &mut service).unwrap();
        assert_eq!(service.numbers, [-3, 16]);
        assert_eq!(
            drawn(&special),
            [
                "-3", "Type", "Special", "A", "B", "C", "D", "E", "+", "16", "F"
            ]
            .map(|value| value.as_bytes().to_vec())
        );
    }

    #[test]
    fn shop_wings_copy_before_gate_keep_missing_right_and_forward_aliases() {
        let frame = 0x800000;
        let mut memory = RenderingMemory::default();
        memory.insert_region(frame - 0x188c, &vec![0xaa; 0x18a0]);
        let mut left = vec![0; WARRIOR_BYTES];
        let mut right = vec![0; WARRIOR_BYTES];
        for (offset, value) in [(26, -3_i16), (2518, 101), (2520, 102)] {
            left[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        }
        for (offset, value) in [(26, 4_i16), (2518, 201), (2520, 202)] {
            right[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        }
        memory.insert_region(0x44721c, &left);
        memory.insert_region(0x447e5e, &right);
        put_word(&mut memory, frame + 8, -2);
        put_word(&mut memory, frame + 10, -1);
        put_word(&mut memory, 0x439582, 0);
        let original = memory.clone();
        let result = update_shop_wings(&memory, frame, &mut Services::default()).unwrap();
        assert_eq!(result.effects.len(), 785);
        assert_eq!(
            read(&result.memory, frame - 0x1884, WARRIOR_BYTES).unwrap(),
            left
        );
        assert_eq!(
            read(&result.memory, frame - 0xc42, WARRIOR_BYTES).unwrap(),
            vec![0xaa; WARRIOR_BYTES]
        );
        assert_eq!(memory, original);
        put_word(&mut memory, 0x439582, 1);
        put_word(&mut memory, 0x402f3c, 3);
        memory.insert_region(0x479ed8, &[0xaa; 256]);
        put_word(&mut memory, frame + 10, -2);
        let mut service = Services::default();
        let displayed = update_shop_wings(&memory, frame, &mut service).unwrap();
        assert_eq!(service.numbers, [-3, 101, 102, 4, 201, 202]);
        assert_eq!(
            moves(&displayed),
            [
                (50, 455),
                (4, 424),
                (42, 424),
                (768, 455),
                (724, 424),
                (762, 424)
            ]
        );
        let pictures: Vec<_> = displayed
            .effects
            .iter()
            .filter_map(|effect| match effect {
                ItemUiEffect::Drawing(DrawingEffect::DrawPicture { handle, bounds }) => {
                    Some((*handle, *bounds))
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            pictures,
            [
                (
                    0,
                    BodyRect {
                        top: 387,
                        left: 0,
                        bottom: 460,
                        right: 80
                    }
                ),
                (
                    0,
                    BodyRect {
                        top: 387,
                        left: 720,
                        bottom: 460,
                        right: 800
                    }
                )
            ]
        );

        let source = 0x44721c;
        let alias_frame = source + 0x1888;
        let mut alias = RenderingMemory::default();
        alias.insert_region(alias_frame - 0x188c, &vec![0xaa; 0x18a0]);
        alias.insert_region(source, &vec![0; WARRIOR_BYTES]);
        alias.insert_region(source, &[0xde, 0xad, 0xbe, 0xef]);
        put_word(&mut alias, alias_frame + 8, -2);
        put_word(&mut alias, alias_frame + 10, -1);
        put_word(&mut alias, 0x439582, 0);
        let copied = update_shop_wings(&alias, alias_frame, &mut Services::default()).unwrap();
        assert_eq!(
            read(&copied.memory, source + 4, 8).unwrap(),
            [0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef]
        );
    }

    #[test]
    fn editor_executes_both_loads_signed_dialog_fields_and_preserves_short_read_tail() {
        let frame = 0x800000;
        let mut memory = common();
        memory.insert_region(frame, &[0; 12]);
        put_word(&mut memory, frame + 8, 1);
        put_word(&mut memory, frame + 10, 2);
        put_word(&mut memory, 0x41fc48, 3);
        let mut old = [0; ITEM_BYTES];
        old[26..28].copy_from_slice(&3_i16.to_be_bytes());
        memory.insert_region(ITEM_TABLE_ADDRESSES[0] + 3 * ITEM_BYTES as u32, &old);
        let mut selected = [0; ITEM_BYTES];
        selected[10..12].copy_from_slice(&56_i16.to_be_bytes());
        selected[26..28].copy_from_slice(&(-7_i16).to_be_bytes());
        selected[102..104].copy_from_slice(&801_i16.to_be_bytes());
        memory.insert_region(ITEM_TABLE_ADDRESSES[1] + 3 * ITEM_BYTES as u32, &selected);
        let mut pack = [0; 22];
        pack[..2].copy_from_slice(&203_i16.to_be_bytes());
        pack[2..4].copy_from_slice(&5_i16.to_be_bytes());
        pack[6..8].copy_from_slice(&(-9_i16).to_be_bytes());
        pack[9] = 1;
        pack[11] = 127;
        pack[13] = 0xfe;
        pack[15] = 0x80;
        pack[16] = 0xff;
        let source = 0x448aa0 + WARRIOR_BYTES as u32 + 0x132 + 44;
        memory.insert_region(source, &pack);
        put_word(&mut memory, 0x4471e8, 0);
        memory.insert_region(0x402f24, b"rb\0");
        memory.insert_region(0x402986, b":Data Files:Data ID DES\0");
        memory.insert_region(0x40299e, b"Data ID DES\0");
        memory.insert_region(0x44260c, b"Scenario\0");
        memory.insert_region(0x44274c, &[7]);
        memory.insert_region(0x4389a4, &[0xaa; 80]);
        let original = memory.clone();
        let mut service = Services::default();
        let result = update_edit_item(&memory, frame, 80, &mut service).unwrap();
        assert_eq!(memory, original);
        assert_eq!(read(&result.memory, 0x41fc48, 22).unwrap(), pack);
        assert_eq!(
            service.numbers,
            [-2, -128, -1, 127, 0, 0, 0, 0, 0, 0, 5, -9, 7]
        );
        assert_eq!(result.memory.word(0x4471e8).unwrap(), 200);
        assert_eq!(service.indexed, [(201, 4)]);
        let selection_write = result.effects.iter().position(|effect| {
            matches!(effect, ItemUiEffect::Write(RecordWrite { address: 0x4471e8, bytes }) if bytes == &[0, 200])
        }).unwrap();
        let name_lookup = result
            .effects
            .iter()
            .position(|effect| {
                matches!(
                    effect,
                    ItemUiEffect::IndexedString {
                        resource_id: 201,
                        index: 4,
                        ..
                    }
                )
            })
            .unwrap();
        assert!(selection_write < name_lookup);
        assert_eq!(service.icons, [176, 56]);
        assert!(service.requests.iter().any(|request| matches!(request,ItemUiRequest::File(FileRequest::Open {name,mode}) if name==b"Scenario:7:Data ID DES\0" && mode==b"rb\0")));
        assert!(service.requests.iter().any(|request| matches!(
            request,
            ItemUiRequest::File(FileRequest::Seek {
                handle: 77,
                offset: 256,
                origin: 0
            })
        )));
        let normalization: Vec<_> = result
            .effects
            .iter()
            .filter_map(|effect| match effect {
                ItemUiEffect::Write(RecordWrite {
                    address: 0x453656,
                    bytes,
                }) => Some(bytes.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(normalization, [vec![0, 3], vec![0, 7]]);
        let description: Vec<_> = result
            .effects
            .iter()
            .filter_map(|effect| match effect {
                ItemUiEffect::SetDialogItemText {
                    handle: 1065, text, ..
                } => Some(text.payload().to_vec()),
                _ => None,
            })
            .collect();
        assert_eq!(description, [b"Xla".to_vec()]);
        assert_eq!(result.memory.long(0x4434a6).unwrap(), 77);
        let mut failing = Services {
            fail_number: true,
            ..Services::default()
        };
        assert!(update_edit_item(&memory, frame, 80, &mut failing).is_err());
        assert_eq!(memory, original);

        selected[102..104].copy_from_slice(&(-1_i16).to_be_bytes());
        memory.insert_region(ITEM_TABLE_ADDRESSES[1] + 3 * ITEM_BYTES as u32, &selected);
        pack[8] = 1;
        pack[9] = 0;
        memory.insert_region(source, &pack);
        let mut service = Services::default();
        update_edit_item(&memory, frame, 0, &mut service).unwrap();
        assert_eq!(service.icons, [56]);
        assert!(service.requests.iter().any(|request| matches!(
            request,
            ItemUiRequest::File(FileRequest::Seek {
                offset: 0xffffff00,
                ..
            })
        )));
    }

    #[test]
    fn item_stats_compose_ammunition_wrap_fields_and_use_word_display_and_negative_gate() {
        let mut memory = common();
        put_long(&mut memory, 0x3fe302, 0x900000);
        memory.insert_region(0x900044, &[0, 7, 0x15, 0, 0, 2, 0, 14]);
        memory.insert_region(
            0x4471f2,
            &rect_bytes(BodyRect {
                top: 100,
                left: 200,
                bottom: 140,
                right: 240,
            }),
        );
        put_word(&mut memory, 0x41fc4c, 2);
        memory.insert_region(0x41fc5a, &[0xff]);
        memory.insert_region(0x45368d, &[127]);
        for (address, value) in [
            (0x453664, 32760),
            (0x45367a, 10),
            (0x45367c, -1),
            (0x453688, 2),
            (0x453652, 3),
        ] {
            put_word(&mut memory, address, value);
        }
        let mut ammo = [0; ITEM_BYTES];
        ammo[26..28].copy_from_slice(&(-3_i16).to_be_bytes());
        ammo[81] = 1;
        for (offset, value) in [(40, 10_i16), (62, -11), (64, -2), (140, -7)] {
            ammo[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        }
        memory.insert_region(ITEM_TABLE_ADDRESSES[0] + 2 * ITEM_BYTES as u32, &ammo);
        for (address, text) in [
            (0x400f74, b"M".as_slice()),
            (0x400f72, b";"),
            (0x400f68, b"D"),
            (0x400f64, b"P"),
            (0x400f5e, b"N"),
            (0x400f14, b"NEG"),
            (0x400f10, b"X"),
            (0x400f0c, b"Y"),
            (0x400f7c, b"TYPE"),
            (0x400f78, b"END"),
        ] {
            put_pascal(&mut memory, address, text);
        }
        let original = memory.clone();
        let mut service = Services::default();
        let result = show_item_stats(&memory, &mut service).unwrap();
        assert_eq!(memory, original);
        assert_eq!(result.memory.byte(0x45368d).unwrap(), 0x80);
        assert_eq!(signed_word(&result.memory, 0x453664).unwrap(), -32766);
        assert_eq!(signed_word(&result.memory, 0x45367a).unwrap(), -1);
        assert_eq!(signed_word(&result.memory, 0x45367c).unwrap(), -3);
        assert_eq!(signed_word(&result.memory, 0x4536c8).unwrap(), -7);
        assert_eq!(signed_word(&result.memory, 0x4536ec).unwrap(), 3);
        assert_eq!(service.numbers, [2, 6, -1]);
        assert_eq!(moves(&result), [(242, 130)]);
        assert_eq!(
            drawn(&result),
            ["M", "2", ";", "D", "6", "P", "-1", "N", "NEG", "X", "Y"]
                .map(|value| value.as_bytes().to_vec())
        );
        assert_eq!(
            &result.effects[result.effects.len() - 5..],
            &[
                ItemUiEffect::Drawing(DrawingEffect::ForeColor(33)),
                ItemUiEffect::Drawing(DrawingEffect::TextFont(7)),
                ItemUiEffect::TextFaceSlot(0x0015),
                ItemUiEffect::Drawing(DrawingEffect::TextSize(14)),
                ItemUiEffect::TextMode(2),
            ]
        );
        memory.insert_region(0x453693, &[0xff]);
        let mut service = Services::default();
        show_item_stats(&memory, &mut service).unwrap();
        assert_eq!(service.indexed, [(-11000, 0)]);

        put_word(&mut memory, 0x41fc4c, 0);
        memory.insert_region(0x41fc5a, &[0]);
        memory.insert_region(0x453693, &[0]);
        put_word(&mut memory, 0x453664, i16::MIN);
        put_word(&mut memory, 0x45367a, 0);
        put_word(&mut memory, 0x45367c, 0);
        put_word(&mut memory, 0x4536c8, -7);
        put_word(&mut memory, 0x453688, -2);
        put_word(&mut memory, 0x453652, 0);
        let mut service = Services::default();
        let zero_word = show_item_stats(&memory, &mut service).unwrap();
        assert_eq!(service.numbers, [-2]);
        assert_eq!(
            drawn(&zero_word),
            ["M", "-2", ";"].map(|value| value.as_bytes().to_vec())
        );
    }

    #[test]
    fn print_stat_keeps_crs_final_empty_draw_word_wrap_and_addressed_copy_aliases() {
        let frame = 0x800000;
        let mut memory = common();
        memory.insert_region(frame - 0x212, &vec![0xaa; 0x21e]);
        put_word(&mut memory, frame + 8, 34);
        put_word(&mut memory, frame + 10, 32760);
        put_word(&mut memory, 0x4434bc, 7);
        let original = memory.clone();
        let mut service = Services {
            dialog_text: PascalString::from_payload(b"A\r\rB").unwrap(),
            ..Services::default()
        };
        let result = print_stat(&memory, frame, &mut service).unwrap();
        assert_eq!(memory, original);
        assert_eq!(result.native_return, None);
        assert_eq!(
            drawn(&result),
            [b"A\r".to_vec(), b"\r".to_vec(), b"B".to_vec()]
        );
        assert_eq!(moves(&result), [(7, 32760), (7, -32761), (7, -32746)]);
        assert_eq!(
            read(&result.memory, frame - 0x200, 4).unwrap(),
            [1, b'B', 13, 0xaa]
        );
        service.dialog_text = PascalString::from_payload(b"A\r").unwrap();
        assert_eq!(
            drawn(&print_stat(&memory, frame, &mut service).unwrap()),
            [b"A\r".to_vec(), Vec::<u8>::new()]
        );
        service.dialog_text = PascalString::default();
        assert_eq!(
            drawn(&print_stat(&memory, frame, &mut service).unwrap()),
            [Vec::<u8>::new()]
        );
        assert!(service.numbers.is_empty());

        let frame = 0x4436bc;
        let mut alias = common();
        alias.insert_region(frame - 0x212, &vec![0xaa; 0x21e]);
        put_word(&mut alias, frame + 8, 34);
        put_word(&mut alias, frame + 10, 20);
        put_word(&mut alias, 0x4434bc, 7);
        let mut service = Services {
            dialog_text: PascalString::from_payload(b"AB").unwrap(),
            ..Services::default()
        };
        let result = print_stat(&alias, frame, &mut service).unwrap();
        assert_eq!(moves(&result), [(0x0241, 20)]);
    }
}
