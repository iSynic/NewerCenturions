//! One-time installation, session and screen initialization.
use super::{option, preferences_path, user_directory};
use crate::{application, assets::Assets, audio::Audio, presentation::UiState, save_files, window};
use centurions_content::{Content, ContentError, installation::Installation};
use centurions_core::{Command, Event, Session, apply, load};
use std::path::{Path, PathBuf};

pub(crate) struct Ready {
    pub capture: Option<String>,
    pub audit_log: Option<String>,
    pub input_log: Option<String>,
    pub saves: save_files::SaveFiles,
    pub resource_root: PathBuf,
    pub loaded: Result<Content, ContentError>,
    pub assets: Option<Result<Assets, ContentError>>,
    pub audio: Result<Audio, String>,
    pub state: Option<Session>,
    pub ui: UiState,
    pub application: application::Application,
    pub window_mode: window::WindowMode,
}

pub(crate) async fn prepare() -> Ready {
    let root = std::env::args()
        .nth(1)
        .filter(|a| !a.starts_with("--"))
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("CENTURIONS_ASSET_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(Installation::default_root)
        });
    let capture = option("--capture");
    let session_file = option("--session");
    let audit_log = option("--audit-log");
    let input_log = option("--input-log");
    let mut saves = save_files::SaveFiles {
        current: None,
        default: user_directory().join("saves/session.json"),
        explicit: option("--save-path").map(PathBuf::from),
        protected: [
            "NewCenturions",
            "Data Files",
            "Scenarios",
            "Warriors",
            "Save",
        ]
        .map(|folder| root.join(folder))
        .into(),
    };
    let installation = Installation::open(&root);
    let resource_root = installation
        .as_ref()
        .map(|i| i.resource_root.clone())
        .unwrap_or_else(|_| root.clone());
    let loaded = installation.map(|i| i.content);
    let assets = loaded
        .as_ref()
        .ok()
        .map(|content| Assets::load(content, &resource_root));
    let audio = if let Ok(content) = &loaded {
        Audio::load(content, &resource_root).await
    } else {
        Err("content unavailable".into())
    };
    let mut state = loaded.as_ref().ok().map(Session::new);
    let mut ui = UiState {
        diagnostic_path: Some(user_directory().join("logs/desktop.log")),
        ..Default::default()
    };
    let screen = match option("--screen").as_deref() {
        Some("logo") => application::Screen::Logo,
        Some("title") => application::Screen::Title,
        Some("squad") => application::Screen::Squad,
        Some("intro") => application::Screen::Intro,
        _ if capture.is_some() || session_file.is_some() => application::Screen::Game,
        _ => application::Screen::Logo,
    };
    let mut application = application::Application::new(screen);
    if let Ok(content) = &loaded
        && let Err(error) = application.load_roster(
            &option("--roster-path")
                .map(PathBuf::from)
                .unwrap_or_else(|| user_directory().join("warriors.json")),
            content,
        )
    {
        ui.failure(error);
    }
    if let Err(error) = application.preferences(preferences_path()) {
        ui.error(error.to_string());
    }
    let window_mode = window::WindowMode::default();
    application.window_mode(false);
    if let Ok(audio) = &audio {
        audio.set_volumes(application.volume, application.music_volume);
    }
    if screen == application::Screen::Squad
        && loaded.is_ok()
        && assets.as_ref().is_some_and(|assets| assets.is_ok())
        && let Err(error) = application.help.open(121, 124)
    {
        ui.error(error.to_string());
    }
    application.show_hover = capture.is_none();
    if capture.is_some()
        && screen == application::Screen::Squad
        && std::env::args().any(|arg| arg == "--selected-starters")
    {
        application.squad = (0..6).collect();
    }
    if let Some(menu) = option("--menu").and_then(|id| id.parse().ok()) {
        application.menu = Some(menu);
    }
    if screen == application::Screen::Logo
        && application.sound
        && let Ok(audio) = &audio
    {
        audio.play(&[Event::Sound { resource_id: 20003 }]);
    }
    if let (Some(path), Ok(content)) = (&session_file, &loaded) {
        match load(Path::new(path), content) {
            Ok(s) => {
                state = Some(s);
                saves.current = Some(PathBuf::from(path));
            }
            Err(e) => ui.error(e.to_string()),
        }
    }
    if capture.is_some()
        && screen == application::Screen::Game
        && session_file.is_none()
        && let (Ok(content), Some(state)) = (&loaded, &mut state)
        && let Err(e) = apply(Command::ImportStarters, state, content).and_then(|_| {
            apply(
                Command::EnterMission {
                    name: "The Thunder of Guns".into(),
                },
                state,
                content,
            )
        })
    {
        ui.error(e.to_string());
    }
    if capture.is_some()
        && option("--screen").as_deref() == Some("quit")
        && let (Ok(content), Some(state)) = (&loaded, &mut state)
        && let Err(e) =
            application.handle(application::ApplicationAction::Menu(129, 6), state, content)
    {
        ui.failure(e);
    }
    if let Err(e) = &audio {
        ui.error(e.clone());
    }
    if capture.is_some()
        && matches!(
            option("--screen").as_deref(),
            Some("editor" | "item-editor" | "editor-level" | "editor-scans")
        )
        && let (Ok(content), Some(state)) = (&loaded, &mut state)
    {
        application.editing = true;
        let action = if option("--screen").as_deref() == Some("item-editor") {
            application::ApplicationAction::OpenItemEditor(0, 0)
        } else {
            application::ApplicationAction::OpenEditor(usize::from(
                option("--screen").as_deref() == Some("editor-scans"),
            ))
        };
        if let Err(error) = application.handle(action, state, content) {
            ui.error(error.to_string());
        }
        if option("--screen").as_deref() == Some("editor-level") {
            for _ in 0..3 {
                if let Err(error) = application.handle(
                    application::ApplicationAction::EditorItem(81),
                    state,
                    content,
                ) {
                    ui.error(error.to_string());
                    break;
                }
            }
        }
        if option("--screen").as_deref() == Some("editor-scans") {
            for item in [39, 2, 11] {
                if let Err(error) = application.handle(
                    application::ApplicationAction::EditorItem(item),
                    state,
                    content,
                ) {
                    ui.error(error.to_string());
                    break;
                }
            }
        }
    }
    Ready {
        capture,
        audit_log,
        input_log,
        saves,
        resource_root,
        loaded,
        assets,
        audio,
        state,
        ui,
        application,
        window_mode,
    }
}
