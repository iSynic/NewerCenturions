//! Scenario menus and input dialog control from CODE 1.

use crate::{
    RuleError,
    recovered_drawing::{self, BuildingWorld, DrawingContext, DrawingEffect, DrawingServices},
    recovered_feedback::{self, FeedbackEffect},
    recovered_load_control::{LoadArg, LoadRequest},
    recovered_rendering::RenderingMemory,
    recovered_storage::{Execution, StorageLimits, StoragePlan, StorageRequest, StorageServices},
    recovered_text::PascalString,
};
use centurions_content::ITEM_BYTES;

pub trait ScenarioMenuServices: StorageServices + DrawingServices {}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScenarioMenuEffect {
    Storage(crate::recovered_storage::StorageEffect),
    Feedback(FeedbackEffect),
    Drawing(DrawingEffect),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenarioMenuPlan {
    pub memory: RenderingMemory,
    pub effects: Vec<ScenarioMenuEffect>,
    pub flow: crate::recovered_load_control::LoadFlow,
    pub native_return: Option<u32>,
    pub qualification: ScenarioMenuQualification,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScenarioMenuQualification {
    SourcePortWithObservedImportsAndProjectedDrawingChildren,
}
struct MenuExecution {
    storage: Execution,
    effects: Vec<ScenarioMenuEffect>,
}
impl MenuExecution {
    fn new(memory: &RenderingMemory, limits: StorageLimits, returned: Option<u32>) -> Self {
        Self {
            storage: Execution::new(memory, limits, returned),
            effects: vec![],
        }
    }
    fn flush(&mut self) {
        self.effects.extend(
            self.storage
                .plan
                .effects
                .drain(..)
                .map(ScenarioMenuEffect::Storage),
        );
    }
    fn finish(mut self) -> ScenarioMenuPlan {
        self.flush();
        let StoragePlan {
            memory,
            flow,
            native_return,
            ..
        } = self.storage.plan;
        ScenarioMenuPlan {
            memory,
            effects: self.effects,
            flow,
            native_return,
            qualification:
                ScenarioMenuQualification::SourcePortWithObservedImportsAndProjectedDrawingChildren,
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn flash(
        &mut self,
        frame: u32,
        message: u32,
        horizontal: i16,
        vertical: i16,
        wait: i16,
        sound: i16,
        services: &(impl ScenarioMenuServices + ?Sized),
    ) -> Result<(), RuleError> {
        let e = &mut self.storage;
        e.lw(
            frame.wrapping_add(16),
            (u32::from(wait as u16) << 16) | u32::from(sound as u16),
        )?;
        e.lw(
            frame.wrapping_add(12),
            (u32::from(horizontal as u16) << 16) | u32::from(vertical as u16),
        )?;
        e.lw(frame.wrapping_add(8), message)?;
        let child = recovered_feedback::flashmessage(
            &e.plan.memory,
            frame,
            message,
            horizontal,
            vertical,
            wait,
            sound,
            e.limits.maximum_polls,
            services,
        )?;
        e.plan.memory = child.memory;
        self.flush();
        self.effects
            .extend(child.effects.into_iter().map(ScenarioMenuEffect::Feedback));
        Ok(())
    }
    fn drawing(&mut self, effects: Vec<DrawingEffect>) -> Result<(), RuleError> {
        self.flush();
        for effect in effects {
            if let DrawingEffect::Write(write) = effect {
                self.storage.write(write.address, &write.bytes)?;
                self.flush();
            } else {
                self.effects.push(ScenarioMenuEffect::Drawing(effect));
            }
        }
        Ok(())
    }
}
use LoadArg::{Long as L, Pointer as P, Word as W};

/// updatemapmenu, 0x13b212. The enable flag is reread after setting each menu label.
pub fn update_map_menu(
    memory: &RenderingMemory,
    frame: u32,
    limits: StorageLimits,
    services: &mut (impl ScenarioMenuServices + ?Sized),
) -> Result<ScenarioMenuPlan, RuleError> {
    let mut result = MenuExecution::new(memory, limits, Some(0));
    let e = &mut result.storage;
    for index in 1_i16..20 {
        let address = 0x442722_u32.wrapping_add(i32::from(index - 1) as u32);
        let resource = if e.byte(address)? != 0 { -102 } else { -101 };
        e.load(
            LoadRequest::GetIndexedString {
                resource,
                index,
                destination: 0x479ed8,
            },
            services,
        )?;
        e.toolbox(
            0xa947,
            None,
            vec![
                L(e.long(0x43872a)?),
                W(index.wrapping_add(3) as u16),
                P(0x479ed8),
            ],
            services,
        )?;
        let opcode = if e.byte(address)? != 0 {
            0xa939
        } else {
            0xa93a
        };
        e.toolbox(
            opcode,
            None,
            vec![L(e.long(0x43872a)?), W(index.wrapping_add(3) as u16)],
            services,
        )?;
    }
    let handle = e.open(0x401e1a, 0x40250e, false, services)?;
    if handle == 0 {
        e.fatal(frame.wrapping_sub(18), 104, false, services)?;
        if !e.active() {
            return Ok(result.finish());
        }
    }
    e.read(handle, 30000, 0x470cbe, services)?;
    e.close(handle, services)?;
    Ok(result.finish())
}
fn indexed_address(index: i16) -> u32 {
    0x4395a6_u32.wrapping_add((i32::from(index) * 2) as u32)
}
fn menu_label(
    e: &mut Execution,
    old_count: i16,
    services: &(impl ScenarioMenuServices + ?Sized),
) -> Result<(), RuleError> {
    let menu = e.long(0x438722)?;
    if old_count < e.word(0x439cf8)? {
        e.toolbox(0xa933, None, vec![L(menu), P(0x438aa4)], services)?;
    } else {
        e.toolbox(
            0xa947,
            None,
            vec![L(menu), W(e.word(0x439cf8)? as u16), P(0x438aa4)],
            services,
        )?;
    }
    Ok(())
}
fn metadata(
    e: &mut Execution,
    frame: u32,
    services: &(impl ScenarioMenuServices + ?Sized),
) -> Result<(), RuleError> {
    e.lw(frame.wrapping_add(12), 0x4389a4)?;
    e.lw(frame.wrapping_add(8), 0x4026ca)?;
    let name = e.long(frame.wrapping_add(12))?;
    e.load(LoadRequest::CToPascal(name), services)?;
    e.service(
        StorageRequest::GetFileInfo {
            name,
            volume: 0,
            info: 0x439cfc,
        },
        services,
    )?;
    e.load(
        LoadRequest::CopyCString {
            source: e.long(frame.wrapping_add(8))?,
            destination: 0x439cfc,
        },
        services,
    )?;
    e.load(
        LoadRequest::CopyCString {
            source: 0x402436,
            destination: 0x439d00,
        },
        services,
    )?;
    e.service(
        StorageRequest::SetFileInfo {
            name,
            volume: 0,
            info: 0x439cfc,
        },
        services,
    )?;
    e.load(LoadRequest::PascalToC(name), services)?;
    Ok(())
}
/// updatemonstermenu, 0x135bb2. Cache construction sorts actual observed menu strings.
pub fn update_monster_menu(
    memory: &RenderingMemory,
    frame: u32,
    limits: StorageLimits,
    services: &mut (impl ScenarioMenuServices + ?Sized),
) -> Result<ScenarioMenuPlan, RuleError> {
    let mut result = MenuExecution::new(memory, limits, Some(0));
    loop {
        let e = &mut result.storage;
        e.step()?;
        e.write(frame.wrapping_sub(0x201), &[0])?;
        let mut record_index = 0_i16;
        e.ww(0x439cf8, 0)?;
        let old_count = e.toolbox(0xa950, None, vec![L(e.long(0x438722)?)], services)? as i16;
        let cache = e.open(0x402700, 0x4026fc, false, services)?;
        if cache != 0 {
            if e.read(cache, 0x1f6, 0x4395a6, services)? != 0 {
                e.write(frame.wrapping_sub(0x201), &[1])?;
            }
            e.close(cache, services)?;
        }
        if e.word(0x4042c8)? == e.word(frame.wrapping_add(8))? {
            return Ok(result.finish());
        }
        let input = e.open(0x4026f4, 0x4026fc, false, services)?;
        if input == 0 {
            e.fatal(frame.wrapping_sub(548), 116, false, services)?;
            if !e.active() {
                return Ok(result.finish());
            }
        }
        if e.byte(frame.wrapping_sub(0x201))? != 0 {
            let mut index = 0_i16;
            while index < 999 {
                e.step()?;
                if e.word(indexed_address(index))? != 0 {
                    e.add(0x439cf8, 1)?;
                    e.seek(
                        input,
                        u32::from(e.word(indexed_address(index))?.wrapping_sub(1) as u16) * 288,
                        services,
                    )?;
                    e.read(input, 288, 0x440dbc, services)?;
                    if e.word(0x440dbc)? != 0 && e.byte(0x440e63)? == 0 {
                        if e.word(0x440dbc)? == 255 {
                            break;
                        }
                        e.load(
                            LoadRequest::CopyCString {
                                source: 0x440e96,
                                destination: 0x438aa4,
                            },
                            services,
                        )?;
                        menu_label(e, old_count, services)?;
                    }
                }
                index = index.wrapping_add(1);
            }
        } else {
            result.flash(frame.wrapping_sub(558), 0x4026d6, 50, 70, -1, 0, services)?;
            let e = &mut result.storage;
            while e.read(input, 288, 0x440dbc, services)? != 0 {
                e.step()?;
                record_index = record_index.wrapping_add(1);
                e.load(
                    LoadRequest::CopyCString {
                        source: 0x4026d4,
                        destination: 0x438aa4,
                    },
                    services,
                )?;
                if e.word(0x439cf8)? > 999 {
                    break;
                }
                if e.word(0x440dbc)? != 0 && e.byte(0x440e63)? == 0 {
                    if e.word(0x440dbc)? == 255 {
                        break;
                    }
                    e.add(0x439cf8, 1)?;
                    e.ww(indexed_address(e.word(0x439cf8)?), record_index)?;
                    e.load(
                        LoadRequest::CopyCString {
                            source: 0x440e96,
                            destination: 0x438aa4,
                        },
                        services,
                    )?;
                    menu_label(e, old_count, services)?;
                }
            }
        }
        let e = &mut result.storage;
        e.close(input, services)?;
        e.ww(0x4042c8, e.word(frame.wrapping_add(8))?)?;
        if e.byte(frame.wrapping_sub(0x201))? != 0 {
            return Ok(result.finish());
        }
        let first = frame.wrapping_sub(256);
        let second = frame.wrapping_sub(512);
        let mut pass = 1_i16;
        while pass < e.word(0x439cf8)? {
            e.step()?;
            let mut index = 1_i16;
            while index < e.word(0x439cf8)? {
                e.step()?;
                e.toolbox(
                    0xa946,
                    None,
                    vec![L(e.long(0x438722)?), W(index as u16), P(first)],
                    services,
                )?;
                e.toolbox(
                    0xa946,
                    None,
                    vec![
                        L(e.long(0x438722)?),
                        W(index.wrapping_add(1) as u16),
                        P(second),
                    ],
                    services,
                )?;
                e.load(LoadRequest::PascalToC(first), services)?;
                e.load(LoadRequest::PascalToC(second), services)?;
                if e.service(StorageRequest::StringLength(first), services)? as i16 != 0
                    && e.service(StorageRequest::StringLength(second), services)? as i16 != 0
                    && e.service(
                        StorageRequest::CompareCString {
                            left: first,
                            right: second,
                        },
                        services,
                    )? as i16
                        > 0
                {
                    let saved = e.word(indexed_address(index))?;
                    e.ww(
                        indexed_address(index),
                        e.word(indexed_address(index.wrapping_add(1)))?,
                    )?;
                    e.ww(indexed_address(index.wrapping_add(1)), saved)?;
                    e.load(LoadRequest::CToPascal(first), services)?;
                    e.load(LoadRequest::CToPascal(second), services)?;
                    e.toolbox(
                        0xa947,
                        None,
                        vec![
                            L(e.long(0x438722)?),
                            W(index.wrapping_add(1) as u16),
                            P(first),
                        ],
                        services,
                    )?;
                    e.toolbox(
                        0xa947,
                        None,
                        vec![L(e.long(0x438722)?), W(index as u16), P(second)],
                        services,
                    )?;
                }
                index = index.wrapping_add(1);
            }
            pass = pass.wrapping_add(1);
        }
        let output = e.open(0x402700, 0x4026d0, false, services)?;
        if output == 0 {
            e.fatal(frame.wrapping_sub(548), 117, false, services)?;
            if !e.active() {
                return Ok(result.finish());
            }
        }
        e.fwrite(output, 0x1f6, 0x4395a6, services)?;
        e.close(output, services)?;
        metadata(e, frame.wrapping_sub(574), services)?;
        result.flash(frame.wrapping_sub(586), 0x4026c9, 50, 70, -1, 0, services)?;
        result.storage.ww(0x4042c8, -1)?;
    }
}
/// Displaymissionname, 0x133f3e. The mission selector is captured as an unsigned WORD.
pub fn display_mission_name(
    memory: &RenderingMemory,
    frame: u32,
    limits: StorageLimits,
    services: &mut (impl ScenarioMenuServices + ?Sized),
) -> Result<ScenarioMenuPlan, RuleError> {
    let mut result = MenuExecution::new(memory, limits, Some(0));
    let e = &mut result.storage;
    let mission = e.word(frame.wrapping_add(8))? as u16;
    let handle = e.open(0x4028c0, 0x4028bc, false, services)?;
    e.lw(0x4434a6, handle)?;
    e.read(e.long(0x4434a6)?, 0x2440, 0x41d1c4, services)?;
    e.close(e.long(0x4434a6)?, services)?;
    let source = 0x41d1c4 + u32::from(mission) * 464;
    for (offset, destination) in [(4, 0x442774), (0, 0x442778), (2, 0x44277c)] {
        e.lw(destination, i32::from(e.word(source + offset)?) as u32)?;
    }
    e.load(
        LoadRequest::CopyCString {
            source: source + 6,
            destination: 0x479ed8,
        },
        services,
    )?;
    e.load(LoadRequest::PascalToC(0x479ed8), services)?;
    result.flash(frame.wrapping_sub(68), 0x479ed8, 50, 50, 0, 133, services)?;
    Ok(result.finish())
}
/// getword, 0x127132. This native void routine ends at an observed RGBForeColor call.
pub fn get_word(
    memory: &RenderingMemory,
    frame: u32,
    limits: StorageLimits,
    services: &mut (impl ScenarioMenuServices + ?Sized),
) -> Result<ScenarioMenuPlan, RuleError> {
    let mut result = MenuExecution::new(memory, limits, None);
    let e = &mut result.storage;
    e.toolbox(0xa874, None, vec![P(frame.wrapping_sub(16))], services)?;
    e.toolbox(0xaa19, None, vec![P(frame.wrapping_sub(6))], services)?;
    e.toolbox(0xaa1a, None, vec![P(frame.wrapping_sub(12))], services)?;
    let cursor = e.toolbox(0xaa1b, None, vec![W(155)], services)?;
    e.toolbox(0xaa1c, None, vec![L(cursor)], services)?;
    e.toolbox(0xaa26, None, vec![L(cursor)], services)?;
    let dialog = e.toolbox(0xa97c, None, vec![W(157), L(0), L(u32::MAX)], services)?;
    e.toolbox(0xa873, None, vec![L(dialog)], services)?;
    e.toolbox(0xaa0b, None, vec![L(e.long(0x4358ce)?)], services)?;
    e.toolbox(0xa887, None, vec![W(e.word(0x402f44)? as u16)], services)?;
    e.toolbox(0xa88a, None, vec![W(18)], services)?;
    e.toolbox(0xa862, None, vec![L(69)], services)?;
    let horizontal = e.word(0x4395a4)?.wrapping_add(30);
    let vertical = e
        .word(0x4395a2)?
        .wrapping_add(110)
        .wrapping_add(e.word(0x40856e)?);
    e.service(
        StorageRequest::MoveWindow {
            window: dialog,
            horizontal,
            vertical,
            front: 0,
        },
        services,
    )?;
    e.toolbox(0xa915, None, vec![L(dialog)], services)?;
    loop {
        e.step()?;
        e.toolbox(0xa032, Some(0xffff), vec![], services)?;
        e.toolbox(0xa991, None, vec![L(0), P(0x4434b2)], services)?;
        if e.word(0x4434b2)? == 4 {
            break;
        }
    }
    e.toolbox(
        0xa98d,
        None,
        vec![L(dialog), W(4), P(0x4434b0), P(0x4434dc), P(0x44292c)],
        services,
    )?;
    let child_frame = frame.wrapping_sub(46);
    e.lw(child_frame + 18, e.long(0x442930)?)?;
    e.lw(child_frame + 14, e.long(0x44292c)?)?;
    e.ww(child_frame + 12, 0)?;
    e.lw(child_frame + 8, 0x10001)?;
    let child = recovered_feedback::pressbutton(&e.plan.memory, child_frame, 1, 1, 0, services)?;
    e.plan.memory = child.memory;
    result.flush();
    result
        .effects
        .extend(child.effects.into_iter().map(ScenarioMenuEffect::Feedback));
    let e = &mut result.storage;
    e.toolbox(
        0xa98d,
        None,
        vec![L(dialog), W(2), P(0x4434b0), P(0x4434dc), P(0x4434ba)],
        services,
    )?;
    e.toolbox(
        0xa990,
        None,
        vec![L(e.long(0x4434dc)?), P(0x438aa4)],
        services,
    )?;
    e.toolbox(0xa983, None, vec![L(dialog)], services)?;
    e.toolbox(0xaa1c, None, vec![L(e.long(0x43952c)?)], services)?;
    e.toolbox(
        0xa873,
        None,
        vec![L(e.long(frame.wrapping_sub(16))?)],
        services,
    )?;
    e.toolbox(0xaa15, None, vec![P(frame.wrapping_sub(12))], services)?;
    e.toolbox(0xaa14, None, vec![P(frame.wrapping_sub(6))], services)?;
    Ok(result.finish())
}
fn drawing_context(e: &Execution, tile: i16) -> Result<DrawingContext, RuleError> {
    let rect = if tile >= 23000 {
        e.plan.memory.rect(0x4471f2)?
    } else {
        crate::recovered_geometry::BodyRect {
            top: 0,
            left: 0,
            bottom: 0,
            right: 0,
        }
    };
    let mut buildings = vec![];
    let mut depth = 0;
    if tile >= 23000 {
        let index = usize::from(tile.wrapping_sub(23000) as u16);
        let pointer = e.long(0x41c220 + index as u32 * 4)?;
        let bounds = if pointer == 0 {
            depth = e.byte(0x43873f)? as i8;
            None
        } else {
            Some(e.plan.memory.rect(pointer.wrapping_add(16))?)
        };
        buildings.resize(
            index + 1,
            BuildingWorld {
                pointer: 0,
                bounds: None,
            },
        );
        buildings[index] = BuildingWorld { pointer, bounds };
    }
    Ok(DrawingContext {
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
        map_port: if tile >= 0 { e.long(0x43879c)? } else { 0 },
        text_port: 0,
        secondary_port: 0,
        backing_port: if tile >= 23000 { e.long(0x4386bc)? } else { 0 },
        tile_port: if (0..23000).contains(&tile) {
            e.long(0x4386c4)?
        } else {
            0
        },
        depth,
        old_world: 0,
        old_device: 0,
        buildings,
    })
}
/// fastplotmap, 0x134d9e. Positive tiles use direct mode-zero CopyBits.
pub fn fast_plot_map(
    memory: &RenderingMemory,
    frame: u32,
    limits: StorageLimits,
    services: &mut (impl ScenarioMenuServices + ?Sized),
) -> Result<ScenarioMenuPlan, RuleError> {
    let mut result = MenuExecution::new(memory, limits, Some(0));
    let e = &mut result.storage;
    let tile = e.word(frame.wrapping_add(8))?;
    e.lw(frame.wrapping_sub(4), 0)?;
    if tile < 0 {
        let destination = e.plan.memory.rect(frame.wrapping_add(10))?;
        let index = i32::from(e.byte(0x402f43)? as i8) * 2;
        let background = e.word(0x435508_u32.wrapping_add(index as u32))?;
        let mut context = drawing_context(e, background)?;
        let child =
            recovered_drawing::fastplot(&mut context, background, destination, 0, 0, services)?;
        result.drawing(child.effects)?;
        let destination = result.storage.plan.memory.rect(frame.wrapping_add(10))?;
        let child = recovered_drawing::ploticon3(&mut context, tile, destination, services)?;
        result.drawing(child.effects)?;
    } else {
        e.ww(0x4434b4, tile.wrapping_sub(1) / 20)?;
        e.ww(
            0x4434b6,
            tile.wrapping_sub(e.word(0x4434b4)?.wrapping_mul(20))
                .wrapping_sub(1),
        )?;
        e.ww(0x4434ba, e.word(0x4434b4)?.wrapping_shl(5))?;
        e.ww(0x4434bc, e.word(0x4434b6)?.wrapping_shl(5))?;
        e.ww(0x4434c0, e.word(0x4434bc)?.wrapping_add(32))?;
        e.ww(0x4434be, e.word(0x4434ba)?.wrapping_add(32))?;
        e.toolbox(
            0xa8ec,
            None,
            vec![
                P(e.long(0x4386c4)?.wrapping_add(2)),
                P(e.long(0x43879c)?.wrapping_add(2)),
                P(0x4434ba),
                P(frame.wrapping_add(10)),
                W(0),
                L(0),
            ],
            services,
        )?;
    }
    Ok(result.finish())
}
