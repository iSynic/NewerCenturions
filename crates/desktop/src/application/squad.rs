//! Squad workflow ownership.
use super::{Application, ApplicationAction, HandleOutcome, Screen};
use crate::{
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_content::Content;
use centurions_core::{Command, Event, Phase, Session, apply};
use macroquad::prelude::*;

impl Application {
    pub(super) fn handle_squad(
        &mut self,
        action: ApplicationAction,
        state: &mut Session,
        content: &Content,
    ) -> Result<HandleOutcome, centurions_core::RuleError> {
        let mut next = None;
        let mut events = Vec::new();
        match action {
            ApplicationAction::EndAnswer(answer) => {
                if !self.selection.end_question {
                    return Err("Mission ending question is not open.".into());
                }
                if state.phase == Phase::Combat && !answer {
                    self.selection.end_question = false;
                } else if answer {
                    self.finish_mission(state, content)?;
                } else {
                    self.selection.end_question = false;
                    self.end_after_save = true;
                    next = Some(Action::Save);
                }
            }
            ApplicationAction::EndSaved => self.finish_mission(state, content)?,
            ApplicationAction::SquadCancel => {
                self.screen = if self.selection.modify_squad {
                    Screen::Game
                } else {
                    Screen::Title
                };
                self.selection.modify_squad = false;
            }
            ApplicationAction::OrderCancel => self.selection.order = None,
            ApplicationAction::OrderMember(member) => {
                let order = self
                    .selection
                    .order
                    .as_mut()
                    .ok_or("Squad ordering is not open.")?;
                if member >= state.warriors.len() {
                    return Err("Ordering member outside squad.".into());
                }
                if let Some(i) = order.iter().position(|m| *m == member) {
                    order.remove(i);
                    events.push(Event::Sound { resource_id: 144 });
                } else {
                    order.push(member);
                    events.push(Event::Sound { resource_id: 141 });
                }
                if order.len() == state.warriors.len() {
                    let mut destinations = vec![0; order.len()];
                    for (i, member) in order.iter().enumerate() {
                        destinations[*member] = order.len() - i - 1;
                    }
                    events.extend(apply(
                        Command::ReorderSquad { destinations },
                        state,
                        content,
                    )?);
                    self.selection.order = None;
                }
            }
            ApplicationAction::Difficulty(level) => {
                if self.screen != Screen::Squad
                    || self.selection.modify_squad
                    || !(-2..=2).contains(&level)
                {
                    return Err("Difficulty selection is unavailable.".into());
                }
                self.selection.difficulty = level;
                events.push(Event::Sound { resource_id: 144 });
            }
            ApplicationAction::Warrior(i) => {
                self.selection.selected = Some(i);
                self.selection.member = None;
            }
            ApplicationAction::Member(i) => self.selection.member = Some(i),
            ApplicationAction::Add => {
                if let Some(i) = self.selection.selected
                    && self.squad.len() < 6
                    && !self.squad.contains(&i)
                {
                    self.squad.push(i);
                    self.selection.member = Some(self.squad.len() - 1);
                    events.push(Event::Sound { resource_id: 141 });
                }
            }
            ApplicationAction::Remove => {
                if let Some(i) = self.selection.member.take()
                    && i < self.squad.len()
                {
                    self.squad.remove(i);
                }
            }
            ApplicationAction::Done => {
                let warriors = self
                    .squad
                    .iter()
                    .map(|i| {
                        self.warriors(content)
                            .get(*i)
                            .map(|record| {
                                if self.selection.modify_squad {
                                    state
                                        .warriors
                                        .iter()
                                        .find(|w| w.original_sha256 == record.original_sha256)
                                        .unwrap_or(record)
                                        .clone()
                                } else {
                                    record.clone()
                                }
                            })
                            .ok_or("Squad roster index missing.")
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if self.selection.modify_squad {
                    if warriors.is_empty() {
                        return Err(centurions_core::RuleError::Refused {
                            feedback: vec![centurions_core::feedback::NativeFeedback::Sound {
                                resource_id: 6000,
                            }],
                        });
                    }
                    events = apply(Command::ReplaceSquad { warriors }, state, content)?;
                    self.selection.modify_squad = false;
                    self.screen = Screen::Game;
                } else if self.squad.is_empty() {
                    self.screen = Screen::Title;
                } else {
                    let mut candidate = Session::new(content);
                    candidate.random = state.random.clone();
                    candidate.library_random_seed = state.library_random_seed;
                    candidate.library_random_draws = state.library_random_draws;
                    apply(
                        Command::SetDifficulty {
                            level: self.selection.difficulty,
                        },
                        &mut candidate,
                        content,
                    )
                    .map_err(|error| error.to_string())?;
                    events = apply(
                        Command::SelectWarriors { warriors },
                        &mut candidate,
                        content,
                    )
                    .and_then(|mut events| {
                        events.extend(apply(
                            Command::EnterMission {
                                name: self
                                    .selected_scenario
                                    .clone()
                                    .or_else(|| content.scenario.as_ref().map(|s| s.name.clone()))
                                    .ok_or("no starting scenario")?,
                            },
                            &mut candidate,
                            content,
                        )?);
                        Ok(events)
                    })
                    .map_err(|e| e.to_string())?;
                    *state = candidate;
                    self.screen = Screen::Game;
                }
            }
            _ => unreachable!("application action routed to wrong screen"),
        }
        Ok(HandleOutcome::Complete(next, events))
    }

    pub fn load_roster(
        &mut self,
        path: &std::path::Path,
        content: &Content,
    ) -> Result<(), centurions_core::RuleError> {
        self.scenario_names = content.scenario_names();
        if let Some(map) = content.map(0) {
            self.note_labels = map.note_labels.clone();
            self.unknown_note_labels = map.unknown_note_labels.clone();
        }
        self.archives =
            centurions_core::reference_views::archive_entries(&Session::new(content), content)?;
        self.roster = Some(centurions_core::roster::Roster::open(path, content)?);
        Ok(())
    }
    pub(super) fn warriors<'a>(
        &'a self,
        content: &'a Content,
    ) -> &'a [centurions_content::Warrior] {
        self.roster
            .as_ref()
            .map_or(&content.warriors, |r| &r.warriors)
    }
    pub(super) fn finish_mission(
        &mut self,
        state: &mut Session,
        content: &Content,
    ) -> Result<(), centurions_core::RuleError> {
        let mut next = state.clone();
        apply(Command::EndMission, &mut next, content)?;
        self.roster
            .as_mut()
            .ok_or("warrior roster missing")?
            .store(&centurions_core::roster::mission_records(state, content)?)?;
        *state = next;
        self.selection.end_question = false;
        self.end_after_save = false;
        self.screen = Screen::Title;
        Ok(())
    }

    pub(super) fn draw_squad(
        &self,
        content: &Content,
        assets: &Assets,
        controls: &mut Vec<Control>,
    ) {
        clear_background(BLACK);
        let origin = vec2(80., 80.);
        assets.panel(203, Rect::new(origin.x, origin.y, 640., 460.));
        assets.text(
            &assets.squad_text[69],
            origin.x + 7.,
            origin.y + 19.,
            12,
            WHITE,
        );
        for (i, warrior) in self
            .warriors(content)
            .iter()
            .enumerate()
            .skip(self.selection.roster_page * 13)
            .take(13)
        {
            let row = i - self.selection.roster_page * 13;
            let rect = Rect::new(origin.x + 12., origin.y + 30. + row as f32 * 20., 155., 14.);
            if self.selection.selected == Some(i) {
                draw_rectangle(rect.x, rect.y, rect.w, rect.h, DARKBLUE);
            }
            assets.text(&warrior.name, rect.x, rect.y + 12., 12, WHITE);
            assets.text(
                &warrior.record().level().unwrap_or(0).to_string(),
                origin.x + 184.,
                rect.y + 12.,
                12,
                WHITE,
            );
            controls.push(Control {
                rect,
                enabled: true,
                action: Action::Application(ApplicationAction::Warrior(i)),
            });
        }
        for (i, warrior) in self.squad.iter().enumerate() {
            let rect = Rect::new(origin.x + 331., origin.y + 50. * i as f32, 50., 50.);
            if let Ok(id) = self.warriors(content)[*warrior].record().portrait()
                && let Some(texture) = assets.portraits.get(&id)
            {
                draw_texture(texture, rect.x, rect.y, WHITE);
            }
            assets.text(
                &self.warriors(content)[*warrior].name,
                rect.x + 59.,
                rect.y + 23.,
                12,
                WHITE,
            );
            if self.selection.member == Some(i) {
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1., YELLOW);
            }
            controls.push(Control {
                rect,
                enabled: true,
                action: Action::Application(ApplicationAction::Member(i)),
            });
        }
        let add = self
            .selection
            .selected
            .is_some_and(|i| !self.squad.contains(&i))
            && self.squad.len() < 6;
        for (id, y, enabled, action) in [
            (160, 22., add, ApplicationAction::Add),
            (
                7,
                72.,
                self.selection.member.is_some(),
                ApplicationAction::Remove,
            ),
            (158, 122., false, ApplicationAction::Remove),
            (168, 222., true, ApplicationAction::Done),
            (183, 272., false, ApplicationAction::Add),
        ] {
            let rect = Rect::new(origin.x + 221., origin.y + y, 50., 50.);
            assets.panel(id, rect);
            if !enabled {
                draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0., 0., 0., 0.55));
            }
            controls.push(Control {
                rect,
                enabled,
                action: Action::Application(action),
            });
        }
        assets.panel(132, Rect::new(origin.x + 221., origin.y + 172., 50., 50.));
        draw_rectangle(
            origin.x + 221.,
            origin.y + 172.,
            50.,
            50.,
            Color::new(0., 0., 0., 0.55),
        );
        assets.text(
            assets.squad_text[66].trim_end(),
            origin.x + 233.,
            origin.y + 360.,
            12,
            YELLOW,
        );
        for (i, name) in assets.squad_text[67].lines().enumerate() {
            assets.text(
                name,
                origin.x + 268.,
                origin.y + 375. + i as f32 * 16.,
                12,
                WHITE,
            );
            controls.push(Control {
                rect: Rect::new(origin.x + 241., origin.y + 364. + i as f32 * 16., 130., 16.),
                enabled: !self.selection.modify_squad,
                action: Action::Application(ApplicationAction::Difficulty(i as i16 - 2)),
            });
        }
        assets.panel(
            179,
            Rect::new(
                origin.x + 241.,
                origin.y + 364. + f32::from(self.selection.difficulty + 2) * 16.,
                13.,
                13.,
            ),
        );
        assets.wrapped_color(
            &assets.squad_text[68],
            Rect::new(origin.x + 12., origin.y + 311., 202., 119.),
            12,
            16.,
            YELLOW,
        );
    }
}
