//! Settings workflow ownership.
use super::{Application, ApplicationAction, HandleOutcome};
use centurions_content::Content;
use centurions_core::{Command, Event, Session};
use macroquad::prelude::*;

impl Application {
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
                    self.native_preferences = dialog.draft.clone();
                    self.preferences_dialog = None;
                }
                events.push(Event::Sound { resource_id: 141 });
            }
            ApplicationAction::Playlist(index) => {
                if !self.supported(145, index + 10, state) {
                    return Err(centurions_core::RuleError::Refused { feedback: vec![] });
                }
                let entry = self.playlist.get_mut(index).ok_or("music playlist index")?;
                *entry = !*entry;
                self.menu = None;
                self.submenu = None;
            }
            ApplicationAction::HelpResetAnswer(answer) => {
                self.help_reset_question = false;
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

    pub fn window_mode(&mut self, fullscreen: bool) {
        self.native_preferences.controls[20 - 9] = fullscreen;
        if let Some(dialog) = &mut self.preferences_dialog {
            dialog.draft.controls[20 - 9] = fullscreen;
        }
    }
    pub fn preferences(&mut self, path: std::path::PathBuf) -> Result<(), String> {
        self.help.preferences(path)?;
        let settings = &self.help.settings;
        self.native_preferences = crate::preferences::Preferences::decode(&settings["native"])?;
        self.volume = settings["volume"]
            .as_u64()
            .filter(|v| *v <= 10)
            .unwrap_or(5) as usize;
        self.sound = self.volume != 0;
        self.music_volume = settings["music_volume"]
            .as_u64()
            .filter(|v| *v <= 10)
            .map_or(self.volume, |v| v as usize);
        self.speed = settings["speed"]
            .as_u64()
            .filter(|v| (1..=11).contains(v))
            .unwrap_or(5) as usize;
        self.music_enabled = settings["music_enabled"].as_bool().unwrap_or(true);
        if let Some(values) = settings["playlist"].as_array().filter(|v| v.len() == 13) {
            for (target, value) in self.playlist.iter_mut().zip(values) {
                *target = value.as_bool().unwrap_or(true);
            }
        }
        Ok(())
    }
    pub fn command(&self, command: Command) -> Command {
        Command::WithPreferences {
            command: Box::new(command),
            preferences: centurions_core::CommandPreferences {
                auto_refit: self.native_preferences.enabled(15),
                suppress_round_message_sound: self.native_preferences.enabled(13),
                animation_delay: self.speed as i16 * 5 - 5,
                small_combat_field: !self.native_preferences.enabled(22),
                trace: self.trace,
            },
        }
    }
}
