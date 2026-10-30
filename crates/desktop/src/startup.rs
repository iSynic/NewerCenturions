//! Process options, per-user storage and desktop diagnostics.
use crate::{layout, preferences};
use centurions_core::Session;
use macroquad::prelude::*;
use std::{io::Write, path::PathBuf};
/// The value that follows the command line option `name`.
pub(crate) fn option(name: &str) -> Option<String> {
    std::env::args()
        .collect::<Vec<_>>()
        .windows(2)
        .find(|a| a[0] == name)
        .map(|a| a[1].clone())
}
/// The window the game opens: titled The New Centurions, at the size asked for
/// but no smaller than the screen size preference needs.
pub(crate) fn config() -> Conf {
    let preferences = startup_preferences();
    let minimum = if preferences.enabled(crate::preferences::pref::SCREEN_800_X_600) {
        (800, 600)
    } else {
        (640, 480)
    };
    let (width, height) = option("--size")
        .and_then(|size| {
            size.split_once('x')
                .and_then(|(w, h)| Some((w.parse::<i32>().ok()?, h.parse::<i32>().ok()?)))
        })
        .unwrap_or(minimum);
    Conf {
        window_title: "The New Centurions".into(),
        window_width: width.max(minimum.0),
        window_height: height.max(minimum.1),
        // The game's Hide The Desktop default is not a request for Windows fullscreen.
        fullscreen: false,
        high_dpi: false,
        ..Default::default()
    }
}
/// Where the preferences are kept: the path given on the command line, or the
/// player's data folder.
pub(crate) fn preferences_path() -> PathBuf {
    option("--preferences-path")
        .map(PathBuf::from)
        .unwrap_or_else(|| user_directory().join("ui.json"))
}
/// The per-user folder where the game keeps its files.
pub(crate) fn user_directory() -> PathBuf {
    #[cfg(windows)]
    {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("NewCenturions")
    }
    #[cfg(not(windows))]
    {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir()));
        #[cfg(target_os = "macos")]
        let base = home.join("Library/Application Support");
        #[cfg(not(target_os = "macos"))]
        let base = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .unwrap_or_else(|| home.join(".local/share"));
        base.join("NewCenturions")
    }
}
/// The saved preferences, read before the window opens to pick its size.
pub(crate) fn startup_preferences() -> preferences::Preferences {
    std::fs::metadata(preferences_path())
        .ok()
        .filter(|m| m.len() <= 8192)
        .and_then(|_| std::fs::read(preferences_path()).ok())
        .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
        .and_then(|v| preferences::Preferences::decode(&v["native"]).ok())
        .unwrap_or_default()
}
/// Appends a line to the audit log: the command, the session before and after,
/// the random seed and draws, and the window scale.
pub(crate) fn audit(
    path: &Option<String>,
    label: &str,
    before: Option<&Session>,
    after: &Session,
    canvas: layout::Canvas,
) {
    if let (Some(path), Some(before)) = (path, before)
        && let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
    {
        let hash =
            |state: &Session| centurions_content::digest(&serde_json::to_vec(state).unwrap());
        let trace = centurions_core::trace::take();
        let _ = writeln!(
            file,
            "{}",
            serde_json::json!({"action":label,"before":hash(before),"after":hash(after),"seed":after.random.seed,"draws":after.random.draws,"scale":canvas.scale,"offset":[canvas.offset.x,canvas.offset.y],"client":[screen_width(),screen_height()],"trace":trace})
        );
    }
}

mod session;
pub(crate) use session::{Ready, prepare};
