//! Combat presentation routines composed from recovered rule and drawing bodies.

use crate::RuleError;
use crate::recovered_drawing::{
    self as drawing, BuildingWorld, DrawingContext, DrawingEffect, DrawingServices,
    PictureObservation,
};
use crate::recovered_geometry::BodyRect;
use crate::recovered_records::RecordWrite;
use crate::recovered_rendering::RenderingMemory;
use crate::recovered_spatial::{self as spatial, Position, RangeWork, SpatialServices};
use crate::recovered_text::{self as text, NumericServices, PascalString, TextContext, TextEffect};
use crate::recovered_timing::{self as timing, TimingServices};
use centurions_content::ITEM_BYTES;
use std::collections::VecDeque;

pub type CombatUiMemory = RenderingMemory;

#[derive(Clone, Debug, PartialEq)]
pub enum CombatUiRequest {
    Drawing(DrawingEffect),
    Text(TextEffect),
    TextMode(i16),
    RgbForeColor(u32),
    SetRect {
        pointer: u32,
        rect: BodyRect,
    },
    OffsetRect {
        pointer: u32,
        horizontal: i16,
        vertical: i16,
    },
    Sane {
        operation: u16,
        source: u32,
        destination: Option<u32>,
    },
    TickCount,
    Button,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombatUiObservation {
    pub value: u32,
    /// Actual full D0 after the external boundary; no upper-WORD preservation is assumed.
    pub d0: u32,
    pub memory: CombatUiMemory,
}

pub trait CombatUiServices {
    fn observe(
        &self,
        request: &CombatUiRequest,
        memory: &CombatUiMemory,
    ) -> Result<CombatUiObservation, RuleError>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum CombatUiEffect {
    Write(RecordWrite),
    Service {
        request: CombatUiRequest,
        observation: CombatUiObservation,
    },
    RangeQuery {
        body: i16,
        target: Position,
        maximum: i16,
        in_range: bool,
        work: RangeWork,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct CombatUiPlan {
    pub memory: CombatUiMemory,
    pub effects: Vec<CombatUiEffect>,
    /// None for native void bodies whose final D0 has no recovered return contract.
    pub native_return: Option<u32>,
}

fn plan(memory: &CombatUiMemory, native_return: Option<u32>) -> CombatUiPlan {
    CombatUiPlan {
        memory: memory.clone(),
        effects: Vec::new(),
        native_return,
    }
}

fn missing(label: &str) -> RuleError {
    RuleError::Invalid(format!("combat UI observation unavailable: {label}"))
}

fn write(result: &mut CombatUiPlan, address: u32, bytes: &[u8]) -> Result<(), RuleError> {
    for offset in 0..bytes.len() {
        result.memory.byte(address.wrapping_add(offset as u32))?;
    }
    result.memory.insert_region(address, bytes);
    result.effects.push(CombatUiEffect::Write(RecordWrite {
        address,
        bytes: bytes.to_vec(),
    }));
    Ok(())
}

fn word(result: &mut CombatUiPlan, address: u32, value: i16) -> Result<(), RuleError> {
    write(result, address, &value.to_be_bytes())
}
fn add_word(result: &mut CombatUiPlan, address: u32, value: i16) -> Result<(), RuleError> {
    let value = (result.memory.word(address)? as i16).wrapping_add(value);
    word(result, address, value)
}

fn observed(
    result: &mut CombatUiPlan,
    request: CombatUiRequest,
    observation: CombatUiObservation,
) -> u32 {
    let value = observation.value;
    result.memory = observation.memory.clone();
    result.effects.push(CombatUiEffect::Service {
        request,
        observation,
    });
    value
}

fn service(
    result: &mut CombatUiPlan,
    request: CombatUiRequest,
    services: &(impl CombatUiServices + ?Sized),
) -> Result<u32, RuleError> {
    let observation = services.observe(&request, &result.memory)?;
    Ok(observed(result, request, observation))
}

fn draw(
    result: &mut CombatUiPlan,
    effect: DrawingEffect,
    services: &(impl CombatUiServices + ?Sized),
) -> Result<u32, RuleError> {
    service(result, CombatUiRequest::Drawing(effect), services)
}

fn half_word(value: i16) -> i16 {
    value.wrapping_add(((value as u16) >> 15) as i16) >> 1
}
fn offset(address: u32, displacement: i32) -> u32 {
    address.wrapping_add(displacement as u32)
}
fn record(body: i16, warrior: bool) -> u32 {
    if warrior {
        0x448aa0_u32.wrapping_add(u32::from(body as u16) * 3138)
    } else {
        0x439d3c_u32.wrapping_add(u32::from(body.wrapping_sub(10) as u16) * 288)
    }
}
fn position_pointer(body: i16, warrior: bool) -> u32 {
    if warrior {
        offset(0x44709a, i32::from(body) * 4)
    } else {
        offset(0x446f0a, i32::from(body.wrapping_sub(10)) * 4)
    }
}

fn position(memory: &CombatUiMemory, body: i16, warrior: bool) -> Result<Position, RuleError> {
    let pointer = position_pointer(body, warrior);
    Ok((
        memory.word(pointer)? as i16,
        memory.word(pointer.wrapping_add(2))? as i16,
    ))
}

fn pascal(memory: &CombatUiMemory, address: u32) -> Result<PascalString, RuleError> {
    let count = usize::from(memory.byte(address)?) + 1;
    let bytes: Result<Vec<_>, _> = (0..count)
        .map(|i| memory.byte(address.wrapping_add(i as u32)))
        .collect();
    PascalString::from_raw(&bytes?)
}

struct NumericBridge<'a, S: CombatUiServices + ?Sized> {
    services: &'a S,
    memory: CombatUiMemory,
    observations: VecDeque<CombatUiObservation>,
}

impl<S: CombatUiServices + ?Sized> NumericServices for NumericBridge<'_, S> {
    fn number_to_string(&mut self, value: i32) -> Result<PascalString, RuleError> {
        let request = CombatUiRequest::Text(TextEffect::NumberToString(value));
        let observation = self.services.observe(&request, &self.memory)?;
        let string = pascal(&observation.memory, 0x479ed8)?;
        self.memory = observation.memory.clone();
        self.observations.push_back(observation);
        Ok(string)
    }
    fn string_to_number(&mut self, _: &PascalString) -> Result<i32, RuleError> {
        Err(missing("unused numeric conversion"))
    }
}

fn string(
    result: &mut CombatUiPlan,
    value: i16,
    services: &(impl CombatUiServices + ?Sized),
) -> Result<(), RuleError> {
    let mut bridge = NumericBridge {
        services,
        memory: result.memory.clone(),
        observations: VecDeque::new(),
    };
    let effects = text::string(&mut TextContext::default(), value, &mut bridge)?;
    for effect in effects {
        let request = CombatUiRequest::Text(effect.clone());
        if matches!(effect, TextEffect::NumberToString(_)) {
            let observation = bridge
                .observations
                .pop_front()
                .ok_or_else(|| missing("numeric reply"))?;
            observed(result, request, observation);
        } else {
            service(result, request, services)?;
        }
    }
    Ok(())
}

struct DrawingBridge<'a, S: CombatUiServices + ?Sized> {
    services: &'a S,
    memory: CombatUiMemory,
    observations: VecDeque<CombatUiObservation>,
}

impl<S: CombatUiServices + ?Sized> DrawingServices for DrawingBridge<'_, S> {
    fn color_icon(&mut self, id: i16) -> Result<u32, RuleError> {
        let request = CombatUiRequest::Drawing(DrawingEffect::GetColorIcon(id));
        let observation = self.services.observe(&request, &self.memory)?;
        let handle = observation.value;
        self.memory = observation.memory.clone();
        self.observations.push_back(observation);
        Ok(handle)
    }
    fn picture(&mut self, id: i16) -> Result<PictureObservation, RuleError> {
        let request = CombatUiRequest::Drawing(DrawingEffect::GetPicture(id));
        let observation = self.services.observe(&request, &self.memory)?;
        let handle = observation.value;
        self.memory = observation.memory.clone();
        self.observations.push_back(observation);
        Ok(PictureObservation {
            handle,
            bounds: None,
        })
    }
    fn indexed_string(&mut self, _: i16, _: i16) -> Result<PascalString, RuleError> {
        Err(missing("unused indexed string"))
    }
    fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
        Err(missing("unused world query"))
    }
    fn new_world(&mut self, _: i16, _: BodyRect) -> Result<(i16, BuildingWorld), RuleError> {
        Err(missing("unused world allocation"))
    }
}

