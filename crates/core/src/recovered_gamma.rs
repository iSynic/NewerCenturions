//! Gamma helpers recovered from original ordered instructions and native memory.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GammaError {
    MissingMemory(u32),
    MissingService(String),
    DivisionByZero,
    LinkedListCycle(u32),
}

impl fmt::Display for GammaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "gamma action unavailable: {self:?}")
    }
}

impl std::error::Error for GammaError {}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GammaMemory {
    pub bytes: BTreeMap<u32, u8>,
}

impl GammaMemory {
    pub fn insert_region(&mut self, address: u32, bytes: &[u8]) {
        for (offset, byte) in bytes.iter().enumerate() {
            self.bytes
                .insert(address.wrapping_add(offset as u32), *byte);
        }
    }

    pub fn byte(&self, address: u32) -> Result<u8, GammaError> {
        self.bytes
            .get(&address)
            .copied()
            .ok_or(GammaError::MissingMemory(address))
    }

    pub fn word(&self, address: u32) -> Result<u16, GammaError> {
        Ok(u16::from_be_bytes([
            self.byte(address)?,
            self.byte(address.wrapping_add(1))?,
        ]))
    }

    pub fn long(&self, address: u32) -> Result<u32, GammaError> {
        Ok(u32::from_be_bytes([
            self.byte(address)?,
            self.byte(address.wrapping_add(1))?,
            self.byte(address.wrapping_add(2))?,
            self.byte(address.wrapping_add(3))?,
        ]))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GammaRequest {
    TrapAddress {
        trap: u16,
        toolbox: bool,
    },
    BlockMove {
        source: u32,
        destination: u32,
        count: u32,
    },
    DisposeHandle(u32),
    DeviceStatus {
        parameter_block: u32,
    },
    DeviceControl {
        parameter_block: u32,
    },
    GetDevice,
    SetDevice(u32),
    SetEntries {
        start: i16,
        count: i16,
        entries: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GammaObservation {
    pub memory: GammaMemory,
    pub value: u32,
}

pub trait GammaServices {
    /// Supply the actual returned pointer/status and memory effects for this call.
    fn observe(
        &self,
        request: &GammaRequest,
        memory: &GammaMemory,
    ) -> Result<GammaObservation, GammaError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GammaEffect {
    Write {
        address: u32,
        bytes: Vec<u8>,
    },
    Service {
        request: GammaRequest,
        value: u32,
        memory_after: GammaMemory,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GammaReturn {
    DataRegister(u32),
    Word(i16),
    Byte(u8),
    Address(u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GammaPlan {
    pub memory: GammaMemory,
    pub effects: Vec<GammaEffect>,
    pub native_return: GammaReturn,
}

fn plan(memory: &GammaMemory, native_return: GammaReturn) -> GammaPlan {
    GammaPlan {
        memory: memory.clone(),
        effects: Vec::new(),
        native_return,
    }
}

fn write(plan: &mut GammaPlan, address: u32, bytes: &[u8]) -> Result<(), GammaError> {
    for offset in 0..bytes.len() {
        plan.memory.byte(address.wrapping_add(offset as u32))?;
    }
    plan.memory.insert_region(address, bytes);
    plan.effects.push(GammaEffect::Write {
        address,
        bytes: bytes.to_vec(),
    });
    Ok(())
}

fn service(
    plan: &mut GammaPlan,
    request: GammaRequest,
    services: &(impl GammaServices + ?Sized),
) -> Result<u32, GammaError> {
    let observation = services.observe(&request, &plan.memory)?;
    plan.effects.push(GammaEffect::Service {
        request,
        value: observation.value,
        memory_after: observation.memory.clone(),
    });
    plan.memory = observation.memory;
    Ok(observation.value)
}

fn memset_into(plan: &mut GammaPlan, address: u32, byte: u8, count: u32) -> Result<(), GammaError> {
    let mut cursor = address;
    let remainder = count & 3;
    let mut remaining = count;
    if remainder != 0 {
        remaining = remaining.wrapping_sub(remainder);
        cursor = cursor.wrapping_sub(1);
        for _ in 0..remainder {
            cursor = cursor.wrapping_add(1);
            write(plan, cursor, &[byte])?;
        }
    }
    if (remaining as i32) > 4 {
        let duplicated = u16::from(byte) | (u16::from(byte) << 8);
        let extended = i32::from(duplicated as i16) as u32;
        let pattern = extended | extended.wrapping_shl(16);
        let mut long_cursor = cursor.wrapping_sub(4);
        for _ in 0..((remaining as i32) >> 2) {
            long_cursor = long_cursor.wrapping_add(4);
            write(plan, long_cursor, &pattern.to_be_bytes())?;
        }
        cursor = long_cursor;
        remaining &= 3;
    }
    cursor = cursor.wrapping_sub(1);
    while (remaining as i32) > 0 {
        cursor = cursor.wrapping_add(1);
        write(plan, cursor, &[byte])?;
        remaining = remaining.wrapping_sub(1);
    }
    Ok(())
}

pub fn color_gamma_memset(
    memory: &GammaMemory,
    address: u32,
    byte: u8,
    count: u32,
) -> Result<GammaPlan, GammaError> {
    let mut result = plan(memory, GammaReturn::Address(address));
    memset_into(&mut result, address, byte, count)?;
    Ok(result)
}

pub fn color_gamma_trap_available(
    memory: &GammaMemory,
    trap: u16,
    services: &(impl GammaServices + ?Sized),
) -> Result<GammaPlan, GammaError> {
    let mut result = plan(memory, GammaReturn::Word(0));
    let unimplemented = service(
        &mut result,
        GammaRequest::TrapAddress {
            trap: 0xa89f,
            toolbox: true,
        },
        services,
    )?;
    let address = service(
        &mut result,
        GammaRequest::TrapAddress {
            trap,
            toolbox: trap & 0x0800 != 0,
        },
        services,
    )?;
    result.native_return = GammaReturn::Word(i16::from(address != unimplemented));
    Ok(result)
}

pub fn is_color_gamma_available(
    memory: &GammaMemory,
    services: &(impl GammaServices + ?Sized),
) -> Result<GammaPlan, GammaError> {
    let cached = memory.long(0x479fe4)?;
    if cached != 0 {
        return Ok(plan(memory, GammaReturn::Byte(cached as u8)));
    }
    let mut result = color_gamma_trap_available(memory, 0xaa29, services)?;
    let GammaReturn::Word(available) = result.native_return else {
        unreachable!()
    };
    let cached = if available == 0 { 1_u32 } else { 2_u32 };
    write(&mut result, 0x479fe4, &cached.to_be_bytes())?;
    result.native_return = GammaReturn::Byte(u8::from(available != 0));
    Ok(result)
}

fn table_data(memory: &GammaMemory, handle: u32) -> Result<u32, GammaError> {
    let table = memory.long(handle)?;
    Ok(table
        .wrapping_add(12)
        .wrapping_add(i32::from(memory.word(table.wrapping_add(4))? as i16) as u32))
}

fn header_copy(
    result: &mut GammaPlan,
    source_handle: u32,
    output_handle: u32,
    services: &(impl GammaServices + ?Sized),
) -> Result<(), GammaError> {
    let source = result.memory.long(source_handle)?;
    let destination = result.memory.long(output_handle)?;
    let count =
        14_i32.wrapping_add(i32::from(result.memory.word(source.wrapping_add(4))? as i16)) as u32;
    service(
        result,
        GammaRequest::BlockMove {
            source,
            destination,
            count,
        },
        services,
    )?;
    Ok(())
}

fn normalized_ratio(numerator: u16, denominator: u16) -> (u16, u16) {
    if denominator > 256 {
        (
            ((u32::from(numerator) << 8) / u32::from(denominator)) as u16,
            256,
        )
    } else {
        (numerator, denominator)
    }
}

fn interpolate(
    source: u8,
    target: u32,
    numerator: u16,
    denominator: u16,
) -> Result<u8, GammaError> {
    if denominator == 0 {
        return Err(GammaError::DivisionByZero);
    }
    let difference = u32::from(source).wrapping_sub(target) as i32;
    let product = difference.wrapping_mul(i32::from(numerator));
    let quotient = product / i32::from(denominator);
    Ok(i32::from(source).wrapping_sub(quotient) as u8)
}

/// FUN_001248f0: fade one native node's original table toward an RGB WORD triple.
pub fn calc_fade_to_color(
    memory: &GammaMemory,
    node_handle: u32,
    rgb_address: u32,
    numerator: u16,
    denominator: u16,
    incoming_d0: u32,
    services: &(impl GammaServices + ?Sized),
) -> Result<GammaPlan, GammaError> {
    let node = memory.long(node_handle)?;
    memory.long(node.wrapping_add(4))?;
    let source_handle = memory.long(node.wrapping_add(8))?;
    let output_handle = memory.long(node.wrapping_add(16))?;
    if memory.byte(node.wrapping_add(22))? == 0 {
        return Ok(plan(
            memory,
            GammaReturn::DataRegister((incoming_d0 & 0xffff0000) | 0xb436),
        ));
    }
    let mut result = plan(memory, GammaReturn::DataRegister(0));
    header_copy(&mut result, source_handle, output_handle, services)?;
    let table = result.memory.long(output_handle)?;
    let shift = u32::from(result.memory.word(table.wrapping_add(10))? & 63);
    let shifted = if shift >= 16 { 0_u16 } else { 1_u16 << shift };
    let maximum = i32::from(shifted.wrapping_sub(1) as i16) as u32;
    let (numerator, denominator) = normalized_ratio(numerator, denominator);
    let mut source = table_data(&result.memory, source_handle)?;
    let mut destination = table_data(&result.memory, output_handle)?;
    let entries = i32::from(result.memory.word(table.wrapping_add(8))? as i16) as u32;
    let mut channel = 0_u32;
    loop {
        let table = result.memory.long(output_handle)?;
        let channels = i32::from(result.memory.word(table.wrapping_add(6))? as i16) as u32;
        if channel >= channels {
            break;
        }
        let color = u32::from(
            result
                .memory
                .word(rgb_address.wrapping_add(channel.min(2) * 2))?,
        );
        let target = color.wrapping_mul(maximum) / 65535;
        for entry in 0..entries {
            let byte = result.memory.byte(source.wrapping_add(entry))?;
            let faded = interpolate(byte, target, numerator, denominator)?;
            write(&mut result, destination.wrapping_add(entry), &[faded])?;
        }
        source = source.wrapping_add(entries);
        destination = destination.wrapping_add(entries);
        channel = channel.wrapping_add(1);
    }
    Ok(result)
}

pub fn x_calc_fade_to_gamma(
    memory: &GammaMemory,
    node_handle: u32,
    targets_head: u32,
    numerator: u16,
    denominator: u16,
    incoming_d0: u32,
    services: &(impl GammaServices + ?Sized),
) -> Result<GammaPlan, GammaError> {
    let node = memory.long(node_handle)?;
    let device = memory.long(node.wrapping_add(4))?;
    let source_handle = memory.long(node.wrapping_add(8))?;
    let output_handle = memory.long(node.wrapping_add(16))?;
    let mut target = targets_head;
    let mut visited = BTreeSet::new();
    while target != 0 {
        if !visited.insert(target) {
            return Err(GammaError::LinkedListCycle(target));
        }
        let node = memory.long(target)?;
        if memory.long(node.wrapping_add(4))? == device {
            break;
        }
        target = memory.long(node)?;
    }
    if target == 0 {
        return Ok(plan(memory, GammaReturn::DataRegister((-43_i32) as u32)));
    }
    let target_handle = memory.long(memory.long(target)?.wrapping_add(8))?;
    let source_table = memory.long(source_handle)?;
    let target_table = memory.long(target_handle)?;
    if memory.word(target_table.wrapping_add(6))? != memory.word(source_table.wrapping_add(6))?
        || memory.word(target_table.wrapping_add(8))?
            != memory.word(source_table.wrapping_add(8))?
    {
        return Ok(plan(
            memory,
            GammaReturn::DataRegister((incoming_d0 & 0xffff0000) | 0xff65),
        ));
    }
    let mut result = plan(memory, GammaReturn::DataRegister(0));
    header_copy(&mut result, source_handle, output_handle, services)?;
    let (numerator, denominator) = normalized_ratio(numerator, denominator);
    let source = table_data(&result.memory, source_handle)?;
    let target = table_data(&result.memory, target_handle)?;
    let destination = table_data(&result.memory, output_handle)?;
    let output = result.memory.long(output_handle)?;
    let entries = result.memory.word(output.wrapping_add(8))? as i16;
    let channels = result.memory.word(output.wrapping_add(6))? as i16;
    let product_word = (i32::from(entries) * i32::from(channels)) as i16;
    let count = i32::from(product_word) as u32;
    for entry in 0..count {
        let target_byte = result.memory.byte(target.wrapping_add(entry))?;
        let source_byte = result.memory.byte(source.wrapping_add(entry))?;
        let faded = interpolate(source_byte, u32::from(target_byte), numerator, denominator)?;
        write(&mut result, destination.wrapping_add(entry), &[faded])?;
    }
    Ok(result)
}

pub fn dispose_gamma_list(
    memory: &GammaMemory,
    head: u32,
    services: &(impl GammaServices + ?Sized),
) -> Result<GammaPlan, GammaError> {
    if head == 0 {
        return Ok(plan(memory, GammaReturn::Word(-109)));
    }
    let mut result = plan(memory, GammaReturn::Word(0));
    let mut handle = head;
    let mut visited = BTreeSet::new();
    while handle != 0 {
        if !visited.insert(handle) {
            return Err(GammaError::LinkedListCycle(handle));
        }
        let node = result.memory.long(handle)?;
        let next = result.memory.long(node)?;
        let backup = result.memory.long(node.wrapping_add(12))?;
        if backup != 0 {
            service(&mut result, GammaRequest::DisposeHandle(backup), services)?;
        }
        let source = result
            .memory
            .long(result.memory.long(handle)?.wrapping_add(8))?;
        service(&mut result, GammaRequest::DisposeHandle(source), services)?;
        let output = result
            .memory
            .long(result.memory.long(handle)?.wrapping_add(16))?;
        service(&mut result, GammaRequest::DisposeHandle(output), services)?;
        service(&mut result, GammaRequest::DisposeHandle(handle), services)?;
        handle = next;
    }
    Ok(result)
}

pub fn get_device_gamma_table(
    memory: &GammaMemory,
    frame_pointer: u32,
    device_handle: u32,
    output_pointer: u32,
    services: &(impl GammaServices + ?Sized),
) -> Result<GammaPlan, GammaError> {
    let mut result = plan(memory, GammaReturn::Word(0));
    write(&mut result, output_pointer, &0_u32.to_be_bytes())?;
    let block = frame_pointer.wrapping_sub(50);
    memset_into(&mut result, block, 0, 50)?;
    write(
        &mut result,
        frame_pointer.wrapping_sub(24),
        &8_u16.to_be_bytes(),
    )?;
    let reference = result.memory.word(result.memory.long(device_handle)?)?;
    write(
        &mut result,
        frame_pointer.wrapping_sub(26),
        &reference.to_be_bytes(),
    )?;
    write(
        &mut result,
        frame_pointer.wrapping_sub(22),
        &output_pointer.to_be_bytes(),
    )?;
    let status = service(
        &mut result,
        GammaRequest::DeviceStatus {
            parameter_block: block,
        },
        services,
    )?;
    result.native_return = GammaReturn::Word(status as i16);
    Ok(result)
}

pub fn set_device_gamma_table(
    memory: &GammaMemory,
    frame_pointer: u32,
    device_handle: u32,
    gamma_address: u32,
    services: &(impl GammaServices + ?Sized),
) -> Result<GammaPlan, GammaError> {
    let mut result = plan(memory, GammaReturn::Word(0));
    let block = frame_pointer.wrapping_sub(50);
    memset_into(&mut result, block, 0, 50)?;
    write(
        &mut result,
        frame_pointer.wrapping_sub(24),
        &4_u16.to_be_bytes(),
    )?;
    let reference = result.memory.word(result.memory.long(device_handle)?)?;
    write(
        &mut result,
        frame_pointer.wrapping_sub(26),
        &reference.to_be_bytes(),
    )?;
    write(
        &mut result,
        frame_pointer.wrapping_sub(22),
        &gamma_address.to_be_bytes(),
    )?;
    let status = service(
        &mut result,
        GammaRequest::DeviceControl {
            parameter_block: block,
        },
        services,
    )? as i16;
    if status == 0 {
        let previous = service(&mut result, GammaRequest::GetDevice, services)?;
        service(
            &mut result,
            GammaRequest::SetDevice(device_handle),
            services,
        )?;
        let device = result.memory.long(device_handle)?;
        let pixmap_handle = result.memory.long(device.wrapping_add(22))?;
        let pixmap = result.memory.long(pixmap_handle)?;
        let table_handle = result.memory.long(pixmap.wrapping_add(42))?;
        let table = result.memory.long(table_handle)?;
        let count = result.memory.word(table.wrapping_add(6))? as i16;
        service(
            &mut result,
            GammaRequest::SetEntries {
                start: 0,
                count,
                entries: table.wrapping_add(8),
            },
            services,
        )?;
        service(&mut result, GammaRequest::SetDevice(previous), services)?;
    }
    result.native_return = GammaReturn::Word(status);
    Ok(result)
}

fn merge(result: &mut GammaPlan, child: GammaPlan) -> GammaReturn {
    result.memory = child.memory;
    result.effects.extend(child.effects);
    child.native_return
}

/// FUN_00124576: restore each node's optional backup, otherwise its original table.
pub fn restore_gamma_list(
    memory: &GammaMemory,
    frame_pointer: u32,
    head: u32,
    services: &(impl GammaServices + ?Sized),
) -> Result<GammaPlan, GammaError> {
    let mut result = plan(memory, GammaReturn::Word(0));
    let mut handle = head;
    let mut visited = BTreeSet::new();
    while handle != 0 {
        if !visited.insert(handle) {
            return Err(GammaError::LinkedListCycle(handle));
        }
        let node = result.memory.long(handle)?;
        let backup = result.memory.long(node.wrapping_add(12))?;
        let gamma = if backup != 0 {
            backup
        } else {
            result.memory.long(node.wrapping_add(8))?
        };
        let device = result.memory.long(node.wrapping_add(4))?;
        let child = set_device_gamma_table(
            &result.memory,
            frame_pointer.wrapping_sub(24),
            device,
            gamma,
            services,
        )?;
        merge(&mut result, child);
        handle = result.memory.long(result.memory.long(handle)?)?;
    }
    Ok(result)
}

#[derive(Clone, Copy)]
enum FadeTarget {
    Color(u32),
    Gamma(u32),
}

fn fade_list(
    memory: &GammaMemory,
    frame_pointer: u32,
    head: u32,
    target: FadeTarget,
    numerator: u16,
    denominator: u16,
    services: &(impl GammaServices + ?Sized),
) -> Result<GammaPlan, GammaError> {
    let mut result = plan(memory, GammaReturn::Word(0));
    write(&mut result, 0x479fe2, &0_u16.to_be_bytes())?;
    let mut handle = head;
    let mut visited = BTreeSet::new();
    while handle != 0 {
        if !visited.insert(handle) {
            return Err(GammaError::LinkedListCycle(handle));
        }
        let child = match target {
            FadeTarget::Color(rgb) => calc_fade_to_color(
                &result.memory,
                handle,
                rgb,
                numerator,
                denominator,
                handle,
                services,
            )?,
            FadeTarget::Gamma(targets) => x_calc_fade_to_gamma(
                &result.memory,
                handle,
                targets,
                numerator,
                denominator,
                handle,
                services,
            )?,
        };
        let GammaReturn::DataRegister(status) = merge(&mut result, child) else {
            unreachable!()
        };
        let status = status as i16;
        if status != 0 {
            write(&mut result, 0x479fe2, &status.to_be_bytes())?;
        } else {
            let node = result.memory.long(handle)?;
            let gamma = result.memory.long(node.wrapping_add(16))?;
            let device = result.memory.long(node.wrapping_add(4))?;
            let child = set_device_gamma_table(
                &result.memory,
                frame_pointer.wrapping_sub(36),
                device,
                gamma,
                services,
            )?;
            let GammaReturn::Word(status) = merge(&mut result, child) else {
                unreachable!()
            };
            if status != 0 {
                write(&mut result, 0x479fe2, &status.to_be_bytes())?;
            }
        }
        handle = result.memory.long(result.memory.long(handle)?)?;
    }
    result.native_return = GammaReturn::Word(result.memory.word(0x479fe2)? as i16);
    Ok(result)
}

/// XCalcFadeToColor: first WORD is denominator, second WORD is numerator.
pub fn x_calc_fade_to_color(
    memory: &GammaMemory,
    frame_pointer: u32,
    denominator: u16,
    numerator: u16,
    rgb_address: u32,
    head: u32,
    services: &(impl GammaServices + ?Sized),
) -> Result<GammaPlan, GammaError> {
    fade_list(
        memory,
        frame_pointer,
        head,
        FadeTarget::Color(rgb_address),
        numerator,
        denominator,
        services,
    )
}

/// FUN_0012485e: retain the last nonzero error while processing the source list.
pub fn fade_gamma_list(
    memory: &GammaMemory,
    frame_pointer: u32,
    denominator: u16,
    numerator: u16,
    targets: u32,
    head: u32,
    services: &(impl GammaServices + ?Sized),
) -> Result<GammaPlan, GammaError> {
    fade_list(
        memory,
        frame_pointer,
        head,
        FadeTarget::Gamma(targets),
        numerator,
        denominator,
        services,
    )
}
