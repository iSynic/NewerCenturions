//! Monitor capture and fade orchestration recovered from ordered instructions.

use crate::recovered_gamma::{
    self as gamma, GammaEffect, GammaError, GammaMemory, GammaObservation, GammaPlan, GammaRequest,
    GammaReturn, GammaServices,
};
use std::collections::BTreeSet;

const ERROR: u32 = 0x479fe2;
const AVAILABILITY: u32 = 0x479fe4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GammaControlRequest {
    DeviceAttribute { device: u32, attribute: u16 },
    FirstDevice,
    NextDevice(u32),
    NewHandle(u32),
    ResizeHandle { handle: u32, size: u32 },
    Runtime400170 { handle_pointer: u32 },
    ToolboxA852,
    ToolboxA853,
}

pub trait GammaControlServices: GammaServices {
    fn observe_control(
        &self,
        request: &GammaControlRequest,
        memory: &GammaMemory,
    ) -> Result<GammaObservation, GammaError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GammaControlEffect {
    Gamma(GammaEffect),
    Service {
        request: GammaControlRequest,
        value: u32,
        memory_after: GammaMemory,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GammaControlPlan {
    pub memory: GammaMemory,
    pub effects: Vec<GammaControlEffect>,
    pub native_return: GammaReturn,
}

fn plan(memory: &GammaMemory, native_return: GammaReturn) -> GammaControlPlan {
    GammaControlPlan {
        memory: memory.clone(),
        effects: Vec::new(),
        native_return,
    }
}

fn write(result: &mut GammaControlPlan, address: u32, bytes: &[u8]) -> Result<(), GammaError> {
    for offset in 0..bytes.len() {
        result.memory.byte(address.wrapping_add(offset as u32))?;
    }
    result.memory.insert_region(address, bytes);
    result
        .effects
        .push(GammaControlEffect::Gamma(GammaEffect::Write {
            address,
            bytes: bytes.to_vec(),
        }));
    Ok(())
}

fn merge(result: &mut GammaControlPlan, child: GammaPlan) -> GammaReturn {
    result.memory = child.memory;
    result
        .effects
        .extend(child.effects.into_iter().map(GammaControlEffect::Gamma));
    child.native_return
}

fn merge_control(result: &mut GammaControlPlan, child: GammaControlPlan) -> GammaReturn {
    result.memory = child.memory;
    result.effects.extend(child.effects);
    child.native_return
}

fn service(
    result: &mut GammaControlPlan,
    request: GammaControlRequest,
    services: &(impl GammaControlServices + ?Sized),
) -> Result<u32, GammaError> {
    let observed = services.observe_control(&request, &result.memory)?;
    result.effects.push(GammaControlEffect::Service {
        request,
        value: observed.value,
        memory_after: observed.memory.clone(),
    });
    result.memory = observed.memory;
    Ok(observed.value)
}

fn gamma_service(
    result: &mut GammaControlPlan,
    request: GammaRequest,
    services: &(impl GammaServices + ?Sized),
) -> Result<u32, GammaError> {
    let observed = services.observe(&request, &result.memory)?;
    result
        .effects
        .push(GammaControlEffect::Gamma(GammaEffect::Service {
            request,
            value: observed.value,
            memory_after: observed.memory.clone(),
        }));
    result.memory = observed.memory;
    Ok(observed.value)
}

fn ensure_available(
    result: &mut GammaControlPlan,
    services: &(impl GammaControlServices + ?Sized),
) -> Result<bool, GammaError> {
    if result.memory.long(AVAILABILITY)? == 0 {
        let child = gamma::is_color_gamma_available(&result.memory, services)?;
        merge(result, child);
    }
    if result.memory.long(AVAILABILITY)? == 1 {
        write(result, ERROR, &(-201_i16).to_be_bytes())?;
        Ok(false)
    } else {
        Ok(true)
    }
}

// The native three shifts implement signed WORD division by eight toward zero.
fn divided_bit_count(product: u16) -> i32 {
    let sign_part = ((product as i16 >> 2) as u16 >> 8) >> 5;
    i32::from((product.wrapping_add(sign_part) as i16) >> 3)
}

/// XHighLevelFade, 00124308: capture one device and build its list node.
pub fn capture_monitor_gamma(
    memory: &GammaMemory,
    frame_pointer: u32,
    device: u32,
    services: &(impl GammaControlServices + ?Sized),
) -> Result<GammaControlPlan, GammaError> {
    let mut result = plan(memory, GammaReturn::Address(0));
    let original_pointer = frame_pointer.wrapping_sub(4);
    let table_pointer = frame_pointer.wrapping_sub(8);
    write(&mut result, original_pointer, &0_u32.to_be_bytes())?;
    write(&mut result, frame_pointer.wrapping_sub(9), &[0])?;
    if !ensure_available(&mut result, services)? {
        return Ok(result);
    }
    let screen = service(
        &mut result,
        GammaControlRequest::DeviceAttribute {
            device,
            attribute: 13,
        },
        services,
    )? as u8;
    let acceptable = if screen != 0 {
        let inactive = service(
            &mut result,
            GammaControlRequest::DeviceAttribute {
                device,
                attribute: 14,
            },
            services,
        )? as u8;
        inactive == 0
            && result
                .memory
                .word(result.memory.long(device)?.wrapping_add(4))?
                != 1
    } else {
        false
    };
    if !acceptable {
        write(&mut result, ERROR, &(-155_i16).to_be_bytes())?;
        return Ok(result);
    }
    let child = gamma::get_device_gamma_table(
        &result.memory,
        frame_pointer.wrapping_sub(58),
        device,
        table_pointer,
        services,
    )?;
    let GammaReturn::Word(status) = merge(&mut result, child) else {
        unreachable!()
    };
    if status != 0 {
        write(&mut result, ERROR, &status.to_be_bytes())?;
        return Ok(result);
    }
    let source = result.memory.long(table_pointer)?;
    let channels = result.memory.word(source.wrapping_add(6))?;
    let entries = result.memory.word(source.wrapping_add(8))?;
    let bits = result.memory.word(source.wrapping_add(10))?;
    let first_product = i32::from(channels as i16).wrapping_mul(i32::from(entries as i16));
    let product = i32::from(first_product as i16).wrapping_mul(i32::from(bits as i16));
    let formula = i32::from(result.memory.word(source.wrapping_add(4))? as i16);
    let mut size = 14_u32
        .wrapping_add(formula as u32)
        .wrapping_add(divided_bit_count(product as u16) as u32);
    let node_handle = service(&mut result, GammaControlRequest::NewHandle(24), services)?;
    let original = service(
        &mut result,
        GammaControlRequest::NewHandle(u32::from(size as u16)),
        services,
    )?;
    write(&mut result, original_pointer, &original.to_be_bytes())?;
    let output = service(
        &mut result,
        GammaControlRequest::NewHandle(u32::from(size as u16)),
        services,
    )?;
    if node_handle == 0 || result.memory.long(original_pointer)? == 0 || output == 0 {
        let error = result.memory.word(0x220)? as i16;
        write(
            &mut result,
            ERROR,
            &(if error == 0 { -108 } else { error }).to_be_bytes(),
        )?;
        if node_handle != 0 {
            gamma_service(
                &mut result,
                GammaRequest::DisposeHandle(node_handle),
                services,
            )?;
        }
        let original = result.memory.long(original_pointer)?;
        if original != 0 {
            gamma_service(&mut result, GammaRequest::DisposeHandle(original), services)?;
        }
        if output != 0 {
            gamma_service(&mut result, GammaRequest::DisposeHandle(output), services)?;
        }
        return Ok(result);
    }
    let source = result.memory.long(table_pointer)?;
    let destination = result.memory.long(result.memory.long(original_pointer)?)?;
    gamma_service(
        &mut result,
        GammaRequest::BlockMove {
            source,
            destination,
            count: u32::from(size as u16),
        },
        services,
    )?;
    let mut backup = 0;
    if result.memory.word(
        result
            .memory
            .long(result.memory.long(original_pointer)?)?
            .wrapping_add(6),
    )? == 1
    {
        // Both copies occur in the original, even though the second is redundant.
        for _ in 0..2 {
            let source = result.memory.long(result.memory.long(original_pointer)?)?;
            let destination = result.memory.long(output)?;
            gamma_service(
                &mut result,
                GammaRequest::BlockMove {
                    source,
                    destination,
                    count: u32::from(size as u16),
                },
                services,
            )?;
        }
        let source = result.memory.long(result.memory.long(original_pointer)?)?;
        let product = i32::from(result.memory.word(source.wrapping_add(8))? as i16).wrapping_mul(
            i32::from(result.memory.word(source.wrapping_add(10))? as i16),
        );
        let channel_bytes = divided_bit_count(product as u16) as u32;
        let formula = i32::from(result.memory.word(source.wrapping_add(4))? as i16) as u32;
        let expanded_size = 14_u32
            .wrapping_add(formula)
            .wrapping_add(channel_bytes.wrapping_mul(3));
        service(
            &mut result,
            GammaControlRequest::ResizeHandle {
                handle: output,
                size: expanded_size,
            },
            services,
        )?;
        let output_data = result.memory.long(output)?;
        let formula = i32::from(result.memory.word(output_data.wrapping_add(4))? as i16) as u32;
        let samples = output_data.wrapping_add(12).wrapping_add(formula);
        for multiplier in [1_u32, 2] {
            gamma_service(
                &mut result,
                GammaRequest::BlockMove {
                    source: samples,
                    destination: samples.wrapping_add(channel_bytes.wrapping_mul(multiplier)),
                    count: channel_bytes,
                },
                services,
            )?;
        }
        let output_data = result.memory.long(output)?;
        write(
            &mut result,
            output_data.wrapping_add(6),
            &3_u16.to_be_bytes(),
        )?;
        let child = gamma::set_device_gamma_table(
            &result.memory,
            frame_pointer.wrapping_sub(58),
            device,
            output,
            services,
        )?;
        let GammaReturn::Word(status) = merge(&mut result, child) else {
            unreachable!()
        };
        if status == 0 {
            backup = result.memory.long(original_pointer)?;
            write(&mut result, original_pointer, &output.to_be_bytes())?;
            service(
                &mut result,
                GammaControlRequest::Runtime400170 {
                    handle_pointer: original_pointer,
                },
                services,
            )?;
            size = (size & 0xffff0000)
                | u32::from((size as u16).wrapping_add(channel_bytes.wrapping_mul(2) as u16));
        }
    }
    // Re-read each master pointer after prior writes and observable calls.
    for (offset, value) in [(0, 0), (4, device)] {
        let node = result.memory.long(node_handle)?;
        write(&mut result, node.wrapping_add(offset), &value.to_be_bytes())?;
    }
    let original = result.memory.long(original_pointer)?;
    let node = result.memory.long(node_handle)?;
    write(&mut result, node.wrapping_add(8), &original.to_be_bytes())?;
    for (offset, value) in [(12, backup), (16, output)] {
        let node = result.memory.long(node_handle)?;
        write(&mut result, node.wrapping_add(offset), &value.to_be_bytes())?;
    }
    let node = result.memory.long(node_handle)?;
    write(
        &mut result,
        node.wrapping_add(20),
        &(size as u16).to_be_bytes(),
    )?;
    let source = result.memory.long(result.memory.long(original_pointer)?)?;
    let flag = u8::from(result.memory.word(source.wrapping_add(6))? == 3);
    let node = result.memory.long(node_handle)?;
    write(&mut result, node.wrapping_add(22), &[flag])?;
    result.native_return = GammaReturn::Address(node_handle);
    Ok(result)
}

/// FUN_0012427c: prepend each successful capture in device enumeration order.
pub fn capture_monitor_list(
    memory: &GammaMemory,
    frame_pointer: u32,
    services: &(impl GammaControlServices + ?Sized),
) -> Result<GammaControlPlan, GammaError> {
    let mut result = plan(memory, GammaReturn::Address(0));
    if !ensure_available(&mut result, services)? {
        return Ok(result);
    }
    let mut device = service(&mut result, GammaControlRequest::FirstDevice, services)?;
    let mut head = 0_u32;
    let mut visited = BTreeSet::new();
    while device != 0 {
        if !visited.insert(device) {
            return Err(GammaError::LinkedListCycle(device));
        }
        let child = capture_monitor_gamma(
            &result.memory,
            frame_pointer.wrapping_sub(28),
            device,
            services,
        )?;
        let GammaReturn::Address(handle) = merge_control(&mut result, child) else {
            unreachable!()
        };
        if handle != 0 {
            let node = result.memory.long(handle)?;
            write(&mut result, node, &head.to_be_bytes())?;
            head = handle;
        }
        device = service(
            &mut result,
            GammaControlRequest::NextDevice(device),
            services,
        )?;
    }
    if head == 0 {
        write(&mut result, ERROR, &(-19400_i16).to_be_bytes())?;
    }
    result.native_return = GammaReturn::Address(head);
    Ok(result)
}

/// GETMONITORSTATE, 00124212: write the list head and return the global error WORD.
pub fn get_monitor_state(
    memory: &GammaMemory,
    frame_pointer: u32,
    output_pointer: u32,
    services: &(impl GammaControlServices + ?Sized),
) -> Result<GammaControlPlan, GammaError> {
    let mut result = plan(memory, GammaReturn::Word(0));
    let child = capture_monitor_list(&result.memory, frame_pointer.wrapping_sub(12), services)?;
    let GammaReturn::Address(head) = merge_control(&mut result, child) else {
        unreachable!()
    };
    write(&mut result, output_pointer, &head.to_be_bytes())?;
    result.native_return = GammaReturn::Word(result.memory.word(ERROR)? as i16);
    Ok(result)
}

/// FUN_00124242: optional restore, followed by unconditional disposal.
pub fn release_monitor_state(
    memory: &GammaMemory,
    frame_pointer: u32,
    restore: u8,
    head: u32,
    services: &(impl GammaControlServices + ?Sized),
) -> Result<GammaControlPlan, GammaError> {
    let mut result = plan(memory, GammaReturn::Word(0));
    if restore != 0 {
        let child = gamma::restore_gamma_list(
            &result.memory,
            frame_pointer.wrapping_sub(14),
            head,
            services,
        )?;
        merge(&mut result, child);
    }
    let child = gamma::dispose_gamma_list(&result.memory, head, services)?;
    result.native_return = merge(&mut result, child);
    Ok(result)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FadeSequence {
    /// Zero captures every device; nonzero captures this device handle.
    pub device: u32,
    pub gamma_target: u8,
    pub target: u32,
    pub steps: u16,
    pub curve: u16,
}

/// FUN_001246a8: inclusive linear or quadratic fade, then dispose the capture.
pub fn run_fade_sequence(
    memory: &GammaMemory,
    frame_pointer: u32,
    sequence: FadeSequence,
    services: &(impl GammaControlServices + ?Sized),
) -> Result<GammaControlPlan, GammaError> {
    let mut result = plan(memory, GammaReturn::Word(-19401));
    let mut steps = sequence.steps;
    if sequence.curve & 4 != 0 {
        while steps > 256 {
            steps >>= 1;
        }
    }
    let denominator = match sequence.curve {
        2 => steps,
        4 | 5 => steps.wrapping_mul(steps),
        _ => return Ok(result),
    };
    service(&mut result, GammaControlRequest::ToolboxA852, services)?;
    let child = if sequence.device == 0 {
        capture_monitor_list(&result.memory, frame_pointer.wrapping_sub(44), services)?
    } else {
        capture_monitor_gamma(
            &result.memory,
            frame_pointer.wrapping_sub(48),
            sequence.device,
            services,
        )?
    };
    let GammaReturn::Address(head) = merge_control(&mut result, child) else {
        unreachable!()
    };
    if head == 0 {
        result.native_return = GammaReturn::Word(result.memory.word(ERROR)? as i16);
        return Ok(result);
    }
    let mut status = -19401;
    for index in 0..=u32::from(steps) {
        let numerator = match sequence.curve {
            2 => index,
            4 => index.wrapping_mul(index),
            5 => u32::from(denominator).wrapping_sub(
                u32::from(steps)
                    .wrapping_sub(index)
                    .wrapping_mul(u32::from(steps).wrapping_sub(index)),
            ),
            _ => unreachable!(),
        } as u16;
        let child_frame = frame_pointer.wrapping_sub(54);
        let child = if sequence.gamma_target == 0 {
            gamma::x_calc_fade_to_color(
                &result.memory,
                child_frame,
                denominator,
                numerator,
                sequence.target,
                head,
                services,
            )?
        } else {
            gamma::fade_gamma_list(
                &result.memory,
                child_frame,
                denominator,
                numerator,
                sequence.target,
                head,
                services,
            )?
        };
        let GammaReturn::Word(child_status) = merge(&mut result, child) else {
            unreachable!()
        };
        status = child_status;
        if status != 0 {
            break;
        }
    }
    let child = gamma::dispose_gamma_list(&result.memory, head, services)?;
    merge(&mut result, child);
    service(&mut result, GammaControlRequest::ToolboxA853, services)?;
    write(&mut result, ERROR, &status.to_be_bytes())?;
    result.native_return = GammaReturn::Word(status);
    Ok(result)
}

/// FUN_00124632: fade all devices toward an existing monitor-state list.
pub fn fade_to_monitor_state(
    memory: &GammaMemory,
    frame_pointer: u32,
    curve: u16,
    steps: u16,
    target: u32,
    services: &(impl GammaControlServices + ?Sized),
) -> Result<GammaControlPlan, GammaError> {
    run_fade_sequence(
        memory,
        frame_pointer.wrapping_sub(22),
        FadeSequence {
            device: 0,
            gamma_target: 1,
            target,
            steps,
            curve,
        },
        services,
    )
}

/// FUN_00124666: copy the six-byte constant RGB before fading all devices.
pub fn fade_to_constant_color(
    memory: &GammaMemory,
    frame_pointer: u32,
    curve: u16,
    steps: u16,
    services: &(impl GammaControlServices + ?Sized),
) -> Result<GammaControlPlan, GammaError> {
    let mut result = plan(memory, GammaReturn::Word(0));
    let target = frame_pointer.wrapping_sub(6);
    let rgb_long = result.memory.long(0x403770)?;
    write(&mut result, target, &rgb_long.to_be_bytes())?;
    let rgb_word = result.memory.word(0x403774)?;
    write(
        &mut result,
        frame_pointer.wrapping_sub(2),
        &rgb_word.to_be_bytes(),
    )?;
    let child = run_fade_sequence(
        &result.memory,
        frame_pointer.wrapping_sub(28),
        FadeSequence {
            device: 0,
            gamma_target: 0,
            target,
            steps,
            curve,
        },
        services,
    )?;
    result.native_return = merge_control(&mut result, child);
    Ok(result)
}