fn drawing_context(rect: BodyRect) -> DrawingContext {
    // These three recovered children only read rect and update icon_handle.
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
        buildings: Vec::new(),
    }
}

fn merge_drawing(
    result: &mut CombatUiPlan,
    effects: Vec<DrawingEffect>,
    replies: &mut VecDeque<CombatUiObservation>,
    global_rect: bool,
    services: &(impl CombatUiServices + ?Sized),
) -> Result<(), RuleError> {
    for effect in effects {
        match effect {
            DrawingEffect::Write(value) => write(result, value.address, &value.bytes)?,
            DrawingEffect::GetColorIcon(_) | DrawingEffect::GetPicture(_) => {
                let observation = replies
                    .pop_front()
                    .ok_or_else(|| missing("drawing resource reply"))?;
                observed(result, CombatUiRequest::Drawing(effect), observation);
            }
            DrawingEffect::PlotColorIcon { rect, .. } => {
                let handle = result.memory.long(0x439598)?;
                let rect = if global_rect {
                    result.memory.rect(0x4471f2)?
                } else {
                    rect
                };
                draw(
                    result,
                    DrawingEffect::PlotColorIcon { handle, rect },
                    services,
                )?;
            }
            DrawingEffect::DisposeColorIcon(_) => {
                let handle = result.memory.long(0x439598)?;
                draw(result, DrawingEffect::DisposeColorIcon(handle), services)?;
            }
            _ => {
                draw(result, effect, services)?;
            }
        }
    }
    Ok(())
}

