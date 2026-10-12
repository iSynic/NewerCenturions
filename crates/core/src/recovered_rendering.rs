//! Combat sprite, queued field, and pointer rendering from original instructions.

use crate::RuleError;
use crate::recovered_drawing::DrawingEffect;
use crate::recovered_geometry::BodyRect;
use crate::recovered_records::RecordWrite;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderingMemory {
    pub bytes: BTreeMap<u32, u8>,
}

impl RenderingMemory {
    pub fn insert_region(&mut self, address: u32, bytes: &[u8]) {
        for (offset, byte) in bytes.iter().enumerate() {
            self.bytes
                .insert(address.wrapping_add(offset as u32), *byte);
        }
    }

    pub fn byte(&self, address: u32) -> Result<u8, RuleError> {
        self.bytes.get(&address).copied().ok_or_else(|| {
            RuleError::Invalid(format!("rendering memory unavailable at {address:08x}"))
        })
    }

    pub fn word(&self, address: u32) -> Result<u16, RuleError> {
        Ok(u16::from_be_bytes([
            self.byte(address)?,
            self.byte(address.wrapping_add(1))?,
        ]))
    }

    pub fn long(&self, address: u32) -> Result<u32, RuleError> {
        Ok(u32::from_be_bytes([
            self.byte(address)?,
            self.byte(address.wrapping_add(1))?,
            self.byte(address.wrapping_add(2))?,
            self.byte(address.wrapping_add(3))?,
        ]))
    }

