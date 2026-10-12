//! Equipment list and ammunition display bodies with addressed child effects.

use crate::RuleError;
use crate::recovered_drawing::{self, DrawingContext, DrawingEffect};
use crate::recovered_equipment::{item_tables, member_address, read, sb, sw};
use crate::recovered_geometry::BodyRect;
use crate::recovered_item_ui::{self, ItemUiEffect, ItemUiServices};
use crate::recovered_records::{self, ItemTables, RecordWrite};
use crate::recovered_rendering::RenderingMemory;
use crate::recovered_text::{self, PascalString, TextContext, TextEffect};
use centurions_content::{ITEM_BYTES, WARRIOR_BYTES};

pub trait EquipmentUiServices: ItemUiServices {
    /// Actual GetPen writes at the supplied Point pointer, including reported aliases.
    fn pen_position(
        &mut self,
        destination: u32,
        memory: &RenderingMemory,
    ) -> Result<Vec<RecordWrite>, RuleError>;
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EquipmentUiEffect {
    Write(RecordWrite),
    Drawing(DrawingEffect),
    ItemUi(ItemUiEffect),
    TextFaceSlot(u16),
    Move(i16, i16),
    GetPen {
        destination: u32,
        writes: Vec<RecordWrite>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EquipmentUiPlan {
    pub memory: RenderingMemory,
    pub effects: Vec<EquipmentUiEffect>,
    pub native_return: u32,
}
fn plan(memory: &RenderingMemory) -> EquipmentUiPlan {
    EquipmentUiPlan {
        memory: memory.clone(),
        effects: vec![],
        native_return: 0,
    }
}
fn write(result: &mut EquipmentUiPlan, address: u32, bytes: &[u8]) -> Result<(), RuleError> {
    read(&result.memory, address, bytes.len())?;
    result.memory.insert_region(address, bytes);
    result.effects.push(EquipmentUiEffect::Write(RecordWrite {
        address,
        bytes: bytes.to_vec(),
    }));
    Ok(())
}
fn word(result: &mut EquipmentUiPlan, address: u32, value: i16) -> Result<(), RuleError> {
    write(result, address, &value.to_be_bytes())
}
fn draw(result: &mut EquipmentUiPlan, effect: DrawingEffect) {
    result.effects.push(EquipmentUiEffect::Drawing(effect));
}
fn literal(result: &mut EquipmentUiPlan, address: u32) -> Result<(), RuleError> {
    let length = usize::from(result.memory.byte(address)?);
    let text = PascalString::from_raw(&read(&result.memory, address, length + 1)?)?;
    draw(result, DrawingEffect::DrawString(text));
    Ok(())
}
fn number(
    result: &mut EquipmentUiPlan,
    value: i16,
    services: &mut impl EquipmentUiServices,
) -> Result<(), RuleError> {
    let mut context = TextContext::default();
    let effects = recovered_text::string(&mut context, value, services)?;
    for effect in effects {
        match effect {
            TextEffect::NumberToString(value) => {
                result
                    .effects
                    .push(EquipmentUiEffect::ItemUi(ItemUiEffect::NumberToString {
                        value,
                        destination: 0x479ed8,
                        result: context.scratch.clone(),
                    }));
                write(result, 0x479ed8, context.scratch.raw())?;
            }
            TextEffect::DrawString(text) => draw(result, DrawingEffect::DrawString(text)),
            _ => {
                return Err(RuleError::Invalid(
                    "unexpected equipment text child effect".into(),
                ));
            }
        }
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
    result: &mut EquipmentUiPlan,
    id: i16,
    rect: BodyRect,
    services: &mut impl EquipmentUiServices,
) -> Result<(), RuleError> {
    let child = recovered_drawing::ploticon3(&mut icon_context(rect), id, rect, services)?;
    for effect in child.effects {
        match effect {
            DrawingEffect::Write(effect) => write(result, effect.address, &effect.bytes)?,
            effect => draw(result, effect),
        }
    }
    Ok(())
}
fn copy_record(
    result: &mut EquipmentUiPlan,
    source: u32,
    destination: u32,
    length: usize,
) -> Result<(), RuleError> {
    for offset in (0..length / 4 * 4).step_by(4) {
        let bytes = read(&result.memory, source.wrapping_add(offset as u32), 4)?;
        write(result, destination.wrapping_add(offset as u32), &bytes)?;
    }
    let offset = length / 4 * 4;
    if offset < length {
        let bytes = read(
            &result.memory,
            source.wrapping_add(offset as u32),
            length - offset,
        )?;
        write(result, destination.wrapping_add(offset as u32), &bytes)?;
    }
    Ok(())
}

fn plot_item(
    result: &mut EquipmentUiPlan,
    id: i16,
    services: &mut impl EquipmentUiServices,
) -> Result<(), RuleError> {
    let backing = item_tables(&result.memory, id)?;
    let tables = ItemTables {
        families: std::array::from_fn(|i| backing[i].as_slice()),
    };
    let current = read(&result.memory, 0x45363c, ITEM_BYTES)?
        .try_into()
        .expect("item");
    let loaded = recovered_records::loaditemnotype(&tables, &current, id)?;
    let item_id = i16::from_be_bytes([loaded.bytes[102], loaded.bytes[103]]);
    let base = crate::rules::selection_base(item_id);
    let rect = result.memory.rect(0x4471f2)?;
    let omit = result.memory.byte(0x442756)?;
    let mut context = icon_context(rect);
    context.item_work = current;
    context.window = result.memory.long(0x439404)?;
    context.item_window = result.memory.long(0x4393fc)?;
    context.half_width_input = sw(&result.memory, 0x40856c)?;
    context.omit_banner = omit;
    if omit == 0 {
        context.banner_rect = result.memory.rect(0x4471ac)?;
    }
    if id == 0 && context.window == context.item_window {
        context.active_member = result.memory.byte(0x44273e)? as i8;
    }
    context.show_item_background = result.memory.byte(0x44275a)?;
    context.text_in_other_port = result.memory.byte(0x442750)?;
    context.erase_pattern = result.memory.long(0x4358da)?;
    context.default_font = sw(&result.memory, 0x402f3c)?;
    if context.text_in_other_port != 0 {
        context.text_port = result.memory.long(0x438798)?;
    }
    context.selection = base;
    let child = recovered_drawing::ploticon(&mut context, &tables, id, services)?;
    for effect in child.effects {
        // getselection's scalar adapter needs its addressed store at the native call point.
        if matches!(effect, DrawingEffect::GetIndexedString { .. }) {
            word(result, 0x4471e8, base)?;
        }
        match effect {
            DrawingEffect::Write(effect) => write(result, effect.address, &effect.bytes)?,
            DrawingEffect::GetIndexedString { resource_id, index } => {
                result
                    .effects
                    .push(EquipmentUiEffect::ItemUi(ItemUiEffect::IndexedString {
                        resource_id,
                        index,
                        destination: 0x479ed8,
                        result: context.text_work.clone(),
                    }));
            }
            effect => draw(result, effect),
        }
    }
    Ok(())
}

/// showcharge2, 0x15498e. Quantity is A6+8; ammo ID is cached from A6+10.
pub fn show_charge(
    memory: &RenderingMemory,
    frame: u32,
    services: &mut impl EquipmentUiServices,
) -> Result<EquipmentUiPlan, RuleError> {
    let mut result = plan(memory);
    let ammunition = sw(&result.memory, frame.wrapping_add(10))?;
    if sw(&result.memory, frame.wrapping_add(8))? > 0 {
        result.effects.push(EquipmentUiEffect::TextFaceSlot(0));
        if ammunition != 0 {
            let destination = frame.wrapping_sub(12);
            let writes = services.pen_position(destination, &result.memory)?;
            result.effects.push(EquipmentUiEffect::GetPen {
                destination,
                writes: writes.clone(),
            });
            for effect in writes {
                write(&mut result, effect.address, &effect.bytes)?;
            }
            let left = sw(&result.memory, frame.wrapping_sub(10))?.wrapping_add(4);
            word(&mut result, frame.wrapping_sub(6), left)?;
            let top = sw(&result.memory, 0x4471f2)?.wrapping_add(2);
            word(&mut result, frame.wrapping_sub(8), top)?;
            let right = sw(&result.memory, frame.wrapping_sub(6))?.wrapping_add(16);
            word(&mut result, frame.wrapping_sub(2), right)?;
            let bottom = sw(&result.memory, frame.wrapping_sub(8))?.wrapping_add(16);
            word(&mut result, frame.wrapping_sub(4), bottom)?;
            let offset = u32::from(ammunition.wrapping_sub(200) as u16) * 150;
            let id = sw(
                &result.memory,
                0x46225e_u32.wrapping_add(offset).wrapping_add(10),
            )?;
            let rect = result.memory.rect(frame.wrapping_sub(8))?;
            icon(&mut result, id, rect, services)?;
            result.effects.push(EquipmentUiEffect::Move(20, 0));
        }
        draw(&mut result, DrawingEffect::ForeColor(409));
        literal(&mut result, 0x401152)?;
        let value = sw(&result.memory, frame.wrapping_add(8))?;
        number(&mut result, value, services)?;
    }
    draw(&mut result, DrawingEffect::ForeColor(33));
    let child = recovered_item_ui::show_item_stats(&result.memory, services)?;
    result.memory = child.memory;
    result
        .effects
        .extend(child.effects.into_iter().map(EquipmentUiEffect::ItemUi));
    result.effects.push(EquipmentUiEffect::TextFaceSlot(0));
    Ok(result)
}
fn copied_node(result: &EquipmentUiPlan, row: i16) -> Result<u32, RuleError> {
    let index = row.wrapping_add(sw(&result.memory, 0x44719c)?) as u16;
    Ok(0x44721c_u32
        .wrapping_add(u32::from(index) * 22)
        .wrapping_add(0x132))
}
/// updateitems, 0x160d08. Both row bounds are cached before the style/copy prefix.
pub fn update_items(
    memory: &RenderingMemory,
    frame: u32,
    services: &mut impl EquipmentUiServices,
) -> Result<EquipmentUiPlan, RuleError> {
    let mut result = plan(memory);
    let first = sw(&result.memory, frame.wrapping_add(8))?;
    let last = sw(&result.memory, frame.wrapping_add(10))?;
    let port = result.memory.long(0x438798)?;
    draw(&mut result, DrawingEffect::SetPort(port));
    word(&mut result, 0x4471be, 10)?;
    let right = sw(&result.memory, 0x40856c)?.wrapping_add(284);
    word(&mut result, 0x4471c2, right)?;
    draw(&mut result, DrawingEffect::TextSize(14));
    result.effects.push(EquipmentUiEffect::TextFaceSlot(0));
    let pattern = result.memory.long(0x4358da)?;
    draw(&mut result, DrawingEffect::BackPixelPattern(pattern));
    result.effects.push(EquipmentUiEffect::TextFaceSlot(0));
    let font = sw(&result.memory, 0x402f3c)?;
    draw(&mut result, DrawingEffect::TextFont(font));
    draw(&mut result, DrawingEffect::ForeColor(33));
    let member = sb(&result.memory, 0x44273f)? as u16;
    copy_record(&mut result, member_address(member), 0x44721c, WARRIOR_BYTES)?;
    let last = last.clamp(0, 10);
    let mut row = first.max(0);
    if result.memory.long(0x439404)? == 0 {
        let handle = result.memory.long(0x439490)?;
        write(&mut result, 0x439404, &handle.to_be_bytes())?;
    }
    while row < last.wrapping_add(1) {
        word(&mut result, 0x4471f2, row.wrapping_mul(40).wrapping_add(15))?;
        let bottom = sw(&result.memory, 0x4471f2)?.wrapping_add(32);
        word(&mut result, 0x4471f6, bottom)?;
        word(&mut result, 0x4471f4, 10)?;
        word(&mut result, 0x4471f8, 42)?;
        let node = copied_node(&result, row)?;
        if result.memory.word(node)? != 0 {
            let equipped = result
                .memory
                .byte(copied_node(&result, row)?.wrapping_add(9))?;
            write(&mut result, 0x44275a, &[equipped])?;
            let id = sw(&result.memory, copied_node(&result, row)?)?;
            plot_item(&mut result, id, services)?;
            let index = row.wrapping_add(sw(&result.memory, 0x44719c)?) as u8;
            write(&mut result, 0x41f708, &[index])?;
            let member = result.memory.byte(0x44273f)?;
            write(&mut result, 0x41f709, &[member])?;
            let index = sb(&result.memory, 0x41f708)? as u16;
            let source = 0x44721c_u32
                .wrapping_add(u32::from(index) * 22)
                .wrapping_add(0x132);
            copy_record(&mut result, source, 0x41fc48, 22)?;
            let child_frame = frame.wrapping_sub(26);
            let ammo = sw(&result.memory, copied_node(&result, row)?.wrapping_add(4))?;
            word(&mut result, child_frame.wrapping_add(10), ammo)?;
            let quantity = sw(&result.memory, copied_node(&result, row)?.wrapping_add(2))?;
            word(&mut result, child_frame.wrapping_add(8), quantity)?;
            let child = show_charge(&result.memory, child_frame, services)?;
            result.memory = child.memory;
            result.effects.extend(child.effects);
        } else {
            word(&mut result, 0x4471f4, 10)?;
            let right = sw(&result.memory, 0x40856c)?.wrapping_add(284);
            word(&mut result, 0x4471f8, right)?;
            let rect = result.memory.rect(0x4471f2)?;
            draw(&mut result, DrawingEffect::EraseRect(rect));
            word(&mut result, 0x4471f8, 42)?;
            let rect = result.memory.rect(0x4471ac)?;
            icon(&mut result, 174, rect, services)?;
        }
        row = row.wrapping_add(1);
    }
    if sw(&result.memory, 0x447236)? <= sw(&result.memory, 0x4471aa)? {
        let selected = sw(&result.memory, 0x447236)?.wrapping_sub(1);
        word(&mut result, 0x4471aa, selected)?;
        if sw(&result.memory, 0x4471aa)? == -1 {
            let value = sw(&result.memory, 0x4471aa)?.wrapping_add(1);
            word(&mut result, 0x4471aa, value)?;
        }
        let rect = result.memory.rect(0x4471ac)?;
        icon(&mut result, 173, rect, services)?;
        let top = sw(&result.memory, 0x4471aa)?
            .wrapping_mul(40)
            .wrapping_add(11);
        word(&mut result, 0x4471ac, top)?;
        let bottom = sw(&result.memory, 0x4471ac)?.wrapping_add(40);
        word(&mut result, 0x4471b0, bottom)?;
        let rect = result.memory.rect(0x4471ac)?;
        icon(&mut result, 174, rect, services)?;
    }
    let member = sb(&result.memory, 0x44273f)? as u16;
    copy_record(&mut result, 0x44721c, member_address(member), WARRIOR_BYTES)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recovered_equipment::test_support::*;

    #[test]
    fn charge_rereads_quantity_after_getpen_and_composes_ammo_stats() {
        let mut memory = fixture();
        let address = item(&mut memory, 200, 0);
        put_word(&mut memory, address + 10, 99);
        put_word(&mut memory, address + 64, 4);
        put_word(&mut memory, 0x41fc4c, 200);
        put_word(&mut memory, FRAME + 8, 2);
        put_word(&mut memory, FRAME + 10, 200);
        let mut services = Services::default();
        services.pen_quantity = Some(7);
        let result = show_charge(&memory, FRAME, &mut services).unwrap();
        assert_eq!(services.numbers, [7, 4]);
        assert_eq!(services.icons, [99]);
        assert_eq!(result.memory.word(FRAME + 8).unwrap(), 7);
        assert_eq!(
            result.memory.rect(FRAME - 8).unwrap(),
            BodyRect {
                top: 2,
                left: 104,
                bottom: 18,
                right: 120
            }
        );
        assert_eq!(result.memory.word(0x45367c).unwrap(), 4);
        assert_eq!(
            result
                .effects
                .iter()
                .filter(|effect| matches!(effect, EquipmentUiEffect::GetPen { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn list_uses_selection_base_and_empty_count_clamp_then_copies_back() {
        let mut memory = fixture();
        let address = item(&mut memory, 203, 0);
        put_word(&mut memory, address + 10, 99);
        put_word(&mut memory, member_address(0) + 0x132, 203);
        put_word(&mut memory, 0x4471e8, 1234);
        put_long(&mut memory, 0x439404, 1);
        put_long(&mut memory, 0x4393fc, 2);
        put_word(&mut memory, FRAME + 8, 0);
        put_word(&mut memory, FRAME + 10, 0);
        let mut services = Services::default();
        let result = update_items(&memory, FRAME, &mut services).unwrap();
        assert_eq!(services.indexed, [(201, 4)]);
        assert_eq!(result.memory.word(0x4471e8).unwrap(), 200);
        assert_eq!(
            read(&result.memory, member_address(0), WARRIOR_BYTES).unwrap(),
            read(&result.memory, 0x44721c, WARRIOR_BYTES).unwrap()
        );
        assert_eq!(result.memory.word(0x41fc48).unwrap(), 203);

        put_word(&mut memory, member_address(0) + 0x132, 0);
        put_word(&mut memory, member_address(0) + 0x1a, 0);
        put_word(&mut memory, 0x4471aa, 8);
        let mut services = Services::default();
        let empty = update_items(&memory, FRAME, &mut services).unwrap();
        assert_eq!(empty.memory.word(0x4471aa).unwrap(), 0);
        assert_eq!(empty.memory.word(0x4471ac).unwrap(), 11);
        assert_eq!(empty.memory.word(0x4471b0).unwrap(), 51);
        assert_eq!(services.icons, [174, 173, 174]);
        assert!(empty.effects.iter().any(|effect| matches!(
            effect,
            EquipmentUiEffect::Drawing(DrawingEffect::EraseRect(BodyRect {
                left: 10,
                right: 284,
                ..
            }))
        )));
    }
}