fn plot(
    result: &mut CombatUiPlan,
    id: i16,
    rect: BodyRect,
    complete: bool,
    services: &(impl CombatUiServices + ?Sized),
) -> Result<(), RuleError> {
    let mut memory = result.memory.clone();
    if !complete {
        memory.insert_region(0x4471f8, &rect.left.wrapping_add(32).to_be_bytes());
        memory.insert_region(0x4471f6, &rect.top.wrapping_add(32).to_be_bytes());
    }
    memory.insert_region(0x439598, &0_u32.to_be_bytes());
    let mut bridge = DrawingBridge {
        services,
        memory,
        observations: VecDeque::new(),
    };
    let mut context = drawing_context(rect);
    let child = if complete {
        drawing::ploticon3(&mut context, id, rect, &mut bridge)?
    } else {
        drawing::ploticon2(&mut context, id, &mut bridge)?
    };
    merge_drawing(
        result,
        child.effects,
        &mut bridge.observations,
        !complete,
        services,
    )
}

fn pict(
    result: &mut CombatUiPlan,
    id: i16,
    rect: BodyRect,
    services: &(impl CombatUiServices + ?Sized),
) -> Result<(), RuleError> {
    let mut bridge = DrawingBridge {
        services,
        memory: result.memory.clone(),
        observations: VecDeque::new(),
    };
    let effects = drawing::pict(id, rect, &mut bridge)?;
    merge_drawing(result, effects, &mut bridge.observations, false, services)
}

struct TimingBridge<'a, S: CombatUiServices + ?Sized> {
    services: &'a S,
    result: &'a mut CombatUiPlan,
    local_limit: u32,
    duration: i16,
    first: bool,
}
impl<S: CombatUiServices + ?Sized> TimingServices for TimingBridge<'_, S> {
    fn tick_count(&mut self) -> Result<u32, RuleError> {
        if self.first {
            self.first = false;
        } else {
            write(
                self.result,
                self.local_limit,
                &(i32::from(self.duration) as u32).to_be_bytes(),
            )?;
        }
        service(self.result, CombatUiRequest::TickCount, self.services)
    }
    fn button(&mut self) -> Result<bool, RuleError> {
        Ok(service(self.result, CombatUiRequest::Button, self.services)? as u8 != 0)
    }
}

fn delay(
    result: &mut CombatUiPlan,
    frame_pointer: u32,
    duration: i16,
    services: &(impl CombatUiServices + ?Sized),
) -> Result<(), RuleError> {
    let mut bridge = TimingBridge {
        services,
        result,
        local_limit: frame_pointer.wrapping_sub(54),
        duration,
        first: true,
    };
    timing::delay(duration, 0, &mut bridge)?;
    Ok(())
}

