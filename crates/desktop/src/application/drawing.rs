//! Draws the current screen, its dialogs and the yes/no questions over it.
use super::{Application, ApplicationAction, Screen};
use crate::{
    assets::Assets,
    game_strings,
    presentation::{self, Action, Control, UiState},
};
use centurions_content::Content;
use centurions_core::{Phase, Session};
use macroquad::prelude::*;

impl Application {
    /// Draws the current screen, then the menu bar, then the topmost dialog, which
    /// takes over the controls. Returns the controls the player can click.
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
                assets.panel(19, crate::layout::DIALOG_FRAME);
            }
            Screen::Title => {
                clear_background(BLACK);
                assets.panel(187, crate::layout::DIALOG_FRAME);
            }
            Screen::Squad => self.draw_squad(content, state, assets, &mut controls),
            Screen::Intro => self.intro.draw(content, state, assets),
        }
        if self.screen != Screen::Logo && self.screen != Screen::Intro {
            self.draw_menus(state, assets, &mut controls);
        }
        if let Some(note) = &self.notes.open {
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
                crate::presentation::push_panel_control(
                    assets,
                    &mut controls,
                    234,
                    rect,
                    true,
                    Action::Application(ApplicationAction::OpenItemEditor(member, slot)),
                );
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
            assets.dialog_text(&text, assets.dialog_rect(166, 17));
            controls.push(Control {
                rect: assets.dialog_rect(166, 12),
                enabled: true,
                action: Action::Application(ApplicationAction::CloseAbout),
            });
        }
        if self.questions.exchange_offer.is_some() || state.out_of_combat_walk_choice.is_some() {
            let choices = if state
                .out_of_combat_walk_choice
                .as_ref()
                .map_or(self.questions.exchange_swap, |choice| choice.exchange)
            {
                [
                    (2, game_strings::ATTACK_FRIEND, Some(false)),
                    (1, game_strings::SWAP_POSITIONS, Some(true)),
                ]
            } else {
                [
                    (2, game_strings::DO_NOT_ATTACK, Some(false)),
                    (1, game_strings::ATTACK_YOUR_FRIEND, None),
                ]
            };
            draw_choice_dialog(
                assets,
                choices.map(|(item, address, choice)| {
                    (item, address, ApplicationAction::ExchangeChoice(choice))
                }),
                &mut controls,
            );
        }
        if self.questions.escape {
            draw_choice_dialog(
                assets,
                [
                    (
                        1,
                        game_strings::STAY_AND_FIGHT,
                        ApplicationAction::EscapeAnswer(false),
                    ),
                    (
                        2,
                        game_strings::EMBRACE_COWARDICE,
                        ApplicationAction::EscapeAnswer(true),
                    ),
                ],
                &mut controls,
            );
        }
        self.draw_question(state, assets, &mut controls);
        if let Some(page) = self.help.page {
            crate::help::draw_page(page, content, assets, &mut controls);
        }
        controls
    }

    /// The yes/no question window: keeping a created warrior, end of game,
    /// editor, help reset or quit. The escape question has its own window,
    /// drawn with the choice dialogs.
    fn draw_question(&self, state: &Session, assets: &Assets, controls: &mut Vec<Control>) {
        let keeping = self
            .characters
            .creation
            .as_ref()
            .is_some_and(|c| c.stage == crate::creation::Stage::Keep);
        if keeping
            || self.questions.quit
            || self.questions.help_reset
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
                } else if self.questions.help_reset {
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
                crate::presentation::push_panel_control(
                    assets,
                    controls,
                    id,
                    rect,
                    true,
                    Action::Application(if keeping {
                        ApplicationAction::CreationAnswer(answer)
                    } else if self.questions.escape {
                        ApplicationAction::EscapeAnswer(answer)
                    } else if self.selection.end_question {
                        ApplicationAction::EndAnswer(answer)
                    } else if self.characters.editor_question {
                        ApplicationAction::EditorAnswer(answer)
                    } else if self.questions.help_reset {
                        ApplicationAction::HelpResetAnswer(answer)
                    } else {
                        ApplicationAction::QuitAnswer(answer)
                    }),
                );
            }
        }
    }
}

/// Dialog 136 with each item's button cleared to black and labelled from the
/// game's own strings.
fn draw_choice_dialog(
    assets: &Assets,
    choices: [(usize, u32, ApplicationAction); 2],
    controls: &mut Vec<Control>,
) {
    controls.clear();
    assets.draw_dialog(136);
    for (item, address, action) in choices {
        let label = assets.game_text.text(address, true).unwrap_or_default();
        let rect = assets.dialog_rect(136, item);
        draw_rectangle(rect.x + 2., rect.y + 2., rect.w - 4., rect.h - 4., BLACK);
        assets.dialog_text(&label, rect);
        controls.push(Control {
            rect,
            enabled: true,
            action: Action::Application(action),
        });
    }
}
