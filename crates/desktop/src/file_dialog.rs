use std::path::{Path, PathBuf};

#[cfg(windows)]
pub fn choose(saving: bool, initial: &Path, title: &str) -> std::io::Result<Option<PathBuf>> {
    use std::{
        ffi::c_void,
        os::windows::ffi::{OsStrExt, OsStringExt},
    };
    // OPENFILENAMEW from the Windows SDK commdlg.h; no additional runtime dependency.
    #[repr(C)]
    struct OpenFileName {
        size: u32,
        owner: *mut c_void,
        instance: *mut c_void,
        filter: *const u16,
        custom_filter: *mut u16,
        custom_filter_size: u32,
        filter_index: u32,
        file: *mut u16,
        file_size: u32,
        file_title: *mut u16,
        file_title_size: u32,
        initial_directory: *const u16,
        title: *const u16,
        flags: u32,
        file_offset: u16,
        extension_offset: u16,
        default_extension: *const u16,
        data: isize,
        hook: *mut c_void,
        template: *const u16,
        reserved: *mut c_void,
        reserved_flags: u32,
        flags_ex: u32,
    }
    #[link(name = "comdlg32")]
    unsafe extern "system" {
        fn GetOpenFileNameW(options: *mut OpenFileName) -> i32;
        fn GetSaveFileNameW(options: *mut OpenFileName) -> i32;
        fn CommDlgExtendedError() -> u32;
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetActiveWindow() -> *mut c_void;
    }
    let directory = initial.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(directory)?;
    let directory = directory
        .as_os_str()
        .encode_wide()
        .chain([0])
        .collect::<Vec<_>>();
    let pattern = if saving { "*.json" } else { "*.json;Data I1" };
    let filter = format!("{title} ({pattern})\0{pattern}\0\0")
        .encode_utf16()
        .collect::<Vec<_>>();
    let title = title.encode_utf16().chain([0]).collect::<Vec<_>>();
    let extension = "json\0".encode_utf16().collect::<Vec<_>>();
    let mut file = vec![0_u16; 32768];
    let name = initial
        .file_name()
        .unwrap_or_default()
        .encode_wide()
        .collect::<Vec<_>>();
    if name.len() >= file.len() {
        return Err(std::io::Error::other(
            "Save filename exceeds the dialog limit.",
        ));
    }
    file[..name.len()].copy_from_slice(&name);
    let mut options: OpenFileName = unsafe { std::mem::zeroed() };
    options.size = std::mem::size_of::<OpenFileName>() as _;
    options.owner = unsafe { GetActiveWindow() };
    options.filter = filter.as_ptr();
    options.filter_index = 1;
    options.file = file.as_mut_ptr();
    options.file_size = file.len() as _;
    options.initial_directory = directory.as_ptr();
    options.title = title.as_ptr();
    options.default_extension = if saving {
        extension.as_ptr()
    } else {
        std::ptr::null()
    };
    // Explorer, sizing, unchanged working directory, existing directory, hidden readonly control.
    options.flags =
        0x0008_0000 | 0x0080_0000 | 0x8 | 0x800 | 0x4 | if saving { 0x2 } else { 0x1000 };
    let accepted = unsafe {
        if saving {
            GetSaveFileNameW(&mut options)
        } else {
            GetOpenFileNameW(&mut options)
        }
    };
    if accepted == 0 {
        let error = unsafe { CommDlgExtendedError() };
        return if error == 0 {
            Ok(None)
        } else {
            Err(std::io::Error::other(format!(
                "Windows file dialog failed: {error:#x}"
            )))
        };
    }
    let end = file.iter().position(|c| *c == 0).unwrap_or(file.len());
    Ok(Some(std::ffi::OsString::from_wide(&file[..end]).into()))
}

#[cfg(not(windows))]
pub fn choose(_saving: bool, _initial: &Path, _title: &str) -> std::io::Result<Option<PathBuf>> {
    Err(std::io::Error::other(
        "Native file dialogs are currently available on Windows only; use --save-path.",
    ))
}