/// combatupdate, 0011edc4: movement, HP, statistic, and two attack-action fields.
pub fn combatupdate(
    memory: &CombatUiMemory,
    frame_pointer: u32,
    body: i16,
    services: &(impl CombatUiServices + ?Sized),
) -> Result<CombatUiPlan, RuleError> {
    let mut result = plan(memory, Some(0));
    if body < 0 {
        return Ok(result);
    }
    let port = result.memory.long(0x43878c)?;
    draw(&mut result, DrawingEffect::SetPort(port), services)?;
    let font = result.memory.word(0x402f3c)? as i16;
    draw(&mut result, DrawingEffect::TextFont(font), services)?;
    draw(&mut result, DrawingEffect::TextSize(12), services)?;
    service(&mut result, CombatUiRequest::TextMode(1), services)?;
    let r = record(body, body < 6);
    let (mut movement, hp, first_attack, second_attack);
    if body < 6 {
        movement = result.memory.word(r.wrapping_add(0x3c))? as i16;
        let maximum = result.memory.word(r.wrapping_add(0x3e))? as i16;
        word(&mut result, frame_pointer.wrapping_sub(2), maximum)?;
        hp = result.memory.word(r.wrapping_add(0x50))? as i16;
        let maximum = result.memory.word(r.wrapping_add(0x52))? as i16;
        word(&mut result, frame_pointer.wrapping_sub(4), maximum)?;
        let statistic = result.memory.word(r.wrapping_add(0x26))? as i16;
        word(&mut result, frame_pointer.wrapping_sub(6), statistic)?;
        let value = result.memory.word(r.wrapping_add(0xaca))? as i16;
        first_attack = value.min(half_word(result.memory.word(r.wrapping_add(0x40))? as i16));
        let value = result.memory.word(r.wrapping_add(0xac6))? as i16;
        second_attack = value.min(half_word(result.memory.word(r.wrapping_add(0x40))? as i16));
    } else {
        movement = i16::from(result.memory.byte(r.wrapping_add(0x106))? as i8);
        let maximum = result.memory.word(r.wrapping_add(6))? as i16;
        word(&mut result, frame_pointer.wrapping_sub(2), maximum)?;
        hp = result.memory.word(r.wrapping_add(0x110))? as i16;
        let maximum = result.memory.word(r.wrapping_add(0x112))? as i16;
        word(&mut result, frame_pointer.wrapping_sub(4), maximum)?;
        let statistic = result.memory.word(r.wrapping_add(0x58))? as i16;
        word(&mut result, frame_pointer.wrapping_sub(6), statistic)?;
        first_attack = half_word(i16::from(result.memory.byte(r.wrapping_add(0x8a))? as i8));
        second_attack = i16::from(result.memory.byte(r.wrapping_add(0x7e))? as i8);
    }
    let top = ui_vertical(&result.memory, 0x179)?;
    word(&mut result, 0x4471bc, top)?;
    word(&mut result, 0x4471be, 112)?;
    word(&mut result, 0x4471c2, 144)?;
    let bottom = (result.memory.word(0x4471bc)? as i16).wrapping_add(14);
    word(&mut result, 0x4471c0, bottom)?;
    erase(&mut result, 0x4471bc, services)?;
    if (result.memory.word(0x4434b6)? as i16)
        < result.memory.word(frame_pointer.wrapping_sub(2))? as i16
    {
        service(
            &mut result,
            CombatUiRequest::RgbForeColor(0x44337a),
            services,
        )?;
    } else {
        draw(&mut result, DrawingEffect::ForeColor(69), services)?;
    }
    move_to(&mut result, 112, 0x186, services)?;
    if movement < 0 {
        movement = 0;
    }
    string(&mut result, movement, services)?;
    let top = ui_vertical(&result.memory, 0x14b)?;
    word(&mut result, 0x4471bc, top)?;
    let bottom = (result.memory.word(0x4471bc)? as i16).wrapping_add(14);
    word(&mut result, 0x4471c0, bottom)?;
    word(&mut result, 0x4471be, 112)?;
    word(&mut result, 0x4471c2, 144)?;
    erase(&mut result, 0x4471bc, services)?;
    if hp < result.memory.word(frame_pointer.wrapping_sub(4))? as i16 {
        service(
            &mut result,
            CombatUiRequest::RgbForeColor(0x44337a),
            services,
        )?;
    } else {
        draw(&mut result, DrawingEffect::ForeColor(69), services)?;
    }
    move_to(&mut result, 112, 0x158, services)?;
    string(&mut result, hp, services)?;
    draw(&mut result, DrawingEffect::ForeColor(69), services)?;
    add_word(&mut result, 0x4471be, 105)?;
    let right = (result.memory.word(0x4471be)? as i16).wrapping_add(35);
    word(&mut result, 0x4471c2, right)?;
    erase(&mut result, 0x4471bc, services)?;
    move_to(&mut result, 217, 0x158, services)?;
    let statistic = result.memory.word(frame_pointer.wrapping_sub(6))? as i16;
    string(&mut result, statistic, services)?;
    word(&mut result, 0x4471be, 257)?;
    word(&mut result, 0x4471c2, 268)?;
    let top = ui_vertical(&result.memory, 0x179)?;
    word(&mut result, 0x4471bc, top)?;
    let bottom = (result.memory.word(0x4471bc)? as i16).wrapping_add(14);
    word(&mut result, 0x4471c0, bottom)?;
    erase(&mut result, 0x4471bc, services)?;
    move_to(&mut result, 257, 0x185, services)?;
    string(&mut result, first_attack, services)?;
    add_word(&mut result, 0x4471be, 34)?;
    add_word(&mut result, 0x4471c2, 34)?;
    erase(&mut result, 0x4471bc, services)?;
    move_to(&mut result, 291, 0x185, services)?;
    string(&mut result, second_attack, services)?;
    draw(&mut result, DrawingEffect::TextFace(0), services)?;
    Ok(result)
}

fn ui_vertical(memory: &CombatUiMemory, addition: i16) -> Result<i16, RuleError> {
    Ok((memory.word(0x40856e)? as i16)
        .wrapping_add(addition)
        .wrapping_sub((memory.word(0x439582)? as i16).wrapping_shl(5)))
}
fn erase(
    result: &mut CombatUiPlan,
    pointer: u32,
    services: &(impl CombatUiServices + ?Sized),
) -> Result<(), RuleError> {
    let rect = result.memory.rect(pointer)?;
    draw(result, DrawingEffect::EraseRect(rect), services)?;
    Ok(())
}
fn move_to(
    result: &mut CombatUiPlan,
    horizontal: i16,
    addition: i16,
    services: &(impl CombatUiServices + ?Sized),
) -> Result<(), RuleError> {
    let vertical = ui_vertical(&result.memory, addition)?;
    draw(
        result,
        DrawingEffect::MoveTo {
            horizontal,
            vertical,
        },
        services,
    )?;
    Ok(())
}

