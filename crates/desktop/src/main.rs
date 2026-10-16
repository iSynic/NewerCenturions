#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod appearance;
mod application;
mod assets;
mod audio;
mod creation;
mod creature;
mod editor;
mod feedback;
mod file_dialog;
mod help;
mod information;
mod input;
mod inventory;
mod layout;
mod note;
#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../tests/unit/original_content.rs"]
mod original_content;
mod preferences;
mod presentation;
mod save_files;
mod window;

use assets::Assets;
use audio::Audio;
use centurions_content::installation::Installation;
use centurions_core::{Command, Event, Session, apply, load};
use macroquad::prelude::*;
use presentation::{Action, UiState};
use std::{
    io::Write,
    path::{Path, PathBuf},
};

fn option(name: &str) -> Option<String> {
    std::env::args()
        .collect::<Vec<_>>()
        .windows(2)
        .find(|a| a[0] == name)
        .map(|a| a[1].clone())
}
fn config() -> Conf {
    let preferences = startup_preferences();
    let minimum = if preferences.enabled(22) {
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
        // The original Hide The Desktop default is not a request for Windows fullscreen.
        fullscreen: false,
        high_dpi: false,
        ..Default::default()
    }
}
fn preferences_path() -> PathBuf {
    option("--preferences-path")
        .map(PathBuf::from)
        .unwrap_or_else(|| user_directory().join("ui.json"))
}
fn user_directory() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("NewCenturions")
}
fn startup_preferences() -> preferences::Preferences {
    std::fs::metadata(preferences_path())
        .ok()
        .filter(|m| m.len() <= 8192)
        .and_then(|_| std::fs::read(preferences_path()).ok())
        .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
        .and_then(|v| preferences::Preferences::decode(&v["native"]).ok())
        .unwrap_or_default()
}
fn audit(
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
        let _ = writeln!(
            file,
            "{}",
            serde_json::json!({"action":label,"before":hash(before),"after":hash(after),"seed":after.random.seed,"draws":after.random.draws,"scale":canvas.scale,"offset":[canvas.offset.x,canvas.offset.y],"client":[screen_width(),screen_height()]})
        );
    }
}
#[macroquad::main(config)]
async fn main() {
    let root = std::env::args()
        .nth(1)
        .filter(|a| !a.starts_with("--"))
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("CENTURIONS_ASSET_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    std::env::current_exe()
                        .unwrap()
                        .parent()
                        .unwrap()
                        .to_path_buf()
                })
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
    let mut assets = loaded
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
        diagnostic_path: std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .map(|root| root.join("NewCenturions/logs/desktop.log")),
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
    let mut window_mode = window::WindowMode::default();
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
    let target = render_target(800, 600);
    target.texture.set_filter(FilterMode::Nearest);
    let mut camera = Camera2D::from_display_rect(Rect::new(0., 0., 800., 600.));
    camera.render_target = Some(target.clone());
    let mut activity = window::Activity::new();
    let mut frames = 0;
    let mut next_enemy = 0.;
    let mut enemy_error = false;
    let mut sync_error = false;
    let mut render_context: Option<((Option<String>, u8), centurions_content::Content)> = None;
    loop {
        if !activity.poll() && capture.is_none() {
            ui.movement.reset();
            std::thread::sleep(std::time::Duration::from_millis(100));
            next_frame().await;
            continue;
        }
        let frame_started = std::time::Instant::now();
        let alt = is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt);
        let enter = is_key_down(KeyCode::Enter) || is_key_down(KeyCode::KpEnter);
        let display_shortcut = capture.is_none() && alt && enter;
        if capture.is_none() && window_mode.shortcut(alt, enter) {
            window::apply(window_mode.change(!window_mode.fullscreen, None));
            application.window_mode(window_mode.fullscreen);
        }
        if display_shortcut {
            while get_char_pressed().is_some() {}
        }
        let minimum = if application.native_preferences.enabled(22) {
            vec2(800., 600.)
        } else {
            vec2(640., 480.)
        };
        if !window_mode.fullscreen && (screen_width() < minimum.x || screen_height() < minimum.y) {
            request_new_screen_size(
                screen_width().max(minimum.x),
                screen_height().max(minimum.y),
            );
        }
        let canvas = layout::Canvas::new(screen_width(), screen_height());
        let mouse = mouse_position();
        let pointer = canvas.pointer(vec2(mouse.0, mouse.1));
        if let Some(path) = &input_log {
            let keys = get_keys_pressed();
            if !keys.is_empty()
                && let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
            {
                let _ = writeln!(file, "pressed: {keys:?}");
            }
        }
        set_camera(&camera);
        if let (Ok(content), Some(state), Some(Ok(assets))) = (&loaded, &mut state, &mut assets) {
            let key = (state.mission.clone(), state.scenario_level);
            if render_context
                .as_ref()
                .is_none_or(|(selected, _)| *selected != key)
            {
                match content.context(key.0.as_deref(), key.1) {
                    Ok(std::borrow::Cow::Owned(selected)) => {
                        if let Err(error) = assets.select_content(&selected, &resource_root) {
                            ui.error(error.to_string());
                        }
                        render_context = Some((key, selected));
                    }
                    Ok(std::borrow::Cow::Borrowed(_)) => {
                        if render_context.take().is_some()
                            && let Err(error) = assets.select_content(content, &resource_root)
                        {
                            ui.error(error.to_string());
                        }
                    }
                    Err(error) => ui.error(error.to_string()),
                }
            }
            let content = render_context
                .as_ref()
                .map_or(content, |(_, selected)| selected);
            ui.preferences = application.native_preferences.clone();
            assets.set_artistic_font(ui.preferences.enabled(21));
            ui.selected = ui.selected.min(state.warriors.len().saturating_sub(1));
            while ui
                .effects
                .front()
                .is_some_and(|effect| get_time() >= effect.started + effect.frame_seconds * 8.)
            {
                ui.effects.pop_front();
            }
            let before = (cfg!(debug_assertions) || audit_log.is_some()).then(|| state.clone());
            if let Err(error) = application.sync(state, capture.is_none()) {
                if !sync_error {
                    ui.error(error.to_string());
                }
                sync_error = true;
            } else {
                sync_error = false;
            }
            if capture.is_none()
                && let Ok(audio) = &audio
            {
                audio.set_volumes(application.volume, application.music_volume);
                audio.set_fast_delays(ui.preferences.enabled(10));
                audio.tick_sounds(get_time());
                audio.music(
                    application.music_enabled,
                    application.music_request.take(),
                    &application.playlist,
                    get_time(),
                );
                application.current_music = audio.current_resource();
            }
            if let Err(error) = application.prepare_information(state, assets) {
                ui.failure(error);
            }
            let controls = application.draw(content, state, assets, &ui);
            ui.movement.repeat_seconds = (application.speed as f64 * 5. - 5.) / 60.;
            if ui.technical_error.is_none()
                && application.help.page.is_none()
                && let Some(event) = ui.feedback_sound(get_time())
                && let Ok(audio) = &audio
                && let Event::Sound { resource_id } = event
            {
                ui.feedback_until = get_time()
                    + if application.sound && application.volume != 0 {
                        audio.duration(resource_id)
                    } else if resource_id < 0 {
                        20. / 60.
                    } else {
                        0.
                    };
                if application.sound {
                    audio.play(&[event]);
                }
            }
            if application.help.page.is_none() || ui.technical_error.is_some() {
                ui.draw_feedback(assets, application.warning_origin());
            }
            if let Some(before) = &before {
                debug_assert_eq!(*state, *before, "rendering changed rule state");
            }
            let was_blocked = application.blocks_game();
            let feedback_blocked = (application.help.page.is_none()
                || ui.technical_error.is_some())
                && ui.feedback_input(
                    get_time(),
                    capture.is_none()
                        && !display_shortcut
                        && (is_mouse_button_pressed(MouseButton::Left)
                            || !get_keys_pressed().is_empty()),
                );
            if let Some((first, last)) = ui.help_request.take()
                && application.help.page.is_none()
                && let Err(error) = application.help.open(first, last)
            {
                ui.error(error);
            }
            let mut action = if capture.is_none()
                && !display_shortcut
                && !feedback_blocked
                && !state.combat.as_ref().is_some_and(|c| c.selection_pending)
            {
                application.input(state, assets, &controls, pointer)
            } else {
                None
            };
            let mut blocked =
                display_shortcut || feedback_blocked || was_blocked || application.blocks_game();
            let mut automatic_turn = false;
            if action.is_none() && !blocked && ui.effects.is_empty() && get_time() >= ui.delay_until
            {
                action = input::action(state, content, &mut ui, &controls, pointer);
            } else {
                ui.movement.reset();
            }
            if let Some(Action::Application(command)) = action.clone() {
                blocked = true;
                action = None;
                let label = format!("UI {command:?}");
                let previous_preferences = application.native_preferences.clone();
                match application.handle(command, state, content) {
                    Ok((next, events)) => {
                        action = next;
                        ui.events(&events);
                        if application.sound
                            && let Ok(audio) = &audio
                        {
                            audio.set_volumes(application.volume, application.music_volume);
                            audio.play(&events);
                        }
                        if let Ok(audio) = &audio {
                            audio.set_volumes(application.volume, application.music_volume);
                        }
                    }
                    Err(e) => ui.failure(e),
                }
                let size = (previous_preferences.enabled(22)
                    != application.native_preferences.enabled(22))
                .then(|| {
                    if application.native_preferences.enabled(22) {
                        (800, 600)
                    } else {
                        (640, 480)
                    }
                });
                window::apply(window_mode.change(application.native_preferences.enabled(20), size));
                if application.native_preferences.enabled(20) != window_mode.fullscreen {
                    application.window_mode(window_mode.fullscreen);
                }
                audit(&audit_log, &label, before.as_ref(), state, canvas);
            }
            if action.is_none()
                && !blocked
                && ui.inspection.is_none()
                && ui.effects.is_empty()
                && get_time() >= ui.delay_until
                && !enemy_error
                && get_time() >= next_enemy
                && capture.is_none()
                && let Some(arena) = state.combat.as_ref()
            {
                if arena.selection_pending {
                    action = Some(Action::Command(Command::ResumeCombatSelection));
                } else if let Some(body) = arena.active_body() {
                    if body >= 10 {
                        action = Some(Action::Command(Command::RunEnemyTurn));
                    } else if let Some(warrior) = state.warriors.get(body as usize) {
                        if !application.native_preferences.enabled(16)
                            && centurions_content::word(&warrior.bytes, 0x90).unwrap_or(0) > 0
                            && centurions_content::word(&warrior.bytes, 0x10).unwrap_or(0) == 0
                            && centurions_content::word(&warrior.bytes, 0x5e).unwrap_or(0) == 0
                            && state.warriors.iter().any(|w| w.bytes[0x9de] != 0)
                        {
                            action = Some(Action::Command(Command::AutomaticBandage));
                        } else if centurions_core::combat::runtime::controlled_warrior(state)
                            .is_some()
                        {
                            action = Some(Action::Command(Command::RunControlledTurn));
                        } else if centurions_content::word(&warrior.bytes, 0x40).unwrap_or(0) < 2 {
                            action = Some(Action::Command(Command::EndTurn));
                        }
                    }
                }
                automatic_turn = action.is_some();
            }
            if let Some(Action::Drop(warrior, slot)) = action {
                action = (!application.native_preferences.enabled(12)
                    || application::command_modifier())
                .then_some(Action::Command(Command::DropItem { warrior, slot }));
            }
            action = match ui.resolve_inventory(action, state, content, get_time()) {
                Ok(next) => next,
                Err(error) => {
                    ui.failure(error);
                    None
                }
            };
            if let Some(action) = action {
                let label = match &action {
                    Action::Application(_) => unreachable!(),
                    Action::Inventory(_) => unreachable!(),
                    Action::Command(command) => format!("{command:?}"),
                    Action::Save => "Save".into(),
                    Action::FastSave => "Fast save".into(),
                    Action::Load => "Reload session".into(),
                    Action::Select(i) => format!("Select warrior {i}"),
                    Action::Open(_) => "Open inspection".into(),
                    Action::InformationInspection(member, _) => {
                        format!("Information child {member}")
                    }
                    Action::Close => "Close inspection".into(),
                    Action::Page(n) => format!("Inspection page {n}"),
                    Action::ChoosePsi(spell) => format!("Choose psi {spell}"),
                    Action::PsiPower(power) => format!("Psi power {power}"),
                    Action::PsiMapTarget => "Choose psi map location".into(),
                    Action::RotatePsi => "Rotate psi target".into(),
                    Action::FireMapTarget => "Choose firearm map location".into(),
                    Action::MedicalCancel => "Confirm medical cancellation".into(),
                    Action::KeepMedicalTarget => "Keep choosing medical target".into(),
                    Action::Drop(_, _) => unreachable!(),
                };
                match action {
                    Action::Application(_) => unreachable!(),
                    Action::Drop(_, _) => unreachable!(),
                    Action::Command(command) => {
                        let enemy = matches!(command, Command::RunEnemyTurn);
                        let psi = matches!(
                            command,
                            Command::CastPsi { .. }
                                | Command::CastEncounterPsi { .. }
                                | Command::CastPartyPsi { .. }
                                | Command::BeginPsiSelection { .. }
                                | Command::FirePsiSelection
                                | Command::CancelPsiSelection
                                | Command::SelectOutdoorPsiTarget { .. }
                                | Command::FinishOutdoorPsi
                                | Command::CancelOutdoorPsi
                        );
                        let shot = matches!(
                            command,
                            Command::ShootEnemy { .. }
                                | Command::ShootAt { .. }
                                | Command::FireSelectedTargets
                        );
                        let medical = matches!(
                            command,
                            Command::BeginMedicalItem { .. }
                                | Command::UseItem { .. }
                                | Command::BeginLearnedHeal { .. }
                                | Command::SelectMedicalTarget { .. }
                                | Command::StopBleeding { .. }
                        );
                        let movement =
                            matches!(command, Command::Move { .. } | Command::CombatClick { .. });
                        let previous_round = state.combat.as_ref().map(|c| c.round);
                        match apply(application.command(command), state, content) {
                            Ok(mut events) => {
                                if shot
                                    && events
                                        .iter()
                                        .any(|event| matches!(event, Event::AttackResolved { .. }))
                                {
                                    ui.firearm_map_target = false;
                                }
                                events.extend(application.combat_notices(previous_round, state));
                                if movement
                                    && !events.iter().any(|event| {
                                        matches!(
                                            event,
                                            Event::TacticalMoved { .. } | Event::Moved { .. }
                                        )
                                    })
                                {
                                    ui.movement.refuse();
                                }
                                if medical {
                                    ui.medical_result(state, &events);
                                }
                                if psi {
                                    ui.inspection = None;
                                    ui.psi_pending = None;
                                    ui.psi_choice = None;
                                    ui.medical_cancel = false;
                                    if state.psi_selection.is_some() {
                                        ui.page = 0;
                                    }
                                }
                                if state
                                    .combat
                                    .as_ref()
                                    .is_some_and(|c| c.firearm_selection.is_some())
                                {
                                    ui.inspection = Some(presentation::Inspection::FireTargets);
                                } else if shot
                                    && ui.inspection == Some(presentation::Inspection::FireTargets)
                                {
                                    ui.inspection = None;
                                }
                                ui.events(&events);
                                application.observe(&events);
                                if events
                                    .iter()
                                    .any(|event| matches!(event, Event::MissionEnded { .. }))
                                {
                                    ui.selected = 0;
                                    ui.inspection = None;
                                    ui.effects.clear();
                                    ui.movement.reset();
                                    ui.psi_pending = None;
                                    ui.psi_choice = None;
                                }
                                if let Ok(audio) = &audio {
                                    audio.play(&events);
                                }
                                enemy_error = false;
                            }
                            Err(e) => {
                                ui.failure(e);
                                enemy_error = enemy || automatic_turn;
                            }
                        }
                        next_enemy =
                            get_time() + (application.speed.saturating_sub(1) * 5) as f64 / 60.;
                    }
                    action @ (Action::Save | Action::FastSave | Action::Load) => {
                        let saving = !matches!(action, Action::Load);
                        let title = &assets.menus[&129].items[if saving { 2 } else { 0 }].text;
                        match saves.perform(&action, state, content, |saving, initial| {
                            file_dialog::choose(saving, initial, title)
                        }) {
                            Ok(true) if saving => {
                                if application.end_after_save
                                    && let Err(error) = application.handle(
                                        application::ApplicationAction::EndSaved,
                                        state,
                                        content,
                                    )
                                {
                                    ui.failure(error);
                                }
                                if application.quit_after_save {
                                    application.quit = true;
                                }
                            }
                            Ok(true) => {
                                application.screen = application::Screen::Game;
                                ui.selected = 0;
                                ui.feedback.clear();
                                if let Ok(audio) = &audio {
                                    audio.clear_pending();
                                }
                                ui.feedback_started = None;
                                ui.feedback_until = 0.;
                                ui.help_request = None;
                                ui.effects.clear();
                                ui.delay_until = 0.;
                                ui.movement = Default::default();
                                ui.inspection = None;
                                ui.psi_pending = None;
                                ui.psi_choice = None;
                                ui.medical_cancel = false;
                                ui.page = 0;
                                enemy_error = false;
                                sync_error = false;
                                next_enemy = get_time() + 0.45;
                            }
                            Ok(false) => {}
                            Err(e) => ui.error(e.to_string()),
                        }
                        application.quit_after_save = false;
                        application.end_after_save = false;
                    }
                    Action::Select(i) => {
                        ui.selected = i;
                        ui.page = 0;
                    }
                    Action::InformationInspection(member, inspection) => {
                        ui.selected = member;
                        ui.return_information = Some(member);
                        ui.inspection = Some(inspection);
                        ui.page = 0;
                        if inspection == presentation::Inspection::Items {
                            ui.inventory.open(member, state.warriors.len());
                            if let Err(error) =
                                apply(ui.inventory.context_command(), state, content)
                            {
                                ui.failure(error);
                            }
                        }
                    }
                    Action::Open(inspection) => {
                        if let Some(centurions_core::Presentation::Clinic { member, .. }) = state
                            .exploration
                            .as_ref()
                            .and_then(|p| p.presentation.as_ref())
                        {
                            ui.selected = *member;
                        }
                        if inspection == presentation::Inspection::Items
                            && state.phase == centurions_core::Phase::Combat
                        {
                            if let Some(body) = state
                                .combat
                                .as_ref()
                                .and_then(|arena| arena.active_body())
                                .filter(|body| (0..6).contains(body))
                            {
                                ui.selected = body as usize;
                            }
                            if let Ok(audio) = &audio {
                                audio.play(&[
                                    Event::Sound { resource_id: 141 },
                                    Event::Sound { resource_id: 20001 },
                                ]);
                            }
                        }
                        if inspection == presentation::Inspection::Psi {
                            let actor = state
                                .combat
                                .as_ref()
                                .and_then(|arena| arena.active_body())
                                .filter(|body| (0..6).contains(body))
                                .map_or(ui.selected, |body| body as usize);
                            if let Some(choice) = state.psi_menu_choices[actor].as_ref() {
                                ui.psi_power = choice.power.clamp(1, 7);
                            }
                        }
                        ui.inspection = Some(inspection);
                        ui.page = 0;
                        if inspection == presentation::Inspection::Items {
                            ui.inventory.open(ui.selected, state.warriors.len());
                            if let Err(error) =
                                apply(ui.inventory.context_command(), state, content)
                            {
                                ui.failure(error);
                            }
                        }
                    }
                    Action::Close => {
                        ui.firearm_map_target = false;
                        if state
                            .combat
                            .as_ref()
                            .is_some_and(|c| c.firearm_selection.is_some())
                        {
                            match apply(Command::CancelFireTargets, state, content) {
                                Ok(events) => application.observe(&events),
                                Err(error) => ui.failure(error),
                            }
                        }
                        ui.inspection = None;
                        ui.psi_pending = None;
                        ui.psi_choice = None;
                        if let Some(member) = ui.return_information.take() {
                            application.restore_information(member);
                        }
                    }
                    Action::Page(change) => ui.page = (ui.page as i32 + change).max(0) as usize,
                    Action::Inventory(_) => unreachable!(),
                    Action::PsiPower(power) => ui.psi_power = power.clamp(1, 7),
                    Action::ChoosePsi(spell) => {
                        let power = ui.psi_power.clamp(1, 7);
                        ui.psi_rotation = 0;
                        ui.psi_preview = None;
                        ui.psi_choice = Some((spell, power));
                        ui.inspection = Some(presentation::Inspection::PsiTargets);
                        ui.page = 0;
                    }
                    Action::RotatePsi => {
                        ui.psi_rotation = (ui.psi_rotation + 1) % 4;
                        ui.feedback
                            .push_back(centurions_core::feedback::NativeFeedback::Sound {
                                resource_id: 130,
                            });
                    }
                    Action::PsiMapTarget => {
                        ui.psi_pending = ui.psi_choice;
                        ui.inspection = None;
                    }
                    Action::FireMapTarget => {
                        ui.firearm_map_target = true;
                        ui.inspection = None;
                    }
                    Action::MedicalCancel => ui.medical_cancel = true,
                    Action::KeepMedicalTarget => ui.medical_cancel = false,
                }
                audit(&audit_log, &label, before.as_ref(), state, canvas);
            }
        } else {
            clear_background(BLACK);
            let error = loaded
                .as_ref()
                .err()
                .map(|e| e.to_string())
                .or_else(|| {
                    assets
                        .as_ref()
                        .and_then(|a| a.as_ref().err())
                        .map(|e| e.to_string())
                })
                .unwrap_or_else(|| "Loading".into());
            draw_text(&error, 24., 64., 20., WHITE);
        }
        canvas.draw(&target);
        if application.quit {
            break;
        }
        if let Some(path) = &capture
            && frames >= 3
        {
            get_screen_data().export_png(path);
            break;
        }
        frames += 1;
        if let Some(remaining) =
            std::time::Duration::from_micros(16_667).checked_sub(frame_started.elapsed())
        {
            std::thread::sleep(remaining);
        }
        next_frame().await;
    }
}
