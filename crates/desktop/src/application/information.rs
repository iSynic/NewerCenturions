//! Information workflow ownership.
use super::{Application, ApplicationAction, HandleOutcome};
use crate::{assets::Assets, presentation::Action};
use centurions_content::Content;
use centurions_core::{Command, Event, Session, apply};
use macroquad::prelude::*;

impl Application {
    pub(super) fn handle_information(
        &mut self,
        action: ApplicationAction,
        state: &mut Session,
        content: &Content,
    ) -> Result<HandleOutcome, centurions_core::RuleError> {
        let mut next = None;
        let mut events = Vec::new();
        match action {
            ApplicationAction::CloseNote => self.note = None,
            ApplicationAction::CreatureItem(item) => {
                let view = self
                    .creature
                    .as_ref()
                    .ok_or("creature dialog is not open")?;
                if matches!(item, 1 | 4) {
                    self.creature = None;
                    self.combat_information = false;
                    events.push(Event::Sound { resource_id: 205 });
                } else if matches!(item, 2 | 3) {
                    if self.combat_information {
                        let c = state.combat.as_ref().ok_or("combat information missing")?;
                        let count = c.enemies.records.len();
                        let index = view.index.saturating_sub(10);
                        let next = if item == 2 {
                            index.saturating_sub(1)
                        } else {
                            (index + 1).min(count.saturating_sub(1))
                        };
                        return self
                            .handle(
                                ApplicationAction::CombatInformation(next as i16 + 10),
                                state,
                                content,
                            )
                            .map(|(next, events)| HandleOutcome::Return(next, events));
                    }
                    let entries: Vec<_> = if view.ally {
                        (0..state.ally_count).collect()
                    } else {
                        self.archives.iter().map(|(id, _)| *id).collect()
                    };
                    let current = entries
                        .iter()
                        .position(|id| *id == view.index)
                        .ok_or("creature selection unavailable")?;
                    let next = if item == 2 {
                        current.saturating_sub(1)
                    } else {
                        (current + 1).min(entries.len().saturating_sub(1))
                    };
                    events = apply(
                        Command::ViewCreature {
                            index: entries[next],
                            ally: view.ally,
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
                    events.push(Event::Sound { resource_id: 202 });
                }
            }
            ApplicationAction::CombatInformation(body) => {
                events = apply(Command::ViewCombatBody { body }, state, content)?;
                if body < 10 {
                    self.characters.information =
                        Some(crate::information::WarriorInformation::new(body as usize));
                } else {
                    self.combat_information = true;
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
                }
            }
            ApplicationAction::OpenInformation(warrior) => {
                events = apply(Command::ViewWarrior { warrior }, state, content)?;
                self.characters.information =
                    Some(crate::information::WarriorInformation::new(warrior));
            }
            ApplicationAction::InformationItem(item) => {
                let member = self
                    .characters
                    .information
                    .as_ref()
                    .ok_or("Warrior information is not open.")?
                    .member;
                if self.characters.information.as_ref().unwrap().detailed {
                    let info = self.characters.information.as_mut().unwrap();
                    match item {
                        58 => {
                            info.detailed = false;
                            events.push(Event::Sound { resource_id: 214 });
                        }
                        72..=77 if item - 72 < state.warriors.len() => {
                            info.member = item - 72;
                            info.attempted = false;
                        }
                        _ => return Err("Statistics control is outside the current squad.".into()),
                    }
                    return Ok(HandleOutcome::Return(next, events));
                }
                match item {
                    1 | 72 => self.characters.information = None,
                    2..=5 => {
                        let inspection = match item {
                            2 => crate::presentation::Inspection::Conditions,
                            3 => crate::presentation::Inspection::Items,
                            4 => crate::presentation::Inspection::Specials,
                            _ => crate::presentation::Inspection::Modifiers,
                        };
                        if item == 3 {
                            self.characters.information = None;
                        } else {
                            events.push(Event::Sound {
                                resource_id: if item == 4 { 647 } else { 609 },
                            });
                        }
                        next = Some(Action::InformationInspection(member, inspection));
                    }
                    96 | 97 => {
                        self.characters.creation = Some(crate::creation::Creation::profile(
                            state,
                            content,
                            member,
                            item == 97,
                        )?);
                    }
                    100 => {
                        self.characters.information.as_mut().unwrap().detailed = true;
                        let mut presentation_random = state.random.clone();
                        events.push(Event::Sound {
                            resource_id: presentation_random.bound(2).wrapping_add(200),
                        });
                    }
                    65 | 105 => {
                        let mut editor = crate::editor::WarriorEditor::new(state, member)?;
                        editor.free = self.editing;
                        editor.enter(content)?;
                        self.characters.editor = Some(editor);
                    }
                    73..=78 | 84..=89 | 90..=95 => {
                        let warrior = item
                            - if item <= 78 {
                                73
                            } else if item <= 89 {
                                84
                            } else {
                                90
                            };
                        events = apply(Command::ViewWarrior { warrior }, state, content)?;
                        self.characters.information =
                            Some(crate::information::WarriorInformation::new(warrior));
                    }
                    _ => return Err("Character information action is not connected.".into()),
                }
            }
            _ => unreachable!("application action routed to wrong screen"),
        }
        Ok(HandleOutcome::Complete(next, events))
    }

    pub fn restore_information(&mut self, member: usize) {
        self.characters.information = Some(crate::information::WarriorInformation::new(member));
    }
    pub fn prepare_information(
        &mut self,
        state: &Session,
        assets: &Assets,
    ) -> Result<(), centurions_core::RuleError> {
        if let Some(creation) = &mut self.characters.creation
            && creation.stage == crate::creation::Stage::Review
            && !creation.fields_attempted
        {
            creation.fields_attempted = true;
            creation.fields = Some(centurions_core::character_display::information(
                &creation.draft,
                0,
                &assets.native_text,
                &assets.character_strings,
            )?);
        }
        if let Some(info) = &mut self.characters.information
            && !info.attempted
        {
            info.attempted = true;
            info.fields = Some(centurions_core::character_display::information(
                state,
                info.member,
                &assets.native_text,
                &assets.character_strings,
            )?);
        }
        Ok(())
    }
}