struct MemorySpatial<'a>(&'a CombatUiMemory);
impl SpatialServices for MemorySpatial<'_> {
    fn warrior_position(&self, index: i16) -> Option<Position> {
        position(self.0, index, true).ok()
    }
    fn enemy_position(&self, index: i16) -> Option<Position> {
        position(self.0, index.wrapping_add(10), false).ok()
    }
    fn warrior_byte(&self, index: i16, field: usize) -> Option<u8> {
        self.0
            .byte(record(index, true).wrapping_add(field as u32))
            .ok()
    }
    fn warrior_word(&self, index: i16, field: usize) -> Option<i16> {
        self.0
            .word(record(index, true).wrapping_add(field as u32))
            .ok()
            .map(|v| v as i16)
    }
    fn enemy_byte(&self, index: i16, field: usize) -> Option<u8> {
        self.0
            .byte(record(index.wrapping_add(10), false).wrapping_add(field as u32))
            .ok()
    }
    fn enemy_word(&self, index: i16, field: usize) -> Option<i16> {
        self.0
            .word(record(index.wrapping_add(10), false).wrapping_add(field as u32))
            .ok()
            .map(|v| v as i16)
    }
    fn map_word(&self, row: i32, column: i32) -> Option<i16> {
        self.0
            .word(offset(
                0x4085b8,
                row.wrapping_mul(180).wrapping_add(column.wrapping_mul(2)),
            ))
            .ok()
            .map(|v| v as i16)
    }
}

// Represent the recovered spatial helper's finite f64 result as native extended bytes.
// Native SANE sqrt precision remains an independent comparison boundary.
fn extended(value: f64) -> Result<[u8; 10], RuleError> {
    if !value.is_finite() || value < 0.0 {
        return Err(missing("finite nonnegative distance"));
    }
    if value == 0.0 {
        return Ok([0; 10]);
    }
    let bits = value.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    if exponent == 0 {
        return Err(missing("normalized distance representation"));
    }
    let exponent = (exponent - 1023 + 16383) as u16;
    let mantissa = ((bits & ((1_u64 << 52) - 1)) | (1_u64 << 52)) << 11;
    let mut bytes = [0; 10];
    bytes[..2].copy_from_slice(&exponent.to_be_bytes());
    bytes[2..].copy_from_slice(&mantissa.to_be_bytes());
    Ok(bytes)
}

/// flashrange, 00123b56. Body nine intentionally uses NPC position index minus one.
pub fn flashrange(
    memory: &CombatUiMemory,
    frame_pointer: u32,
    target_body: i16,
    actor: i16,
    services: &(impl CombatUiServices + ?Sized),
) -> Result<CombatUiPlan, RuleError> {
    let mut result = plan(memory, None);
    let target = position(&result.memory, target_body, target_body < 9)?;
    let maximum = result.memory.word(0x4435bc)? as i16;
    let (in_range, work) =
        spatial::checkrange(actor, target, maximum, &MemorySpatial(&result.memory))
            .map_err(|e| RuleError::Invalid(e.to_string()))?;
    result.effects.push(CombatUiEffect::RangeQuery {
        body: actor,
        target,
        maximum,
        in_range,
        work,
    });
    write(&mut result, 0x44349c, &extended(work.distance)?)?;
    word(&mut result, 0x4434b6, work.range_word)?;
    let local = frame_pointer.wrapping_sub(10);
    for (native, destination, size) in [
        (0x44349c, local, 4),
        (0x4434a0, local.wrapping_add(4), 4),
        (0x4434a4, local.wrapping_add(8), 2),
    ] {
        let bytes: Result<Vec<_>, _> = (0..size).map(|i| result.memory.byte(native + i)).collect();
        write(&mut result, destination, &bytes?)?;
    }
    service(
        &mut result,
        CombatUiRequest::Sane {
            operation: 0x16,
            source: local,
            destination: None,
        },
        services,
    )?;
    service(
        &mut result,
        CombatUiRequest::Sane {
            operation: 0x2810,
            source: local,
            destination: Some(local),
        },
        services,
    )?;
    let shown_range = result.memory.word(local.wrapping_add(2))? as i16;
    let port = result.memory.long(0x43878c)?;
    draw(&mut result, DrawingEffect::SetPort(port), services)?;
    let bounds = result.memory.rect(0x44370a)?;
    pict(&mut result, 156, bounds, services)?;
    draw(&mut result, DrawingEffect::TextSize(18), services)?;
    service(
        &mut result,
        CombatUiRequest::RgbForeColor(0x443374),
        services,
    )?;
    let horizontal = (result.memory.word(0x40856c)? as i16).wrapping_add(425);
    move_to(&mut result, horizontal, 0x1b8, services)?;
    let prefix = pascal(&result.memory, 0x403786)?;
    service(
        &mut result,
        CombatUiRequest::Text(TextEffect::DrawString(prefix)),
        services,
    )?;
    string(&mut result, shown_range, services)?;
    Ok(result)
}

fn copy_bits(
    result: &mut CombatUiPlan,
    source_bitmap: u32,
    destination_bitmap: u32,
    source: u32,
    destination: u32,
    mode: i16,
    services: &(impl CombatUiServices + ?Sized),
) -> Result<(), RuleError> {
    let source = result.memory.rect(source)?;
    let destination = result.memory.rect(destination)?;
    draw(
        result,
        DrawingEffect::CopyBits {
            source_bitmap,
            destination_bitmap,
            source,
            destination,
            mode,
            mask: 0,
        },
        services,
    )?;
    Ok(())
}

