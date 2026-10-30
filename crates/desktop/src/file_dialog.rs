use std::path::{Path, PathBuf};

/// Asks the player, with the Windows Open or Save dialog, for a file starting at
/// `initial`; `None` if the player cancels.
#[cfg(windows)]
pub fn choose(saving: bool, initial: &Path, title: &str) -> std::io::Result<Option<PathBuf>> {
    use std::{
        ffi::c_void,
        os::windows::ffi::{OsStrExt, OsStringExt},
    };
    // OPENFILENAMEW from the Windows SDK commdlg.h; no additional runtime dependency.
    /// The Windows OPENFILENAMEW options for the file dialog.
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
        /// Shows the Windows Open dialog; nonzero when a file was chosen.
        fn GetOpenFileNameW(options: *mut OpenFileName) -> i32;
        /// Shows the Windows Save dialog; nonzero when a file was chosen.
        fn GetSaveFileNameW(options: *mut OpenFileName) -> i32;
        /// The reason a Windows file dialog failed.
        fn CommDlgExtendedError() -> u32;
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        /// The window that owns the file dialog.
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

/// Asks the player, with the system file chooser, for a file starting at
/// `initial`; `None` if the player cancels.
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn choose(saving: bool, initial: &Path, title: &str) -> std::io::Result<Option<PathBuf>> {
    let directory = initial.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(directory)?;
    let initial = directory
        .canonicalize()?
        .join(initial.file_name().unwrap_or_default());
    platform_choose(saving, &initial, title)
}

/// Shows the macOS file chooser through a script.
#[cfg(target_os = "macos")]
fn platform_choose(saving: bool, initial: &Path, title: &str) -> std::io::Result<Option<PathBuf>> {
    // Arguments remain separate from the script, including filenames containing quotes.
    /// The script that shows the macOS Open or Save chooser.
    const SCRIPT: &str = r#"function run(args) {
        var app = Application.currentApplication();
        app.includeStandardAdditions = true;
        try {
            var options = {withPrompt: args[3], defaultLocation: Path(args[1])};
            var file;
            if (args[0] === 'save') {
                options.defaultName = args[2];
                file = app.chooseFileName(options);
            } else {
                file = app.chooseFile(options);
            }
            return JSON.stringify(file.toString());
        } catch (error) {
            if (error.errorNumber === -128) return 'null';
            throw error;
        }
    }"#;
    let output = std::process::Command::new("/usr/bin/osascript")
        .args(["-l", "JavaScript", "-e", SCRIPT, "--"])
        .arg(if saving { "save" } else { "open" })
        .arg(initial.parent().unwrap())
        .arg(initial.file_name().unwrap_or_default())
        .arg(title)
        .output()?;
    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "macOS file dialog failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    serde_json::from_slice::<Option<String>>(&output.stdout)
        .map(|path| path.map(PathBuf::from))
        .map_err(std::io::Error::other)
}

/// Shows the zenity file chooser.
#[cfg(target_os = "linux")]
fn platform_choose(saving: bool, initial: &Path, title: &str) -> std::io::Result<Option<PathBuf>> {
    let mut zenity = std::process::Command::new("zenity");
    zenity
        .args(["--file-selection", "--title", title, "--filename"])
        .arg(initial);
    if saving {
        zenity.args(["--save", "--confirm-overwrite"]);
    }
    let output = match zenity.output() {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut kdialog = std::process::Command::new("kdialog");
            kdialog
                .arg(if saving {
                    "--getsavefilename"
                } else {
                    "--getopenfilename"
                })
                .arg(initial)
                .args(["--title", title]);
            kdialog.output().map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    std::io::Error::other("Save/Open requires zenity or kdialog. Install one with your system package manager, or use --save-path.")
                } else { error }
            })?
        }
        result => result?,
    };
    linux_selection(output.status.code(), &output.stdout, &output.stderr)
}

/// The chosen file from zenity's result: none if the player cancelled, an error
/// if the chooser failed.
#[cfg(any(target_os = "linux", test))]
fn linux_selection(
    code: Option<i32>,
    stdout: &[u8],
    stderr: &[u8],
) -> std::io::Result<Option<PathBuf>> {
    match code {
        Some(1) => Ok(None),
        Some(0) => {
            let bytes = stdout.strip_suffix(b"\n").unwrap_or(stdout);
            if bytes.is_empty() {
                return Err(std::io::Error::other(
                    "File dialog returned an empty filename.",
                ));
            }
            #[cfg(unix)]
            let path = {
                use std::os::unix::ffi::OsStringExt;
                PathBuf::from(std::ffi::OsString::from_vec(bytes.to_vec()))
            };
            #[cfg(not(unix))]
            let path =
                PathBuf::from(String::from_utf8(bytes.to_vec()).map_err(std::io::Error::other)?);
            if !path.is_absolute() {
                return Err(std::io::Error::other(
                    "File dialog returned a relative filename.",
                ));
            }
            Ok(Some(path))
        }
        _ => Err(std::io::Error::other(format!(
            "File dialog failed: {}",
            String::from_utf8_lossy(stderr).trim()
        ))),
    }
}

/// Reports that file dialogs are unavailable on this platform.
#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
pub fn choose(_saving: bool, _initial: &Path, _title: &str) -> std::io::Result<Option<PathBuf>> {
    Err(std::io::Error::other(
        "Native file dialogs are unavailable on this platform; use --save-path.",
    ))
}

#[cfg(test)]
#[path = "../tests/unit/file_dialog.rs"]
mod tests;
