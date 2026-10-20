#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../../core/tests/hex.rs"]
mod fixture_hex;

mod appearance;
mod application;
mod assets;
mod audio;
mod collection;
mod combat_controls;
mod command_result;
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
mod scan_picker;
mod startup;
mod status_popup;
mod story_dialogs;
mod targeting;
mod window;
use startup::{audit, config};
mod view_actions;

use centurions_core::{Command, Event};
use macroquad::prelude::*;
use presentation::Action;
use std::io::Write;

#[macroquad::main(config)]
async fn main() {
    let startup::Ready {
        capture,
        audit_log,
        input_log,
        mut saves,
        resource_root,
        loaded,
        mut assets,
        audio,
        mut state,
        mut ui,
        mut application,
        mut window_mode,
    } = startup::prepare().await;
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
            if status_popup::active(&ui)
                && ui.technical_error.is_none()
                && application.help.page.is_none()
            {
                status_popup::scroll(state, assets, &mut ui);
            }
            let controls = application.draw(content, state, assets, &ui);
            ui.movement.repeat_seconds = (application.speed as f64 * 5. - 5.) / 60.;
            let script_pause = {
                ui.prepare_capture_prompt(state);
                ui.prepare_script_pause(state)
            };
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
            let mut action = if script_pause && feedback_blocked && ui.feedback.is_empty() {
                Some(Action::Command(Command::Continue))
            } else if status_popup::active(&ui)
                && !feedback_blocked
                && !display_shortcut
                && application.help.page.is_none()
            {
                status_popup::input()
            } else if capture.is_none()
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
            blocked |= status_popup::active(&ui);
            if !blocked
                && capture.is_none()
                && ui.inspection.is_none()
                && state.exploration.as_ref().is_some_and(|p| {
                    matches!(
                        p.presentation,
                        Some(
                            centurions_core::Presentation::Dialogue { .. }
                                | centurions_core::Presentation::Messages { .. }
                        )
                    )
                })
            {
                // Scenario text owns input; negative IDs resume without a click.
                action = input::story_action(
                    state,
                    is_mouse_button_pressed(MouseButton::Left) || !get_keys_pressed().is_empty(),
                );
                blocked = true;
            }
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
                && !targeting::active(state, &ui)
                && state
                    .exploration
                    .as_ref()
                    .is_some_and(|position| position.presentation.is_none())
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
                            && warrior.record().faction().unwrap_or(0) == 0
                            && centurions_content::word(&warrior.bytes, 0x5e).unwrap_or(0) == 0
                            && state.warriors.iter().any(|w| w.bytes[0x9de] != 0)
                        {
                            action = Some(Action::Command(Command::AutomaticBandage));
                        } else if centurions_core::combat::runtime::controlled_warrior(state)
                            .is_some()
                        {
                            action = Some(Action::Command(Command::RunControlledTurn));
                        } else if warrior.record().action_points().unwrap_or(0) < 2 {
                            action = Some(Action::Command(Command::EndTurn));
                        }
                    }
                }
                automatic_turn = action.is_some();
            }
            action = match ui.resolve_inventory(action, state, content, get_time()) {
                Ok(next) => next,
                Err(error) => {
                    ui.failure(error);
                    None
                }
            };
            if let Some(Action::Drop(warrior, slot)) = action {
                action = (!application.native_preferences.enabled(12)
                    || application::command_modifier())
                .then_some(Action::Command(Command::DropItem { warrior, slot }));
            }
            if let Some(action) = action {
                let label = match &action {
                    Action::Application(_) => unreachable!(),
                    Action::Inventory(_) | Action::Collection(_) => unreachable!(),
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
                    Action::CombatView(view) => format!("Combat view {view:?}"),
                    Action::PanCombat(x, y) => format!("Combat view {x},{y}"),
                    Action::Page(n) => format!("Inspection page {n}"),
                    Action::ChoosePsi(spell) => format!("Choose psi {spell}"),
                    Action::PsiPower(power) => format!("Psi power {power}"),
                    Action::RotatePsi => "Rotate psi target".into(),
                    Action::TargetQuantity(increase) => {
                        format!("Target quantity increase {increase}")
                    }
                    Action::MedicalCancel => "Confirm medical cancellation".into(),
                    Action::TextEntry(open) => format!("Text entry {open}"),
                    Action::PreviewPsi(spell) => format!("Preview Psi {spell}"),
                    Action::KeepMedicalTarget => "Keep choosing medical target".into(),
                    Action::Drop(_, _) => unreachable!(),
                };
                match action {
                    Action::Application(_) => unreachable!(),
                    Action::Drop(_, _) => unreachable!(),
                    Action::Command(command) => {
                        enemy_error = command_result::execute(
                            command,
                            state,
                            content,
                            &mut application,
                            &mut ui,
                            audio.as_ref().ok(),
                            automatic_turn,
                        );
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
                                ui.persistent_feedback = None;
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
                    action => {
                        if !view_actions::handle(
                            action,
                            state,
                            content,
                            &mut application,
                            &mut ui,
                            audio.as_ref().ok(),
                        ) {
                            continue;
                        }
                    }
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