fn flash_one(
    result: &mut CombatUiPlan,
    frame_pointer: u32,
    target: u32,
    saved: u32,
    other: u32,
    services: &(impl CombatUiServices + ?Sized),
) -> Result<(), RuleError> {
    let backup = result.memory.long(0x4386d4)?.wrapping_add(2);
    copy_bits(result, 0x3fe288, backup, target, saved, 0, services)?;
    let flash = result.memory.long(0x4386d0)?.wrapping_add(2);
    copy_bits(
        result,
        flash,
        0x3fe288,
        frame_pointer.wrapping_sub(40),
        target,
        36,
        services,
    )?;
    delay(result, frame_pointer, 3, services)?;
    let backup = result.memory.long(0x4386d4)?.wrapping_add(2);
    copy_bits(result, backup, 0x3fe288, saved, target, 0, services)?;
    let duration = if result.memory.word(other)? == 0 {
        10
    } else {
        5
    };
    delay(result, frame_pointer, duration, services)
}

/// flashrect, 00123f96: save, illuminate, wait, restore each configured Rect.
pub fn flashrect(
    memory: &CombatUiMemory,
    frame_pointer: u32,
    index: i16,
    services: &(impl CombatUiServices + ?Sized),
) -> Result<CombatUiPlan, RuleError> {
    let mut result = plan(memory, None);
    if result.memory.word(0x439582)? == 0 {
        return Ok(result);
    }
    service(
        &mut result,
        CombatUiRequest::SetRect {
            pointer: frame_pointer.wrapping_sub(40),
            rect: BodyRect {
                top: 0,
                left: 0,
                bottom: 50,
                right: 50,
            },
        },
        services,
    )?;
    let table = offset(0x4046ec, i32::from(index).wrapping_shl(4));
    let first = frame_pointer.wrapping_sub(8);
    let second = frame_pointer.wrapping_sub(16);
    for (source, destination) in [(table, first), (table.wrapping_add(8), second)] {
        let value = result.memory.long(source)?;
        write(&mut result, destination, &value.to_be_bytes())?;
        let value = result.memory.long(source.wrapping_add(4))?;
        write(
            &mut result,
            destination.wrapping_add(4),
            &value.to_be_bytes(),
        )?;
    }
    for (destination, global) in [
        (first.wrapping_add(2), 0x4395a4),
        (first.wrapping_add(6), 0x4395a4),
        (first, 0x4395a2),
        (first.wrapping_add(4), 0x4395a2),
        (second.wrapping_add(2), 0x4395a4),
        (second.wrapping_add(6), 0x4395a4),
        (second, 0x4395a2),
        (second.wrapping_add(4), 0x4395a2),
    ] {
        let value = result.memory.word(global)? as i16;
        add_word(&mut result, destination, value)?;
    }
    for (target, saved) in [
        (first, frame_pointer.wrapping_sub(24)),
        (second, frame_pointer.wrapping_sub(32)),
    ] {
        word(&mut result, saved.wrapping_add(2), 0)?;
        word(&mut result, saved, 0)?;
        let width = (result.memory.word(target.wrapping_add(6))? as i16)
            .wrapping_sub(result.memory.word(target.wrapping_add(2))? as i16);
        word(&mut result, saved.wrapping_add(6), width)?;
        let height = (result.memory.word(target.wrapping_add(4))? as i16)
            .wrapping_sub(result.memory.word(target)? as i16);
        word(&mut result, saved.wrapping_add(4), height)?;
    }
    draw(&mut result, DrawingEffect::ForeColor(33), services)?;
    draw(&mut result, DrawingEffect::BackColor(30), services)?;
    if result.memory.word(first)? != 0 {
        flash_one(
            &mut result,
            frame_pointer,
            first,
            frame_pointer.wrapping_sub(24),
            second,
            services,
        )?;
    }
    if result.memory.word(second)? != 0 {
        flash_one(
            &mut result,
            frame_pointer,
            second,
            frame_pointer.wrapping_sub(32),
            first,
            services,
        )?;
    }
    Ok(result)
}

fn centered_body_rect(result: &mut CombatUiPlan, body: i16, pointer: u32) -> Result<(), RuleError> {
    let enemy = body > 9;
    let size = if enemy {
        (result.memory.word(record(body, false).wrapping_add(16))? as i16).wrapping_abs()
    } else {
        0
    };
    let position = position_pointer(body, !enemy);
    let vertical = result.memory.word(position.wrapping_add(2))? as i16;
    word(result, pointer, vertical.wrapping_shl(5))?;
    let horizontal = result.memory.word(position)? as i16;
    word(result, pointer.wrapping_add(2), horizontal.wrapping_shl(5))?;
    if matches!(size, 1 | 3) {
        add_word(result, pointer, -16)?;
    }
    if size > 1 {
        add_word(result, pointer.wrapping_add(2), -16)?;
    }
    let bottom = (result.memory.word(pointer)? as i16).wrapping_add(32);
    word(result, pointer.wrapping_add(4), bottom)?;
    let right = (result.memory.word(pointer.wrapping_add(2))? as i16).wrapping_add(32);
    word(result, pointer.wrapping_add(6), right)
}

