#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../../core/tests/hex.rs"]
mod fixture_hex;

mod acting;
mod appearance;
mod application;
mod assets;
mod audio;
mod collection;
mod combat_controls;
mod combat_flight;
mod combat_playback;
mod command_result;
mod creation;
mod creature;
mod editor;
mod feedback;
mod file_dialog;
mod frame;
mod game_strings;
mod help;
mod information;
mod input;
mod inventory;
mod layout;
mod note;
#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../tests/unit/original_content.rs"]
mod original_content;
mod portrait;
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

use centurions_core::Command;
use macroquad::prelude::*;
use presentation::Action;

/// delay counts TickCount ticks, sixty a second; no preference changes it.
const TICKS_PER_SECOND: f64 = 60.;

/// Starts the game: reads the command line, loads the content, sound and
/// preferences, then runs the frames until the player quits.
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
            ui.movement.stop_pointer();
            std::thread::sleep(std::time::Duration::from_millis(100));
            // Macroquad clears every frame, so present the last rendered frame again.
            layout::Canvas::new(screen_width(), screen_height()).draw(&target);
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
        let minimum = frame::minimum_window(&application);
        if !window_mode.fullscreen && (screen_width() < minimum.x || screen_height() < minimum.y) {
            request_new_screen_size(
                screen_width().max(minimum.x),
                screen_height().max(minimum.y),
            );
        }
        let canvas = layout::Canvas::new(screen_width(), screen_height());
        let mouse = mouse_position();
        let pointer = canvas.pointer(vec2(mouse.0, mouse.1));
        frame::log_pressed_keys(&input_log);
        set_camera(&camera);
        if let (Ok(content), Some(state), Some(Ok(assets))) = (&loaded, &mut state, &mut assets) {
            frame::select_render_context(
                content,
                state,
                &mut render_context,
                assets,
                &resource_root,
                &mut ui,
            );
            let content = render_context
                .as_ref()
                .map_or(content, |(_, selected)| selected);
            ui.preferences = application.preferences.clone();
            assets.set_artistic_font(
                ui.preferences
                    .enabled(crate::preferences::pref::TERMINATOR_FONT),
            );
            ui.selected = ui.selected.min(state.warriors.len().saturating_sub(1));
            while ui
                .effects
                .front()
                .is_some_and(|effect| get_time() >= effect.started + effect.frame_seconds * 8.)
            {
                ui.effects.pop_front();
            }
            frame::advance_playback(&mut ui, audio.as_ref().ok());
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
                audio.set_volumes(application.audio.volume, application.audio.music_volume);
                audio.tick_sounds(get_time());
                audio.music(
                    application.audio.music_enabled,
                    application.audio.music_request.take(),
                    &application.audio.playlist,
                    get_time(),
                );
                application.audio.current_music = audio.current_resource();
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
            // A playing combat command shows its pre-command copy until each step plays.
            let shown = ui
                .playback
                .as_ref()
                .map_or(&*state, |playback| &playback.session);
            ui.held_pointer = pointer.filter(|_| is_mouse_button_down(MouseButton::Left));
            let controls = application.draw(content, shown, assets, &ui);
            ui.movement.repeat_seconds = (application.speed as f64 * 5. - 5.) / 60.;
            let script_pause = {
                ui.prepare_capture_prompt(state);
                ui.prepare_script_pause(state)
            };
            frame::play_feedback_sound(&application, &mut ui, audio.as_ref().ok());
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
            let mut action = if script_pause && feedback_blocked && ui.feedback.queue.is_empty() {
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
                && ui.playback.is_none()
                && !state.combat.as_ref().is_some_and(|c| c.selection_pending)
            {
                application.input(state, assets, &controls, pointer)
            } else {
                None
            };
            let mut blocked = display_shortcut
                || feedback_blocked
                || was_blocked
                || application.blocks_game()
                || ui.playback.is_some();
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
                if blocked {
                    ui.movement.stop_pointer();
                }
            }
            if let Some(Action::Application(command)) = action.clone() {
                blocked = true;
                let label = format!("UI {command:?}");
                action = frame::run_application_command(
                    command,
                    state,
                    content,
                    &mut application,
                    &mut ui,
                    audio.as_ref().ok(),
                    &mut window_mode,
                );
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
                && state.combat.is_some()
            {
                action = frame::automatic_combat_action(state, &application);
                automatic_turn = action.is_some();
            }
            ui.inventory.quick_info = application
                .preferences
                .enabled(crate::preferences::pref::SCREEN_800_X_600);
            action = match ui.resolve_inventory(action, state, content, get_time()) {
                Ok(next) => next,
                Err(error) => {
                    ui.failure(error);
                    None
                }
            };
            action = frame::protect_drop(action, &application);
            if let Some(action) = action {
                let label = frame::action_label(&action);
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
                        if frame::perform_file_action(
                            action,
                            state,
                            content,
                            assets,
                            &mut saves,
                            &mut application,
                            &mut ui,
                            audio.as_ref().ok(),
                        ) {
                            enemy_error = false;
                            sync_error = false;
                            next_enemy = get_time() + 0.45;
                        }
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
