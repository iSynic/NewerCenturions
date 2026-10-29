//! Menu actions workflow ownership.
use super::{Application, ApplicationAction, HandleOutcome, Screen};
use crate::presentation::Action;
use centurions_content::Content;
use centurions_core::{Command, Event, Session, apply};
use macroquad::prelude::*;

impl Application {
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
                self.menu = None;
                self.submenu = None;
                match (id, item) {
                    (128, 1) => {
                        self.about = true;
                        events.extend([
                            Event::Sound { resource_id: 10107 },
                            Event::Sound { resource_id: 30000 },
                            Event::Delay { ticks: 20 },
                            Event::Sound { resource_id: 10141 },
                        ]);
                    }
                    (200, 4..=22) => {
                        events = apply(Command::ViewNote { index: item - 4 }, state, content)?
                    }
                    (132 | 146, _) => {
                        let index = if id == 146 {
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
                                ally: id == 146,
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
                    (136, 5) => {
                        self.characters.creation =
                            Some(crate::creation::Creation::new(state, content));
                        events = apply(Command::CreationDialog, state, content)?;
                        events.push(Event::Feedback(
                            centurions_core::feedback::NativeFeedback::Tutorial {
                                first: 125,
                                last: 125,
                            },
                        ));
                    }
                    (136, 1) => {
                        self.selection.order = Some(Vec::new());
                        events.push(Event::Feedback(
                            centurions_core::feedback::NativeFeedback::Warning { index: 33 },
                        ));
                        events.push(Event::Feedback(
                            centurions_core::feedback::NativeFeedback::Tutorial {
                                first: 105,
                                last: 105,
                            },
                        ));
                    }
                    (136, 7) => {
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
                    (130, 3) => self.selection.end_question = true,
                    (136, 2 | 3) => {
                        self.characters.appearance =
                            Some(crate::appearance::Appearance::new(state, item == 2));
                        events.push(Event::Feedback(
                            centurions_core::feedback::NativeFeedback::Tutorial {
                                first: if item == 2 { 129 } else { 130 },
                                last: if item == 2 { 129 } else { 130 },
                            },
                        ));
                    }
                    (136, 8) => {
                        if self.editing {
                            self.editing = false;
                            events.push(Event::Feedback(
                                centurions_core::feedback::NativeFeedback::Flash {
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
                    (129, 1 | 2) => {
                        next = Some(Action::Load);
                    }
                    (129, 3) => next = Some(Action::Save),
                    (129, 4) => next = Some(Action::FastSave),
                    (129, 6) => {
                        if self.screen == Screen::Title {
                            self.quit = true;
                        } else {
                            self.quit_question = true;
                            events.push(Event::Sound { resource_id: 20005 });
                        }
                    }
                    (130, 1) => {
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
                    (130, 2) | (200, 1) => self.open_intro(self.screen, None),
                    (130, 5..=254) => {
                        self.selected_scenario = Some(
                            self.scenario_names
                                .get(item - 5)
                                .ok_or("scenario menu entry missing")?
                                .clone(),
                        );
                    }
                    (137, 3) => {
                        self.preferences_dialog =
                            Some(crate::preferences::Dialog::new(&self.native_preferences))
                    }
                    (137, 4) => {}
                    (137, 5) => {
                        self.help.resume()?;
                        self.help_reset_question = true;
                    }
                    (145, 1) => self.music_enabled = !self.music_enabled,
                    (145, 10..=22) => {
                        if self.playlist.iter().any(|enabled| *enabled) {
                            self.music_enabled = true;
                            self.music_request = Some(20128 + item as i16 - 10);
                        } else {
                            self.music_enabled = false;
                            self.music_request = None;
                            events.push(Event::Feedback(
                                centurions_core::feedback::NativeFeedback::Warning { index: 155 },
                            ));
                        }
                    }
                    (134, 1..=11) => self.speed = item,
                    (135, 2..=12) => {
                        self.volume = item - 2;
                        self.sound = self.volume != 0;
                        events.push(Event::Sound { resource_id: 642 });
                    }
                    (135, 16..=26) => self.music_volume = item - 16,
                    _ => return Err("This original menu action has not been reconstructed.".into()),
                }
            }
            _ => unreachable!("application action routed to wrong screen"),
        }
        Ok(HandleOutcome::Complete(next, events))
    }
}