/// showdamage, 001553bc: selected status icon and optional signed WORD amount.
pub fn showdamage(
    memory: &CombatUiMemory,
    body: i16,
    icon: i16,
    amount: i16,
    services: &(impl CombatUiServices + ?Sized),
) -> Result<CombatUiPlan, RuleError> {
    let mut result = plan(memory, Some(0));
    centered_body_rect(&mut result, body, 0x44371a)?;
    let port = result.memory.long(0x43879c)?;
    draw(&mut result, DrawingEffect::SetPort(port), services)?;
    let mut icon = if icon < 0 {
        result.memory.word(0x41f710)? as i16
    } else {
        icon
    };
    if icon == 0 {
        icon = 158;
    }
    let rect = result.memory.rect(0x44371a)?;
    plot(&mut result, icon, rect, true, services)?;
    draw(&mut result, DrawingEffect::ForeColor(30), services)?;
    draw(&mut result, DrawingEffect::TextFace(1), services)?;
    draw(
        &mut result,
        DrawingEffect::ForeColor(if (160..=163).contains(&icon) { 33 } else { 30 }),
        services,
    )?;
    let left = result.memory.word(0x44371c)? as i16;
    let width = (result.memory.word(0x443720)? as i16).wrapping_sub(left);
    let horizontal = half_word(width)
        .wrapping_add(left)
        .wrapping_sub(if amount > 9 { 8 } else { 5 });
    let top = result.memory.word(0x44371a)? as i16;
    let height = (result.memory.word(0x44371e)? as i16).wrapping_sub(top);
    let vertical = top.wrapping_add(4).wrapping_add(half_word(height));
    draw(
        &mut result,
        DrawingEffect::MoveTo {
            horizontal,
            vertical,
        },
        services,
    )?;
    draw(
        &mut result,
        DrawingEffect::TextSize(if amount > 9 { 10 } else { 12 }),
        services,
    )?;
    if amount != 0 {
        string(&mut result, amount, services)?;
    }
    draw(&mut result, DrawingEffect::TextFace(0), services)?;
    Ok(result)
}

fn target_size(result: &mut CombatUiPlan, size: i16) -> Result<(), RuleError> {
    word(result, 0x4435ce, size.wrapping_abs())?;
    if matches!(result.memory.word(0x4435ce)? as i16, 1 | 3) {
        add_word(result, 0x4471f2, -16)?;
    }
    if (result.memory.word(0x4435ce)? as i16) > 1 {
        add_word(result, 0x4471f4, -16)?;
    }
    Ok(())
}

/// showtargets, 0015a1f6. Unrecognized kinds retain the original stale-Rect path.
pub fn showtargets(
    memory: &CombatUiMemory,
    services: &(impl CombatUiServices + ?Sized),
) -> Result<CombatUiPlan, RuleError> {
    let mut result = plan(memory, Some(0));
    if result.memory.word(0x4435c0)? == 0 {
        return Ok(result);
    }
    let port = result.memory.long(0x43879c)?;
    draw(&mut result, DrawingEffect::SetPort(port), services)?;
    word(&mut result, 0x4434aa, 0)?;
    let mut iterations = 0_u32;
    while (result.memory.word(0x4434aa)? as i16) < result.memory.word(0x4435c0)? as i16 {
        if iterations >= 65536 {
            return Err(missing("terminating target loop"));
        }
        iterations += 1;
        let index = result.memory.word(0x4434aa)?;
        let descriptor = 0x443722_u32.wrapping_add(u32::from(index) * 6);
        let kind = result.memory.word(descriptor.wrapping_add(4))? as i16;
        let body = result.memory.word(descriptor)? as i16;
        let mut render = true;
        if kind == 1 || kind == 2 {
            let warrior = kind == 1;
            let hp = result
                .memory
                .word(record(body, warrior).wrapping_add(if warrior { 0x50 } else { 0x110 }))?
                as i16;
            if hp < 1 {
                render = false;
            } else {
                let position = position_pointer(body, warrior);
                let vertical = result.memory.word(position.wrapping_add(2))? as i16;
                word(&mut result, 0x4471f2, vertical.wrapping_shl(5))?;
                let horizontal = result.memory.word(position)? as i16;
                word(&mut result, 0x4471f4, horizontal.wrapping_shl(5))?;
                if !warrior {
                    let size = result.memory.word(record(body, false).wrapping_add(16))? as i16;
                    target_size(&mut result, size)?;
                }
            }
        } else if body != -1 {
            let vertical = result.memory.word(descriptor.wrapping_add(2))? as i16;
            let top = (i32::from(vertical) as u32)
                .wrapping_sub(result.memory.long(0x442784)?)
                .wrapping_shl(5) as i16;
            word(&mut result, 0x4471f2, top)?;
            let horizontal = result.memory.word(descriptor)? as i16;
            let left = (i32::from(horizontal) as u32)
                .wrapping_sub(result.memory.long(0x442780)?)
                .wrapping_shl(5) as i16;
            word(&mut result, 0x4471f4, left)?;
            let size = result.memory.word(0x440dcc)? as i16;
            target_size(&mut result, size)?;
        }
        if render {
            let bottom = (result.memory.word(0x4471f2)? as i16).wrapping_add(32);
            word(&mut result, 0x4471f6, bottom)?;
            let right = (result.memory.word(0x4471f4)? as i16).wrapping_add(32);
            word(&mut result, 0x4471f8, right)?;
            let rect = result.memory.rect(0x4471f2)?;
            let id = result.memory.word(0x4434aa)?.wrapping_add(15000) as i16;
            plot(&mut result, id, rect, true, services)?;
        }
        add_word(&mut result, 0x4434aa, 1)?;
    }
    Ok(result)
}

