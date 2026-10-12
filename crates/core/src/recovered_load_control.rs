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
