//! CODE 1 preference resource and indexed-string control flow.

use std::collections::BTreeMap;

use crate::recovered_files::{FileReply, FileRequest};
use crate::recovered_text::PascalString;
use crate::{RandomState, RuleError};
use centurions_content::span;

pub const PREFERENCE_SIZE: usize = 0x16a;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandleMemory {
    /// Accessible bytes can include the old tail after a native resize.
    pub bytes: Vec<u8>,
    pub logical_size: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreferenceContext {
    pub globals: BTreeMap<u32, Vec<u8>>,
    pub handles: BTreeMap<u32, HandleMemory>,
    pub random: RandomState,
}

impl PreferenceContext {
    pub fn new(random: RandomState) -> Self {
        Self {
            globals: BTreeMap::new(),
            handles: BTreeMap::new(),
            random,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreferenceRequest {
    CurrentResourceFile,
    UseResourceFile(i16),
    FindFolder {
        volume: i16,
        kind: u32,
        create: bool,
    },
    MakeFileSpecification {
        volume: i16,
        directory: i32,
        name: PascalString,
    },
    OpenResourceFile {
        specification: Vec<u8>,
        permission: i8,
    },
    CreateResourceFile {
        specification: Vec<u8>,
        creator: u32,
        kind: u32,
        script: i16,
    },
    GetResource {
        kind: u32,
        id: i16,
        current_only: bool,
    },
    ResourceInfo(u32),
    HomeResourceFile(u32),
    DetachResource(u32),
    AddResource {
        handle: u32,
        kind: u32,
        id: i16,
        name: PascalString,
    },
    SetResourceAttributes {
        handle: u32,
        attributes: i16,
    },
    ResourceChanged(u32),
    WriteResource(u32),
    ReleaseResource(u32),
    RemoveResource(u32),
    CloseResourceFile(i16),
    NewHandleClear(u32),
    LockHandle(u32),
    UnlockHandle(u32),
    NoPurgeHandle(u32),
    PurgeHandle(u32),
    HandleSize(u32),
    ResizeHandle {
        handle: u32,
        size: i32,
    },
    ResourceError,
    File(FileRequest),
    WriteFile {
        handle: u32,
        size: u32,
        count: u32,
        bytes: Vec<u8>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreferenceReply {
    Word(i16),
    Long(i32),
    Folder {
        volume: i16,
        directory: i32,
        error: i16,
    },
    FileSpecification {
        bytes: Vec<u8>,
        error: i16,
    },
    Resource {
        handle: u32,
        memory: Option<HandleMemory>,
    },
    ResourceInfo {
        kind: u32,
        id: i16,
        name: PascalString,
    },
    Handle(u32),
    Resized {
        memory: HandleMemory,
        error: i16,
    },
    File(FileReply),
    Ignored,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreferenceObservation {
    pub request: PreferenceRequest,
    pub reply: PreferenceReply,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreferenceEffect {
    Service(PreferenceObservation),
    GlobalWrite {
        address: u32,
        bytes: Vec<u8>,
    },
    HandleWrite {
        handle: u32,
        offset: usize,
        bytes: Vec<u8>,
    },
    HandleMove {
        handle: u32,
        source: usize,
        destination: usize,
        bytes: Vec<u8>,
    },
    Random {
        bound: i16,
        result: i16,
        before: RandomState,
        after: RandomState,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreferencePlan {
    pub effects: Vec<PreferenceEffect>,
    pub consumed_observations: usize,
    /// All five bodies return a WORD result; savepref/getpref return zero.
    pub result: i16,
}

struct Execution<'a> {
    observations: &'a [PreferenceObservation],
    consumed: usize,
    effects: Vec<PreferenceEffect>,
}

impl Execution<'_> {
    fn request(&mut self, request: PreferenceRequest) -> Result<PreferenceReply, RuleError> {
        let observation = self.observations.get(self.consumed).ok_or(RuleError::Unsupported {
            action: "preferences.service",
            dependency: "the next Resource Manager, Memory Manager, File Manager or stdio result has not been supplied",
        })?;
        if observation.request != request {
            return Err(RuleError::Invalid(format!(
                "preference observation {} does not match request {request:?}",
                self.consumed
            )));
        }
        self.consumed += 1;
        self.effects
            .push(PreferenceEffect::Service(observation.clone()));
        Ok(observation.reply.clone())
    }

    fn ignored(&mut self, request: PreferenceRequest) -> Result<(), RuleError> {
        match self.request(request)? {
            PreferenceReply::Ignored | PreferenceReply::Word(_) | PreferenceReply::Long(_) => {
                Ok(())
            }
            _ => Err(RuleError::Invalid(
                "unexpected discarded preference service result".into(),
            )),
        }
    }

    fn word(&mut self, request: PreferenceRequest) -> Result<i16, RuleError> {
        match self.request(request)? {
            PreferenceReply::Word(value) => Ok(value),
            _ => Err(RuleError::Invalid(
                "missing WORD preference service result".into(),
            )),
        }
    }

    fn resource(
        &mut self,
        context: &mut PreferenceContext,
        kind: u32,
        id: i16,
        current_only: bool,
    ) -> Result<u32, RuleError> {
        match self.request(PreferenceRequest::GetResource {
            kind,
            id,
            current_only,
        })? {
            PreferenceReply::Resource { handle, memory } => {
                if let Some(memory) = memory {
                    if handle == 0 {
                        return Err(RuleError::Invalid(
                            "null resource handle has supplied memory".into(),
                        ));
                    }
                    context.handles.insert(handle, memory);
                } else {
                    context.handles.remove(&handle);
                }
                Ok(handle)
            }
            _ => Err(RuleError::Invalid(
                "missing resource handle observation".into(),
            )),
        }
    }

    fn info(&mut self, handle: u32) -> Result<(u32, i16, PascalString), RuleError> {
        match self.request(PreferenceRequest::ResourceInfo(handle))? {
            PreferenceReply::ResourceInfo { kind, id, name } => Ok((kind, id, name)),
            _ => Err(RuleError::Invalid(
                "missing resource identity observation".into(),
            )),
        }
    }

    fn file(&mut self, request: FileRequest) -> Result<FileReply, RuleError> {
        match self.request(PreferenceRequest::File(request))? {
            PreferenceReply::File(reply) => Ok(reply),
            _ => Err(RuleError::Invalid(
                "missing stdio/application service observation".into(),
            )),
        }
    }
}

fn execute(
    context: &mut PreferenceContext,
    observations: &[PreferenceObservation],
    body: impl FnOnce(&mut PreferenceContext, &mut Execution<'_>) -> Result<i16, RuleError>,
) -> Result<PreferencePlan, RuleError> {
    let mut candidate = context.clone();
    let mut execution = Execution {
        observations,
        consumed: 0,
        effects: vec![],
    };
    let result = body(&mut candidate, &mut execution)?;
    *context = candidate;
    Ok(PreferencePlan {
        effects: execution.effects,
        consumed_observations: execution.consumed,
        result,
    })
}

fn global(context: &PreferenceContext, address: u32, length: usize) -> Result<Vec<u8>, RuleError> {
    let bytes = context
        .globals
        .get(&address)
        .ok_or(RuleError::Unsupported {
            action: "preferences.global",
            dependency: "the native global bytes have not been supplied",
        })?;
    Ok(span(bytes, 0, length)?.to_vec())
}

fn write_global(
    context: &mut PreferenceContext,
    execution: &mut Execution<'_>,
    address: u32,
    bytes: &[u8],
) {
    let destination = context.globals.entry(address).or_default();
    if destination.len() < bytes.len() {
        destination.resize(bytes.len(), 0);
    }
    destination[..bytes.len()].copy_from_slice(bytes);
    execution.effects.push(PreferenceEffect::GlobalWrite {
        address,
        bytes: bytes.to_vec(),
    });
}

fn read_handle(
    context: &PreferenceContext,
    handle: u32,
    offset: usize,
    length: usize,
) -> Result<Vec<u8>, RuleError> {
    let memory = context.handles.get(&handle).ok_or(RuleError::Unsupported {
        action: "preferences.handle",
        dependency: "the source dereferences a null or unobserved handle data pointer",
    })?;
    Ok(span(&memory.bytes, offset, length)?.to_vec())
}

fn write_handle(
    context: &mut PreferenceContext,
    execution: &mut Execution<'_>,
    handle: u32,
    offset: usize,
    bytes: &[u8],
) -> Result<(), RuleError> {
    read_handle(context, handle, offset, bytes.len())?;
    context
        .handles
        .get_mut(&handle)
        .expect("checked handle")
        .bytes[offset..offset + bytes.len()]
        .copy_from_slice(bytes);
    execution.effects.push(PreferenceEffect::HandleWrite {
        handle,
        offset,
        bytes: bytes.to_vec(),
    });
    Ok(())
}

fn word_global(context: &PreferenceContext, address: u32) -> Result<i16, RuleError> {
    Ok(i16::from_be_bytes(
        global(context, address, 2)?.try_into().expect("two bytes"),
    ))
}

fn long_global(context: &PreferenceContext, address: u32) -> Result<i32, RuleError> {
    Ok(i32::from_be_bytes(
        global(context, address, 4)?.try_into().expect("four bytes"),
    ))
}

fn pascal_global(context: &PreferenceContext, address: u32) -> Result<PascalString, RuleError> {
    let length = usize::from(global(context, address, 1)?[0]);
    PascalString::from_raw(&global(context, address, length + 1)?)
}

fn specification(bytes: &[u8]) -> Result<(), RuleError> {
    if bytes.len() != 70 {
        return Err(RuleError::Invalid(
            "source FSSpec must supply exactly 70 raw bytes".into(),
        ));
    }
    Ok(())
}

const FIELDS: [(usize, u32, usize); 19] = [
    (0x00, 0x439568, 2),
    (0x02, 0x43956a, 2),
    (0x04, 0x43956c, 4),
    (0x08, 0x439570, 4),
    (0x0c, 0x439574, 2),
    (0x0e, 0x439578, 2),
    (0x10, 0x43957a, 2),
    (0x12, 0x43957c, 2),
    (0x14, 0x43957e, 2),
    (0x16, 0x439580, 2),
    (0x18, 0x439582, 2),
    (0x1a, 0x439584, 2),
    (0x1c, 0x439586, 2),
    (0x1e, 0x439588, 2),
    (0x20, 0x43958a, 2),
    (0x22, 0x43958c, 2),
    (0x24, 0x43958e, 2),
    (0x26, 0x439590, 2),
    (0x28, 0x439592, 2),
];

/// CODE 1 0x14c622. Nonzero modes observe the caller's preserved D4 WORD.
pub fn open_preferences(
    context: &mut PreferenceContext,
    mode: i16,
    incoming_d4: i16,
    incoming_specification: Option<&[u8]>,
    observations: &[PreferenceObservation],
) -> Result<PreferencePlan, RuleError> {
    if let Some(bytes) = incoming_specification {
        specification(bytes)?;
    }
    execute(context, observations, |context, execution| {
        open_body(
            context,
            execution,
            mode,
            incoming_d4,
            incoming_specification,
            0,
        )
    })
}

fn open_body(
    context: &mut PreferenceContext,
    execution: &mut Execution<'_>,
    mode: i16,
    incoming_d4: i16,
    incoming_specification: Option<&[u8]>,
    depth: usize,
) -> Result<i16, RuleError> {
    if depth >= 64 {
        return Err(RuleError::Invalid(
            "preference creation recursion exceeds the supplied execution domain".into(),
        ));
    }
    let (volume, directory) = match execution.request(PreferenceRequest::FindFolder {
        volume: i16::MIN,
        kind: 0x70726566,
        create: false,
    })? {
        PreferenceReply::Folder {
            volume, directory, ..
        } => (volume, directory),
        _ => return Err(RuleError::Invalid("missing FindFolder output bytes".into())),
    };
    let mut result = incoming_d4;
    let mut spec = incoming_specification.map(<[u8]>::to_vec);
    loop {
        if mode == 0 {
            let bytes = match execution.request(PreferenceRequest::MakeFileSpecification {
                volume,
                directory,
                name: PascalString::from_payload(b"New Centurions Prefs")?,
            })? {
                PreferenceReply::FileSpecification { bytes, .. } => bytes,
                _ => {
                    return Err(RuleError::Invalid(
                        "missing FSMakeFSSpec output bytes".into(),
                    ));
                }
            };
            specification(&bytes)?;
            result = execution.word(PreferenceRequest::OpenResourceFile {
                specification: bytes.clone(),
                permission: 0,
            })?;
            spec = Some(bytes);
        }
        if result != -1 || (mode != 0 && mode != 3) {
            break;
        }
        let raw = spec.as_ref().ok_or(RuleError::Unsupported {
            action: "preferences.open",
            dependency: "mode 3 with incoming D4=-1 copies uninitialized FSSpec stack bytes; supply those bytes explicitly",
        })?;
        create_body(context, execution, raw, mode, depth + 1)?;
    }
    Ok(if mode == 0 { result } else { 0 })
}

/// CODE 1 0x14c01c: builds a new PRFN handle and replaces the old resource.
pub fn save_preferences(
    context: &mut PreferenceContext,
    observations: &[PreferenceObservation],
) -> Result<PreferencePlan, RuleError> {
    execute(context, observations, |context, execution| {
        save_body(context, execution, 0)
    })
}

fn save_body(
    context: &mut PreferenceContext,
    execution: &mut Execution<'_>,
    depth: usize,
) -> Result<i16, RuleError> {
    let old_file = execution.word(PreferenceRequest::CurrentResourceFile)?;
    let file = open_body(context, execution, 0, 0, None, depth)?;
    execution.ignored(PreferenceRequest::UseResourceFile(file))?;
    let handle =
        match execution.request(PreferenceRequest::NewHandleClear(PREFERENCE_SIZE as u32))? {
            PreferenceReply::Handle(handle) if handle != 0 => handle,
            PreferenceReply::Handle(_) => {
                return Err(RuleError::Unsupported {
                    action: "preferences.save",
                    dependency: "the source dereferences a failed NewHandleClear allocation",
                });
            }
            _ => return Err(RuleError::Invalid("missing new preference handle".into())),
        };
    context.handles.insert(
        handle,
        HandleMemory {
            bytes: vec![0; PREFERENCE_SIZE],
            logical_size: PREFERENCE_SIZE as u32,
        },
    );
    execution.ignored(PreferenceRequest::LockHandle(handle))?;
    for (offset, address, length) in FIELDS {
        let bytes = global(context, address, length)?;
        write_handle(context, execution, handle, offset, &bytes)?;
    }
    let signed_byte = i16::from(global(context, 0x438589, 1)?[0] as i8);
    write_handle(context, execution, handle, 0x2a, &signed_byte.to_be_bytes())?;
    let bytes = global(context, 0x402f44, 2)?;
    write_handle(context, execution, handle, 0x2e, &bytes)?;
    let bytes = global(context, 0x442560, 2)?;
    write_handle(context, execution, handle, 0x2c, &bytes)?;
    for index in 0..20_u32 {
        let byte = global(context, 0x44270e + index, 1)?[0] as i8;
        write_handle(
            context,
            execution,
            handle,
            0x30 + index as usize * 2,
            &i16::from(byte).to_be_bytes(),
        )?;
    }
    let seed = global(context, 0x438598, 4)?;
    write_global(context, execution, 0x43859c, &seed);
    let mut flagged = seed.clone();
    flagged[0] |= 0x02;
    write_global(context, execution, 0x43859c, &flagged);
    write_handle(context, execution, handle, 0x166, &seed)?;
    for (offset, address) in [(0x58, 0x439596), (0x5a, 0x4046ea)] {
        let bytes = global(context, address, 2)?;
        write_handle(context, execution, handle, offset, &bytes)?;
    }
    for offset in [0x5c, 0x5e, 0x60, 0x62, 0x64] {
        write_handle(context, execution, handle, offset, &[0, 0])?;
    }
    let name = pascal_global(context, 0x4385a0)?;
    write_handle(context, execution, handle, 0x66, name.raw())?;
    let old_handle = execution.resource(context, 0x5052464e, 128, false)?;
    let (kind, id, name) = execution.info(old_handle)?;
    // The native routine leaves a two-byte result slot on the stack here.
    execution.word(PreferenceRequest::HomeResourceFile(old_handle))?;
    execution.ignored(PreferenceRequest::RemoveResource(old_handle))?;
    execution.ignored(PreferenceRequest::AddResource {
        handle,
        kind,
        id,
        name,
    })?;
    execution.ignored(PreferenceRequest::WriteResource(handle))?;
    execution.ignored(PreferenceRequest::UnlockHandle(handle))?;
    execution.ignored(PreferenceRequest::ReleaseResource(handle))?;
    execution.ignored(PreferenceRequest::CloseResourceFile(file))?;
    execution.ignored(PreferenceRequest::UseResourceFile(old_file))?;
    Ok(0)
}

/// CODE 1 0x14bca4. Seed bookkeeping is preserved; no registration gate is exposed.
pub fn get_preferences(
    context: &mut PreferenceContext,
    observations: &[PreferenceObservation],
) -> Result<PreferencePlan, RuleError> {
    execute(context, observations, |context, execution| {
        get_body(context, execution, 0)
    })
}

fn get_body(
    context: &mut PreferenceContext,
    execution: &mut Execution<'_>,
    depth: usize,
) -> Result<i16, RuleError> {
    let old_file = execution.word(PreferenceRequest::CurrentResourceFile)?;
    let file = open_body(context, execution, 0, 0, None, depth)?;
    execution.ignored(PreferenceRequest::UseResourceFile(file))?;
    let handle = execution.resource(context, 0x5052464e, 128, false)?;
    for (offset, address, length) in FIELDS {
        let bytes = read_handle(context, handle, offset, length)?;
        write_global(context, execution, address, &bytes);
    }
    let bytes = read_handle(context, handle, 0x2b, 1)?;
    write_global(context, execution, 0x438589, &bytes);
    let bytes = read_handle(context, handle, 0x2e, 2)?;
    write_global(context, execution, 0x402f44, &bytes);
    let bytes = read_handle(context, handle, 0x2d, 1)?;
    write_global(context, execution, 0x43858a, &bytes);
    for index in 0..20_u32 {
        let bytes = read_handle(context, handle, 0x31 + index as usize * 2, 1)?;
        write_global(context, execution, 0x44270e + index, &bytes);
    }
    let bytes = read_handle(context, handle, 0x166, 4)?;
    write_global(context, execution, 0x438598, &bytes);
    for (offset, address) in [
        (0x58, 0x439596),
        (0x5a, 0x4046ea),
        (0x5c, 0x43858c),
        (0x5e, 0x43858e),
        (0x60, 0x438590),
        (0x62, 0x438592),
        (0x64, 0x438594),
    ] {
        let bytes = read_handle(context, handle, offset, 2)?;
        write_global(context, execution, address, &bytes);
    }
    let length = usize::from(read_handle(context, handle, 0x66, 1)?[0]);
    let bytes = read_handle(context, handle, 0x66, length + 1)?;
    write_global(context, execution, 0x4385a0, &bytes);
    let seed = global(context, 0x438598, 4)?;
    if long_global(context, 0x438598)? == 0 || seed[1] & 0x80 == 0 {
        let mut value = 0_i32;
        for (step, bound) in [0x55e3, 0x29a, 32000].into_iter().enumerate() {
            let before = context.random.clone();
            let result = context.random.bound(bound);
            execution.effects.push(PreferenceEffect::Random {
                bound,
                result,
                before,
                after: context.random.clone(),
            });
            value = match step {
                0 => i32::from(result),
                1 => value.wrapping_mul(i32::from(result)),
                _ => value.wrapping_add(i32::from(result)),
            };
            write_global(context, execution, 0x438598, &value.to_be_bytes());
        }
        let mut bytes = value.to_be_bytes();
        bytes[0] |= 0x02;
        write_global(context, execution, 0x438598, &bytes);
        bytes[1] |= 0x80;
        write_global(context, execution, 0x438598, &bytes);
        write_global(context, execution, 0x479fdc, &bytes);
        let name = match execution.file(FileRequest::GetApplicationParameters)? {
            FileReply::ApplicationParameters { name, .. } => name,
            _ => return Err(RuleError::Invalid("missing GetAppParms name".into())),
        };
        write_global(context, execution, 0x479ed8, name.raw());
        let mut c_name = name.payload().to_vec();
        c_name.push(0);
        write_global(context, execution, 0x479ed8, &c_name);
        let path_end = c_name
            .iter()
            .position(|byte| *byte == 0)
            .expect("appended NUL");
        let file_handle = match execution.file(FileRequest::Open {
            name: c_name[..=path_end].to_vec(),
            mode: b"r+b\0".to_vec(),
        })? {
            FileReply::Handle(handle) => handle,
            _ => {
                return Err(RuleError::Invalid(
                    "missing application fopen result".into(),
                ));
            }
        };
        write_global(context, execution, 0x4434a6, &file_handle.to_be_bytes());
        if file_handle != 0 {
            match execution.request(PreferenceRequest::WriteFile {
                handle: file_handle,
                size: 4,
                count: 1,
                bytes: bytes.to_vec(),
            })? {
                PreferenceReply::Ignored | PreferenceReply::Long(_) | PreferenceReply::Word(_) => {}
                _ => {
                    return Err(RuleError::Invalid(
                        "unexpected application fwrite result".into(),
                    ));
                }
            }
            match execution.file(FileRequest::Close(file_handle))? {
                FileReply::Ignored | FileReply::Integer(_) | FileReply::Error(_) => (),
                _ => {
                    return Err(RuleError::Invalid(
                        "unexpected application fclose result".into(),
                    ));
                }
            }
        }
    }
    let mut seed = global(context, 0x438598, 4)?;
    write_global(context, execution, 0x43859c, &seed);
    seed[0] |= 0x02;
    write_global(context, execution, 0x43859c, &seed);
    execution.ignored(PreferenceRequest::CloseResourceFile(file))?;
    execution.ignored(PreferenceRequest::UseResourceFile(old_file))?;
    Ok(0)
}

/// CODE 1 0x14c440. Copies the template resource, writes defaults, saves and reloads.
pub fn create_preferences(
    context: &mut PreferenceContext,
    raw_specification: &[u8],
    mode: i16,
    observations: &[PreferenceObservation],
) -> Result<PreferencePlan, RuleError> {
    specification(raw_specification)?;
    execute(context, observations, |context, execution| {
        create_body(context, execution, raw_specification, mode, 0)
    })
}

fn create_body(
    context: &mut PreferenceContext,
    execution: &mut Execution<'_>,
    raw_specification: &[u8],
    mode: i16,
    depth: usize,
) -> Result<i16, RuleError> {
    let old_file = execution.word(PreferenceRequest::CurrentResourceFile)?;
    execution.ignored(PreferenceRequest::UseResourceFile(word_global(
        context, 0x438596,
    )?))?;
    let handle = execution.resource(context, 0x5052464e, 128, false)?;
    let (kind, id, name) = execution.info(handle)?;
    let home = execution.word(PreferenceRequest::HomeResourceFile(handle))?;
    execution.ignored(PreferenceRequest::DetachResource(handle))?;
    let (creator, file_kind) = if mode == 0 {
        (0x4e455743, 0x50524546)
    } else {
        (0x4d414353, 0x70726566)
    };
    execution.ignored(PreferenceRequest::CreateResourceFile {
        specification: raw_specification.to_vec(),
        creator,
        kind: file_kind,
        script: -1,
    })?;
    let file = execution.word(PreferenceRequest::OpenResourceFile {
        specification: raw_specification.to_vec(),
        permission: 0,
    })?;
    execution.ignored(PreferenceRequest::UseResourceFile(file))?;
    execution.ignored(PreferenceRequest::AddResource {
        handle,
        kind,
        id,
        name,
    })?;
    execution.ignored(PreferenceRequest::SetResourceAttributes {
        handle,
        attributes: home,
    })?;
    execution.ignored(PreferenceRequest::ResourceChanged(handle))?;
    execution.ignored(PreferenceRequest::WriteResource(handle))?;
    execution.ignored(PreferenceRequest::ReleaseResource(handle))?;
    let defaults: [(u32, i32, usize); 25] = [
        (0x439568, 20, 2),
        (0x43956a, 5, 2),
        (0x43956c, 5, 4),
        (0x439570, 5, 4),
        (0x439574, 1, 2),
        (0x439578, 5, 2),
        (0x43957a, 0x103, 2),
        (0x43957c, 1, 2),
        (0x43957e, 1, 2),
        (0x439580, 1, 2),
        (0x439582, 1, 2),
        (0x439584, 1, 2),
        (0x439586, 1, 2),
        (0x439588, 1, 2),
        (0x43958a, 1, 2),
        (0x43958c, 0, 2),
        (0x43958e, 0, 2),
        (0x439590, 0, 2),
        (0x439592, 1, 2),
        (0x438589, 0x41, 1),
        (0x43957a, 0x18, 2),
        (0x402f44, 0x7788, 2),
        (0x43858a, 5, 1),
        (0x439596, 1, 2),
        (0x4046ea, 0, 2),
    ];
    for (address, value, width) in defaults {
        let bytes = value.to_be_bytes();
        write_global(context, execution, address, &bytes[4 - width..]);
    }
    write_global(context, execution, 0x4434aa, &[0, 0]);
    for index in 0..20_u32 {
        write_global(context, execution, 0x44270e + index, &[1]);
        write_global(
            context,
            execution,
            0x4434aa,
            &(index as i16 + 1).to_be_bytes(),
        );
    }
    for address in [0x43858c, 0x43858e, 0x438590, 0x438592, 0x438594] {
        write_global(context, execution, address, &[0, 0]);
    }
    execution.ignored(PreferenceRequest::UseResourceFile(old_file))?;
    save_body(context, execution, depth)?;
    get_body(context, execution, depth)?;
    Ok(file)
}

/// CODE 1 0x1213f6. Traversal and old lengths are signed BYTEs; new length is unsigned.
pub fn set_indexed_string(
    context: &mut PreferenceContext,
    string: &PascalString,
    resource_id: i16,
    index: i16,
    observations: &[PreferenceObservation],
) -> Result<PreferencePlan, RuleError> {
    execute(context, observations, |context, execution| {
        let handle = execution.resource(context, 0x53545223, resource_id, true)?;
        if execution.word(PreferenceRequest::ResourceError)? != 0 {
            return execution.word(PreferenceRequest::ResourceError);
        }
        if handle == 0 || !context.handles.contains_key(&handle) {
            return Ok(-192);
        }
        execution.ignored(PreferenceRequest::LockHandle(handle))?;
        execution.ignored(PreferenceRequest::NoPurgeHandle(handle))?;
        let count = i16::from_be_bytes(
            read_handle(context, handle, 0, 2)?
                .try_into()
                .expect("two bytes"),
        );
        if index > count {
            return Ok(-192);
        }
        let mut offset = 2_i32;
        let mut current = 1_i16;
        while current < index {
            let position = usize::try_from(offset).map_err(|_| {
                RuleError::Invalid("signed indexed-string offset leaves the supplied buffer".into())
            })?;
            let advance = i32::from(read_handle(context, handle, position, 1)?[0] as i8) + 1;
            offset = offset.wrapping_add(advance);
            current = current.wrapping_add(1);
        }
        let old_size = match execution.request(PreferenceRequest::HandleSize(handle))? {
            PreferenceReply::Long(size) => size,
            _ => return Err(RuleError::Invalid("missing GetHandleSize result".into())),
        };
        let position = usize::try_from(offset)
            .map_err(|_| RuleError::Invalid("negative indexed-string pointer offset".into()))?;
        let old_length = i32::from(read_handle(context, handle, position, 1)?[0] as i8);
        let new_size = old_size
            .wrapping_sub(old_length)
            .wrapping_add(string.payload().len() as i32);
        execution.ignored(PreferenceRequest::UnlockHandle(handle))?;
        let error = match execution.request(PreferenceRequest::ResizeHandle {
            handle,
            size: new_size,
        })? {
            PreferenceReply::Resized { memory, error } => {
                context.handles.insert(handle, memory);
                error
            }
            _ => {
                return Err(RuleError::Invalid(
                    "missing SetHandleSize/MemErr observation".into(),
                ));
            }
        };
        if error != 0 {
            execution.ignored(PreferenceRequest::ReleaseResource(handle))?;
            return Ok(-1);
        }
        execution.ignored(PreferenceRequest::LockHandle(handle))?;
        let resized_old_length = i32::from(read_handle(context, handle, position, 1)?[0] as i8);
        let source = offset.wrapping_add(resized_old_length).wrapping_add(1);
        let length = old_size
            .wrapping_sub(offset)
            .wrapping_sub(resized_old_length)
            .wrapping_sub(1);
        let source = usize::try_from(source).map_err(|_| {
            RuleError::Invalid("signed indexed-string source leaves the handle".into())
        })?;
        let length = usize::try_from(length).map_err(|_| {
            RuleError::Invalid("signed indexed-string tail length is negative".into())
        })?;
        let destination = position + string.payload().len() + 1;
        let tail = read_handle(context, handle, source, length)?;
        read_handle(context, handle, destination, length)?;
        context
            .handles
            .get_mut(&handle)
            .expect("checked handle")
            .bytes
            .copy_within(source..source + length, destination);
        execution.effects.push(PreferenceEffect::HandleMove {
            handle,
            source,
            destination,
            bytes: tail,
        });
        write_handle(context, execution, handle, position, string.raw())?;
        execution.ignored(PreferenceRequest::ResourceChanged(handle))?;
        execution.ignored(PreferenceRequest::WriteResource(handle))?;
        execution.ignored(PreferenceRequest::PurgeHandle(handle))?;
        execution.ignored(PreferenceRequest::ReleaseResource(handle))?;
        execution.word(PreferenceRequest::ResourceError)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observed(request: PreferenceRequest, reply: PreferenceReply) -> PreferenceObservation {
        PreferenceObservation { request, reply }
    }

    fn ignored(request: PreferenceRequest) -> PreferenceObservation {
        observed(request, PreferenceReply::Ignored)
    }

    fn memory(bytes: &[u8]) -> HandleMemory {
        HandleMemory {
            bytes: bytes.to_vec(),
            logical_size: bytes.len() as u32,
        }
    }

    fn open_trace(result: i16) -> Vec<PreferenceObservation> {
        vec![
            observed(
                PreferenceRequest::FindFolder {
                    volume: i16::MIN,
                    kind: 0x70726566,
                    create: false,
                },
                PreferenceReply::Folder {
                    volume: -3,
                    directory: 42,
                    error: -43,
                },
            ),
            observed(
                PreferenceRequest::MakeFileSpecification {
                    volume: -3,
                    directory: 42,
                    name: PascalString::from_payload(b"New Centurions Prefs").unwrap(),
                },
                PreferenceReply::FileSpecification {
                    bytes: vec![0x77; 70],
                    error: -43,
                },
            ),
            observed(
                PreferenceRequest::OpenResourceFile {
                    specification: vec![0x77; 70],
                    permission: 0,
                },
                PreferenceReply::Word(result),
            ),
        ]
    }

    fn resource(
        handle: u32,
        bytes: &[u8],
        kind: u32,
        id: i16,
        current_only: bool,
    ) -> PreferenceObservation {
        observed(
            PreferenceRequest::GetResource {
                kind,
                id,
                current_only,
            },
            PreferenceReply::Resource {
                handle,
                memory: Some(memory(bytes)),
            },
        )
    }

    fn load_prefix(bytes: &[u8]) -> Vec<PreferenceObservation> {
        let mut trace = vec![observed(
            PreferenceRequest::CurrentResourceFile,
            PreferenceReply::Word(12),
        )];
        trace.extend(open_trace(24));
        trace.push(ignored(PreferenceRequest::UseResourceFile(24)));
        trace.push(resource(800, bytes, 0x5052464e, 128, false));
        trace
    }

    fn load_trace(bytes: &[u8]) -> Vec<PreferenceObservation> {
        let mut trace = load_prefix(bytes);
        trace.extend([
            ignored(PreferenceRequest::CloseResourceFile(24)),
            ignored(PreferenceRequest::UseResourceFile(12)),
        ]);
        trace
    }

    fn save_trace() -> Vec<PreferenceObservation> {
        let mut trace = vec![observed(
            PreferenceRequest::CurrentResourceFile,
            PreferenceReply::Word(12),
        )];
        trace.extend(open_trace(24));
        trace.extend([
            ignored(PreferenceRequest::UseResourceFile(24)),
            observed(
                PreferenceRequest::NewHandleClear(362),
                PreferenceReply::Handle(700),
            ),
            ignored(PreferenceRequest::LockHandle(700)),
            resource(800, &[0; 362], 0x5052464e, 128, false),
            observed(
                PreferenceRequest::ResourceInfo(800),
                PreferenceReply::ResourceInfo {
                    kind: 0x5052464e,
                    id: 128,
                    name: PascalString::from_payload(b"Prefs").unwrap(),
                },
            ),
            observed(
                PreferenceRequest::HomeResourceFile(800),
                PreferenceReply::Word(24),
            ),
            ignored(PreferenceRequest::RemoveResource(800)),
            ignored(PreferenceRequest::AddResource {
                handle: 700,
                kind: 0x5052464e,
                id: 128,
                name: PascalString::from_payload(b"Prefs").unwrap(),
            }),
            ignored(PreferenceRequest::WriteResource(700)),
            ignored(PreferenceRequest::UnlockHandle(700)),
            ignored(PreferenceRequest::ReleaseResource(700)),
            ignored(PreferenceRequest::CloseResourceFile(24)),
            ignored(PreferenceRequest::UseResourceFile(12)),
        ]);
        trace
    }

    #[test]
    fn load_preserves_raw_field_widths_and_open_discards_folder_errors() {
        let mut bytes = vec![0xa5; PREFERENCE_SIZE];
        bytes[0x166..].copy_from_slice(&0x00801234_i32.to_be_bytes());
        bytes[0x66..0x6a].copy_from_slice(&[3, 0x80, 0, b'B']);
        bytes[0x2c..0x2e].copy_from_slice(&[0x99, 0x7f]);
        bytes[0x30..0x32].copy_from_slice(&[0xff, 0x80]);
        let mut context = PreferenceContext::new(RandomState::new(17));
        context.globals.insert(0x4385a0, vec![0xdd; 16]);
        let plan = get_preferences(&mut context, &load_trace(&bytes)).unwrap();
        assert_eq!(plan.result, 0);
        assert_eq!(plan.consumed_observations, 8);
        assert_eq!(global(&context, 0x43858a, 1).unwrap(), [0x7f]);
        assert_eq!(global(&context, 0x44270e, 1).unwrap(), [0x80]);
        assert_eq!(global(&context, 0x43956c, 4).unwrap(), [0xa5; 4]);
        assert_eq!(
            global(&context, 0x43859c, 4).unwrap(),
            [2, 0x80, 0x12, 0x34]
        );
        assert_eq!(&context.globals[&0x4385a0][..5], &[3, 0x80, 0, b'B', 0xdd]);
        assert_eq!(context.random.draws, 0);
        let mut other = context.clone();
        let mut nonzero = open_trace(5);
        nonzero.truncate(1);
        let plan = open_preferences(&mut other, 2, -1, None, &nonzero).unwrap();
        assert_eq!(plan.result, 0);
        assert_eq!(other, context);
        assert!(open_preferences(&mut other, 3, -1, None, &nonzero).is_err());
        assert_eq!(other, context);
    }

    #[test]
    fn absent_seed_consumes_three_draws_and_missing_file_observation_is_atomic() {
        let mut bytes = vec![0; PREFERENCE_SIZE];
        bytes[0x66] = 0;
        let original = PreferenceContext::new(RandomState::new(123));
        let mut context = original.clone();
        let mut trace = load_prefix(&bytes);
        assert!(get_preferences(&mut context, &trace).is_err());
        assert_eq!(context, original);
        trace.push(observed(
            PreferenceRequest::File(FileRequest::GetApplicationParameters),
            PreferenceReply::File(FileReply::ApplicationParameters {
                name: PascalString::from_payload(b"Centurions").unwrap(),
                reference: 2,
                parameter_handle: 900,
            }),
        ));
        trace.push(observed(
            PreferenceRequest::File(FileRequest::Open {
                name: b"Centurions\0".to_vec(),
                mode: b"r+b\0".to_vec(),
            }),
            PreferenceReply::File(FileReply::Handle(0)),
        ));
        trace.extend([
            ignored(PreferenceRequest::CloseResourceFile(24)),
            ignored(PreferenceRequest::UseResourceFile(12)),
        ]);
        let plan = get_preferences(&mut context, &trace).unwrap();
        let mut expected_rng = original.random;
        let first = i32::from(expected_rng.bound(21987));
        let second = i32::from(expected_rng.bound(666));
        let third = i32::from(expected_rng.bound(32000));
        let mut seed = first.wrapping_mul(second).wrapping_add(third).to_be_bytes();
        seed[0] |= 2;
        seed[1] |= 0x80;
        assert_eq!(context.random, expected_rng);
        assert_eq!(global(&context, 0x438598, 4).unwrap(), seed);
        assert_eq!(global(&context, 0x479fdc, 4).unwrap(), seed);
        assert_eq!(global(&context, 0x4434a6, 4).unwrap(), [0; 4]);
        assert_eq!(
            plan.effects
                .iter()
                .filter(|effect| matches!(effect, PreferenceEffect::Random { .. }))
                .count(),
            3
        );
        assert!(!plan.effects.iter().any(|effect| matches!(
            effect,
            PreferenceEffect::Service(PreferenceObservation {
                request: PreferenceRequest::WriteFile { .. },
                ..
            })
        )));
    }

    #[test]
    fn save_sign_extends_bytes_and_create_runs_save_then_reload() {
        let mut context = PreferenceContext::new(RandomState::new(5));
        for (_, address, length) in FIELDS {
            context.globals.insert(address, vec![0x11; length]);
        }
        for index in 0..20_u32 {
            context.globals.insert(0x44270e + index, vec![0x80]);
        }
        for (address, value) in [
            (0x438589, vec![0xff]),
            (0x402f44, vec![0x77, 0x88]),
            (0x442560, vec![0x12, 0x34]),
            (0x438598, vec![0, 0x80, 0, 1]),
            (0x439596, vec![0, 1]),
            (0x4046ea, vec![0, 0]),
            (0x4385a0, vec![3, b'A', 0, 0x81]),
            (0x438596, vec![0, 9]),
        ] {
            context.globals.insert(address, value);
        }
        save_preferences(&mut context, &save_trace()).unwrap();
        let saved = &context.handles[&700].bytes;
        assert_eq!(&saved[0x2a..0x30], &[0xff, 0xff, 0x12, 0x34, 0x77, 0x88]);
        assert_eq!(&saved[0x30..0x32], &[0xff, 0x80]);
        assert_eq!(&saved[0x5c..0x66], &[0; 10]);
        assert_eq!(&saved[0x66..0x6a], &[3, b'A', 0, 0x81]);
        assert_eq!(&saved[0x166..], &[0, 0x80, 0, 1]);
        assert_eq!(global(&context, 0x43859c, 4).unwrap(), [2, 0x80, 0, 1]);
        let mut loaded = vec![0; 362];
        loaded[0..4].copy_from_slice(&[0, 20, 0, 5]);
        loaded[0x10..0x12].copy_from_slice(&[0, 24]);
        loaded[0x2d] = 0x34;
        loaded[0x66..0x6a].copy_from_slice(&[3, b'A', 0, 0x81]);
        loaded[0x166..].copy_from_slice(&[0, 0x80, 0, 1]);
        let raw_spec = vec![0xaa; 70];
        let mut trace = vec![
            observed(
                PreferenceRequest::CurrentResourceFile,
                PreferenceReply::Word(12),
            ),
            ignored(PreferenceRequest::UseResourceFile(9)),
            resource(900, &[0; 362], 0x5052464e, 128, false),
            observed(
                PreferenceRequest::ResourceInfo(900),
                PreferenceReply::ResourceInfo {
                    kind: 0x5052464e,
                    id: 128,
                    name: PascalString::default(),
                },
            ),
            observed(
                PreferenceRequest::HomeResourceFile(900),
                PreferenceReply::Word(77),
            ),
            ignored(PreferenceRequest::DetachResource(900)),
            ignored(PreferenceRequest::CreateResourceFile {
                specification: raw_spec.clone(),
                creator: 0x4e455743,
                kind: 0x50524546,
                script: -1,
            }),
            observed(
                PreferenceRequest::OpenResourceFile {
                    specification: raw_spec.clone(),
                    permission: 0,
                },
                PreferenceReply::Word(15),
            ),
            ignored(PreferenceRequest::UseResourceFile(15)),
            ignored(PreferenceRequest::AddResource {
                handle: 900,
                kind: 0x5052464e,
                id: 128,
                name: PascalString::default(),
            }),
            ignored(PreferenceRequest::SetResourceAttributes {
                handle: 900,
                attributes: 77,
            }),
            ignored(PreferenceRequest::ResourceChanged(900)),
            ignored(PreferenceRequest::WriteResource(900)),
            ignored(PreferenceRequest::ReleaseResource(900)),
            ignored(PreferenceRequest::UseResourceFile(12)),
        ];
        trace.extend(save_trace());
        trace.extend(load_trace(&loaded));
        let plan = create_preferences(&mut context, &raw_spec, 0, &trace).unwrap();
        assert_eq!(plan.result, 15);
        assert_eq!(plan.consumed_observations, trace.len());
        let writes: Vec<_> = plan
            .effects
            .iter()
            .filter_map(|effect| match effect {
                PreferenceEffect::GlobalWrite {
                    address: 0x43957a,
                    bytes,
                } => Some(bytes.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(writes, [vec![1, 3], vec![0, 24], vec![0, 24]]);
        assert_eq!(global(&context, 0x4434aa, 2).unwrap(), [0, 20]);
        assert_eq!(global(&context, 0x43858a, 1).unwrap(), [0x34]);
        assert_eq!(context.random.draws, 0);
    }

    #[test]
    fn indexed_string_moves_tail_and_preserves_native_error_returns() {
        let old = [0, 3, 1, b'A', 2, b'B', b'C', 1, b'D'];
        let replacement = PascalString::from_payload(b"XYZ").unwrap();
        let mut resized = old.to_vec();
        resized.push(0xdd);
        let mut context = PreferenceContext::new(RandomState::new(5));
        let trace = vec![
            resource(55, &old, 0x53545223, 128, true),
            observed(PreferenceRequest::ResourceError, PreferenceReply::Word(0)),
            ignored(PreferenceRequest::LockHandle(55)),
            ignored(PreferenceRequest::NoPurgeHandle(55)),
            observed(PreferenceRequest::HandleSize(55), PreferenceReply::Long(9)),
            ignored(PreferenceRequest::UnlockHandle(55)),
            observed(
                PreferenceRequest::ResizeHandle {
                    handle: 55,
                    size: 10,
                },
                PreferenceReply::Resized {
                    memory: memory(&resized),
                    error: 0,
                },
            ),
            ignored(PreferenceRequest::LockHandle(55)),
            ignored(PreferenceRequest::ResourceChanged(55)),
            ignored(PreferenceRequest::WriteResource(55)),
            ignored(PreferenceRequest::PurgeHandle(55)),
            ignored(PreferenceRequest::ReleaseResource(55)),
            observed(PreferenceRequest::ResourceError, PreferenceReply::Word(-49)),
        ];
        let plan = set_indexed_string(&mut context, &replacement, 128, 2, &trace).unwrap();
        assert_eq!(
            context.handles[&55].bytes,
            [0, 3, 1, b'A', 3, b'X', b'Y', b'Z', 1, b'D']
        );
        assert_eq!(plan.result, -49);
        let error_trace = [
            resource(55, &old, 0x53545223, 128, true),
            observed(
                PreferenceRequest::ResourceError,
                PreferenceReply::Word(-192),
            ),
            observed(PreferenceRequest::ResourceError, PreferenceReply::Word(-43)),
        ];
        assert_eq!(
            set_indexed_string(&mut context, &replacement, 128, 2, &error_trace)
                .unwrap()
                .result,
            -43
        );
        let range_trace = &trace[..4];
        let plan = set_indexed_string(&mut context, &replacement, 128, 4, range_trace).unwrap();
        assert_eq!(plan.result, -192);
        assert_eq!(plan.consumed_observations, 4);
        let original = context.clone();
        assert!(set_indexed_string(&mut context, &replacement, 128, 2, &trace[..7]).is_err());
        assert_eq!(context, original);
    }
}
