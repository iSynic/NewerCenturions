//! Drawing workflow ownership.
use super::{Application, ApplicationAction, Screen};
use crate::{
    assets::Assets,
    presentation::{self, Action, Control, UiState},
};
use centurions_content::Content;
use centurions_core::{Phase, Session};
use macroquad::prelude::*;

impl Application {
    pub fn draw(
        &self,
        content: &Content,
        state: &Session,
        assets: &Assets,
        ui: &UiState,
    ) -> Vec<Control> {
        let mut controls = Vec::new();
        match self.screen {
            Screen::Game => controls = presentation::draw(content, state, assets, ui),
            Screen::Logo => {
                clear_background(BLACK);
                assets.panel(19, Rect::new(80., 80., 640., 460.));
            }
            Screen::Title => {
                clear_background(BLACK);
                assets.panel(187, Rect::new(80., 80., 640., 460.));
            }
            Screen::Squad => self.draw_squad(content, assets, &mut controls),
            Screen::Intro => {
                clear_background(BLACK);
                let rect = Rect::new(160., 92., 480., 416.);
                let background = &assets.movie_background;
                for y in (0..rect.h as usize).step_by(background.height() as usize) {
                    for x in (0..rect.w as usize).step_by(background.width() as usize) {
                        draw_texture_ex(
                            background,
                            rect.x + x as f32,
                            rect.y + y as f32,
                            WHITE,
                            DrawTextureParams {
                                source: Some(Rect::new(
                                    0.,
                                    0.,
                                    background.width().min(rect.w - x as f32),
                                    background.height().min(rect.h - y as f32),
                                )),
                                ..Default::default()
                            },
                        );
                    }
                }
                let id = self.intro.prologue.unwrap_or_else(|| {
                    state
                        .exploration
                        .as_ref()
                        .map_or(1000, |_| state.mission_stage.wrapping_add(1000))
                });
                let text = content
                    .map(state.exploration.as_ref().map_or(0, |p| p.map))
                    .and_then(|map| map.movies.get(&id))
                    .map(String::as_str)
                    .unwrap_or("");
                let mut y = rect.y + 16. + self.intro.intro_y;
                for paragraph in text.replace('\r', "\n").split('\n') {
                    let mut line = String::new();
                    for word in paragraph.split_whitespace() {
                        let proposed = if line.is_empty() {
                            word.to_owned()
                        } else {
                            format!("{line} {word}")
                        };
                        if !line.is_empty()
                            && measure_text(&proposed, Some(&assets.font), 24, 1.).width
                                > rect.w - 10.
                        {
                            if y >= rect.y + 16. && y <= rect.bottom() - 9. {
                                assets.text(
                                    &line,
                                    rect.x + 5.,
                                    y,
                                    24,
                                    Color::from_rgba(252, 243, 15, 255),
                                );
                            }
                            y += 25.;
                            line = word.to_owned();
                        } else {
                            line = proposed;
                        }
                    }
                    if y >= rect.y + 16. && y <= rect.bottom() - 9. {
                        assets.text(
                            &line,
                            rect.x + 5.,
                            y,
                            24,
                            Color::from_rgba(252, 243, 15, 255),
                        );
                    }
                    y += 25.;
                }
            }
        }
        if self.screen != Screen::Logo && self.screen != Screen::Intro {
            self.draw_menus(state, assets, &mut controls);
        }
        if let Some(note) = &self.note {
            controls.clear();
            note.draw(state, content, assets);
        } else if let Some(creature) = &self.creature {
            controls.clear();
            creature.draw(assets, &mut controls);
        } else if let Some(creation) = &self.characters.creation
            && self.characters.editor.is_none()
            && creation.stage != crate::creation::Stage::Keep
        {
            controls.clear();
            creation.draw(content, assets, &mut controls);
        } else if let Some(editor) = &self.characters.appearance {
            controls.clear();
            editor.draw(assets, &mut controls);
        } else if let Some(editor) = &self.characters.item_editor {
            controls.clear();
            editor.draw(content, assets, &mut controls);
        } else if let Some(editor) = &self.characters.editor {
            controls.clear();
            editor.draw(content, assets, &mut controls);
        } else if let Some(info) = &self.characters.information {
            controls.clear();
            info.draw(state, assets, &mut controls);
            if ui.inspection == Some(presentation::Inspection::Specials) {
                controls.clear();
                presentation::draw_specials(state, assets, ui, &mut controls);
            } else if crate::status_popup::active(ui) {
                controls.clear();
                crate::status_popup::draw(state, assets, ui, &mut controls);
            }
        } else if self.screen == Screen::Game && state.phase == Phase::Exploration {
            if self.editing
                && ui.inspection == Some(presentation::Inspection::Items)
                && let Some((side, slot)) = ui.inventory.selected
            {
                let member = ui.inventory.members[side];
                let rect = Rect::new(724., 558., 70., 35.);
                assets.panel(234, rect);
                controls.push(Control {
                    rect,
                    enabled: true,
                    action: Action::Application(ApplicationAction::OpenItemEditor(member, slot)),
                });
            }
            for (index, _) in state.warriors.iter().enumerate().filter(|_| {
                ui.inspection.is_none()
                    && centurions_core::field_controls::free(state)
                    && !crate::targeting::active(state, ui)
            }) {
                let mut rect = crate::layout::row(index);
                rect.w = 50.;
                controls.push(Control {
                    rect,
                    enabled: true,
                    action: Action::Application(ApplicationAction::OpenInformation(index)),
                });
            }
        }
        if let Some(order) = &self.selection.order {
            controls.clear();
            for (member, _) in state.warriors.iter().enumerate() {
                let rect = crate::layout::row(member);
                if let Some(i) = order.iter().position(|m| *m == member)
                    && let Some(icon) = assets
                        .icons
                        .get(&(2200 + (state.warriors.len() - i) as i16))
                {
                    draw_texture(icon, rect.x + 60., rect.y + 9., WHITE);
                }
                controls.push(Control {
                    rect,
                    enabled: true,
                    action: Action::Application(ApplicationAction::OrderMember(member)),
                });
            }
        }
        if let Some(dialog) = &self.preferences_dialog {
            dialog.draw(assets, &mut controls);
        }
        if self.about {
            controls.clear();
            assets.draw_dialog(166);
            if let Some(icon) = assets.icons.get(&384) {
                let rect = assets.dialog_rect(166, 3);
                draw_texture(icon, rect.x, rect.y, WHITE);
            }
            let text = assets.dialogs[&166][16]
                .text()
                .replace("^0", "")
                .replace("^1", "")
                .replace("^2", &assets.executable_version);
            assets.wrapped_color(&text, assets.dialog_rect(166, 17), 12, 14., YELLOW);
            controls.push(Control {
                rect: assets.dialog_rect(166, 12),
                enabled: true,
                action: Action::Application(ApplicationAction::CloseAbout),
            });
        }
        if self.exchange_offer.is_some() {
            controls.clear();
            assets.draw_dialog(136);
            let choices = if self.exchange_swap {
                [(2, 0x403bfa, Some(false)), (1, 0x403c08, Some(true))]
            } else {
                [(2, 0x403bd8, Some(false)), (1, 0x403be6, None)]
            };
            for (item, address, choice) in choices {
                let label = assets.native_text.text(address, true).unwrap_or_default();
                let rect = assets.dialog_rect(136, item);
                draw_rectangle(rect.x + 2., rect.y + 2., rect.w - 4., rect.h - 4., BLACK);
                assets.wrapped_color(&label, rect, 12, 14., YELLOW);
                controls.push(Control {
                    rect,
                    enabled: true,
                    action: Action::Application(ApplicationAction::ExchangeChoice(choice)),
                });
            }
        }
        if self.escape_question {
            controls.clear();
            assets.draw_dialog(136);
            for (item, address, answer) in [(1, 0x403df4, false), (2, 0x403e04, true)] {
                let rect = assets.dialog_rect(136, item);
                draw_rectangle(rect.x + 2., rect.y + 2., rect.w - 4., rect.h - 4., BLACK);
                assets.wrapped_color(
                    &assets.native_text.text(address, true).unwrap_or_default(),
                    rect,
                    12,
                    14.,
                    YELLOW,
                );
                controls.push(Control {
                    rect,
                    enabled: true,
                    action: Action::Application(ApplicationAction::EscapeAnswer(answer)),
                });
            }
        }
        let keeping = self
            .characters
            .creation
            .as_ref()
            .is_some_and(|c| c.stage == crate::creation::Stage::Keep);
        if keeping
            || self.quit_question
            || self.help_reset_question
            || self.characters.editor_question
            || self.selection.end_question
        {
            let origin = vec2(0., 20.);
            draw_texture_ex(
                &assets.panels[&214],
                origin.x,
                origin.y,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(140., 167., 358., 195.)),
                    ..Default::default()
                },
            );
            assets.panel(182, Rect::new(origin.x + 22., origin.y + 23., 320., 84.));
            assets.wrapped(
                &assets.questions[if keeping {
                    1
                } else if self.selection.end_question {
                    if state.phase == Phase::Combat { 9 } else { 0 }
                } else if self.characters.editor_question {
                    18
                } else if self.help_reset_question {
                    8
                } else if state.phase == Phase::Combat {
                    9
                } else {
                    0
                }],
                Rect::new(origin.x + 32., origin.y + 44., 294., 72.),
                14,
                18.,
            );
            for (id, x, answer) in [(2, 66., true), (4, 208., false)] {
                let rect = Rect::new(origin.x + x, origin.y + 127., 70., 35.);
                assets.panel(id, rect);
                controls.push(Control {
                    rect,
                    enabled: true,
                    action: Action::Application(if keeping {
                        ApplicationAction::CreationAnswer(answer)
                    } else if self.escape_question {
                        ApplicationAction::EscapeAnswer(answer)
                    } else if self.selection.end_question {
                        ApplicationAction::EndAnswer(answer)
                    } else if self.characters.editor_question {
                        ApplicationAction::EditorAnswer(answer)
                    } else if self.help_reset_question {
                        ApplicationAction::HelpResetAnswer(answer)
                    } else {
                        ApplicationAction::QuitAnswer(answer)
                    }),
                });
            }
        }
        if let Some(page) = self.help.page {
            let origin = vec2(5., 25.);
            draw_texture_ex(
                &assets.panels[&214],
                origin.x,
                origin.y,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(12., 18., 320., 200.)),
                    ..Default::default()
                },
            );
            draw_rectangle_lines(origin.x, origin.y, 320., 200., 1., MAGENTA);
            assets.panel(221, Rect::new(origin.x + 6., origin.y + 7., 308., 139.));
            let text = content
                .tutorials
                .get(page)
                .map(String::as_str)
                .unwrap_or("");
            let mut y = origin.y + 30.;
            let mut line = String::new();
            for word in text.split_whitespace() {
                let proposed = if line.is_empty() {
                    word.to_owned()
                } else {
                    format!("{line} {word}")
                };
                if measure_text(&proposed, Some(&assets.font), 14, 1.).width > 286.
                    && !line.is_empty()
                {
                    draw_text_ex(
                        &line,
                        origin.x + 17.,
                        y,
                        TextParams {
                            font: Some(&assets.font),
                            font_size: 14,
                            color: YELLOW,
                            ..Default::default()
                        },
                    );
                    y += 16.;
                    line.clear();
                }
                if !line.is_empty() {
                    line.push(' ');
                }
                line.push_str(word);
            }
            draw_text_ex(
                &line,
                origin.x + 17.,
                y,
                TextParams {
                    font: Some(&assets.font),
                    font_size: 14,
                    color: YELLOW,
                    ..Default::default()
                },
            );
            for (id, rect, items) in [
                (168, Rect::new(196., 146., 50., 50.), vec![(3, 0., 50.)]),
                (
                    142,
                    Rect::new(264., 146., 50., 50.),
                    vec![(4, 0., 25.), (5, 25., 25.)],
                ),
                (180, Rect::new(6., 152., 170., 31.), vec![(6, 0., 170.)]),
            ] {
                let rect = Rect::new(origin.x + rect.x, origin.y + rect.y, rect.w, rect.h);
                assets.panel(id, rect);
                for (item, x, width) in items {
                    controls.push(Control {
                        rect: Rect::new(rect.x + x, rect.y, width, rect.h),
                        enabled: true,
                        action: Action::Application(ApplicationAction::Help(item)),
                    });
                }
            }
        }
        controls
    }
}
