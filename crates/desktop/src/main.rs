#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod application;
mod assets;
mod audio;
mod input;
mod layout;
mod presentation;

use assets::Assets;
use audio::Audio;
use centurions_content::installation::Installation;
use centurions_core::{Command, Event, Session, apply, load, save};
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
    let (width, height) = option("--size")
        .and_then(|size| {
            size.split_once('x')
                .and_then(|(w, h)| Some((w.parse::<i32>().ok()?, h.parse::<i32>().ok()?)))
        })
        .unwrap_or((800, 600));
    Conf {
        window_title: "The New Centurions".into(),
        window_width: width.max(800),
        window_height: height.max(600),
        high_dpi: false,
        ..Default::default()
    }
}
fn audit(
    path: &Option<String>,
    label: &str,
    before: &Session,
    after: &Session,
    canvas: layout::Canvas,
) {
    if let Some(path) = path
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
    let save_path = option("--save-path").map(PathBuf::from).unwrap_or_else(|| {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("NewCenturions/saves/session.json")
    });
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
    let mut ui = UiState::default();
    let screen = match option("--screen").as_deref() {
        Some("logo") => application::Screen::Logo,
        Some("title") => application::Screen::Title,
        Some("squad") => application::Screen::Squad,
        Some("intro") => application::Screen::Intro,
        _ if capture.is_some() || session_file.is_some() => application::Screen::Game,
        _ => application::Screen::Logo,
    };
    let mut application = application::Application::new(screen);
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
        && let Ok(audio) = &audio
    {
        audio.play(&[Event::Sound { resource_id: 20003 }]);
    }
    if let (Some(path), Ok(content)) = (&session_file, &loaded) {
        match load(Path::new(path), content) {
            Ok(s) => state = Some(s),
            Err(e) => ui.message(e.to_string()),
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
        ui.message(e.to_string());
    }
    if capture.is_some()
        && option("--screen").as_deref() == Some("quit")
        && let (Ok(content), Some(state)) = (&loaded, &mut state)
        && let Err(e) =
            application.handle(application::ApplicationAction::Menu(129, 6), state, content)
    {
        ui.message(e);
    }
    if let Err(e) = &audio {
        ui.message(e.clone());
    }
    let target = render_target(800, 600);
    target.texture.set_filter(FilterMode::Nearest);
    let mut camera = Camera2D::from_display_rect(Rect::new(0., 0., 800., 600.));
    camera.render_target = Some(target.clone());
    let mut frames = 0;
    let mut next_enemy = 0.;
    let mut enemy_error = false;
    loop {
        if screen_width() < 800. || screen_height() < 600. {
            request_new_screen_size(screen_width().max(800.), screen_height().max(600.));
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
        if let (Ok(content), Some(state), Some(Ok(assets))) = (&loaded, &mut state, &assets) {
            while ui
                .effects
                .front()
                .is_some_and(|effect| get_time() >= effect.started + effect.frame_seconds * 8.)
            {
                ui.effects.pop_front();
            }
            let before = state.clone();
            application.sync(state, capture.is_none());
            if capture.is_none()
                && let Ok(audio) = &audio
            {
                audio.music(
                    application.music_enabled,
                    application.music_request.take(),
                    get_time(),
                );
            }
            let controls = application.draw(content, state, assets, &ui);
            debug_assert_eq!(*state, before, "rendering changed rule state");
            let was_blocked = application.blocks_game();
            let mut action = if capture.is_none() {
                application.input(state, assets, &controls, pointer)
            } else {
                None
            };
            let mut blocked =
                was_blocked || application.blocks_game() || application::command_modifier();
            if action.is_none() && !blocked && ui.effects.is_empty() && get_time() >= ui.delay_until
            {
                action = input::action(state, &mut ui, &controls, pointer);
            } else {
                ui.movement.reset();
            }
            if let Some(Action::Application(command)) = action.clone() {
                blocked = true;
                action = None;
                let label = format!("UI {command:?}");
                match application.handle(command, state, content) {
                    Ok((next, events)) => {
                        action = next;
                        ui.events(&events, state);
                        if application.sound
                            && let Ok(audio) = &audio
                        {
                            audio.set_volume(application.volume);
                            audio.play(&events);
                        }
                        if let Ok(audio) = &audio {
                            audio.set_volume(application.volume);
                        }
                    }
                    Err(e) => ui.message(e),
                }
                audit(&audit_log, &label, &before, state, canvas);
            }
            if action.is_none()
                && !blocked
                && ui.inspection.is_none()
                && ui.effects.is_empty()
                && get_time() >= ui.delay_until
                && !enemy_error
                && get_time() >= next_enemy
                && capture.is_none()
                && state
                    .combat
                    .as_ref()
                    .and_then(|c| c.active_body())
                    .is_some_and(|b| b >= 10)
            {
                action = Some(Action::Command(Command::RunEnemyTurn));
            }
            if let Some(action) = action {
                let label = match &action {
                    Action::Application(_) => unreachable!(),
                    Action::Command(command) => format!("{command:?}"),
                    Action::Save => "Save".into(),
                    Action::Load => "Reload session".into(),
                    Action::Select(i) => format!("Select warrior {i}"),
                    Action::Open(_) => "Open inspection".into(),
                    Action::Close => "Close inspection".into(),
                    Action::Page(n) => format!("Inspection page {n}"),
                    Action::ChoosePsi(spell) => format!("Choose psi {spell}"),
                    Action::PsiPower(power) => format!("Psi power {power}"),
                    Action::PsiMapTarget => "Choose psi map location".into(),
                    Action::MedicalCancel => "Confirm medical cancellation".into(),
                    Action::KeepMedicalTarget => "Keep choosing medical target".into(),
                };
                match action {
                    Action::Application(_) => unreachable!(),
                    Action::Command(command) => {
                        let enemy = matches!(command, Command::RunEnemyTurn);
                        let psi = matches!(
                            command,
                            Command::CastPsi { .. } | Command::CastPartyPsi { .. }
                        );
                        let shot = matches!(command, Command::ShootEnemy { .. });
                        let medical = matches!(
                            command,
                            Command::BeginMedicalItem { .. }
                                | Command::BeginLearnedHeal { .. }
                                | Command::SelectMedicalTarget { .. }
                                | Command::StopBleeding { .. }
                        );
                        match apply(command, state, content) {
                            Ok(events) => {
                                if medical {
                                    ui.inspection = None;
                                    ui.psi_pending = None;
                                    ui.medical_cancel = false;
                                }
                                if psi {
                                    ui.inspection = None;
                                    ui.psi_pending = None;
                                    ui.psi_choice = None;
                                }
                                if shot
                                    && ui.inspection == Some(presentation::Inspection::FireTargets)
                                {
                                    ui.inspection = None;
                                }
                                ui.events(&events, state);
                                if let Ok(audio) = &audio {
                                    audio.play(&events);
                                }
                                enemy_error = false;
                            }
                            Err(e) => {
                                ui.message(e.to_string());
                                enemy_error = enemy;
                            }
                        }
                        next_enemy =
                            get_time() + (application.speed.saturating_sub(1) * 5) as f64 / 60.;
                    }
                    Action::Save => {
                        match save(&save_path, state, content) {
                            Ok(()) => {
                                ui.message("Session saved.".into());
                                if application.quit_after_save {
                                    application.quit = true;
                                }
                            }
                            Err(e) => {
                                ui.message(format!("Could not save {}: {e}", save_path.display()))
                            }
                        }
                        application.quit_after_save = false;
                    }
                    Action::Load => match load(&save_path, content) {
                        Ok(saved) => {
                            *state = saved;
                            application.screen = application::Screen::Game;
                            ui.inspection = None;
                            ui.psi_pending = None;
                            ui.psi_choice = None;
                            ui.medical_cancel = false;
                            ui.page = 0;
                            enemy_error = false;
                            next_enemy = get_time() + 0.45;
                            ui.message("Session reloaded.".into());
                        }
                        Err(centurions_core::RuleError::Io(e))
                            if e.kind() == std::io::ErrorKind::NotFound =>
                        {
                            ui.message(format!(
                                "No saved session at {}. Save a mission first.",
                                save_path.display()
                            ))
                        }
                        Err(e) => {
                            ui.message(format!("Could not load {}: {e}", save_path.display()))
                        }
                    },
                    Action::Select(i) => {
                        ui.selected = i;
                        ui.page = 0;
                    }
                    Action::Open(inspection) => {
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
                                ui.psi_power = choice.power.clamp(1, 4);
                            }
                        }
                        ui.inspection = Some(inspection);
                        ui.page = 0;
                    }
                    Action::Close => {
                        ui.inspection = None;
                        ui.psi_pending = None;
                        ui.psi_choice = None;
                    }
                    Action::Page(change) => ui.page = (ui.page as i32 + change).max(0) as usize,
                    Action::PsiPower(power) => ui.psi_power = power.clamp(1, 4),
                    Action::ChoosePsi(spell) => {
                        let power = ui.psi_power.clamp(1, 4);
                        ui.psi_choice = Some((spell, power));
                        ui.inspection = Some(presentation::Inspection::PsiTargets);
                        ui.page = 0;
                    }
                    Action::PsiMapTarget => {
                        ui.psi_pending = ui.psi_choice;
                        ui.inspection = None;
                    }
                    Action::MedicalCancel => ui.medical_cancel = true,
                    Action::KeepMedicalTarget => ui.medical_cancel = false,
                }
                audit(&audit_log, &label, &before, state, canvas);
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
        next_frame().await;
    }
}
