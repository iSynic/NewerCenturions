//! The preferences dialog, music playlist, popup help and window mode.
use super::{Application, ApplicationAction, HandleOutcome};
use centurions_content::Content;
use centurions_core::{Command, Event, Session};
use macroquad::prelude::*;

impl Application {
    /// Carries out the choices in the preferences dialog, music playlist and popup
    /// help.
    pub(super) fn handle_settings(
        &mut self,
        action: ApplicationAction,
        state: &mut Session,
        _content: &Content,
    ) -> Result<HandleOutcome, centurions_core::RuleError> {
        let next = None;
        let mut events = Vec::new();
        match action {
            ApplicationAction::PreferenceCancel => self.preferences_dialog = None,
            ApplicationAction::PreferenceItem(item) => {
                let dialog = self
                    .preferences_dialog
                    .as_mut()
                    .ok_or("Preferences are not open.")?;
                if dialog.item(item)? {
                    let settings = self.help.settings.clone();
                    self.help.settings["native"] = dialog.draft.encode();
                    if let Err(error) = self.help.save() {
                        self.help.settings = settings;
                        return Err(error.into());
                    }
                    self.preferences = dialog.draft.clone();
                    self.preferences_dialog = None;
                }
                events.push(Event::Sound { resource_id: 141 });
            }
            ApplicationAction::Playlist(index) => {
                if !self.supported(145, index + 10, state) {
                    return Err(centurions_core::RuleError::Refused { feedback: vec![] });
                }
                let entry = self
                    .audio
                    .playlist
                    .get_mut(index)
                    .ok_or("music playlist index")?;
                *entry = !*entry;
                self.menu_bar.menu = None;
                self.menu_bar.submenu = None;
            }
            ApplicationAction::HelpResetAnswer(answer) => {
                self.questions.help_reset = false;
                if answer {
                    self.help.reset()?;
                }
            }
            ApplicationAction::Help(item) => {
                let resource_id = self.help.action(item)?;
                if resource_id != 0 {
                    events.push(Event::Sound { resource_id });
                }
            }
            _ => unreachable!("application action routed to wrong screen"),
        }
        Ok(HandleOutcome::Complete(next, events))
    }

    /// Records whether the window hides the desktop, in the preferences and in an
    /// open preferences dialog.
    pub fn window_mode(&mut self, fullscreen: bool) {
        self.preferences.controls[crate::preferences::pref::HIDE_THE_DESKTOP - 9] = fullscreen;
        if let Some(dialog) = &mut self.preferences_dialog {
            dialog.draft.controls[crate::preferences::pref::HIDE_THE_DESKTOP - 9] = fullscreen;
        }
    }
    /// Reads the saved preferences, volume, music volume, speed and playlist from `path`. A
    /// missing or out-of-range sound volume becomes 5, music volume then follows the sound
    /// volume, and speed becomes 5.
    pub fn preferences(&mut self, path: std::path::PathBuf) -> Result<(), String> {
        self.help.preferences(path)?;
        let settings = &self.help.settings;
        self.preferences = crate::preferences::Preferences::decode(&settings["native"])?;
        self.audio.volume = settings["volume"]
            .as_u64()
            .filter(|v| *v <= 10)
            .unwrap_or(5) as usize;
        self.audio.sound = self.audio.volume != 0;
        self.audio.music_volume = settings["music_volume"]
            .as_u64()
            .filter(|v| *v <= 10)
            .map_or(self.audio.volume, |v| v as usize);
        self.speed = settings["speed"]
            .as_u64()
            .filter(|v| (1..=11).contains(v))
            .unwrap_or(5) as usize;
        self.audio.music_enabled = settings["music_enabled"].as_bool().unwrap_or(true);
        if let Some(values) = settings["playlist"].as_array().filter(|v| v.len() == 13) {
            for (target, value) in self.audio.playlist.iter_mut().zip(values) {
                *target = value.as_bool().unwrap_or(true);
            }
        }
        Ok(())
    }
    /// Attaches the preference choices the rules read (auto refit, round message
    /// sound, animation delay, combat field size) to `command`.
    pub fn command(&self, command: Command) -> Command {
        Command::WithPreferences {
            command: Box::new(command),
            preferences: centurions_core::CommandPreferences {
                auto_refit: self
                    .preferences
                    .enabled(crate::preferences::pref::AUTO_REFIT),
                suppress_round_message_sound: self
                    .preferences
                    .enabled(crate::preferences::pref::IGNORE_NEXT_ROUND_MESSAGE),
                animation_delay: crate::combat_playback::animation_delay(self.speed as i16),
                small_combat_field: !self
                    .preferences
                    .enabled(crate::preferences::pref::SCREEN_800_X_600),
                trace: self.trace,
            },
        }
    }
}