/// tickcheck, 0015ec56: animate condition and activity icons at an unsigned deadline.
pub fn tickcheck(
    memory: &CombatUiMemory,
    frame_pointer: u32,
    services: &(impl CombatUiServices + ?Sized),
) -> Result<CombatUiPlan, RuleError> {
    let mut result = plan(memory, None);
    if result.memory.long(0x43879c)? == 0 {
        return Ok(result);
    }
    if result.memory.word(0x402f44)? != 0x7788 && result.memory.word(0x402f44)? != 0x7788 {
        word(&mut result, 0x402f44, 0x7788)?;
    }
    let deadline = result.memory.long(0x442bd0)?.wrapping_add(10);
    write(
        &mut result,
        frame_pointer.wrapping_sub(4),
        &deadline.to_be_bytes(),
    )?;
    let now = service(&mut result, CombatUiRequest::TickCount, services)?;
    if now <= result.memory.long(frame_pointer.wrapping_sub(4))? {
        return Ok(result);
    }
    let port = result.memory.long(0x43878c)?;
    draw(&mut result, DrawingEffect::SetPort(port), services)?;
    let now = service(&mut result, CombatUiRequest::TickCount, services)?;
    write(&mut result, 0x442bd0, &now.to_be_bytes())?;
    let meter = result.memory.word(0x442562)? as i16;
    if meter != 0 {
        let bias = (((meter >> 1) as u16 >> 8) >> 6) as i16;
        let quarter = meter.wrapping_add(bias) >> 2;
        let top = (result.memory.word(0x40856e)? as i16)
            .wrapping_add(367)
            .wrapping_sub(quarter);
        word(&mut result, 0x4471f2, top)?;
        if (result.memory.word(0x4471f2)? as i16) < 331 {
            word(&mut result, 0x4471f2, 331)?;
        }
        let left = (result.memory.word(0x40856c)? as i16).wrapping_add(526);
        word(&mut result, 0x4471f4, left)?;
        let right = (result.memory.word(0x4471f4)? as i16).wrapping_add(16);
        word(&mut result, 0x4471f8, right)?;
        let bottom = (result.memory.word(0x4471f2)? as i16).wrapping_add(30);
        word(&mut result, 0x4471f6, bottom)?;
        let rect = result.memory.rect(0x4471f2)?;
        let id = result.memory.word(0x442bcc)?.wrapping_add(146) as i16;
        plot(&mut result, id, rect, true, services)?;
    }
    let top = ui_vertical(&result.memory, 0x14a)?;
    word(&mut result, 0x4471f2, top)?;
    let left = (result.memory.word(0x40856c)? as i16).wrapping_add(327);
    word(&mut result, 0x4471f4, left)?;
    word(&mut result, 0x4434aa, 1)?;
    let mut iterations = 0_u32;
    while (result.memory.word(0x4434aa)? as i16) < 9 {
        if iterations >= 65536 {
            return Err(missing("terminating animation loop"));
        }
        iterations += 1;
        service(
            &mut result,
            CombatUiRequest::OffsetRect {
                pointer: 0x4471f2,
                horizontal: 39,
                vertical: 0,
            },
            services,
        )?;
        if result.memory.word(0x4434aa)? == 5 {
            service(
                &mut result,
                CombatUiRequest::OffsetRect {
                    pointer: 0x4471f2,
                    horizontal: -156,
                    vertical: 40,
                },
                services,
            )?;
        }
        let index = result.memory.word(0x4434aa)? as i16;
        if result.memory.word(offset(0x442562, i32::from(index) * 2))? != 0 {
            let id = (index as u16)
                .wrapping_shl(3)
                .wrapping_add(0x36a8)
                .wrapping_add(result.memory.word(0x442bcc)?) as i16;
            let rect = result.memory.rect(0x4471f2)?;
            plot(&mut result, id, rect, false, services)?;
        }
        add_word(&mut result, 0x4434aa, 1)?;
    }
    add_word(&mut result, 0x442bcc, 1)?;
    if (result.memory.word(0x442bcc)? as i16) > 7 {
        word(&mut result, 0x442bcc, 0)?;
    }
    Ok(result)
}
