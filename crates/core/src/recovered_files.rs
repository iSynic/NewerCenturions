//! CODE 1 filename, Finder metadata, and profile-loading control flow.

use crate::RuleError;
use crate::recovered_text::PascalString;
use centurions_content::span;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileContext {
    pub scenario_directory: Vec<u8>,
    pub variant: u8,
    pub filename: Vec<u8>,
    pub finder_info: [u8; 16],
    pub file_handle: u32,
    pub race_profile: Vec<u8>,
    pub caste_profile: Vec<u8>,
    pub application_name: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileBuffer {
    Filename,
    FinderInfo,
    MetadataFilename,
    RaceProfile,
    CasteProfile,
    ApplicationName,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileRequest {
    NumberToString(i32),
    GetFileInfo {
        name: PascalString,
        volume: i16,
    },
    SetFileInfo {
        name: PascalString,
        volume: i16,
        info: [u8; 16],
    },
    Open {
        name: Vec<u8>,
        mode: Vec<u8>,
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
        destination: FileBuffer,
    },
    Close(u32),
    GetApplicationParameters,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileReply {
    Pascal(PascalString),
    FinderInfo {
        bytes: [u8; 16],
        error: i16,
    },
    Handle(u32),
    Read {
        bytes_written: Vec<u8>,
        elements_read: u32,
    },
    ApplicationParameters {
        name: PascalString,
        reference: i16,
        parameter_handle: u32,
    },
    Integer(i32),
    Error(i16),
    /// The native source discards this service's result.
    Ignored,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileObservation {
    pub request: FileRequest,
    pub reply: FileReply,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileEffect {
    Service(FileObservation),
    Write {
        destination: FileBuffer,
        offset: usize,
        bytes: Vec<u8>,
    },
    LocalNumber {
        pascal: PascalString,
        c_string: Vec<u8>,
    },
    FileHandle(u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilePlan {
    pub effects: Vec<FileEffect>,
    pub consumed_observations: usize,
}

struct Execution<'a> {
    observations: &'a [FileObservation],
    consumed: usize,
    effects: Vec<FileEffect>,
}

impl Execution<'_> {
    fn request(&mut self, request: FileRequest) -> Result<FileReply, RuleError> {
        let observation = self.observations.get(self.consumed).ok_or(RuleError::Unsupported {
            action: "file.service",
            dependency: "the next numeric, file, or application service result has not been supplied",
        })?;
        if observation.request != request {
            return Err(RuleError::Invalid(format!(
                "file observation {} does not match request {request:?}",
                self.consumed
            )));
        }
        self.consumed += 1;
        self.effects.push(FileEffect::Service(observation.clone()));
        Ok(observation.reply.clone())
    }

    fn ignored(&mut self, request: FileRequest) -> Result<(), RuleError> {
        match self.request(request)? {
            FileReply::Ignored | FileReply::Integer(_) | FileReply::Error(_) => Ok(()),
            _ => Err(RuleError::Invalid(
                "unexpected discarded file service result".into(),
            )),
        }
    }

    fn number(&mut self, variant: u8) -> Result<Vec<u8>, RuleError> {
        let pascal = match self.request(FileRequest::NumberToString(i32::from(variant)))? {
            FileReply::Pascal(value) => value,
            _ => {
                return Err(RuleError::Invalid(
                    "missing filename number conversion result".into(),
                ));
            }
        };
        let mut c_string = pascal.raw().to_vec();
        pascal_to_c(&mut c_string)?;
        self.effects.push(FileEffect::LocalNumber {
            pascal,
            c_string: c_string.clone(),
        });
        Ok(c_string)
    }

    fn write(&mut self, destination: FileBuffer, offset: usize, bytes: &[u8]) {
        self.effects.push(FileEffect::Write {
            destination,
            offset,
            bytes: bytes.to_vec(),
        });
    }
}

fn execute(
    context: &mut FileContext,
    observations: &[FileObservation],
    body: impl FnOnce(&mut FileContext, &mut Execution<'_>) -> Result<(), RuleError>,
) -> Result<FilePlan, RuleError> {
    let mut candidate = context.clone();
    let mut execution = Execution {
        observations,
        consumed: 0,
        effects: vec![],
    };
    body(&mut candidate, &mut execution)?;
    *context = candidate;
    Ok(FilePlan {
        effects: execution.effects,
        consumed_observations: execution.consumed,
    })
}

fn c_string(bytes: &[u8]) -> Result<&[u8], RuleError> {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .ok_or_else(|| RuleError::Invalid("unterminated source C string".into()))?;
    Ok(&bytes[..=end])
}

fn copy_c(destination: &mut [u8], offset: usize, source: &[u8]) -> Result<Vec<u8>, RuleError> {
    let bytes = c_string(source)?;
    span(destination, offset, bytes.len())?;
    destination[offset..offset + bytes.len()].copy_from_slice(bytes);
    Ok(bytes.to_vec())
}

/// CODE 2 0x20014a: shift the declared Pascal payload left and append NUL.
fn pascal_to_c(bytes: &mut [u8]) -> Result<usize, RuleError> {
    let length = usize::from(
        *bytes
            .first()
            .ok_or_else(|| RuleError::Invalid("missing Pascal string length".into()))?,
    );
    span(bytes, 0, length + 1)?;
    bytes.copy_within(1..length + 1, 0);
    bytes[length] = 0;
    Ok(length + 1)
}

/// CODE 2 0x200114 shifts the entire C payload but clamps its length BYTE.
fn c_to_pascal(bytes: &mut [u8]) -> Result<usize, RuleError> {
    let length = c_string(bytes)?.len() - 1;
    bytes.copy_within(0..length, 1);
    bytes[0] = length.min(255) as u8;
    Ok(length + 1)
}

fn filename_copy(
    context: &mut FileContext,
    source: &[u8],
    execution: &mut Execution<'_>,
) -> Result<(), RuleError> {
    let bytes = copy_c(&mut context.filename, 0, source)?;
    execution.write(FileBuffer::Filename, 0, &bytes);
    Ok(())
}

fn filename_append(
    context: &mut FileContext,
    source: &[u8],
    execution: &mut Execution<'_>,
) -> Result<(), RuleError> {
    let offset = c_string(&context.filename)?.len() - 1;
    let bytes = copy_c(&mut context.filename, offset, source)?;
    execution.write(FileBuffer::Filename, offset, &bytes);
    Ok(())
}

fn filename_body(
    context: &mut FileContext,
    name: &[u8],
    execution: &mut Execution<'_>,
) -> Result<(), RuleError> {
    let directory = context.scenario_directory.clone();
    filename_copy(context, &directory, execution)?;
    let number = execution.number(context.variant)?;
    filename_append(context, b":\0", execution)?;
    filename_append(context, &number, execution)?;
    filename_append(context, b":\0", execution)?;
    filename_append(context, name, execution)
}

/// getfilename, 0x136cfa. Colons and variant bytes retain the authored path order.
pub fn get_filename(
    context: &mut FileContext,
    name: &[u8],
    observations: &[FileObservation],
) -> Result<FilePlan, RuleError> {
    execute(context, observations, |context, execution| {
        filename_body(context, name, execution)
    })
}

/// gettempfilename, 0x136d86.
pub fn get_temp_filename(
    context: &mut FileContext,
    name: &[u8],
    observations: &[FileObservation],
) -> Result<FilePlan, RuleError> {
    execute(context, observations, |context, execution| {
        filename_copy(context, b":Data Files:\0", execution)?;
        let number = execution.number(context.variant)?;
        filename_append(context, &number, execution)?;
        filename_append(context, b":\0", execution)?;
        filename_append(context, name, execution)
    })
}

/// getsavefilename, 0x136e04. Only WORD selectors 4..13 add a Game A..J folder.
pub fn get_save_filename(
    context: &mut FileContext,
    name: &[u8],
    selector: i16,
    observations: &[FileObservation],
) -> Result<FilePlan, RuleError> {
    execute(context, observations, |context, execution| {
        filename_copy(context, b":Save:\0", execution)?;
        if (4..=13).contains(&selector) {
            let game = [
                b'G',
                b'a',
                b'm',
                b'e',
                b' ',
                b'A' + (selector - 4) as u8,
                b':',
                0,
            ];
            filename_append(context, &game, execution)?;
        }
        let number = execution.number(context.variant)?;
        filename_append(context, &number, execution)?;
        filename_append(context, b":\0", execution)?;
        filename_append(context, name, execution)
    })
}

/// setfileinfo, 0x13777a. The supplied filename buffer is converted in place.
pub fn set_file_info(
    context: &mut FileContext,
    file_type: &[u8],
    name_buffer: &mut [u8],
    observations: &[FileObservation],
) -> Result<FilePlan, RuleError> {
    let mut name_candidate = name_buffer.to_vec();
    let result = execute(context, observations, |context, execution| {
        let count = c_to_pascal(&mut name_candidate)?;
        execution.write(FileBuffer::MetadataFilename, 0, &name_candidate[..count]);
        let name = PascalString::from_raw(&name_candidate)?;
        context.finder_info = match execution.request(FileRequest::GetFileInfo {
            name: name.clone(),
            volume: 0,
        })? {
            FileReply::FinderInfo { bytes, .. } => bytes,
            _ => {
                return Err(RuleError::Invalid(
                    "missing Finder info service bytes".into(),
                ));
            }
        };
        execution.write(FileBuffer::FinderInfo, 0, &context.finder_info);
        let bytes = copy_c(&mut context.finder_info, 0, file_type)?;
        execution.write(FileBuffer::FinderInfo, 0, &bytes);
        let bytes = copy_c(&mut context.finder_info, 4, b"NEWC\0")?;
        execution.write(FileBuffer::FinderInfo, 4, &bytes);
        execution.ignored(FileRequest::SetFileInfo {
            name,
            volume: 0,
            info: context.finder_info,
        })?;
        let count = pascal_to_c(&mut name_candidate)?;
        execution.write(FileBuffer::MetadataFilename, 0, &name_candidate[..count]);
        Ok(())
    })?;
    name_buffer.copy_from_slice(&name_candidate);
    Ok(result)
}

fn open_body(
    context: &mut FileContext,
    race: bool,
    execution: &mut Execution<'_>,
) -> Result<(), RuleError> {
    let (name, fallback): (&[u8], &[u8]) = if race {
        (b"Data Race\0", b":Data Files:Data Race\0")
    } else {
        (b"Data MOS\0", b":Data Files:Data MOS\0")
    };
    filename_body(context, name, execution)?;
    let paths = [c_string(&context.filename)?.to_vec(), fallback.to_vec()];
    for path in paths {
        context.file_handle = match execution.request(FileRequest::Open {
            name: path,
            mode: b"rb\0".to_vec(),
        })? {
            FileReply::Handle(handle) => handle,
            _ => return Err(RuleError::Invalid("missing opened file handle".into())),
        };
        execution
            .effects
            .push(FileEffect::FileHandle(context.file_handle));
        if context.file_handle != 0 {
            break;
        }
    }
    Ok(())
}

/// openrace, 0x13ba6c. Null scenario handles trigger exactly one stock fallback.
pub fn open_race(
    context: &mut FileContext,
    observations: &[FileObservation],
) -> Result<FilePlan, RuleError> {
    execute(context, observations, |context, execution| {
        open_body(context, true, execution)
    })
}

/// opencaste, 0x13bac6.
pub fn open_caste(
    context: &mut FileContext,
    observations: &[FileObservation],
) -> Result<FilePlan, RuleError> {
    execute(context, observations, |context, execution| {
        open_body(context, false, execution)
    })
}

fn profile_body(
    context: &mut FileContext,
    selector: i16,
    race: bool,
    execution: &mut Execution<'_>,
) -> Result<(), RuleError> {
    if selector == 0 {
        return Ok(());
    }
    open_body(context, race, execution)?;
    let size = if race { 408 } else { 576 };
    let destination = if race {
        FileBuffer::RaceProfile
    } else {
        FileBuffer::CasteProfile
    };
    let offset = u32::from(selector.wrapping_sub(1) as u16) * size;
    execution.ignored(FileRequest::Seek {
        handle: context.file_handle,
        offset,
        origin: 0,
    })?;
    let bytes = match execution.request(FileRequest::Read {
        handle: context.file_handle,
        size,
        count: 1,
        destination,
    })? {
        FileReply::Read { bytes_written, .. } if bytes_written.len() <= size as usize => {
            bytes_written
        }
        _ => {
            return Err(RuleError::Invalid(
                "invalid profile read service bytes".into(),
            ));
        }
    };
    let target = if race {
        &mut context.race_profile
    } else {
        &mut context.caste_profile
    };
    span(target, 0, size as usize)?;
    target[..bytes.len()].copy_from_slice(&bytes);
    execution.write(destination, 0, &bytes);
    execution.ignored(FileRequest::Close(context.file_handle))?;
    Ok(())
}

/// loadprofile, 0x13a7c4. Race precedes caste; short reads retain prior tail bytes.
pub fn load_profile(
    context: &mut FileContext,
    race: i16,
    caste: i16,
    observations: &[FileObservation],
) -> Result<FilePlan, RuleError> {
    execute(context, observations, |context, execution| {
        profile_body(context, race, true, execution)?;
        profile_body(context, caste, false, execution)
    })
}

/// getappname, 0x14bfe8: GetAppParms returns Pascal bytes, then PtoCstr runs.
pub fn get_application_name(
    context: &mut FileContext,
    observations: &[FileObservation],
) -> Result<FilePlan, RuleError> {
    execute(context, observations, |context, execution| {
        let name = match execution.request(FileRequest::GetApplicationParameters)? {
            FileReply::ApplicationParameters { name, .. } => name,
            _ => {
                return Err(RuleError::Invalid(
                    "missing application-name service bytes".into(),
                ));
            }
        };
        span(&context.application_name, 0, name.raw().len())?;
        context.application_name[..name.raw().len()].copy_from_slice(name.raw());
        execution.write(FileBuffer::ApplicationName, 0, name.raw());
        let count = pascal_to_c(&mut context.application_name)?;
        execution.write(
            FileBuffer::ApplicationName,
            0,
            &context.application_name[..count],
        );
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> FileContext {
        FileContext {
            scenario_directory: b":Campaign\0".to_vec(),
            variant: 255,
            filename: vec![0xcc; 256],
            finder_info: [0xaa; 16],
            file_handle: 9,
            race_profile: vec![0x88; 408],
            caste_profile: vec![0x99; 576],
            application_name: vec![0xdd; 256],
        }
    }

    fn observed(request: FileRequest, reply: FileReply) -> FileObservation {
        FileObservation { request, reply }
    }

    fn number() -> FileObservation {
        observed(
            FileRequest::NumberToString(255),
            FileReply::Pascal(PascalString::from_raw(b"\x03255").unwrap()),
        )
    }

    fn open(name: &[u8], handle: u32) -> FileObservation {
        observed(
            FileRequest::Open {
                name: name.to_vec(),
                mode: b"rb\0".to_vec(),
            },
            FileReply::Handle(handle),
        )
    }

    #[test]
    fn builders_preserve_unsigned_variant_and_native_colon_and_game_folder_order() {
        let mut context = context();
        get_filename(&mut context, b"Data MD\0unused", &[number()]).unwrap();
        assert_eq!(
            c_string(&context.filename).unwrap(),
            b":Campaign:255:Data MD\0"
        );
        get_temp_filename(&mut context, b"temp\0", &[number()]).unwrap();
        assert_eq!(
            c_string(&context.filename).unwrap(),
            b":Data Files:255:temp\0"
        );
        get_save_filename(&mut context, b"slot\0", 13, &[number()]).unwrap();
        assert_eq!(
            c_string(&context.filename).unwrap(),
            b":Save:Game J:255:slot\0"
        );
        get_save_filename(&mut context, b"slot\0", -1, &[number()]).unwrap();
        assert_eq!(c_string(&context.filename).unwrap(), b":Save:255:slot\0");
        let before = context.clone();
        assert!(get_filename(&mut context, b"file\0", &[]).is_err());
        assert_eq!(context, before);
    }

    #[test]
    fn finder_copy_terminators_clear_flag_byte_and_filename_roundtrip_preserves_tail() {
        let mut context = context();
        let mut name_buffer = vec![0xee; 64];
        name_buffer[..4].copy_from_slice(b"foo\0");
        let original_name = name_buffer.clone();
        let name = PascalString::from_payload(b"foo").unwrap();
        let mut info = [0xab; 16];
        info[..9].copy_from_slice(b"ABCDNEWC\0");
        let observations = [
            observed(
                FileRequest::GetFileInfo {
                    name: name.clone(),
                    volume: 0,
                },
                FileReply::FinderInfo {
                    bytes: [0xab; 16],
                    error: -43,
                },
            ),
            observed(
                FileRequest::SetFileInfo {
                    name,
                    volume: 0,
                    info,
                },
                FileReply::Error(-1),
            ),
        ];
        set_file_info(&mut context, b"ABCD\0", &mut name_buffer, &observations).unwrap();
        assert_eq!(context.finder_info, info);
        assert_eq!(name_buffer, original_name);
        let before = context.clone();
        assert!(set_file_info(&mut context, b"ABCD\0", &mut name_buffer, &[]).is_err());
        assert_eq!(context, before);
        assert_eq!(name_buffer, original_name);
    }

    #[test]
    fn profiles_use_fallback_unsigned_offsets_and_partial_bytes_despite_error_returns() {
        let mut context = context();
        let observations = [
            number(),
            open(b":Campaign:255:Data Race\0", 0),
            open(b":Data Files:Data Race\0", 11),
            observed(
                FileRequest::Seek {
                    handle: 11,
                    offset: 408,
                    origin: 0,
                },
                FileReply::Integer(-1),
            ),
            observed(
                FileRequest::Read {
                    handle: 11,
                    size: 408,
                    count: 1,
                    destination: FileBuffer::RaceProfile,
                },
                FileReply::Read {
                    bytes_written: vec![1, 2],
                    elements_read: 0,
                },
            ),
            observed(FileRequest::Close(11), FileReply::Integer(-1)),
            number(),
            open(b":Campaign:255:Data MOS\0", 22),
            observed(
                FileRequest::Seek {
                    handle: 22,
                    offset: 37_747_584,
                    origin: 0,
                },
                FileReply::Integer(0),
            ),
            observed(
                FileRequest::Read {
                    handle: 22,
                    size: 576,
                    count: 1,
                    destination: FileBuffer::CasteProfile,
                },
                FileReply::Read {
                    bytes_written: vec![4; 576],
                    elements_read: 1,
                },
            ),
            observed(FileRequest::Close(22), FileReply::Integer(0)),
        ];
        let plan = load_profile(&mut context, 2, -1, &observations).unwrap();
        assert_eq!(plan.consumed_observations, observations.len());
        assert_eq!(&context.race_profile[..3], &[1, 2, 0x88]);
        assert!(context.caste_profile.iter().all(|byte| *byte == 4));
        assert_eq!(context.file_handle, 22);
        assert!(
            load_profile(&mut context, 0, 0, &[])
                .unwrap()
                .effects
                .is_empty()
        );
    }

    #[test]
    fn application_name_conversion_uses_pascal_length_including_embedded_zero_bytes() {
        let mut context = context();
        let observations = [observed(
            FileRequest::GetApplicationParameters,
            FileReply::ApplicationParameters {
                name: PascalString::from_raw(b"\x03\x80\0B").unwrap(),
                reference: 7,
                parameter_handle: 44,
            },
        )];
        get_application_name(&mut context, &observations).unwrap();
        assert_eq!(&context.application_name[..5], &[0x80, 0, b'B', 0, 0xdd]);
        let mut overlong = vec![b'x'; 258];
        overlong[257] = 0;
        c_to_pascal(&mut overlong).unwrap();
        assert_eq!(overlong[0], 255);
        pascal_to_c(&mut overlong).unwrap();
        assert_eq!(&overlong[254..], &[b'x', 0, b'x', b'x']);
    }
}
