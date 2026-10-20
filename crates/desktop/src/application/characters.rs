//! Characters workflow ownership.
use super::{Application, ApplicationAction, HandleOutcome, Screen};
use centurions_content::Content;
use centurions_core::{Command, Event, Phase, Session, apply};

impl Application {
    pub(super) fn handle_characters(
        &mut self,
        action: ApplicationAction,
        state: &mut Session,
        content: &Content,
    ) -> Result<HandleOutcome, centurions_core::RuleError> {
        let next = None;
        let mut events = Vec::new();
        match action {
            ApplicationAction::CreationCancel => {
                self.characters.creation = None;
                if self.characters.information.is_some() {
                    return Ok(HandleOutcome::Return(next, events));
                }
                self.characters.editor = None;
            }
            ApplicationAction::CreationChoice(choice) => {
                events = self
                    .characters
                    .creation
                    .as_mut()
                    .ok_or("Creation is not open.")?
                    .choice(choice, content)?;
            }
            ApplicationAction::CreationAnswer(answer) => {
                let creation = self
                    .characters
                    .creation
                    .as_ref()
                    .ok_or("Creation is not open.")?;
                if creation.stage != crate::creation::Stage::Keep {
                    return Err("Creation confirmation is not open.".into());
                }
                if answer {
                    self.roster
                        .as_mut()
                        .unwrap()
                        .store(&creation.draft.warriors)?;
                }
                self.characters.creation = None;
            }
            ApplicationAction::CreationItem(item) => {
                if self
                    .characters
                    .creation
                    .as_ref()
                    .is_some_and(|creation| creation.read_only)
                {
                    if matches!(item, 1 | 2 | 56) {
                        self.characters.creation = None;
                    } else if matches!(item, 57 | 58 | 74) {
                        events = self
                            .characters
                            .creation
                            .as_mut()
                            .unwrap()
                            .item(item, state, content)?;
                    }
                    return Ok(HandleOutcome::Return(next, events));
                }
                let creation = self
                    .characters
                    .creation
                    .as_mut()
                    .ok_or("Creation is not open.")?;
                events = creation.item(item, state, content)?;
                if creation.stage == crate::creation::Stage::Scans {
                    let mut editor = crate::editor::WarriorEditor::new(&creation.draft, 0)?;
                    editor.free = false;
                    events.extend(editor.handle(39, content)?);
                    self.characters.editor = Some(editor);
                }
            }
            ApplicationAction::AppearanceItem(item) => {
                let editor = self
                    .characters
                    .appearance
                    .as_mut()
                    .ok_or("Appearance dialog is not open.")?;
                if item < 3 {
                    editor.draft.validate(content)?;
                    *state = editor.draft.clone();
                    self.characters.appearance = None;
                    events.push(Event::Sound { resource_id: 141 });
                } else {
                    events = editor.handle(item, content)?;
                }
            }
            ApplicationAction::OpenItemEditor(warrior, slot) => {
                if !self.editing || self.screen != Screen::Game || state.phase != Phase::Exploration
                {
                    return Err("Item editing is unavailable.".into());
                }
                self.characters.item_editor =
                    Some(crate::editor::ItemEditor::new(state, warrior, slot)?);
            }
            ApplicationAction::ItemEditorItem(item) => {
                let editor = self
                    .characters
                    .item_editor
                    .as_mut()
                    .ok_or("Item editor is not open.")?;
                if item < 3 {
                    editor.draft.validate(content).map_err(|e| e.to_string())?;
                    *state = editor.draft.clone();
                    self.characters.item_editor = None;
                } else {
                    events = editor.handle(item, content)?;
                }
            }
            ApplicationAction::EditorAnswer(answer) => {
                self.characters.editor_question = false;
                self.editing = answer;
                if answer {
                    for address in [0x40361a, 0x4035fa, 0x4035be] {
                        events.push(Event::Feedback(
                            centurions_core::feedback::NativeFeedback::Flash {
                                address,
                                x: 50,
                                y: 50,
                                wait: 0,
                                sound: 5000,
                            },
                        ));
                    }
                }
            }
            ApplicationAction::OpenEditor(warrior) => {
                if self.screen != Screen::Game || state.phase != Phase::Exploration {
                    return Err("Warrior editing is unavailable.".into());
                }
                let mut editor = crate::editor::WarriorEditor::new(state, warrior)?;
                editor.free = self.editing;
                self.characters.editor = Some(editor);
            }
            ApplicationAction::EditorItem(item) => {
                let editor = self
                    .characters
                    .editor
                    .as_mut()
                    .ok_or("Warrior editor is not open.")?;
                if editor.level_information || editor.scans.is_some() {
                    events = editor.handle(item, content)?;
                    events.extend(self.finish_creation_scans(state, content)?);
                    return Ok(HandleOutcome::Return(next, events));
                }
                match item {
                    1 | 4 => {
                        if item == 1 {
                            editor.cancel_current()?;
                        }
                        editor.draft.validate(content).map_err(|e| e.to_string())?;
                        if let Some(creation) = &mut self.characters.creation {
                            creation.draft = editor.draft.clone();
                            creation.stage = crate::creation::Stage::Keep;
                            state.library_random_seed = creation.draft.library_random_seed;
                            state.library_random_draws = creation.draft.library_random_draws;
                        } else {
                            *state = editor.draft.clone();
                        }
                        self.characters.editor = None;
                        if let Some(info) = &mut self.characters.information {
                            info.fields = None;
                            info.attempted = false;
                            events = apply(
                                Command::ViewWarrior {
                                    warrior: info.member,
                                },
                                state,
                                content,
                            )?;
                        }
                    }
                    _ => events = editor.handle(item, content)?,
                }
            }
            ApplicationAction::ScanAnswer(answer) => {
                self.characters
                    .editor
                    .as_mut()
                    .ok_or("Warrior editor is not open.")?
                    .scan_answer(answer, content)?;
                events.extend(self.finish_creation_scans(state, content)?);
            }
            _ => unreachable!("application action routed to wrong screen"),
        }
        Ok(HandleOutcome::Complete(next, events))
    }

    pub(super) fn finish_creation_scans(
        &mut self,
        state: &mut Session,
        content: &Content,
    ) -> Result<Vec<Event>, centurions_core::RuleError> {
        if let Some(creation) = &mut self.characters.creation
            && creation.stage == crate::creation::Stage::Scans
            && let Some(editor) = &self.characters.editor
            && editor.scans.is_none()
        {
            creation.draft = editor.draft.clone();
            let events = creation.finish(state, content)?;
            self.characters.editor = None;
            return Ok(events);
        }
        Ok(vec![])
    }
}
