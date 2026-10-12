//! File, map and error-dialog control with observed library and Toolbox services.

use crate::RuleError;
use crate::recovered_fade_ui;
use crate::recovered_feedback::{self as feedback, FeedbackEffect, FeedbackServices};
use crate::recovered_files::{
    self as files, FileBuffer, FileContext, FileEffect, FileObservation, FileReply, FileRequest,
};
use crate::recovered_gamma::GammaMemory;
use crate::recovered_gamma_control::{GammaControlEffect, GammaControlServices};
use crate::recovered_records::RecordWrite;
use crate::recovered_rendering::RenderingMemory;
use crate::recovered_text::{self as text, PascalString, TextEffect};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadFlow {
    Returned,
    Terminated(i16),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadArg {
    Word(u16),
    Long(u32),
    Pointer(u32),
}
use LoadArg::{Long as L, Pointer as P, Word as W};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadRequest {
    CopyCString {
        source: u32,
        destination: u32,
    },
    AppendCString {
        source: u32,
        destination: u32,
    },
    PascalToC(u32),
    CToPascal(u32),
    NumberToString {
        value: i32,
        destination: u32,
    },
    GetIndexedString {
        resource: i16,
        index: i16,
        destination: u32,
    },
    Toolbox {
        opcode: u16,
        selector: Option<u32>,
        arguments: Vec<LoadArg>,
    },
    Open {
        name: u32,
        mode: u32,
    },
    Seek {
        handle: u32,
        offset: u32,
        origin: i16,
    },
    Read {
        handle: u32,
        size: u32,
        count: u32,
        destination: u32,
    },
    Close(u32),
    Version(TextEffect),
    Runtime400028(i16),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadObservation {
    pub memory: RenderingMemory,
    pub value: u32,
    pub d0: u32,
    /// Only Runtime400028 may finish the application within this adapter.
    pub flow: LoadFlow,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionInputs {
    pub current_resource_file: i16,
    pub resource: Vec<u8>,
}

pub trait LoadServices: FeedbackServices + GammaControlServices {
    fn observe_load(
        &self,
        request: &LoadRequest,
        memory: &RenderingMemory,
    ) -> Result<LoadObservation, RuleError>;
    /// The resource bytes and resource-file selector consumed by the owned helper.
    fn version_inputs(
        &self,
        id: i16,
        file: i16,
        memory: &RenderingMemory,
    ) -> Result<VersionInputs, RuleError>;
    fn filename_number(
        &self,
        variant: u8,
        memory: &RenderingMemory,
    ) -> Result<PascalString, RuleError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoadLimits {
    pub filename_capacity: usize,
    pub maximum_polls: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadEffect {
    Write(RecordWrite),
    Service {
        request: LoadRequest,
        observation: LoadObservation,
    },
    Feedback(FeedbackEffect),
    Fade(GammaControlEffect),
    File(FileEffect),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadPlan {
    pub memory: RenderingMemory,
    pub effects: Vec<LoadEffect>,
    pub flow: LoadFlow,
    pub native_return: Option<u32>,
}
fn plan(memory: &RenderingMemory, native_return: Option<u32>) -> LoadPlan {
    LoadPlan {
        memory: memory.clone(),
        effects: vec![],
        flow: LoadFlow::Returned,
        native_return,
    }
}
fn write(result: &mut LoadPlan, address: u32, bytes: &[u8]) -> Result<(), RuleError> {
    for i in 0..bytes.len() {
        result.memory.byte(address.wrapping_add(i as u32))?;
    }
    result.memory.insert_region(address, bytes);
    result.effects.push(LoadEffect::Write(RecordWrite {
        address,
        bytes: bytes.to_vec(),
    }));
    Ok(())
}
fn word(result: &mut LoadPlan, address: u32, value: u16) -> Result<(), RuleError> {
    write(result, address, &value.to_be_bytes())
}
fn long(result: &mut LoadPlan, address: u32, value: u32) -> Result<(), RuleError> {
    write(result, address, &value.to_be_bytes())
}
fn observe(
    result: &mut LoadPlan,
    request: LoadRequest,
    services: &(impl LoadServices + ?Sized),
) -> Result<u32, RuleError> {
    let observed = services.observe_load(&request, &result.memory)?;
    if observed.flow != LoadFlow::Returned && !matches!(request, LoadRequest::Runtime400028(_)) {
        return Err(RuleError::Invalid(
            "unexpected termination outside the runtime exit boundary".into(),
        ));
    }
    let value = observed.value;
    result.memory = observed.memory.clone();
    result.flow = observed.flow;
    if result.flow != LoadFlow::Returned {
        result.native_return = None;
    }
    result.effects.push(LoadEffect::Service {
        request,
        observation: observed,
    });
    Ok(value)
}
fn toolbox(
    result: &mut LoadPlan,
    opcode: u16,
    selector: Option<u32>,
    arguments: Vec<LoadArg>,
    services: &(impl LoadServices + ?Sized),
) -> Result<u32, RuleError> {
    observe(
        result,
        LoadRequest::Toolbox {
            opcode,
            selector,
            arguments,
        },
        services,
    )
}
fn append(
    result: &mut LoadPlan,
    destination: u32,
    source: u32,
    services: &(impl LoadServices + ?Sized),
) -> Result<(), RuleError> {
    observe(
        result,
        LoadRequest::AppendCString {
            source,
            destination,
        },
        services,
    )?;
    Ok(())
}
fn convert(
    result: &mut LoadPlan,
    value: i32,
    destination: u32,
    services: &(impl LoadServices + ?Sized),
) -> Result<(), RuleError> {
    observe(
        result,
        LoadRequest::NumberToString { value, destination },
        services,
    )?;
    observe(result, LoadRequest::PascalToC(destination), services)?;
    Ok(())
}
fn fade(
    result: &mut LoadPlan,
    frame: u32,
    direction: i16,
    services: &(impl LoadServices + ?Sized),
) -> Result<(), RuleError> {
    let memory = GammaMemory {
        bytes: result.memory.bytes.clone(),
    };
    let child = recovered_fade_ui::fadeinout(&memory, frame, 15, direction, services)
        .map_err(|error| RuleError::Invalid(error.to_string()))?;
    result.memory.bytes = child.memory.bytes;
    result
        .effects
        .extend(child.effects.into_iter().map(LoadEffect::Fade));
    Ok(())
}
fn merge_feedback(result: &mut LoadPlan, child: feedback::FeedbackPlan) {
    result.memory = child.memory;
    result
        .effects
        .extend(child.effects.into_iter().map(LoadEffect::Feedback));
}
fn sound(
    result: &mut LoadPlan,
    id: i16,
    services: &(impl LoadServices + ?Sized),
) -> Result<(), RuleError> {
    let child = feedback::sound(&result.memory, id, services)?;
    merge_feedback(result, child);
    Ok(())
}
fn merge(result: &mut LoadPlan, child: LoadPlan) {
    result.memory = child.memory;
    result.effects.extend(child.effects);
    result.flow = child.flow;
    if result.flow != LoadFlow::Returned {
        result.native_return = None;
    }
}
fn version(
    result: &mut LoadPlan,
    services: &(impl LoadServices + ?Sized),
) -> Result<(), RuleError> {
    let resource_file = result.memory.word(0x438596)? as i16;
    let inputs = services.version_inputs(1, resource_file, &result.memory)?;
    let mut destination = PascalString::default();
    let (_, effects) = text::get_version_string(
        &mut destination,
        1,
        resource_file,
        inputs.current_resource_file,
        &inputs.resource,
    )?;
    for effect in effects {
        if matches!(effect, TextEffect::CopyVersionBytes { .. }) {
            write(result, 0x4387a4, destination.raw())?;
        } else {
            observe(result, LoadRequest::Version(effect), services)?;
        }
    }
    Ok(())
}

/// scratch / scratch2: source dialog construction, including an observed exit.
fn scratch_impl(
    memory: &RenderingMemory,
    frame: u32,
    second: bool,
    services: &(impl LoadServices + ?Sized),
) -> Result<LoadPlan, RuleError> {
    let mut result = plan(memory, Some(0));
    fade(
        &mut result,
        frame.wrapping_sub(if second { 532 } else { 528 }),
        i16::from(second),
        services,
    )?;
    let message = frame.wrapping_sub(256);
    let numeric = frame.wrapping_sub(512);
    observe(
        &mut result,
        LoadRequest::CopyCString {
            source: 0x40250d,
            destination: message,
        },
        services,
    )?;
    // scratch2 captures the adjusted argument before any registration/file request.
    let cached_code = if second {
        Some(
            (result.memory.word(frame.wrapping_add(8))? as i16)
                .wrapping_abs()
                .wrapping_add(1000),
        )
    } else {
        None
    };
    let registered =
        toolbox(&mut result, 0xa85d, None, vec![P(0x438598), L(6)], services)? as u8 != 0;
    if !registered {
        append(&mut result, message, 0x402268, services)?;
    }
    if result.memory.byte(0x44260b)? == b'B' {
        append(&mut result, message, 0x402266, services)?;
    }
    let names: &[u32] = if second {
        &[0x402254, 0x402192]
    } else {
        &[0x402254, 0x40223a, 0x402224]
    };
    for name in names {
        let handle = observe(
            &mut result,
            LoadRequest::Open {
                name: *name,
                mode: 0x40250e,
            },
            services,
        )?;
        long(&mut result, 0x4434a6, handle)?;
        if handle != 0 {
            let handle = result.memory.long(0x4434a6)?;
            observe(&mut result, LoadRequest::Close(handle), services)?;
            append(&mut result, message, 0x402252, services)?;
            break;
        }
    }
    append(&mut result, message, 0x402220, services)?;
    let code = if let Some(code) = cached_code {
        code
    } else {
        (result.memory.word(frame.wrapping_add(8))? as i16).wrapping_abs()
    };
    convert(&mut result, i32::from(code), numeric, services)?;
    append(&mut result, message, numeric, services)?;
    if !second {
        append(&mut result, message, 0x40221c, services)?;
        let value = (result.memory.word(0x442560)? as i16).wrapping_sub(9);
        convert(&mut result, i32::from(value), numeric, services)?;
        append(&mut result, message, numeric, services)?;
        append(&mut result, message, 0x40221a, services)?;
        version(&mut result, services)?;
        observe(&mut result, LoadRequest::PascalToC(0x4387a4), services)?;
        append(&mut result, message, 0x4387a4, services)?;
        observe(&mut result, LoadRequest::CToPascal(0x4387a4), services)?;
    }
    append(&mut result, message, 0x402216, services)?;
    let registration = result.memory.long(0x438598)?;
    long(&mut result, 0x43859c, registration)?;
    toolbox(&mut result, 0xa85f, None, vec![P(0x43859c), L(6)], services)?;
    let value = result.memory.long(0x43859c)? as i32;
    convert(&mut result, value, numeric, services)?;
    append(&mut result, message, numeric, services)?;
    if second {
        version(&mut result, services)?;
    }
    append(&mut result, message, 0x402216, services)?;
    if second {
        observe(&mut result, LoadRequest::PascalToC(0x4387a4), services)?;
        append(&mut result, message, 0x4387a4, services)?;
        observe(&mut result, LoadRequest::CToPascal(0x4387a4), services)?;
        append(&mut result, message, 0x40221a, services)?;
    }
    observe(&mut result, LoadRequest::PascalToC(0x438acc), services)?;
    append(&mut result, message, 0x438acc, services)?;
    observe(&mut result, LoadRequest::CToPascal(0x438acc), services)?;
    observe(&mut result, LoadRequest::CToPascal(message), services)?;
    if second {
        observe(
            &mut result,
            LoadRequest::GetIndexedString {
                resource: 5,
                index: code.wrapping_sub(1000),
                destination: 0x479ed8,
            },
            services,
        )?;
    }
    sound(&mut result, 10141, services)?;
    let argument = if second {
        0x479ed8
    } else if result.memory.byte(0x4386a9)? != 0 {
        0x4021aa
    } else {
        0x4021a9
    };
    toolbox(
        &mut result,
        0xa98b,
        None,
        vec![P(message), P(argument), P(0x4021a9), P(0x4021a9)],
        services,
    )?;
    let dialog = toolbox(
        &mut result,
        0xa97c,
        None,
        vec![W(168), L(0), L(u32::MAX)],
        services,
    )?;
    long(&mut result, frame.wrapping_sub(516), dialog)?;
    let dialog = result.memory.long(frame.wrapping_sub(516))?;
    toolbox(&mut result, 0xa873, None, vec![L(dialog)], services)?;
    let pattern = result.memory.long(0x4358ce)?;
    toolbox(&mut result, 0xaa0b, None, vec![L(pattern)], services)?;
    toolbox(&mut result, 0xa862, None, vec![L(69)], services)?;
    toolbox(&mut result, 0xa032, Some(0xffff), vec![], services)?;
    toolbox(&mut result, 0xa991, None, vec![L(0), P(0x4434b2)], services)?;
    observe(&mut result, LoadRequest::Runtime400028(0), services)?;
    Ok(result)
}

pub fn scratch(
    memory: &RenderingMemory,
    frame: u32,
    services: &(impl LoadServices + ?Sized),
) -> Result<LoadPlan, RuleError> {
    scratch_impl(memory, frame, false, services)
}
pub fn scratch2(
    memory: &RenderingMemory,
    frame: u32,
    services: &(impl LoadServices + ?Sized),
) -> Result<LoadPlan, RuleError> {
    scratch_impl(memory, frame, true, services)
}

fn c_string(memory: &RenderingMemory, address: u32) -> Result<Vec<u8>, RuleError> {
    let mut bytes = vec![];
    for i in 0..=memory.bytes.len() {
        let b = memory.byte(address.wrapping_add(i as u32))?;
        bytes.push(b);
        if b == 0 {
            return Ok(bytes);
        }
    }
    Err(RuleError::Invalid(
        "unterminated load-control C string".into(),
    ))
}
fn filename(
    result: &mut LoadPlan,
    name: u32,
    temporary: bool,
    capacity: usize,
    services: &(impl LoadServices + ?Sized),
) -> Result<(), RuleError> {
    let mut context = FileContext {
        scenario_directory: if temporary {
            vec![]
        } else {
            c_string(&result.memory, 0x44260c)?
        },
        variant: result.memory.byte(0x44274c)?,
        filename: (0..capacity)
            .map(|i| result.memory.byte(0x4389a4 + i as u32))
            .collect::<Result<Vec<_>, _>>()?,
        ..FileContext::default()
    };
    let number = services.filename_number(context.variant, &result.memory)?;
    let observation = FileObservation {
        request: FileRequest::NumberToString(i32::from(context.variant)),
        reply: FileReply::Pascal(number),
    };
    let name = c_string(&result.memory, name)?;
    let child = if temporary {
        files::get_temp_filename(&mut context, &name, &[observation])?
    } else {
        files::get_filename(&mut context, &name, &[observation])?
    };
    for effect in child.effects {
        if let FileEffect::Write {
            destination: FileBuffer::Filename,
            offset,
            bytes,
        } = &effect
        {
            write(result, 0x4389a4_u32.wrapping_add(*offset as u32), bytes)?;
        } else {
            result.effects.push(LoadEffect::File(effect));
        }
    }
    Ok(())
}
fn fatal(
    result: &mut LoadPlan,
    child_frame: u32,
    message: u16,
    services: &(impl LoadServices + ?Sized),
) -> Result<(), RuleError> {
    word(result, child_frame.wrapping_add(8), message)?;
    let child = scratch(&result.memory, child_frame, services)?;
    merge(result, child);
    Ok(())
}

/// loaddoor2, 00123c60. Existing header bytes replace the first eight read bytes.
pub fn load_door(
    memory: &RenderingMemory,
    frame: u32,
    limits: LoadLimits,
    services: &(impl LoadServices + ?Sized),
) -> Result<LoadPlan, RuleError> {
    let mut result = plan(memory, Some(0xffff));
    filename(
        &mut result,
        0x403778,
        false,
        limits.filename_capacity,
        services,
    )?;
    let handle = observe(
        &mut result,
        LoadRequest::Open {
            name: 0x4389a4,
            mode: 0x403782,
        },
        services,
    )?;
    if handle == 0 {
        fatal(&mut result, frame.wrapping_sub(56), 58, services)?;
        if result.flow != LoadFlow::Returned {
            return Ok(result);
        }
    }
    let offset = u32::from(result.memory.word(frame.wrapping_add(8))?) * 42;
    observe(
        &mut result,
        LoadRequest::Seek {
            handle,
            offset,
            origin: 0,
        },
        services,
    )?;
    observe(
        &mut result,
        LoadRequest::Read {
            handle,
            size: 42,
            count: 1,
            destination: frame.wrapping_sub(42),
        },
        services,
    )?;
    observe(&mut result, LoadRequest::Close(handle), services)?;
    for (source, local) in [
        (0x479eaf, 37),
        (0x479eb0, 36),
        (0x479eb1, 35),
        (0x479eae, 38),
    ] {
        let value = result.memory.byte(source)?;
        write(&mut result, frame.wrapping_sub(local), &[value])?;
    }
    let value = result.memory.long(0x479eaa)?;
    long(&mut result, frame.wrapping_sub(42), value)?;
    for offset in (0..40).step_by(4) {
        let value = result
            .memory
            .long(frame.wrapping_sub(42).wrapping_add(offset))?;
        long(&mut result, 0x479eaa + offset, value)?;
    }
    let value = result.memory.word(frame.wrapping_sub(2))?;
    word(&mut result, 0x479ed2, value)?;
    Ok(result)
}

/// loadextracode, 001383e6. Unsigned WORD record product and observed short read.
pub fn load_extra_code(
    memory: &RenderingMemory,
    frame: u32,
    limits: LoadLimits,
    services: &(impl LoadServices + ?Sized),
) -> Result<LoadPlan, RuleError> {
    let mut result = plan(memory, None);
    filename(
        &mut result,
        0x402270,
        false,
        limits.filename_capacity,
        services,
    )?;
    let handle = observe(
        &mut result,
        LoadRequest::Open {
            name: 0x4389a4,
            mode: 0x40250e,
        },
        services,
    )?;
    if handle == 0 {
        fatal(&mut result, frame.wrapping_sub(14), 118, services)?;
        if result.flow != LoadFlow::Returned {
            return Ok(result);
        }
    }
    let offset = u32::from(result.memory.word(frame.wrapping_add(8))?) * 10;
    observe(
        &mut result,
        LoadRequest::Seek {
            handle,
            offset,
            origin: 0,
        },
        services,
    )?;
    observe(
        &mut result,
        LoadRequest::Read {
            handle,
            size: 10,
            count: 1,
            destination: 0x44330a,
        },
        services,
    )?;
    observe(&mut result, LoadRequest::Close(handle), services)?;
    if let Some(LoadEffect::Service { observation, .. }) = result.effects.last() {
        result.native_return = Some(observation.d0);
    }
    Ok(result)
}

fn map_failure(
    result: &mut LoadPlan,
    frame: u32,
    limits: LoadLimits,
    services: &(impl LoadServices + ?Sized),
) -> Result<(), RuleError> {
    for (shift, pointer) in [(36, 0x403050), (48, 0x403016)] {
        let child_frame = frame.wrapping_sub(shift);
        long(result, child_frame.wrapping_add(8), pointer)?;
        for (off, value) in [(12, 50), (14, 50), (16, 0), (18, 5000)] {
            word(result, child_frame.wrapping_add(off), value)?;
        }
        let child = feedback::flashmessage(
            &result.memory,
            child_frame,
            pointer,
            50,
            50,
            0,
            5000,
            limits.maximum_polls,
            services,
        )?;
        merge_feedback(result, child);
    }
    fatal(result, frame.wrapping_sub(50), 412, services)
}

/// loadpixmap, 00131182. Signed BYTE cache, picture/GWorld and four file reads.
pub fn load_pixmap(
    memory: &RenderingMemory,
    frame: u32,
    limits: LoadLimits,
    services: &(impl LoadServices + ?Sized),
) -> Result<LoadPlan, RuleError> {
    let mut result = plan(memory, Some(0));
    let selection = result.memory.word(frame.wrapping_add(8))? as i16;
    if i16::from(result.memory.byte(0x402f43)? as i8) == selection {
        return Ok(result);
    }
    let scenario = selection >= 15;
    if scenario {
        filename(
            &mut result,
            0x403103,
            false,
            limits.filename_capacity,
            services,
        )?;
    } else {
        observe(
            &mut result,
            LoadRequest::CopyCString {
                source: 0x4030f6,
                destination: 0x4389a4,
            },
            services,
        )?;
    }
    let suffix = match selection {
        1 => Some(0x4030ec),
        2 => Some(0x4030de),
        3 => Some(0x4030d0),
        4 => Some(0x4030c4),
        5 => Some(0x4030b4),
        6 => Some(0x4030a6),
        7 => Some(0x403096),
        8 => Some(0x403086),
        9 => Some(0x403074),
        _ => None,
    };
    if let Some(suffix) = suffix {
        append(&mut result, 0x4389a4, suffix, services)?;
    }
    let picture = toolbox(
        &mut result,
        0xa9bc,
        None,
        vec![W(selection.wrapping_add(299) as u16)],
        services,
    )?;
    if picture == 0 {
        map_failure(&mut result, frame, limits, services)?;
        if result.flow != LoadFlow::Returned {
            return Ok(result);
        }
    } else {
        toolbox(
            &mut result,
            0xa8a7,
            None,
            vec![P(0x4434ba), L(0), L(0x01e00280)],
            services,
        )?;
        toolbox(
            &mut result,
            0xab1d,
            Some(0x80005),
            vec![P(0x4046da), P(0x4046de)],
            services,
        )?;
        let world = result.memory.long(0x4386c4)?;
        toolbox(
            &mut result,
            0xab1d,
            Some(0x80006),
            vec![L(world), L(0)],
            services,
        )?;
        let rect = result.memory.long(picture)?.wrapping_add(2);
        toolbox(
            &mut result,
            0xa8f6,
            None,
            vec![L(picture), P(rect)],
            services,
        )?;
        let world = result.memory.long(0x4046da)?;
        let device = result.memory.long(0x4046de)?;
        toolbox(
            &mut result,
            0xab1d,
            Some(0x80006),
            vec![L(world), L(device)],
            services,
        )?;
    }
    if scenario {
        append(&mut result, 0x4389a4, 0x403012, services)?;
    }
    let handle = observe(
        &mut result,
        LoadRequest::Open {
            name: 0x4389a4,
            mode: 0x403104,
        },
        services,
    )?;
    if handle == 0 {
        map_failure(&mut result, frame, limits, services)?;
        if result.flow != LoadFlow::Returned {
            return Ok(result);
        }
    } else {
        let index = (i32::from(selection) * 2) as u32;
        for (destination, size) in [
            (0x435508_u32.wrapping_add(index), 2),
            (0x435530_u32.wrapping_add(index), 2),
            (0x40857c, 60),
            (0x43167c, 16000),
        ] {
            observe(
                &mut result,
                LoadRequest::Read {
                    handle,
                    size,
                    count: 1,
                    destination,
                },
                services,
            )?;
        }
        observe(&mut result, LoadRequest::Close(handle), services)?;
    }
    write(&mut result, 0x402f43, &[selection as u8])?;
    Ok(result)
}

/// loadland, 0013107a. The native index is a wrapping LONG product.
pub fn load_land(
    memory: &RenderingMemory,
    frame: u32,
    limits: LoadLimits,
    services: &(impl LoadServices + ?Sized),
) -> Result<LoadPlan, RuleError> {
    let mut result = plan(memory, Some(0));
    filename(
        &mut result,
        0x403108,
        true,
        limits.filename_capacity,
        services,
    )?;
    let handle = observe(
        &mut result,
        LoadRequest::Open {
            name: 0x4389a4,
            mode: 0x403104,
        },
        services,
    )?;
    if handle == 0 {
        fatal(&mut result, frame.wrapping_sub(14), 107, services)?;
        if result.flow != LoadFlow::Returned {
            return Ok(result);
        }
    }
    let offset = result
        .memory
        .long(frame.wrapping_add(8))?
        .wrapping_mul(0x12fb0);
    observe(
        &mut result,
        LoadRequest::Seek {
            handle,
            offset,
            origin: 0,
        },
        services,
    )?;
    for (destination, size) in [
        (0x478e42, 0x1068),
        (0x443086, 0x284),
        (0x4358e6, 0x1fa4),
        (0x4085b8, 0x3f48),
        (0x40c500, 0x3f48),
        (0x414390, 0x7e90),
    ] {
        observe(
            &mut result,
            LoadRequest::Read {
                handle,
                size,
                count: 1,
                destination,
            },
            services,
        )?;
    }
    observe(&mut result, LoadRequest::Close(handle), services)?;
    let selection = i16::from(result.memory.byte(0x44328e)? as i8);
    let child_frame = frame.wrapping_sub(124);
    word(&mut result, child_frame.wrapping_add(8), selection as u16)?;
    let child = load_pixmap(&result.memory, child_frame, limits, services)?;
    merge(&mut result, child);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recovered_audio::{AudioContext, AudioObservation};
    use crate::recovered_feedback::{
        FeedbackArg, FeedbackMemory, FeedbackObservation, FeedbackRequest,
    };
    use crate::recovered_gamma::{GammaError, GammaObservation, GammaRequest, GammaServices};
    use crate::recovered_gamma_control::GammaControlRequest;
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;

    struct Services {
        requests: RefCell<Vec<LoadRequest>>,
        opens: RefCell<VecDeque<u32>>,
        tick: Cell<u32>,
        picture: u32,
        failed_read: bool,
        changed_argument: Option<(u32, u16)>,
    }
    impl Services {
        fn new(opens: &[u32]) -> Self {
            Self {
                requests: RefCell::new(vec![]),
                opens: RefCell::new(opens.iter().copied().collect()),
                tick: Cell::new(0),
                picture: 0x70000,
                failed_read: false,
                changed_argument: None,
            }
        }
    }
    fn copy_string(
        memory: &mut RenderingMemory,
        source: u32,
        destination: u32,
        append: bool,
    ) -> Result<(), RuleError> {
        let bytes = c_string(memory, source)?;
        let offset = if append {
            c_string(memory, destination)?.len() - 1
        } else {
            0
        };
        memory.insert_region(destination + offset as u32, &bytes);
        Ok(())
    }
    fn p_to_c(memory: &mut RenderingMemory, address: u32) -> Result<(), RuleError> {
        let length = memory.byte(address)? as usize;
        let bytes = (0..length)
            .map(|i| memory.byte(address + 1 + i as u32))
            .collect::<Result<Vec<_>, _>>()?;
        memory.insert_region(address, &bytes);
        memory.insert_region(address + length as u32, &[0]);
        Ok(())
    }
    fn c_to_p(memory: &mut RenderingMemory, address: u32) -> Result<(), RuleError> {
        let bytes = c_string(memory, address)?;
        let string = PascalString::from_payload(&bytes[..bytes.len() - 1])?;
        memory.insert_region(address, string.raw());
        Ok(())
    }
    impl GammaServices for Services {
        fn observe(
            &self,
            _: &GammaRequest,
            _: &GammaMemory,
        ) -> Result<GammaObservation, GammaError> {
            Err(GammaError::MissingService("unused disabled fade".into()))
        }
    }
    impl GammaControlServices for Services {
        fn observe_control(
            &self,
            _: &GammaControlRequest,
            _: &GammaMemory,
        ) -> Result<GammaObservation, GammaError> {
            Err(GammaError::MissingService("unused disabled fade".into()))
        }
    }
    impl FeedbackServices for Services {
        fn observe(
            &self,
            request: &FeedbackRequest,
            memory: &FeedbackMemory,
        ) -> Result<FeedbackObservation, RuleError> {
            let mut memory = memory.clone();
            let mut value = 0;
            match request {
                FeedbackRequest::TickCount => {
                    value = self.tick.get();
                    self.tick.set(value.wrapping_add(100));
                }
                FeedbackRequest::BlockMove {
                    source,
                    destination,
                    count,
                } => {
                    let bytes = (0..*count)
                        .map(|i| memory.byte(source.wrapping_add(i)))
                        .collect::<Result<Vec<_>, _>>()?;
                    memory.insert_region(*destination, &bytes);
                }
                FeedbackRequest::CToPascal(address) => c_to_p(&mut memory, *address)?,
                FeedbackRequest::Toolbox {
                    opcode: 0xa874,
                    arguments,
                } => {
                    if let [FeedbackArg::Pointer(pointer)] = arguments.as_slice() {
                        memory.insert_region(*pointer, &0x76543210_u32.to_be_bytes());
                    }
                }
                FeedbackRequest::Toolbox { opcode: 0xa97c, .. } => value = 0x12340000,
                FeedbackRequest::Toolbox { opcode: 0xa860, .. } => {
                    memory.insert_region(0x44720a, &[0, 3])
                }
                _ => (),
            }
            Ok(FeedbackObservation {
                memory,
                value,
                d0: 0x33445566,
            })
        }
        fn sound_observations(
            &self,
            _: i16,
            _: &AudioContext,
            _: &FeedbackMemory,
        ) -> Result<Vec<AudioObservation>, RuleError> {
            Err(RuleError::Invalid("unused muted audio".into()))
        }
    }
    impl LoadServices for Services {
        fn observe_load(
            &self,
            request: &LoadRequest,
            memory: &RenderingMemory,
        ) -> Result<LoadObservation, RuleError> {
            self.requests.borrow_mut().push(request.clone());
            let mut memory = memory.clone();
            let mut value = 0;
            let mut flow = LoadFlow::Returned;
            match request {
                LoadRequest::CopyCString {
                    source,
                    destination,
                } => copy_string(&mut memory, *source, *destination, false)?,
                LoadRequest::AppendCString {
                    source,
                    destination,
                } => copy_string(&mut memory, *source, *destination, true)?,
                LoadRequest::PascalToC(address) => p_to_c(&mut memory, *address)?,
                LoadRequest::CToPascal(address) => c_to_p(&mut memory, *address)?,
                LoadRequest::NumberToString { value, destination } => memory.insert_region(
                    *destination,
                    PascalString::from_payload(value.to_string().as_bytes())?.raw(),
                ),
                LoadRequest::GetIndexedString { destination, .. } => {
                    memory.insert_region(*destination, &[4, b'o', b'o', b'p', b's'])
                }
                LoadRequest::Open { .. } => {
                    value = self.opens.borrow_mut().pop_front().unwrap_or(9);
                    if let Some((address, argument)) = self.changed_argument {
                        memory.insert_region(address, &argument.to_be_bytes());
                    }
                }
                LoadRequest::Read { destination, .. } => {
                    if self.failed_read {
                        return Err(RuleError::Invalid(
                            "missing fixture read observation".into(),
                        ));
                    }
                    memory.insert_region(*destination, &[0xaa, 0xbb]);
                }
                LoadRequest::Runtime400028(status) => flow = LoadFlow::Terminated(*status),
                LoadRequest::Toolbox { opcode: 0xa85d, .. } => value = 1,
                LoadRequest::Toolbox { opcode: 0xa85f, .. } => {
                    let old = memory.byte(0x43859c)?;
                    memory.insert_region(0x43859c, &[old & !2]);
                }
                LoadRequest::Toolbox { opcode: 0xa97c, .. } => value = 0x11112222,
                LoadRequest::Toolbox { opcode: 0xa9bc, .. } => value = self.picture,
                LoadRequest::Toolbox { opcode: 0xa8a7, .. } => {
                    memory.insert_region(0x4434ba, &[0, 0, 0, 0, 1, 224, 2, 128])
                }
                LoadRequest::Toolbox {
                    opcode: 0xab1d,
                    selector: Some(0x80005),
                    ..
                } => {
                    memory.insert_region(0x4046da, &0xaaaabbbb_u32.to_be_bytes());
                    memory.insert_region(0x4046de, &0xccccdddd_u32.to_be_bytes());
                }
                _ => (),
            }
            Ok(LoadObservation {
                memory,
                value,
                d0: 0xdeadbeef,
                flow,
            })
        }
        fn version_inputs(
            &self,
            _: i16,
            _: i16,
            _: &RenderingMemory,
        ) -> Result<VersionInputs, RuleError> {
            let mut resource = vec![0; 6];
            resource.extend_from_slice(&[5, b'1', b'.', b'3', b'.', b'2']);
            Ok(VersionInputs {
                current_resource_file: 7,
                resource,
            })
        }
        fn filename_number(
            &self,
            variant: u8,
            _: &RenderingMemory,
        ) -> Result<PascalString, RuleError> {
            PascalString::from_payload(variant.to_string().as_bytes())
        }
    }
    fn fixture(frame: u32) -> RenderingMemory {
        let mut memory = RenderingMemory::default();
        for (address, size) in [
            (frame - 4096, 8192),
            (0x402000, 8192),
            (0x438500, 2048),
            (0x439500, 256),
            (0x442600, 512),
            (0x479e00, 1024),
            (0x443300, 768),
            (0x404600, 256),
            (0x4434ba, 16),
            (0x402f40, 4),
            (0x435500, 1024),
            (0x43167c, 16000),
            (0x40857c, 128),
            (0x44720a, 24),
            (0x41d1c0, 4),
        ] {
            memory.insert_region(address, &vec![0xcc; size]);
        }
        memory.insert_region(0x438587, &[1, 0]);
        memory.insert_region(0x439570, &[0, 0, 0, 0]);
        memory.insert_region(0x438598, &0x02001234_u32.to_be_bytes());
        memory.insert_region(0x44260b, b"BScenario\0");
        memory.insert_region(0x44274c, &[3]);
        memory.insert_region(0x4389a4, &[0; 128]);
        memory.insert_region(0x438acc, &[3, b'a', b'p', b'p']);
        for (address, text) in [
            (0x40250d, b"\0".as_slice()),
            (0x402268, b"U\0"),
            (0x402266, b"B\0"),
            (0x402254, b"first\0"),
            (0x40223a, b"second\0"),
            (0x402224, b"third\0"),
            (0x402192, b"copy\0"),
            (0x402252, b"E\0"),
            (0x402220, b"#\0"),
            (0x40221c, b"-\0"),
            (0x40221a, b"+\0"),
            (0x402216, b"/\0"),
            (0x403778, b"Door\0"),
            (0x402270, b"Extra\0"),
            (0x403108, b"Land\0"),
            (0x403103, b"Map\0"),
            (0x4030f6, b"Stock\0"),
            (0x4030ec, b"One\0"),
            (0x403012, b".dat\0"),
            (0x403050, b"Missing map\0"),
            (0x403016, b"Update map\0"),
        ] {
            memory.insert_region(address, text);
        }
        memory.insert_region(0x4386b4, &[0; 4]);
        memory.insert_region(0x43879c, &[0; 4]);
        memory.insert_region(0x479fd8, &[0; 4]);
        memory.insert_region(0x4427b4, &[0]);
        memory.insert_region(0x4395a2, &[0; 4]);
        memory.insert_region(0x41d1c0, &[0; 4]);
        memory.insert_region(0x442560, &[0, 8]);
        memory.insert_region(0x402f43, &[0xff]);
        memory.insert_region(0x70000, &0x71000_u32.to_be_bytes());
        memory
    }
    fn limits() -> LoadLimits {
        LoadLimits {
            filename_capacity: 128,
            maximum_polls: 8,
        }
    }

    #[test]
    fn scratch_preserves_fallback_order_late_argument_and_explicit_termination() {
        let frame = 0x90000;
        let mut input = fixture(frame);
        input.insert_region(frame + 8, &(-40_i16).to_be_bytes());
        let mut services = Services::new(&[0, 0, 9]);
        services.changed_argument = Some((frame + 8, 42));
        let result = scratch(&input, frame, &services).unwrap();
        assert_eq!(result.flow, LoadFlow::Terminated(0));
        assert_eq!(result.native_return, None);
        let requests = services.requests.borrow();
        assert_eq!(
            requests
                .iter()
                .filter(|x| matches!(x, LoadRequest::Open { .. }))
                .count(),
            3
        );
        assert!(requests.contains(&LoadRequest::NumberToString {
            value: 42,
            destination: frame - 512
        }));
        assert_eq!(
            (
                result.memory.byte(0x438598).unwrap(),
                result.memory.byte(0x43859c).unwrap()
            ),
            (2, 0)
        );
        assert!(matches!(
            requests.last(),
            Some(LoadRequest::Runtime400028(0))
        ));
        assert_eq!(input.word(frame + 8).unwrap() as i16, -40);
        drop(requests);
        let services = Services::new(&[0, 9]);
        let result = scratch2(&input, frame, &services).unwrap();
        assert_eq!(result.flow, LoadFlow::Terminated(0));
        assert!(
            services
                .requests
                .borrow()
                .contains(&LoadRequest::NumberToString {
                    value: 1040,
                    destination: frame - 512
                })
        );
        assert!(
            services
                .requests
                .borrow()
                .contains(&LoadRequest::GetIndexedString {
                    resource: 5,
                    index: 40,
                    destination: 0x479ed8
                })
        );
    }
    #[test]
    fn door_retains_existing_header_and_extra_uses_post_service_word_index() {
        let frame = 0x90000;
        let mut input = fixture(frame);
        input.insert_region(frame + 8, &0xffff_u16.to_be_bytes());
        let prior: Vec<u8> = (0..42).map(|x| x + 10).collect();
        input.insert_region(0x479eaa, &prior);
        let result = load_door(&input, frame, limits(), &Services::new(&[9])).unwrap();
        assert_eq!(
            (0..8)
                .map(|x| result.memory.byte(0x479eaa + x).unwrap())
                .collect::<Vec<_>>(),
            prior[..8]
        );
        assert_eq!(result.memory.byte(0x479eb2).unwrap(), 0xcc);
        assert_eq!(result.memory.word(0x479ed2).unwrap(), 0xcccc);
        assert_eq!(result.native_return, Some(0xffff));
        let mut services = Services::new(&[9]);
        services.changed_argument = Some((frame + 8, 65534));
        let result = load_extra_code(&input, frame, limits(), &services).unwrap();
        assert_eq!(result.native_return, Some(0xdeadbeef));
        assert!(services.requests.borrow().contains(&LoadRequest::Seek {
            handle: 9,
            offset: 655340,
            origin: 0
        }));
        assert_eq!(result.memory.word(0x44330a).unwrap(), 0xaabb);
        assert_eq!(result.memory.byte(0x44330c).unwrap(), 0xcc);
        let result = load_extra_code(&input, frame, limits(), &Services::new(&[0, 9])).unwrap();
        assert_eq!(result.flow, LoadFlow::Terminated(0));
    }
    #[test]
    fn land_wraps_long_seek_and_preserves_six_reads_and_cached_pixmap() {
        let frame = 0x90000;
        let mut input = fixture(frame);
        input.insert_region(frame + 8, &u32::MAX.to_be_bytes());
        input.insert_region(0x44328e, &[0xff]);
        let services = Services::new(&[9]);
        let result = load_land(&input, frame, limits(), &services).unwrap();
        assert_eq!(result.native_return, Some(0));
        assert!(services.requests.borrow().contains(&LoadRequest::Seek {
            handle: 9,
            offset: u32::MAX.wrapping_mul(0x12fb0),
            origin: 0
        }));
        let reads: Vec<_> = services
            .requests
            .borrow()
            .iter()
            .filter_map(|r| {
                if let LoadRequest::Read {
                    destination, size, ..
                } = r
                {
                    Some((*destination, *size))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            reads,
            vec![
                (0x478e42, 0x1068),
                (0x443086, 0x284),
                (0x4358e6, 0x1fa4),
                (0x4085b8, 0x3f48),
                (0x40c500, 0x3f48),
                (0x414390, 0x7e90)
            ]
        );
        assert_eq!(result.memory.word(frame - 116).unwrap(), 0xffff);
        let before = input.clone();
        let mut missing = Services::new(&[9]);
        missing.failed_read = true;
        assert!(load_land(&input, frame, limits(), &missing).is_err());
        assert_eq!(input, before);
    }
    #[test]
    fn pixmap_keeps_resource_world_order_negative_index_and_missing_map_exit() {
        let frame = 0x90000;
        let mut input = fixture(frame);
        input.insert_region(frame + 8, &0xfffe_u16.to_be_bytes());
        let services = Services::new(&[9]);
        let result = load_pixmap(&input, frame, limits(), &services).unwrap();
        assert_eq!(result.memory.byte(0x402f43).unwrap(), 0xfe);
        assert!(services.requests.borrow().contains(&LoadRequest::Toolbox {
            opcode: 0xa9bc,
            selector: None,
            arguments: vec![W(297)]
        }));
        assert!(services.requests.borrow().contains(&LoadRequest::Toolbox {
            opcode: 0xa8f6,
            selector: None,
            arguments: vec![L(0x70000), P(0x71002)]
        }));
        assert!(services.requests.borrow().contains(&LoadRequest::Read {
            handle: 9,
            size: 2,
            count: 1,
            destination: 0x435504
        }));
        assert!(services.requests.borrow().contains(&LoadRequest::Read {
            handle: 9,
            size: 2,
            count: 1,
            destination: 0x43552c
        }));
        let cached = load_pixmap(&result.memory, frame, limits(), &Services::new(&[])).unwrap();
        assert!(cached.effects.is_empty());
        input.insert_region(frame + 8, &[0, 1]);
        let mut missing = Services::new(&[9]);
        missing.picture = 0;
        let result = load_pixmap(&input, frame, limits(), &missing).unwrap();
        assert_eq!(result.flow, LoadFlow::Terminated(0));
        assert_eq!(result.memory.byte(0x402f43).unwrap(), 0xff);
        assert!(
            !missing
                .requests
                .borrow()
                .iter()
                .any(|x| matches!(x, LoadRequest::Read { .. }))
        );
        let frames: Vec<_> = result
            .effects
            .iter()
            .filter_map(|effect| {
                if let LoadEffect::Feedback(FeedbackEffect::Service {
                    request:
                        FeedbackRequest::BlockMove {
                            source,
                            destination,
                            count,
                        },
                    ..
                }) = effect
                {
                    Some((*source, *destination, *count))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            frames,
            vec![
                (0x403050, frame - 36 - 272, 255),
                (0x403016, frame - 48 - 272, 255)
            ]
        );
    }
}
