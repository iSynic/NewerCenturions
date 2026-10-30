//! What each menu item does when chosen.
use super::{Application, ApplicationAction, HandleOutcome, Screen};
use crate::presentation::Action;
use centurions_content::Content;
use centurions_core::{Command, Event, Session, apply};
use macroquad::prelude::*;

impl Application {
    /// Carries out a chosen menu item, refusing items that are not available now.
    pub(super) fn handle_menu_actions(
        &mut self,
        action: ApplicationAction,
        state: &mut Session,
        content: &Content,
    ) -> Result<HandleOutcome, centurions_core::RuleError> {
        let mut next = None;
        let mut events = Vec::new();
        match action {
            ApplicationAction::Menu(id, item) => {
                if !self.supported(id, item, state) {
                    return Err(centurions_core::RuleError::Refused { feedback: vec![] });
                }
                self.menu_bar.menu = None;
                self.menu_bar.submenu = None;
                use super::menus::menu::{
                    self, apple, character, game, game_speed, maps_notes, mission, music,
                    preferences, volume,
                };
                match (id, item) {
                    (menu::APPLE, apple::ABOUT) => {
                        self.about = true;
                        events.extend([
                            Event::Sound { resource_id: 10107 },
                            Event::Sound { resource_id: 30000 },
                            Event::Delay { ticks: 20 },
                            Event::Sound { resource_id: 10141 },
                        ]);
                    }
                    (menu::MAPS_NOTES, maps_notes::FIRST_NOTE..=maps_notes::LAST_NOTE) => {
                        events = apply(
                            Command::ViewNote {
                                index: item - maps_notes::FIRST_NOTE,
                            },
                            state,
                            content,
                        )?
                    }
                    (menu::ARCHIVES | menu::ALLIES, _) => {
                        let index = if id == menu::ALLIES {
                            item - 1
                        } else {
                            self.archives
                                .get(item - 1)
                                .ok_or("archive selection unavailable")?
                                .0
                        };
                        events = apply(
                            Command::ViewCreature {
                                index,
                                ally: id == menu::ALLIES,
                            },
                            state,
                            content,
                        )?;
                        self.creature = events.iter().find_map(|event| {
                            if let Event::CreatureViewed {
                                index,
                                ally,
                                record,
                                description,
                            } = event
                            {
                                Some(crate::creature::Creature {
                                    index: *index,
                                    ally: *ally,
                                    record: record.clone(),
                                    description: description.clone(),
                                })
                            } else {
                                None
                            }
                        });
                        events.push(Event::Sound { resource_id: 201 });
                    }
                    (menu::CHARACTER, character::GENERATE_WARRIOR) => {
                        self.characters.creation =
                            Some(crate::creation::Creation::new(state, content));
                        events = apply(Command::CreationDialog, state, content)?;
                        events.push(Event::Feedback(
                            centurions_core::feedback::GameFeedback::Tutorial {
                                first: 125,
                                last: 125,
                            },
                        ));
                    }
                    (menu::CHARACTER, character::SQUAD_ORDER) => {
                        self.selection.order = Some(Vec::new());
                        events.push(Event::Feedback(
                            centurions_core::feedback::GameFeedback::Warning { index: 33 },
                        ));
                        events.push(Event::Feedback(
                            centurions_core::feedback::GameFeedback::Tutorial {
                                first: 105,
                                last: 105,
                            },
                        ));
                    }
                    (menu::CHARACTER, character::MODIFY_SQUAD) => {
                        self.roster
                            .as_mut()
                            .unwrap()
                            .store(&centurions_core::roster::mission_records(state, content)?)?;
                        self.squad = state
                            .warriors
                            .iter()
                            .map(|w| {
                                self.warriors(content)
                                    .iter()
                                    .position(|r| r.original_sha256 == w.original_sha256)
                                    .ok_or("Squad warrior missing from roster.")
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        self.selection.modify_squad = true;
                        self.screen = Screen::Squad;
                        self.selection.selected = None;
                        self.selection.member = None;
                    }
                    (menu::MISSION, mission::END) => self.selection.end_question = true,
                    (menu::CHARACTER, character::CHANGE_PORTRAIT | character::CHANGE_ICON) => {
                        self.characters.appearance = Some(crate::appearance::Appearance::new(
                            state,
                            item == character::CHANGE_PORTRAIT,
                        ));
                        events.push(Event::Feedback(
                            centurions_core::feedback::GameFeedback::Tutorial {
                                first: if item == character::CHANGE_PORTRAIT {
                                    129
                                } else {
                                    130
                                },
                                last: if item == character::CHANGE_PORTRAIT {
                                    129
                                } else {
                                    130
                                },
                            },
                        ));
                    }
                    (menu::CHARACTER, character::WARRIOR_EDIT) => {
                        if self.editing {
                            self.editing = false;
                            events.push(Event::Feedback(
                                centurions_core::feedback::GameFeedback::Flash {
                                    address: 0x403686,
                                    x: 50,
                                    y: 50,
                                    wait: 0,
                                    sound: 5000,
                                },
                            ));
                        } else {
                            self.characters.editor_question = true;
                        }
                    }
                    (menu::GAME, game::LOAD | game::REVERT) => {
                        next = Some(Action::Load);
                    }
                    (menu::GAME, game::SAVE) => next = Some(Action::Save),
                    (menu::GAME, game::FAST_SAVE) => next = Some(Action::FastSave),
                    (menu::GAME, game::QUIT) => {
                        if self.screen == Screen::Title {
                            self.quit = true;
                        } else {
                            self.questions.quit = true;
                            events.push(Event::Sound { resource_id: 20005 });
                        }
                    }
                    (menu::MISSION, mission::BEGIN) => {
                        self.selection.modify_squad = false;
                        self.selection.roster_top = 0;
                        self.selection.difficulty = state.difficulty;
                        self.screen = Screen::Squad;
                        self.squad.clear();
                        self.selection.selected = None;
                        self.selection.member = None;
                        self.help.open(121, 124)?;
                        events.push(Event::Sound { resource_id: 10004 });
                    }
                    (menu::MISSION, mission::BRIEFING)
                    | (menu::MAPS_NOTES, maps_notes::BRIEFING) => {
                        self.open_intro(self.screen, None)
                    }
                    (menu::MISSION, mission::FIRST_SCENARIO..=mission::LAST_SCENARIO) => {
                        self.selected_scenario = Some(
                            self.scenario_names
                                .get(item - mission::FIRST_SCENARIO)
                                .ok_or("scenario menu entry missing")?
                                .clone(),
                        );
                    }
                    (menu::PREFERENCES, preferences::SET_PREFERENCES) => {
                        self.preferences_dialog =
                            Some(crate::preferences::Dialog::new(&self.preferences))
                    }
                    (menu::PREFERENCES, preferences::REFRESH_SCREEN) => {}
                    (menu::PREFERENCES, preferences::START_POPUP_HELP) => {
                        self.help.resume()?;
                        self.questions.help_reset = true;
                    }
                    (menu::MUSIC, music::STOP) => {
                        self.audio.music_enabled = !self.audio.music_enabled
                    }
                    (menu::MUSIC, music::FIRST_TITLE..=music::LAST_TITLE) => {
                        if self.audio.playlist.iter().any(|enabled| *enabled) {
                            self.audio.music_enabled = true;
                            self.audio.music_request = Some(20128 + item as i16 - 10);
                        } else {
                            self.audio.music_enabled = false;
                            self.audio.music_request = None;
                            events.push(Event::Feedback(
                                centurions_core::feedback::GameFeedback::Warning { index: 155 },
                            ));
                        }
                    }
                    (menu::GAME_SPEED, game_speed::FASTEST..=game_speed::SLOWEST) => {
                        self.speed = item
                    }
                    (menu::VOLUME, volume::SOUND_SILENT..=volume::SOUND_LOUDEST) => {
                        self.audio.volume = item - volume::SOUND_SILENT;
                        self.audio.sound = self.audio.volume != 0;
                        events.push(Event::Sound { resource_id: 642 });
                    }
                    (menu::VOLUME, volume::MUSIC_SILENT..=volume::MUSIC_LOUDEST) => {
                        self.audio.music_volume = item - volume::MUSIC_SILENT
                    }
                    _ => return Err("This original menu action has not been reconstructed.".into()),
                }
            }
            _ => unreachable!("application action routed to wrong screen"),
        }
        Ok(HandleOutcome::Complete(next, events))
    }
}