    pub fn rect(&self, address: u32) -> Result<BodyRect, RuleError> {
        Ok(BodyRect {
            top: self.word(address)? as i16,
            left: self.word(address.wrapping_add(2))? as i16,
            bottom: self.word(address.wrapping_add(4))? as i16,
            right: self.word(address.wrapping_add(6))? as i16,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderingRequest {
    Drawing(DrawingEffect),
    InsetRect {
        pointer: u32,
        horizontal: i16,
        vertical: i16,
    },
    PointInRect {
        point: u32,
        rect_pointer: u32,
    },
    GetColorCursor(i16),
    SetColorCursor(u32),
    DisposeColorCursor(u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderingObservation {
    /// Returned resource handle or low BYTE Boolean for PointInRect.
    pub value: u32,
    pub memory: RenderingMemory,
}

pub trait RenderingServices {
    /// Observe the external call without applying the returned rendering plan.
    fn observe(
        &self,
        request: &RenderingRequest,
        memory: &RenderingMemory,
    ) -> Result<RenderingObservation, RuleError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderingEffect {
    Write(RecordWrite),
    Service {
        request: RenderingRequest,
        value: u32,
        memory_after: RenderingMemory,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderingPlan {
    pub memory: RenderingMemory,
    pub effects: Vec<RenderingEffect>,
}

fn plan(memory: &RenderingMemory) -> RenderingPlan {
    RenderingPlan {
        memory: memory.clone(),
        effects: Vec::new(),
    }
}

fn write(result: &mut RenderingPlan, address: u32, bytes: &[u8]) -> Result<(), RuleError> {
    for offset in 0..bytes.len() {
        result.memory.byte(address.wrapping_add(offset as u32))?;
    }
    result.memory.insert_region(address, bytes);
    result.effects.push(RenderingEffect::Write(RecordWrite {
        address,
        bytes: bytes.to_vec(),
    }));
    Ok(())
}

fn word(result: &mut RenderingPlan, address: u32, value: u16) -> Result<(), RuleError> {
    write(result, address, &value.to_be_bytes())
}

fn add_word(result: &mut RenderingPlan, address: u32, increment: u16) -> Result<(), RuleError> {
    let value = result.memory.word(address)?.wrapping_add(increment);
    word(result, address, value)
}

fn service(
    result: &mut RenderingPlan,
    request: RenderingRequest,
    services: &(impl RenderingServices + ?Sized),
) -> Result<u32, RuleError> {
    let observation = services.observe(&request, &result.memory)?;
    result.effects.push(RenderingEffect::Service {
        request,
        value: observation.value,
        memory_after: observation.memory.clone(),
    });
    result.memory = observation.memory;
    Ok(observation.value)
}

fn drawing(
    result: &mut RenderingPlan,
    effect: DrawingEffect,
    services: &(impl RenderingServices + ?Sized),
) -> Result<u32, RuleError> {
    service(result, RenderingRequest::Drawing(effect), services)
}

fn signed_offset(address: u32, offset: i32) -> u32 {
    address.wrapping_add(offset as u32)
}

fn copy_bits(
    result: &mut RenderingPlan,
    source_port: u32,
    destination_port: u32,
    source_pointer: u32,
    destination_pointer: u32,
    mode: i16,
    services: &(impl RenderingServices + ?Sized),
) -> Result<(), RuleError> {
    let effect = DrawingEffect::CopyBits {
        source_bitmap: result.memory.long(source_port)?.wrapping_add(2),
        destination_bitmap: result.memory.long(destination_port)?.wrapping_add(2),
        source: result.memory.rect(source_pointer)?,
        destination: result.memory.rect(destination_pointer)?,
        mode,
        mask: 0,
    };
    drawing(result, effect, services)?;
    Ok(())
}

/// drawbody, 00120c14. Caller WORD arguments and frame-local writes are retained.
pub fn drawbody(
    memory: &RenderingMemory,
    frame_pointer: u32,
    body: i16,
    force_background: i16,
    secondary_destination: i16,
    services: &(impl RenderingServices + ?Sized),
) -> Result<RenderingPlan, RuleError> {
    if memory.word(frame_pointer.wrapping_add(10))? as i16 != force_background {
        return Err(RuleError::Invalid(
            "drawbody force WORD disagrees with caller frame".into(),
        ));
    }
    let mut result = plan(memory);
    let source_rect = frame_pointer.wrapping_sub(8);
    write(&mut result, frame_pointer.wrapping_sub(17), &[0])?;
    word(&mut result, source_rect, 0)?;
    word(&mut result, source_rect.wrapping_add(2), 768)?;
    word(&mut result, source_rect.wrapping_add(6), 800)?;
    word(&mut result, source_rect.wrapping_add(4), 32)?;
    if result.memory.word(0x438776)? == 0 {
        return Ok(result);
    }
    write(&mut result, 0x439598, &0_u32.to_be_bytes())?;
    let enemy = body > 9;
    let index = body.wrapping_sub(10);
    let record = if enemy {
        0x439d3c_u32.wrapping_add(u32::from(index as u16).wrapping_mul(288))
    } else {
        0x448aa0_u32.wrapping_add(u32::from(body as u16).wrapping_mul(3138))
    };
    let ward;
    let size;
    let mut picture = 0;
    let position;
    if enemy {
        if (result.memory.word(record.wrapping_add(0x110))? as i16) < 1 {
            return Ok(result);
        }
        ward = result.memory.byte(record.wrapping_add(0xb3))? != 0;
        let raw_size = result.memory.word(record.wrapping_add(0x10))? as i16;
        if raw_size == -3 {
            picture = result.memory.word(record.wrapping_add(0x50))?;
        }
        size = raw_size.wrapping_abs() as u8 as i8;
        let icon_id = result.memory.word(record.wrapping_add(0x50))?;
        word(&mut result, 0x4434b6, icon_id)?;
        if result.memory.byte(record.wrapping_add(0x107))? == 1 {
            add_word(&mut result, 0x4434b6, 1000)?;
        }
        let icon_id = result.memory.word(0x4434b6)? as i16;
        let handle = drawing(&mut result, DrawingEffect::GetColorIcon(icon_id), services)?;
        write(&mut result, 0x439598, &handle.to_be_bytes())?;
        position = signed_offset(0x446f0a, i32::from(index) * 4);
        let top = result
            .memory
            .word(position.wrapping_add(2))?
            .wrapping_shl(5);
        word(&mut result, 0x44371a, top)?;
        let left = result.memory.word(position)?.wrapping_shl(5);
        word(&mut result, 0x44371c, left)?;
        if result.memory.byte(record.wrapping_add(0xb0))? == 0xff
            || result.memory.byte(record.wrapping_add(0xb3))? != 0
        {
            word(&mut result, frame_pointer.wrapping_add(10), 1)?;
            add_word(&mut result, source_rect.wrapping_add(2), 128)?;
            add_word(&mut result, source_rect.wrapping_add(6), 128)?;
        } else if result.memory.word(record.wrapping_add(0x0e))? != 0 {
            add_word(&mut result, source_rect.wrapping_add(2), 64)?;
            add_word(&mut result, source_rect.wrapping_add(6), 64)?;
        }
    } else {
        if (result.memory.word(record.wrapping_add(0x50))? as i16) < 1 {
            return Ok(result);
        }
        ward = result.memory.word(record.wrapping_add(0x60))? != 0;
        size = 0;
        let icon_id = u16::from(result.memory.byte(record.wrapping_add(0x9e0))?)
            .wrapping_mul(10000)
            .wrapping_add(result.memory.word(record.wrapping_add(0x56))?);
        let handle = drawing(
            &mut result,
            DrawingEffect::GetColorIcon(icon_id as i16),
            services,
        )?;
        write(&mut result, 0x439598, &handle.to_be_bytes())?;
        position = signed_offset(0x44709a, i32::from(body) * 4);
        let top = result
            .memory
            .word(position.wrapping_add(2))?
            .wrapping_shl(5);
        word(&mut result, 0x44371a, top)?;
        let left = result.memory.word(position)?.wrapping_shl(5);
        word(&mut result, 0x44371c, left)?;
        if result.memory.word(record.wrapping_add(0x10))? != 0 {
            add_word(&mut result, source_rect.wrapping_add(2), 64)?;
            add_word(&mut result, source_rect.wrapping_add(6), 64)?;
        }
        if ward {
            word(&mut result, frame_pointer.wrapping_add(10), 1)?;
            add_word(&mut result, source_rect.wrapping_add(2), 64)?;
            add_word(&mut result, source_rect.wrapping_add(6), 64)?;
            if result.memory.word(record.wrapping_add(0x10))? == 0 {
                add_word(&mut result, source_rect.wrapping_add(2), 64)?;
                add_word(&mut result, source_rect.wrapping_add(6), 64)?;
            }
        }
    }
    let bottom = result.memory.word(0x44371a)?.wrapping_add(32);
    word(&mut result, 0x44371e, bottom)?;
    let right = result.memory.word(0x44371c)?.wrapping_add(32);
    word(&mut result, 0x443720, right)?;
    if matches!(size, 1 | 3) {
        add_word(&mut result, 0x44371a, (-32_i16) as u16)?;
        add_word(&mut result, source_rect.wrapping_add(4), 32)?;
    }
    if size > 1 {
        add_word(&mut result, 0x44371c, (-32_i16) as u16)?;
        add_word(&mut result, source_rect.wrapping_add(6), 32)?;
    }
    let port = result.memory.long(0x43879c)?;
    drawing(&mut result, DrawingEffect::SetPort(port), services)?;
    let order_cursor = result.memory.word(0x447194)? as i16;
    let current_body = result
        .memory
        .word(signed_offset(0x4470b8, i32::from(order_cursor) * 2))? as i16;
    if body == current_body
        || (result.memory.word(frame_pointer.wrapping_add(10))? as i16) > 0
        || ward
    {
        copy_bits(
            &mut result,
            0x4386c8,
            if secondary_destination == 0 {
                0x43879c
            } else {
                0x4386c0
            },
            source_rect,
            0x44371a,
            0,
            services,
        )?;
    }
    if picture != 0 {
        let destination = frame_pointer.wrapping_sub(16);
        let first = result.memory.long(0x44371a)?;
        write(&mut result, destination, &first.to_be_bytes())?;
        let second = result.memory.long(0x44371e)?;
        write(
            &mut result,
            destination.wrapping_add(4),
            &second.to_be_bytes(),
        )?;
        let handle = drawing(
            &mut result,
            DrawingEffect::GetPicture(picture as i16),
            services,
        )?;
        let pointer = result.memory.long(handle)?;
        let first = result.memory.long(pointer.wrapping_add(2))?;
        write(&mut result, 0x4434ba, &first.to_be_bytes())?;
        let second = result.memory.long(pointer.wrapping_add(6))?;
        write(&mut result, 0x4434be, &second.to_be_bytes())?;
        let top = result
            .memory
            .word(destination.wrapping_add(4))?
            .wrapping_sub(result.memory.word(0x4434be)?);
        word(&mut result, destination, top)?;
        let left = result
            .memory
            .word(destination.wrapping_add(6))?
            .wrapping_sub(result.memory.word(0x4434c0)?);
        word(&mut result, destination.wrapping_add(2), left)?;
        drawing(&mut result, DrawingEffect::GetWorld, services)?;
        let world = result.memory.long(0x4386bc)?;
        drawing(
            &mut result,
            DrawingEffect::SetWorld { world, device: 0 },
            services,
        )?;
        let bounds = result.memory.rect(0x4434ba)?;
        drawing(
            &mut result,
            DrawingEffect::DrawPicture { handle, bounds },
            services,
        )?;
        restore_world(&mut result, services)?;
        copy_bits(
            &mut result,
            0x4386bc,
            if secondary_destination == 0 {
                0x43879c
            } else {
                0x4386c0
            },
            0x4434ba,
            destination,
            36,
            services,
        )?;
    } else {
        if secondary_destination == 0 && result.memory.long(0x439598)? != 0 {
            plot_loaded_icon(&mut result, services)?;
        } else if result.memory.long(0x439598)? != 0 {
            drawing(&mut result, DrawingEffect::GetWorld, services)?;
            let world = result.memory.long(0x4386c0)?;
            drawing(
                &mut result,
                DrawingEffect::SetWorld { world, device: 0 },
                services,
            )?;
            plot_loaded_icon(&mut result, services)?;
            restore_world(&mut result, services)?;
        }
        let handle = result.memory.long(0x439598)?;
        if handle != 0 {
            drawing(
                &mut result,
                DrawingEffect::DisposeColorIcon(handle),
                services,
            )?;
        }
    }
    Ok(result)
}

fn plot_loaded_icon(
    result: &mut RenderingPlan,
    services: &(impl RenderingServices + ?Sized),
) -> Result<(), RuleError> {
    let handle = result.memory.long(0x439598)?;
    let rect = result.memory.rect(0x44371a)?;
    drawing(
        result,
        DrawingEffect::PlotColorIcon { handle, rect },
        services,
    )?;
    Ok(())
}

fn restore_world(
    result: &mut RenderingPlan,
    services: &(impl RenderingServices + ?Sized),
) -> Result<(), RuleError> {
    let world = result.memory.long(0x4046da)?;
    let device = result.memory.long(0x4046de)?;
    drawing(result, DrawingEffect::SetWorld { world, device }, services)?;
    Ok(())
}

/// showque, 00157ef8: paint eligible cells for all sixty active field records.
pub fn showque(
    memory: &RenderingMemory,
    frame_pointer: u32,
    services: &(impl RenderingServices + ?Sized),
) -> Result<RenderingPlan, RuleError> {
    let mut result = plan(memory);
    let counter = frame_pointer.wrapping_sub(18);
    let destination = frame_pointer.wrapping_sub(8);
    let source = frame_pointer.wrapping_sub(16);
    word(&mut result, counter, 0)?;
    let mut iterations = 0_u32;
    while (result.memory.word(counter)? as i16) < 60 {
        if iterations >= 65536 {
            return Err(RuleError::Invalid(
                "showque field counter does not terminate".into(),
            ));
        }
        iterations += 1;
        let slot = result.memory.word(counter)?;
        let field = 0x442bd6_u32.wrapping_add(u32::from(slot) * 20);
        if result.memory.byte(field.wrapping_add(15))? != 0 {
            let shape =
                (i16::from(result.memory.byte(field.wrapping_add(14))? as i8)).wrapping_sub(1);
            let x = i16::from(result.memory.byte(field.wrapping_add(12))? as i8);
            let y = i16::from(result.memory.byte(field.wrapping_add(13))? as i8);
            let top = (i32::from(y) as u32)
                .wrapping_sub(result.memory.long(0x442784)?)
                .wrapping_shl(5)
                .wrapping_sub(32) as u16;
            word(&mut result, destination, top)?;
            word(
                &mut result,
                destination.wrapping_add(4),
                top.wrapping_add(32),
            )?;
            word(&mut result, source, 64)?;
            word(&mut result, source.wrapping_add(2), (-32_i16) as u16)?;
            word(&mut result, source.wrapping_add(6), 0)?;
            word(&mut result, source.wrapping_add(4), 96)?;
            let source_shift = result.memory.word(field.wrapping_add(10))?.wrapping_shl(5);
            add_word(&mut result, source.wrapping_add(2), source_shift)?;
            let source_shift = result.memory.word(field.wrapping_add(10))?.wrapping_shl(5);
            add_word(&mut result, source.wrapping_add(6), source_shift)?;
            for outer in 0..7_i16 {
                add_word(&mut result, destination, 32)?;
                add_word(&mut result, destination.wrapping_add(4), 32)?;
                let left = (i32::from(x) as u32)
                    .wrapping_sub(result.memory.long(0x442780)?)
                    .wrapping_shl(5)
                    .wrapping_sub(32) as u16;
                word(&mut result, destination.wrapping_add(2), left)?;
                word(
                    &mut result,
                    destination.wrapping_add(6),
                    left.wrapping_add(32),
                )?;
                for inner in 0..7_i16 {
                    add_word(&mut result, destination.wrapping_add(2), 32)?;
                    add_word(&mut result, destination.wrapping_add(6), 32)?;
                    let mask = 0x438210_u32
                        .wrapping_add(u32::from(shape as u16) * 49)
                        .wrapping_add((outer as u32) * 7)
                        .wrapping_add(inner as u32);
                    if result.memory.byte(mask)? != 0 {
                        let map = 0x4085b8_u32
                            .wrapping_add(u32::from(x.wrapping_add(inner) as u16) * 180);
                        let tile_pointer = signed_offset(map, i32::from(y.wrapping_add(outer)) * 2);
                        let tile = result.memory.word(tile_pointer)? as i16;
                        if tile > 999 {
                            let tile = result.memory.word(tile_pointer)?.wrapping_sub(1000);
                            let building = 0x43167c_u32.wrapping_add(u32::from(tile) * 40);
                            if result.memory.word(building.wrapping_add(4))? == 0 {
                                copy_bits(
                                    &mut result,
                                    0x4386c8,
                                    0x4386c0,
                                    source,
                                    destination,
                                    36,
                                    services,
                                )?;
                            }
                        }
                    }
                }
            }
        }
        add_word(&mut result, counter, 1)?;
    }
    Ok(result)
}

/// updatearrow, 0015f210. A2 is required for the native branch that never loads it.
pub fn updatearrow(
    memory: &RenderingMemory,
    frame_pointer: u32,
    map_position_mode: i16,
    incoming_a2: u32,
    services: &(impl RenderingServices + ?Sized),
) -> Result<RenderingPlan, RuleError> {
    let mut result = plan(memory);
    let local_rect = frame_pointer.wrapping_sub(8);
    let first = result.memory.long(0x4471b4)?;
    write(&mut result, local_rect, &first.to_be_bytes())?;
    let second = result.memory.long(0x4471b8)?;
    write(
        &mut result,
        local_rect.wrapping_add(4),
        &second.to_be_bytes(),
    )?;
    service(
        &mut result,
        RenderingRequest::InsetRect {
            pointer: local_rect,
            horizontal: 64,
            vertical: 64,
        },
        services,
    )?;
    word(&mut result, 0x4471a2, 0)?;
    for (vertical, threshold_offset, increment) in [
        (true, -32_i32, 3_u16),
        (true, 64, 3),
        (false, -32, 1),
        (false, 64, 1),
    ] {
        let point_word = result
            .memory
            .word(if vertical { 0x4471ee } else { 0x4471f0 })? as i16;
        let beyond = if map_position_mode != 0 {
            let origin = result
                .memory
                .long(if vertical { 0x442768 } else { 0x442764 })?;
            let threshold = origin.wrapping_shl(5).wrapping_add(threshold_offset as u32) as i32;
            i32::from(point_word) > threshold
        } else {
            let coordinate = if result.memory.byte(0x43565c)? == 0 {
                result
                    .memory
                    .word(0x439582)?
                    .wrapping_mul(2)
                    .wrapping_add(5)
            } else {
                let body = result.memory.word(0x4435d2)? as i16;
                let position = signed_offset(0x44709a, i32::from(body) * 4);
                result
                    .memory
                    .word(position.wrapping_add(if vertical { 2 } else { 0 }))?
            };
            let threshold = coordinate
                .wrapping_shl(5)
                .wrapping_add(threshold_offset as u16) as i16;
            point_word > threshold
        };
        if beyond {
            add_word(&mut result, 0x4471a2, increment)?;
        }
    }
    let point = result.memory.long(0x4471ee)?;
    let inside = service(
        &mut result,
        RenderingRequest::PointInRect {
            point,
            rect_pointer: local_rect,
        },
        services,
    )? as u8
        != 0;
    write(&mut result, 0x43565c, &[u8::from(inside)])?;
    if result.memory.word(0x4471a2)? == result.memory.word(0x4471a4)?
        && result.memory.byte(0x43565c)? == result.memory.byte(0x43565d)?
    {
        return Ok(result);
    }
    let battle = result.memory.word(0x438776)? != 0;
    let point = result.memory.long(0x4471ee)?;
    let inside = service(
        &mut result,
        RenderingRequest::PointInRect {
            point,
            rect_pointer: if battle { local_rect } else { 0x4471b4 },
        },
        services,
    )? as u8
        != 0;
    let mut cursor = incoming_a2;
    if battle || inside {
        let id = result
            .memory
            .word(0x4471a2)?
            .wrapping_add(if inside { 138 } else { 164 });
        cursor = service(
            &mut result,
            RenderingRequest::GetColorCursor(id as i16),
            services,
        )?;
        if battle {
            write(&mut result, 0x43565c, &[u8::from(inside)])?;
        }
    }
    service(
        &mut result,
        RenderingRequest::SetColorCursor(cursor),
        services,
    )?;
    service(
        &mut result,
        RenderingRequest::DisposeColorCursor(cursor),
        services,
    )?;
    let arrow = result.memory.word(0x4471a2)?;
    word(&mut result, 0x4471a4, arrow)?;
    let flag = result.memory.byte(0x43565c)?;
    write(&mut result, 0x43565d, &[flag])?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct Observations {
        requests: RefCell<Vec<RenderingRequest>>,
        point_results: RefCell<Vec<u32>>,
        picture_handle: u32,
    }

    impl RenderingServices for Observations {
        fn observe(
            &self,
            request: &RenderingRequest,
            memory: &RenderingMemory,
        ) -> Result<RenderingObservation, RuleError> {
            self.requests.borrow_mut().push(request.clone());
            let mut memory = memory.clone();
            let value = match request {
                RenderingRequest::Drawing(DrawingEffect::GetColorIcon(_)) => 0x7000,
                RenderingRequest::Drawing(DrawingEffect::GetPicture(_)) => self.picture_handle,
                RenderingRequest::Drawing(DrawingEffect::GetWorld) => {
                    memory.insert_region(0x4046da, &0xaaaa_u32.to_be_bytes());
                    memory.insert_region(0x4046de, &0xbbbb_u32.to_be_bytes());
                    0
                }
                RenderingRequest::InsetRect {
                    pointer,
                    horizontal,
                    vertical,
                } => {
                    let rect = memory.rect(*pointer)?;
                    let values = [
                        rect.top.wrapping_add(*vertical),
                        rect.left.wrapping_add(*horizontal),
                        rect.bottom.wrapping_sub(*vertical),
                        rect.right.wrapping_sub(*horizontal),
                    ];
                    for (index, value) in values.iter().enumerate() {
                        memory.insert_region(
                            pointer.wrapping_add(index as u32 * 2),
                            &value.to_be_bytes(),
                        );
                    }
                    0
                }
                RenderingRequest::PointInRect { .. } => self.point_results.borrow_mut().remove(0),
                RenderingRequest::GetColorCursor(id) => u32::from(*id as u16) + 0x8000,
                RenderingRequest::Drawing(_)
                | RenderingRequest::SetColorCursor(_)
                | RenderingRequest::DisposeColorCursor(_) => 0,
            };
            Ok(RenderingObservation { value, memory })
        }
    }

    fn input() -> RenderingMemory {
        let mut memory = RenderingMemory::default();
        memory.insert_region(8900, &[0; 200]);
        for (address, length) in [
            (0x438776, 2),
            (0x439598, 4),
            (0x4434b6, 12),
            (0x44371a, 8),
            (0x447194, 2),
            (0x4470b8, 220),
            (0x446f0a, 400),
            (0x44709a, 40),
            (0x439d3c, 288),
            (0x448aa0, 3138),
            (0x4046da, 8),
            (0x442bd6, 1200),
            (0x438210, 49),
            (0x4085b8, 16200),
            (0x43167c, 80),
            (0x442780, 8),
            (0x442764, 8),
            (0x4471b4, 8),
            (0x4471ee, 4),
            (0x4471a2, 4),
            (0x43565c, 2),
            (0x439582, 2),
            (0x4435d2, 2),
        ] {
            memory.insert_region(address, &vec![0; length]);
        }
        for (address, pointer) in [
            (0x43879c, 0x1000_u32),
            (0x4386c0, 0x2000),
            (0x4386bc, 0x3000),
            (0x4386c8, 0x4000),
        ] {
            memory.insert_region(address, &pointer.to_be_bytes());
        }
        memory.insert_region(0x7100, &0x7200_u32.to_be_bytes());
        memory.insert_region(0x7200, &[0; 12]);
        memory.insert_region(0x7206, &96_u16.to_be_bytes());
        memory.insert_region(0x7208, &128_u16.to_be_bytes());
        memory
    }

    fn services(point_results: Vec<u32>) -> Observations {
        Observations {
            requests: RefCell::new(Vec::new()),
            point_results: RefCell::new(point_results),
            picture_handle: 0x7100,
        }
    }

    #[test]
    fn picture_body_preserves_overlay_rect_and_icon_lifetime() {
        let mut memory = input();
        memory.insert_region(0x438776, &1_u16.to_be_bytes());
        memory.insert_region(0x439e4c, &5_u16.to_be_bytes());
        memory.insert_region(0x439d4c, &(-3_i16).to_be_bytes());
        memory.insert_region(0x439d8c, &77_u16.to_be_bytes());
        memory.insert_region(0x439def, &[1]);
        memory.insert_region(0x439e43, &[1]);
        memory.insert_region(0x446f0a, &5_u16.to_be_bytes());
        memory.insert_region(0x446f0c, &6_u16.to_be_bytes());
        let observed = services(Vec::new());
        let result = drawbody(&memory, 9000, 10, 0, 1, &observed).unwrap();
        assert_eq!(
            result.memory.rect(0x44371a).unwrap(),
            BodyRect {
                top: 160,
                left: 128,
                bottom: 224,
                right: 192
            }
        );
        assert_eq!(result.memory.word(9010).unwrap(), 1);
        let requests = observed.requests.borrow();
        assert_eq!(
            requests[0],
            RenderingRequest::Drawing(DrawingEffect::GetColorIcon(1077))
        );
        let copies: Vec<_> = requests
            .iter()
            .filter_map(|request| match request {
                RenderingRequest::Drawing(DrawingEffect::CopyBits {
                    source,
                    destination,
                    mode,
                    ..
                }) => Some((*source, *destination, *mode)),
                _ => None,
            })
            .collect();
        assert_eq!(
            copies,
            [
                (
                    BodyRect {
                        top: 0,
                        left: 896,
                        bottom: 64,
                        right: 960
                    },
                    BodyRect {
                        top: 160,
                        left: 128,
                        bottom: 224,
                        right: 192
                    },
                    0
                ),
                (
                    BodyRect {
                        top: 0,
                        left: 0,
                        bottom: 96,
                        right: 128
                    },
                    BodyRect {
                        top: 128,
                        left: 64,
                        bottom: 224,
                        right: 192
                    },
                    36
                ),
            ]
        );
        assert!(!requests.iter().any(|r| matches!(
            r,
            RenderingRequest::Drawing(DrawingEffect::DisposeColorIcon(_))
        )));
        assert!(
            requests.contains(&RenderingRequest::Drawing(DrawingEffect::SetWorld {
                world: 0xaaaa,
                device: 0xbbbb
            }))
        );
        drop(requests);
        // The first bounds copy overwrites an aliased master pointer. A0 is cached.
        let mut aliased = memory.clone();
        aliased.insert_region(0x4434ba, &0x7200_u32.to_be_bytes());
        let mut alias_observation = services(Vec::new());
        alias_observation.picture_handle = 0x4434ba;
        let alias_result = drawbody(&aliased, 9000, 10, 0, 1, &alias_observation).unwrap();
        assert_eq!(
            alias_result.memory.rect(0x4434ba).unwrap(),
            BodyRect {
                top: 0,
                left: 0,
                bottom: 96,
                right: 128,
            }
        );
        let before = memory.clone();
        memory.insert_region(0x439e4c, &[0, 0]);
        let no_draw = services(Vec::new());
        let result = drawbody(&memory, 9000, 10, 0, 0, &no_draw).unwrap();
        assert!(no_draw.requests.borrow().is_empty());
        assert_eq!(result.memory.long(0x439598).unwrap(), 0);
        assert_eq!(before.word(0x439e4c).unwrap(), 5);
    }

    #[test]
    fn field_queue_uses_mask_map_building_word_and_shifted_rects() {
        let mut memory = input();
        memory.insert_region(0x442be5, &[1]);
        memory.insert_region(0x442be4, &[1]);
        memory.insert_region(0x442be2, &[2, 3]);
        memory.insert_region(0x442be0, &5_u16.to_be_bytes());
        memory.insert_region(0x438210, &[1]);
        memory.insert_region(0x438219, &[1]);
        memory.insert_region(0x442780, &1_u32.to_be_bytes());
        memory.insert_region(0x442784, &2_u32.to_be_bytes());
        memory.insert_region(0x4085b8 + 2 * 180 + 3 * 2, &1001_u16.to_be_bytes());
        memory.insert_region(0x4085b8 + 4 * 180 + 4 * 2, &1000_u16.to_be_bytes());
        memory.insert_region(0x43167c + 40 + 4, &1_u16.to_be_bytes());
        let observed = services(Vec::new());
        let result = showque(&memory, 9000, &observed).unwrap();
        assert_eq!(result.memory.word(8982).unwrap(), 60);
        assert_eq!(
            observed.requests.borrow().as_slice(),
            [RenderingRequest::Drawing(DrawingEffect::CopyBits {
                source_bitmap: 0x4002,
                destination_bitmap: 0x2002,
                source: BodyRect {
                    top: 64,
                    left: 128,
                    bottom: 96,
                    right: 160
                },
                destination: BodyRect {
                    top: 64,
                    left: 96,
                    bottom: 96,
                    right: 128
                },
                mode: 36,
                mask: 0,
            })]
        );
        assert_eq!(memory.word(8982).unwrap(), 0);
    }

    #[test]
    fn arrow_keeps_incoming_cursor_on_legacy_miss_and_long_thresholds() {
        let mut memory = input();
        memory.insert_region(0x4471ee, &200_u16.to_be_bytes());
        memory.insert_region(0x4471f0, &100_u16.to_be_bytes());
        memory.insert_region(0x4471b8, &160_u16.to_be_bytes());
        memory.insert_region(0x4471ba, &80_u16.to_be_bytes());
        let observed = services(vec![0, 0]);
        let result = updatearrow(&memory, 9000, 0, 0x12345678, &observed).unwrap();
        assert_eq!(result.memory.word(0x4471a2).unwrap(), 3);
        assert_eq!(result.memory.word(0x4471a4).unwrap(), 3);
        assert_eq!(result.memory.byte(0x43565c).unwrap(), 0);
        assert_eq!(result.memory.byte(0x43565d).unwrap(), 0);
        assert_eq!(
            result.memory.rect(8992).unwrap(),
            BodyRect {
                top: 64,
                left: 64,
                bottom: 96,
                right: 16,
            }
        );
        let requests = observed.requests.borrow();
        assert!(
            !requests
                .iter()
                .any(|r| matches!(r, RenderingRequest::GetColorCursor(_)))
        );
        assert!(requests.contains(&RenderingRequest::SetColorCursor(0x12345678)));
        assert!(requests.contains(&RenderingRequest::DisposeColorCursor(0x12345678)));
        drop(requests);
        memory.insert_region(0x442764, &0x08000000_u32.to_be_bytes());
        memory.insert_region(0x442768, &0x08000000_u32.to_be_bytes());
        memory.insert_region(0x4471b8, &240_u16.to_be_bytes());
        memory.insert_region(0x4471ba, &160_u16.to_be_bytes());
        let observed = services(vec![0, 1]);
        let result = updatearrow(&memory, 9000, 1, 7, &observed).unwrap();
        assert_eq!(result.memory.word(0x4471a2).unwrap(), 8);
        assert!(
            observed
                .requests
                .borrow()
                .contains(&RenderingRequest::GetColorCursor(146))
        );
        assert_eq!(memory.word(0x4471a2).unwrap(), 0);
    }
}
