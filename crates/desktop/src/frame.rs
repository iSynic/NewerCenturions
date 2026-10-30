//! Steps of one desktop frame. `main` calls them in frame order.
use crate::{
    application::{self, Application},
    assets::Assets,
    audio::Audio,
    preferences::pref,
    presentation::{Action, UiState},
    save_files, window,
};
use centurions_content::Content;
use centurions_core::{Command, Session};
use macroquad::prelude::*;
use std::{io::Write, path::Path};

/// The scenario and level whose content the renderer last selected.
pub(crate) type RenderContext = Option<((Option<String>, u8), Content)>;

/// The smallest window the screen-size preference allows.
pub(crate) fn minimum_window(application: &Application) -> Vec2 {
    if application.preferences.enabled(pref::SCREEN_800_X_600) {
        vec2(800., 600.)
    } else {
        vec2(640., 480.)
    }
}

/// Appends this frame's pressed keys to the optional input log.
pub(crate) fn log_pressed_keys(input_log: &Option<String>) {
    if let Some(path) = input_log {
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
}

/// Switches drawing assets when the session's scenario or level changes.
pub(crate) fn select_render_context(
    content: &Content,
    state: &Session,
    render_context: &mut RenderContext,
    assets: &mut Assets,
    resource_root: &Path,
    ui: &mut UiState,
) {
    let key = (state.mission.clone(), state.scenario_level);
    if render_context
        .as_ref()
        .is_none_or(|(selected, _)| *selected != key)
    {
        match content.context(key.0.as_deref(), key.1) {
            Ok(std::borrow::Cow::Owned(selected)) => {
                if let Err(error) = assets.select_content(&selected, resource_root) {
                    ui.error(error.to_string());
                }
                *render_context = Some((key, selected));
            }
            Ok(std::borrow::Cow::Borrowed(_)) => {
                if render_context.take().is_some()
                    && let Err(error) = assets.select_content(content, resource_root)
                {
                    ui.error(error.to_string());
                }
            }
            Err(error) => ui.error(error.to_string()),
        }
    }
}

/// Starts the next step of a playing combat command once feedback is idle.
pub(crate) fn advance_playback(ui: &mut UiState, audio: Option<&Audio>) {
    if let Some(playback) = ui.playback.as_mut() {
        let idle = ui.feedback.queue.is_empty() && get_time() >= ui.delay_until;
        match playback.advance(get_time(), idle) {
            crate::combat_playback::Advance::Waiting => {}
            crate::combat_playback::Advance::Step(events) => {
                ui.events(&events);
                if let Some(audio) = audio {
                    audio.play(events.iter());
                }
            }
            crate::combat_playback::Advance::Finished => ui.playback = None,
        }
    }
}

/// Plays the next feedback sound and holds feedback for its duration.
pub(crate) fn play_feedback_sound(
    application: &Application,
    ui: &mut UiState,
    audio: Option<&Audio>,
) {
    if ui.technical_error.is_none()
        && application.help.page.is_none()
        && let Some(event) = ui.feedback_sound(get_time())
        && let Some(audio) = audio
        && let Some(resource_id) = event.sound()
    {
        ui.feedback.until = get_time()
            + if application.audio.sound && application.audio.volume != 0 {
                audio.duration(resource_id)
            } else if resource_id < 0 {
                20. / 60.
            } else {
                0.
            };
        if application.audio.sound {
            audio.play(&[event]);
        }
    }
}

/// Runs an application menu command and applies any window preference it changed.
pub(crate) fn run_application_command(
    command: application::ApplicationAction,
    state: &mut Session,
    content: &Content,
    application: &mut Application,
    ui: &mut UiState,
    audio: Option<&Audio>,
    window_mode: &mut window::WindowMode,
) -> Option<Action> {
    let mut action = None;
    let previous_preferences = application.preferences.clone();
    match application.handle(command, state, content) {
        Ok((next, events)) => {
            action = next;
            ui.events(&events);
            if application.audio.sound
                && let Some(audio) = audio
            {
                audio.set_volumes(application.audio.volume, application.audio.music_volume);
                audio.play(&events);
            }
            if let Some(audio) = audio {
                audio.set_volumes(application.audio.volume, application.audio.music_volume);
            }
        }
        Err(e) => ui.failure(e),
    }
    let size = (previous_preferences.enabled(pref::SCREEN_800_X_600)
        != application.preferences.enabled(pref::SCREEN_800_X_600))
    .then(|| {
        if application.preferences.enabled(pref::SCREEN_800_X_600) {
            (800, 600)
        } else {
            (640, 480)
        }
    });
    window::apply(window_mode.change(
        application.preferences.enabled(pref::HIDE_THE_DESKTOP),
        size,
    ));
    if application.preferences.enabled(pref::HIDE_THE_DESKTOP) != window_mode.fullscreen {
        application.window_mode(window_mode.fullscreen);
    }
    action
}

/// The command the game issues by itself for the acting combat body, if any.
pub(crate) fn automatic_combat_action(
    state: &Session,
    application: &Application,
) -> Option<Action> {
    let arena = state.combat.as_ref()?;
    if arena.selection_pending {
        return Some(Action::Command(Command::ResumeCombatSelection));
    }
    let body = arena.active_body()?;
    if !crate::acting::is_warrior_body(body) {
        return Some(Action::Command(Command::RunEnemyTurn));
    }
    let warrior = state.warriors.get(body as usize)?;
    if !application
        .preferences
        .enabled(pref::MANUAL_BANDAGE_ONLY_IN_COMBAT)
        && warrior.record().computer_control().unwrap_or(0) > 0
        && warrior.record().faction().unwrap_or(0) == 0
        && warrior.record().fleeing_duration().unwrap_or(0) == 0
        && state.warriors.iter().any(|w| w.record().bleeding() != 0)
    {
        Some(Action::Command(Command::AutomaticBandage))
    } else if centurions_core::combat::runtime::controlled_warrior(state).is_some() {
        Some(Action::Command(Command::RunControlledTurn))
    } else if warrior.record().attacks().unwrap_or(0) < 2 {
        Some(Action::Command(Command::EndTurn))
    } else {
        None
    }
}

/// Turns a drop into a command unless drop protection holds it back.
pub(crate) fn protect_drop(action: Option<Action>, application: &Application) -> Option<Action> {
    if let Some(Action::Drop(warrior, slot)) = action {
        (!application.preferences.enabled(pref::DROP_ITEM_PROTECTION)
            || application::command_modifier())
        .then_some(Action::Command(Command::DropItem { warrior, slot }))
    } else {
        action
    }
}

/// The audit-log label for a dispatched action.
pub(crate) fn action_label(action: &Action) -> String {
    match action {
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
    }
}

/// Saves, fast-saves or loads; returns true when a session was loaded.
#[allow(clippy::too_many_arguments)]
pub(crate) fn perform_file_action(
    action: Action,
    state: &mut Session,
    content: &Content,
    assets: &Assets,
    saves: &mut save_files::SaveFiles,
    application: &mut Application,
    ui: &mut UiState,
    audio: Option<&Audio>,
) -> bool {
    let mut loaded = false;
    let saving = !matches!(action, Action::Load);
    let title = &assets.menus[&129].items[if saving { 2 } else { 0 }].text;
    match saves.perform(&action, state, content, |saving, initial| {
        crate::file_dialog::choose(saving, initial, title)
    }) {
        Ok(true) if saving => {
            if application.end_after_save
                && let Err(error) =
                    application.handle(application::ApplicationAction::EndSaved, state, content)
            {
                ui.failure(error);
            }
            if application.quit_after_save {
                application.quit = true;
            }
        }
        Ok(true) => {
            application.screen = application::Screen::Game;
            reset_after_load(ui, audio);
            loaded = true;
        }
        Ok(false) => {}
        Err(e) => ui.error(e.to_string()),
    }
    application.quit_after_save = false;
    application.end_after_save = false;
    loaded
}

/// Clears transient presentation after a session is loaded.
pub(crate) fn reset_after_load(ui: &mut UiState, audio: Option<&Audio>) {
    ui.selected = 0;
    ui.feedback.queue.clear();
    ui.feedback.persistent = None;
    if let Some(audio) = audio {
        audio.clear_pending();
    }
    ui.feedback.started = None;
    ui.feedback.until = 0.;
    ui.help_request = None;
    ui.effects.clear();
    ui.playback = None;
    ui.delay_until = 0.;
    ui.movement = Default::default();
    ui.inspection = None;
    ui.psi.pending = None;
    ui.psi.choice = None;
    ui.targeting.medical_cancel = false;
    ui.page = 0;
}
