//! Shared warning, message and button bodies with observed Toolbox boundaries.

use crate::RuleError;
use crate::recovered_audio::{self as audio, AudioContext, AudioEffect, AudioObservation};
use crate::recovered_records::RecordWrite;
use crate::recovered_rendering::RenderingMemory;
use crate::recovered_timing::{self as timing, TimingServices};

pub type FeedbackMemory = RenderingMemory;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeedbackArg {
    Word(u16),
    Long(u32),
    Pointer(u32),
    Byte(u8),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeedbackRequest {
    /// Arguments are in native Pascal push order; pointers reference candidate memory.
    Toolbox {
        opcode: u16,
        arguments: Vec<FeedbackArg>,
    },
    BlockMove {
        source: u32,
        destination: u32,
        count: u32,
    },
    CToPascal(u32),
    GetIndexedString {
        destination: u32,
        resource: i16,
        index: i16,
    },
    TickCount,
    Button,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeedbackObservation {
    pub value: u32,
    pub d0: u32,
    pub memory: FeedbackMemory,
}

pub trait FeedbackServices {
    fn observe(
        &self,
        request: &FeedbackRequest,
        memory: &FeedbackMemory,
    ) -> Result<FeedbackObservation, RuleError>;
    /// Supplies Sound Manager/resource replies, not an application-child result.
    fn sound_observations(
        &self,
        id: i16,
        context: &AudioContext,
        memory: &FeedbackMemory,
    ) -> Result<Vec<AudioObservation>, RuleError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeedbackEffect {
    Write(RecordWrite),
    Service {
        request: FeedbackRequest,
        observation: FeedbackObservation,
    },
    Audio(AudioEffect),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeedbackPlan {
    pub memory: FeedbackMemory,
    pub effects: Vec<FeedbackEffect>,
    pub native_return: Option<u32>,
}

fn plan(memory: &FeedbackMemory, native_return: Option<u32>) -> FeedbackPlan {
    FeedbackPlan {
        memory: memory.clone(),
        effects: Vec::new(),
        native_return,
    }
}
fn write(result: &mut FeedbackPlan, address: u32, bytes: &[u8]) -> Result<(), RuleError> {
    for i in 0..bytes.len() {
        result.memory.byte(address.wrapping_add(i as u32))?;
    }
    result.memory.insert_region(address, bytes);
    result.effects.push(FeedbackEffect::Write(RecordWrite {
        address,
        bytes: bytes.to_vec(),
    }));
    Ok(())
}
fn word(result: &mut FeedbackPlan, address: u32, value: u16) -> Result<(), RuleError> {
    write(result, address, &value.to_be_bytes())
}
fn long(result: &mut FeedbackPlan, address: u32, value: u32) -> Result<(), RuleError> {
    write(result, address, &value.to_be_bytes())
}
fn service(
    result: &mut FeedbackPlan,
    request: FeedbackRequest,
    services: &(impl FeedbackServices + ?Sized),
) -> Result<u32, RuleError> {
    let observation = services.observe(&request, &result.memory)?;
    let value = observation.value;
    result.memory = observation.memory.clone();
    result.effects.push(FeedbackEffect::Service {
        request,
        observation,
    });
    Ok(value)
}
fn trap(
    result: &mut FeedbackPlan,
    opcode: u16,
    arguments: Vec<FeedbackArg>,
    services: &(impl FeedbackServices + ?Sized),
) -> Result<u32, RuleError> {
    service(
        result,
        FeedbackRequest::Toolbox { opcode, arguments },
        services,
    )
}
use FeedbackArg::{Byte as B, Long as L, Pointer as P, Word as W};
fn colors(
    result: &mut FeedbackPlan,
    services: &(impl FeedbackServices + ?Sized),
) -> Result<(), RuleError> {
    trap(result, 0xa862, vec![L(33)], services)?;
    trap(result, 0xa863, vec![L(30)], services)?;
    Ok(())
}
#[allow(clippy::too_many_arguments)]
fn copy(
    result: &mut FeedbackPlan,
    source: u32,
    destination: u32,
    source_rect: u32,
    destination_rect: u32,
    mode: u16,
    services: &(impl FeedbackServices + ?Sized),
) -> Result<(), RuleError> {
    trap(
        result,
        0xa8ec,
        vec![
            P(source),
            P(destination),
            P(source_rect),
            P(destination_rect),
            W(mode),
            L(0),
        ],
        services,
    )?;
    Ok(())
}
fn port(
    result: &mut FeedbackPlan,
    pointer: u32,
    services: &(impl FeedbackServices + ?Sized),
) -> Result<(), RuleError> {
    trap(result, 0xa873, vec![L(pointer)], services)?;
    Ok(())
}
fn cursor(
    result: &mut FeedbackPlan,
    address: u32,
    services: &(impl FeedbackServices + ?Sized),
) -> Result<(), RuleError> {
    let cursor = result.memory.long(address)?;
    trap(result, 0xaa1c, vec![L(cursor)], services)?;
    Ok(())
}
fn flush(
    result: &mut FeedbackPlan,
    services: &(impl FeedbackServices + ?Sized),
) -> Result<(), RuleError> {
    trap(result, 0xa032, vec![L(0xffff)], services)?;
    Ok(())
}

struct ClockBridge<'a, S: FeedbackServices + ?Sized> {
    result: &'a mut FeedbackPlan,
    services: &'a S,
    remaining: usize,
}
impl<S: FeedbackServices + ?Sized> TimingServices for ClockBridge<'_, S> {
    fn tick_count(&mut self) -> Result<u32, RuleError> {
        if self.remaining == 0 {
            return Err(RuleError::Invalid(
                "feedback timing observation limit exhausted".into(),
            ));
        }
        self.remaining -= 1;
        service(self.result, FeedbackRequest::TickCount, self.services)
    }
    fn button(&mut self) -> Result<bool, RuleError> {
        Ok(service(self.result, FeedbackRequest::Button, self.services)? as u8 != 0)
    }
}
fn delay(
    result: &mut FeedbackPlan,
    duration: i16,
    limit: usize,
    services: &(impl FeedbackServices + ?Sized),
) -> Result<(), RuleError> {
    timing::delay(
        duration,
        0,
        &mut ClockBridge {
            result,
            services,
            remaining: limit,
        },
    )?;
    Ok(())
}
fn event(
    result: &mut FeedbackPlan,
    limit: usize,
    services: &(impl FeedbackServices + ?Sized),
) -> Result<(), RuleError> {
    for _ in 0..limit {
        trap(
            result,
            0xa860,
            vec![W(0xffff), P(0x44720a), L(0), L(0)],
            services,
        )?;
        if matches!(result.memory.word(0x44720a)?, 1 | 3) {
            return Ok(());
        }
    }
    Err(RuleError::Invalid(
        "feedback event observation limit exhausted".into(),
    ))
}

/// Execute the recovered sound body, projecting its ordered global effects.
pub fn sound(
    memory: &FeedbackMemory,
    id: i16,
    services: &(impl FeedbackServices + ?Sized),
) -> Result<FeedbackPlan, RuleError> {
    let mut result = plan(memory, Some(0));
    let volume = result.memory.long(0x439570)? as i32;
    let last = if volume == 0 {
        -1
    } else {
        result.memory.word(0x439576)? as i16
    };
    if volume == 0 || last < 0 {
        // The actual child short-circuits without touching the channel tables.
        let mut context = AudioContext {
            sound_volume: volume,
            last_channel: last,
            ..AudioContext::default()
        };
        let observations = if id < 0 {
            vec![AudioObservation {
                request: audio::AudioRequest::Delay(20),
                reply: audio::AudioReply::Ignored,
            }]
        } else {
            Vec::new()
        };
        let child = audio::sound(&mut context, id, &observations)?;
        for effect in child.effects {
            if matches!(
                effect,
                AudioEffect::Service(AudioObservation {
                    request: audio::AudioRequest::Delay(20),
                    ..
                })
            ) {
                delay(&mut result, 20, 65536, services)?;
            } else {
                result.effects.push(FeedbackEffect::Audio(effect));
            }
        }
        return Ok(result);
    }
    let mut context = AudioContext {
        sound_volume: volume,
        last_channel: last,
        queue_index: result.memory.word(0x402f3a)? as i16,
        last_error: result.memory.word(0x4386ac)? as i16,
        ..AudioContext::default()
    };
    for index in 0..=last.max(2) {
        context.channels.push(
            result
                .memory
                .long(0x4436b6_u32.wrapping_add(index as u32 * 4))?,
        );
        context.resource_handles.push(
            result
                .memory
                .long(0x4394dc_u32.wrapping_add(index as u32 * 4))?,
        );
    }
    let observations = services.sound_observations(id, &context, &result.memory)?;
    let child = audio::sound(&mut context, id, &observations)?;
    for effect in child.effects {
        match &effect {
            AudioEffect::QueueIndex(value) => word(&mut result, 0x402f3a, *value as u16)?,
            AudioEffect::ResourceHandle { index, handle } => long(
                &mut result,
                0x4394dc_u32.wrapping_add(*index as u32 * 4),
                *handle,
            )?,
            AudioEffect::LastError(value) => word(&mut result, 0x4386ac, *value as u16)?,
            AudioEffect::Service(_) => (),
        }
        result.effects.push(FeedbackEffect::Audio(effect));
    }
    Ok(result)
}
fn play(
    result: &mut FeedbackPlan,
    id: i16,
    services: &(impl FeedbackServices + ?Sized),
) -> Result<(), RuleError> {
    let child = sound(&result.memory, id, services)?;
    result.memory = child.memory;
    result.effects.extend(child.effects);
    Ok(())
}

/// pressbutton, 0014ce2a. The caller's two Points are mutable argument bytes.
pub fn pressbutton(
    memory: &FeedbackMemory,
    frame: u32,
    kind: i16,
    phase: i16,
    with_sound: i16,
    services: &(impl FeedbackServices + ?Sized),
) -> Result<FeedbackPlan, RuleError> {
    let mut result = plan(memory, None);
    if with_sound != 0 {
        play(&mut result, 141, services)?;
    }
    for (offset, value) in [(6, 0), (8, 0), (4, 50), (2, 50)] {
        word(&mut result, frame.wrapping_sub(offset), value)?;
    }
    let settings: &[(u32, u16)] = match kind {
        3 => &[(6, 100), (2, 125)],
        2 => &[(6, 150), (2, 200), (4, 25)],
        4 => &[(6, 0), (2, 71), (8, 50), (4, 90)],
        5 => &[(6, 0), (2, 50), (8, 90), (4, 129)],
        6 => &[(6, 142), (2, 174), (8, 50), (4, 100)],
        _ => &[],
    };
    for &(offset, value) in settings {
        word(&mut result, frame.wrapping_sub(offset), value)?;
    }
    if phase == 2 {
        let (first, second, increment) = match kind {
            1 => (6, 2, 50),
            3 => (6, 2, 25),
            2 => (8, 4, 25),
            4 => (6, 2, 71),
            5 => (6, 2, 50),
            6 => (6, 2, 32),
            _ => (0, 0, 0),
        };
        if increment != 0 {
            for offset in [first, second] {
                let value = result
                    .memory
                    .word(frame.wrapping_sub(offset))?
                    .wrapping_add(increment);
                word(&mut result, frame.wrapping_sub(offset), value)?;
            }
        }
    }
    for (source, destination) in [(14, 12), (16, 10), (18, 16), (20, 14)] {
        let value = result.memory.word(frame.wrapping_add(source))?;
        word(&mut result, frame.wrapping_sub(destination), value)?;
    }
    trap(
        &mut result,
        0xa870,
        vec![P(frame.wrapping_sub(12))],
        services,
    )?;
    trap(
        &mut result,
        0xa870,
        vec![P(frame.wrapping_sub(16))],
        services,
    )?;
    for (source, destination) in [(12, 14), (10, 16), (16, 18), (14, 20)] {
        let value = result.memory.word(frame.wrapping_sub(source))?;
        word(&mut result, frame.wrapping_add(destination), value)?;
    }
    colors(&mut result, services)?;
    let bitmap = result.memory.long(0x4386cc)?.wrapping_add(2);
    copy(
        &mut result,
        bitmap,
        0x3fe288,
        frame.wrapping_sub(8),
        frame.wrapping_add(14),
        36,
        services,
    )?;
    Ok(result)
}

/// warn, 0016474a. Warning priority, dialog, wait and restoration are source ordered.
pub fn warn(
    memory: &FeedbackMemory,
    frame: u32,
    message: i16,
    maximum_polls: usize,
    services: &(impl FeedbackServices + ?Sized),
) -> Result<FeedbackPlan, RuleError> {
    let mut result = plan(memory, Some(0));
    long(&mut result, frame.wrapping_sub(36), 0)?;
    trap(
        &mut result,
        0xa874,
        vec![P(frame.wrapping_sub(32))],
        services,
    )?;
    trap(
        &mut result,
        0xaa19,
        vec![P(frame.wrapping_sub(22))],
        services,
    )?;
    trap(
        &mut result,
        0xaa1a,
        vec![P(frame.wrapping_sub(28))],
        services,
    )?;
    cursor(&mut result, 0x43952c, services)?;
    colors(&mut result, services)?;
    for (offset, value) in [(16, 0), (12, 62), (14, 0), (10, 261), (8, 40)] {
        word(&mut result, frame.wrapping_sub(offset), value)?;
    }
    let left = 30_u16.wrapping_sub(u16::from(result.memory.byte(0x442750)?) * 18);
    word(&mut result, frame.wrapping_sub(6), left)?;
    let right = result.memory.word(frame.wrapping_sub(6))?.wrapping_add(261);
    word(&mut result, frame.wrapping_sub(2), right)?;
    word(&mut result, frame.wrapping_sub(4), 102)?;
    let h = result.memory.word(0x4395a4)?;
    let v = result.memory.word(0x4395a2)?;
    trap(
        &mut result,
        0xa8a8,
        vec![P(frame.wrapping_sub(8)), W(h), W(v)],
        services,
    )?;
    if result.memory.long(0x43879c)? != 0 {
        let value = result.memory.long(0x43878c)?;
        port(&mut result, value, services)?;
    }
    let mut selected = false;
    for (predicate, address) in [
        (0x442751, 0x4394cc),
        (0x442752, 0x438790),
        (0x442750, 0x438798),
        (0x442756, 0x438788),
        (0x442755, 0x4394d8),
    ] {
        if result.memory.byte(predicate)? != 0 {
            let value = result.memory.long(address)?;
            port(&mut result, value, services)?;
            selected = true;
            break;
        }
    }
    if !selected {
        for address in [0x4394d4, 0x479fd8, 0x4387a0, 0x43878c] {
            let value = result.memory.long(address)?;
            if value != 0 {
                port(&mut result, value, services)?;
                selected = true;
                break;
            }
        }
    }
    if !selected && result.memory.byte(0x43565a)? != 0 {
        let value = result.memory.long(0x4394c4)?;
        port(&mut result, value, services)?;
    }
    colors(&mut result, services)?;
    let backup = result.memory.long(0x4386b4)?.wrapping_add(2);
    copy(
        &mut result,
        0x3fe288,
        backup,
        frame.wrapping_sub(8),
        frame.wrapping_sub(16),
        0,
        services,
    )?;
    if message > 0 {
        service(
            &mut result,
            FeedbackRequest::GetIndexedString {
                destination: 0x479ed8,
                resource: 3,
                index: message,
            },
            services,
        )?;
        trap(
            &mut result,
            0xa98b,
            vec![P(0x479ed8), P(0x4005a5), P(0x4005a5), P(0x4005a5)],
            services,
        )?;
    }
    let dialog = trap(
        &mut result,
        0xa97c,
        vec![W(140), L(0), L(u32::MAX)],
        services,
    )?;
    port(&mut result, dialog, services)?;
    let value = result.memory.long(0x4358ce)?;
    trap(&mut result, 0xaa0b, vec![L(value)], services)?;
    let font = result.memory.word(0x402f3c)?;
    trap(&mut result, 0xa887, vec![W(font)], services)?;
    cursor(&mut result, 0x439538, services)?;
    trap(&mut result, 0xa862, vec![L(69)], services)?;
    let horizontal = result
        .memory
        .word(0x4395a4)?
        .wrapping_add(31)
        .wrapping_sub(u16::from(result.memory.byte(0x442750)?) * 18);
    let vertical = result.memory.word(0x4395a2)?.wrapping_add(41);
    trap(
        &mut result,
        0xa91b,
        vec![L(dialog), W(horizontal), W(vertical), B(0)],
        services,
    )?;
    trap(&mut result, 0xa915, vec![L(dialog)], services)?;
    trap(&mut result, 0xa981, vec![L(dialog)], services)?;
    play(&mut result, 6000, services)?;
    delay(&mut result, 10, maximum_polls, services)?;
    flush(&mut result, services)?;
    event(&mut result, maximum_polls, services)?;
    trap(&mut result, 0xa983, vec![L(dialog)], services)?;
    cursor(&mut result, 0x43952c, services)?;
    word(&mut result, 0x4471a2, 0)?;
    word(&mut result, 0x4434b2, 0)?;
    colors(&mut result, services)?;
    let saved = result.memory.long(frame.wrapping_sub(32))?;
    port(&mut result, saved, services)?;
    let backup = result.memory.long(0x4386b4)?.wrapping_add(2);
    copy(
        &mut result,
        backup,
        0x3fe288,
        frame.wrapping_sub(16),
        frame.wrapping_sub(8),
        0,
        services,
    )?;
    trap(
        &mut result,
        0xaa15,
        vec![P(frame.wrapping_sub(28))],
        services,
    )?;
    trap(
        &mut result,
        0xaa14,
        vec![P(frame.wrapping_sub(22))],
        services,
    )?;
    flush(&mut result, services)?;
    Ok(result)
}

/// flashmessage, 00123d1c. A positive wait ends only on an exactly matching tick.
#[allow(clippy::too_many_arguments)]
pub fn flashmessage(
    memory: &FeedbackMemory,
    frame: u32,
    message_pointer: u32,
    horizontal: i16,
    vertical: i16,
    wait: i16,
    sound_id: i16,
    maximum_polls: usize,
    services: &(impl FeedbackServices + ?Sized),
) -> Result<FeedbackPlan, RuleError> {
    let mut result = plan(memory, None);
    if result.memory.long(frame.wrapping_add(8))? != message_pointer {
        return Err(RuleError::Invalid(
            "flashmessage caller pointer disagrees with the supplied argument".into(),
        ));
    }
    trap(
        &mut result,
        0xa874,
        vec![P(frame.wrapping_sub(288))],
        services,
    )?;
    trap(
        &mut result,
        0xaa19,
        vec![P(frame.wrapping_sub(278))],
        services,
    )?;
    trap(
        &mut result,
        0xaa1a,
        vec![P(frame.wrapping_sub(284))],
        services,
    )?;
    let message_pointer = result.memory.long(frame.wrapping_add(8))?;
    service(
        &mut result,
        FeedbackRequest::BlockMove {
            source: message_pointer,
            destination: frame.wrapping_sub(272),
            count: 255,
        },
        services,
    )?;
    service(
        &mut result,
        FeedbackRequest::CToPascal(frame.wrapping_sub(272)),
        services,
    )?;
    for (offset, value) in [
        (16, 0),
        (12, 62),
        (14, 0),
        (10, 260),
        (8, (vertical as u16).wrapping_sub(1)),
        (6, (horizontal as u16).wrapping_sub(1)),
        (2, (horizontal as u16).wrapping_add(260)),
        (4, (vertical as u16).wrapping_add(61)),
    ] {
        word(&mut result, frame.wrapping_sub(offset), value)?;
    }
    let h = result.memory.word(0x4395a4)?;
    let v = result.memory.word(0x4395a2)?;
    trap(
        &mut result,
        0xa8a8,
        vec![P(frame.wrapping_sub(8)), W(h), W(v)],
        services,
    )?;
    let value = result.memory.long(0x43879c)?;
    if value != 0 {
        port(&mut result, value, services)?;
    }
    colors(&mut result, services)?;
    if sound_id != 0 {
        play(&mut result, sound_id, services)?;
    }
    if result.memory.byte(0x4427b4)? == 0 {
        let backup = result.memory.long(0x4386b4)?.wrapping_add(2);
        copy(
            &mut result,
            0x3fe288,
            backup,
            frame.wrapping_sub(8),
            frame.wrapping_sub(16),
            0,
            services,
        )?;
        trap(
            &mut result,
            0xa98b,
            vec![
                P(0x403776),
                P(frame.wrapping_sub(272)),
                P(0x403776),
                P(0x403776),
            ],
            services,
        )?;
        let dialog = trap(
            &mut result,
            0xa97c,
            vec![W(156), L(0), L(u32::MAX)],
            services,
        )?;
        long(&mut result, 0x41d1c0, dialog)?;
        let value = result.memory.long(0x41d1c0)?;
        port(&mut result, value, services)?;
        let value = result.memory.long(0x4358ce)?;
        trap(&mut result, 0xaa0b, vec![L(value)], services)?;
        let font = result.memory.word(0x402f44)?;
        trap(&mut result, 0xa887, vec![W(font)], services)?;
        trap(&mut result, 0xa862, vec![L(69)], services)?;
        let dialog = result.memory.long(0x41d1c0)?;
        let h = (horizontal as u16).wrapping_add(result.memory.word(0x4395a4)?);
        let v = (vertical as u16).wrapping_add(result.memory.word(0x4395a2)?);
        trap(
            &mut result,
            0xa91b,
            vec![L(dialog), W(h), W(v), B(0)],
            services,
        )?;
        let dialog = result.memory.long(0x41d1c0)?;
        trap(&mut result, 0xa915, vec![L(dialog)], services)?;
        let dialog = result.memory.long(0x41d1c0)?;
        trap(&mut result, 0xa981, vec![L(dialog)], services)?;
    }
    if wait > 0 {
        let start = service(&mut result, FeedbackRequest::TickCount, services)?;
        let deadline = start.wrapping_add(wait as i32 as u32);
        let mut matched = false;
        for _ in 0..maximum_polls {
            if service(&mut result, FeedbackRequest::TickCount, services)? == deadline {
                matched = true;
                break;
            }
        }
        if !matched {
            return Err(RuleError::Invalid(
                "flashmessage exact tick observation limit exhausted".into(),
            ));
        }
    } else if wait < 0 {
        if result.memory.byte(0x4427b4)? == 0 {
            write(&mut result, 0x4427b4, &[1])?;
            restore_message(&mut result, frame, services)?;
            return Ok(result);
        }
        write(&mut result, 0x4427b4, &[0])?;
    } else {
        cursor(&mut result, 0x439538, services)?;
        delay(&mut result, 15, maximum_polls, services)?;
        flush(&mut result, services)?;
        event(&mut result, maximum_polls, services)?;
        flush(&mut result, services)?;
        cursor(&mut result, 0x43952c, services)?;
    }
    let dialog = result.memory.long(0x41d1c0)?;
    if dialog != 0 {
        trap(&mut result, 0xa983, vec![L(dialog)], services)?;
        long(&mut result, 0x41d1c0, 0)?;
    }
    let value = result.memory.long(0x43879c)?;
    if value != 0 {
        port(&mut result, value, services)?;
        colors(&mut result, services)?;
    }
    let value = result.memory.long(0x479fd8)?;
    if value != 0 {
        port(&mut result, value, services)?;
    }
    let backup = result.memory.long(0x4386b4)?.wrapping_add(2);
    copy(
        &mut result,
        backup,
        0x3fe288,
        frame.wrapping_sub(16),
        frame.wrapping_sub(8),
        0,
        services,
    )?;
    restore_message(&mut result, frame, services)?;
    Ok(result)
}
fn restore_message(
    result: &mut FeedbackPlan,
    frame: u32,
    services: &(impl FeedbackServices + ?Sized),
) -> Result<(), RuleError> {
    let saved = result.memory.long(frame.wrapping_sub(288))?;
    port(result, saved, services)?;
    trap(result, 0xaa15, vec![P(frame.wrapping_sub(284))], services)?;
    trap(result, 0xaa14, vec![P(frame.wrapping_sub(278))], services)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    struct Replies {
        ticks: RefCell<VecDeque<u32>>,
        events: RefCell<VecDeque<u16>>,
        caller_pointer: Option<(u32, u32)>,
    }
    impl Replies {
        fn new(ticks: &[u32], events: &[u16]) -> Self {
            Self {
                ticks: RefCell::new(ticks.iter().copied().collect()),
                events: RefCell::new(events.iter().copied().collect()),
                caller_pointer: None,
            }
        }
    }
    impl FeedbackServices for Replies {
        fn observe(
            &self,
            request: &FeedbackRequest,
            memory: &FeedbackMemory,
        ) -> Result<FeedbackObservation, RuleError> {
            let mut memory = memory.clone();
            let mut value = 0;
            match request {
                FeedbackRequest::TickCount => {
                    value = self
                        .ticks
                        .borrow_mut()
                        .pop_front()
                        .ok_or_else(|| RuleError::Invalid("missing test tick".into()))?
                }
                FeedbackRequest::Toolbox { opcode: 0xa860, .. } => {
                    let event = self
                        .events
                        .borrow_mut()
                        .pop_front()
                        .ok_or_else(|| RuleError::Invalid("missing test event".into()))?;
                    memory.insert_region(0x44720a, &event.to_be_bytes());
                }
                FeedbackRequest::Toolbox {
                    opcode: 0xa874,
                    arguments,
                } => {
                    if let [P(pointer)] = arguments.as_slice() {
                        memory.insert_region(*pointer, &0x12345678_u32.to_be_bytes());
                        if let Some((address, value)) = self.caller_pointer {
                            memory.insert_region(address, &value.to_be_bytes());
                        }
                    }
                }
                FeedbackRequest::Toolbox { opcode: 0xa97c, .. } => value = 0xabcdef01,
                FeedbackRequest::Toolbox {
                    opcode: 0xa870,
                    arguments,
                } => {
                    if let [P(pointer)] = arguments.as_slice() {
                        for off in [0, 2] {
                            let val = memory.word(pointer.wrapping_add(off))?.wrapping_add(7);
                            memory.insert_region(pointer.wrapping_add(off), &val.to_be_bytes());
                        }
                    }
                }
                FeedbackRequest::Toolbox {
                    opcode: 0xa8a8,
                    arguments,
                } => {
                    if let [P(pointer), W(h), W(v)] = arguments.as_slice() {
                        for (off, amount) in [(0, *v), (2, *h), (4, *v), (6, *h)] {
                            let val = memory.word(pointer.wrapping_add(off))?.wrapping_add(amount);
                            memory.insert_region(pointer.wrapping_add(off), &val.to_be_bytes());
                        }
                    }
                }
                _ => (),
            }
            Ok(FeedbackObservation {
                value,
                d0: 0x55667788,
                memory,
            })
        }
        fn sound_observations(
            &self,
            _: i16,
            _: &AudioContext,
            _: &FeedbackMemory,
        ) -> Result<Vec<AudioObservation>, RuleError> {
            Err(RuleError::Invalid(
                "unexpected sound manager request in muted fixture".into(),
            ))
        }
    }
    fn memory(frame: u32) -> FeedbackMemory {
        let mut memory = FeedbackMemory::default();
        memory.insert_region(frame - 1200, &vec![0; 1250]);
        for (address, count) in [
            (0x439570, 8),
            (0x43952c, 16),
            (0x4395a2, 4),
            (0x442750, 8),
            (0x43878c, 24),
            (0x4394c4, 24),
            (0x479fd8, 4),
            (0x43565a, 1),
            (0x4386b4, 4),
            (0x4386cc, 4),
            (0x4358ce, 4),
            (0x402f3c, 10),
            (0x44720a, 16),
            (0x4471a2, 2),
            (0x4434b2, 2),
            (0x4427b4, 1),
            (0x41d1c0, 4),
        ] {
            memory.insert_region(address, &vec![0; count]);
        }
        memory.insert_region(0x4395a2, &[0, 9, 0, 11]);
        memory.insert_region(frame + 8, &0x9000_u32.to_be_bytes());
        memory
    }
    fn requests(result: &FeedbackPlan) -> Vec<FeedbackRequest> {
        result
            .effects
            .iter()
            .filter_map(|effect| {
                if let FeedbackEffect::Service { request, .. } = effect {
                    Some(request.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    #[test]
    fn button_retains_source_rect_and_updates_both_caller_points() {
        let frame = 0x10000;
        let mut input = memory(frame);
        for (off, value) in [(14, 100_u16), (16, 200), (18, 300), (20, 400)] {
            input.insert_region(frame + off, &value.to_be_bytes());
        }
        let result = pressbutton(&input, frame, 6, 2, 1, &Replies::new(&[], &[])).unwrap();
        assert_eq!(
            (
                result.memory.word(frame - 8).unwrap(),
                result.memory.word(frame - 6).unwrap(),
                result.memory.word(frame - 4).unwrap(),
                result.memory.word(frame - 2).unwrap()
            ),
            (50, 174, 100, 206)
        );
        for (off, value) in [(14, 107), (16, 207), (18, 307), (20, 407)] {
            assert_eq!(result.memory.word(frame + off).unwrap(), value);
        }
        assert_eq!(input.word(frame + 14).unwrap(), 100);
        assert_eq!(result.native_return, None);
        assert!(requests(&result).contains(&FeedbackRequest::Toolbox {
            opcode: 0xa8ec,
            arguments: vec![P(2), P(0x3fe288), P(frame - 8), P(frame + 14), W(36), L(0)]
        }));
    }
    #[test]
    fn warning_selects_first_priority_waits_and_restores_in_order() {
        let frame = 0x10000;
        let mut input = memory(frame);
        input.insert_region(0x442751, &[1]);
        input.insert_region(0x442752, &[1]);
        input.insert_region(0x4394cc, &0x11223344_u32.to_be_bytes());
        input.insert_region(0x4471a2, &[0, 9]);
        input.insert_region(0x4434b2, &[0, 9]);
        let result = warn(
            &input,
            frame,
            148,
            8,
            &Replies::new(&[100, 110, 111], &[4, 3]),
        )
        .unwrap();
        assert_eq!(result.native_return, Some(0));
        assert_eq!(result.memory.word(0x4471a2).unwrap(), 0);
        assert_eq!(result.memory.word(0x4434b2).unwrap(), 0);
        let ports: Vec<u32> = requests(&result)
            .iter()
            .filter_map(|request| {
                if let FeedbackRequest::Toolbox {
                    opcode: 0xa873,
                    arguments,
                } = request
                {
                    if let [L(value)] = arguments.as_slice() {
                        Some(*value)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(ports, vec![0x11223344, 0xabcdef01, 0x12345678]);
        assert_eq!(result.memory.word(frame - 6).unwrap(), 41);
        assert_eq!(input.word(0x4471a2).unwrap(), 9);
        assert!(warn(&input, frame, 148, 2, &Replies::new(&[100, 111], &[4, 4])).is_err());
        assert_eq!(input.word(0x4471a2).unwrap(), 9);
    }
    #[test]
    fn flashmessage_preserves_toggle_and_exact_wrapping_tick_gate() {
        let frame = 0x10000;
        let input = memory(frame);
        let mut changed_pointer = Replies::new(&[], &[]);
        changed_pointer.caller_pointer = Some((frame + 8, 0xa000));
        let shown =
            flashmessage(&input, frame, 0x9000, 31, 40, -1, 0, 8, &changed_pointer).unwrap();
        assert_eq!(shown.memory.byte(0x4427b4).unwrap(), 1);
        assert_eq!(shown.memory.long(0x41d1c0).unwrap(), 0xabcdef01);
        assert!(requests(&shown).contains(&FeedbackRequest::BlockMove {
            source: 0xa000,
            destination: frame - 272,
            count: 255
        }));
        assert_eq!(input.long(frame + 8).unwrap(), 0x9000);
        assert!(
            !requests(&shown)
                .iter()
                .any(|x| matches!(x, FeedbackRequest::Toolbox { opcode: 0xa983, .. }))
        );
        let hidden = flashmessage(
            &shown.memory,
            frame,
            0xa000,
            31,
            40,
            -1,
            0,
            8,
            &Replies::new(&[], &[]),
        )
        .unwrap();
        assert_eq!(hidden.memory.byte(0x4427b4).unwrap(), 0);
        assert_eq!(hidden.memory.long(0x41d1c0).unwrap(), 0);
        let result = flashmessage(
            &input,
            frame,
            0x9000,
            31,
            40,
            3,
            0,
            4,
            &Replies::new(&[u32::MAX - 1, 2, 0, 1], &[]),
        )
        .unwrap();
        assert_eq!(
            requests(&result)
                .iter()
                .filter(|x| **x == FeedbackRequest::TickCount)
                .count(),
            4
        );
        assert!(
            flashmessage(
                &input,
                frame,
                0x9000,
                31,
                40,
                3,
                0,
                2,
                &Replies::new(&[10, 14, 15], &[])
            )
            .is_err()
        );
        assert_eq!(input.byte(0x4427b4).unwrap(), 0);
    }
}
