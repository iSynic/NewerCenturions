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
        let large = self
            .preferences
            .enabled(crate::preferences::pref::SCREEN_800_X_600);
        let placement = crate::placement::Placement::new(large);
        match self.screen {
            Screen::Game => controls = presentation::draw(content, state, assets, ui),
            Screen::Logo => {
                clear_background(BLACK);
                let logo = placement.logo();
                assets.panel(19, Rect::new(logo.x, logo.y, 640., 460.));
            }
            Screen::Title => {
                clear_background(BLACK);
                assets.panel(187, crate::layout::DIALOG_FRAME);
            }
            Screen::Squad => self.draw_squad(content, state, assets, &mut controls),
            Screen::Intro => self.intro.draw(content, state, assets, large),
        }
        if self.screen != Screen::Logo && self.screen != Screen::Intro {
            self.draw_menus(state, assets, &mut controls);
        }
        if self.selection.restrictions && self.screen == Screen::Squad {
            controls.clear();
            self.draw_restrictions(content, state, assets, &mut controls);
        } else if let Some(note) = &self.notes.open {
            controls.clear();
            note.draw(state, content, assets, placement);
        } else if let Some(creature) = &self.creature {
            controls.clear();
            creature.draw(assets, placement, &mut controls);
        } else if let Some(creation) = &self.characters.creation
            && self.characters.editor.is_none()
            && creation.stage != crate::creation::Stage::Keep
        {
            controls.clear();
            creation.draw(content, assets, placement, &mut controls);
        } else if let Some(editor) = &self.characters.appearance {
            controls.clear();
            editor.draw(assets, &mut controls);
        } else if let Some(editor) = &self.characters.item_editor {
            controls.clear();
            editor.draw(content, assets, &mut controls);
        } else if let Some(editor) = &self.characters.editor {
            controls.clear();
            editor.draw(content, assets, placement, &mut controls);
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
            // While warriors can be edited, PICT 234 covers the Items window's View button,
            // which then edits the highlighted item.
            if self.editing
                && crate::inventory::in_items_window(state, ui)
                && !ui.inventory.popup_open()
            {
                let (member, slot) = ui.inventory.highlighted_slot();
                crate::presentation::push_panel_control(
                    assets,
                    &mut controls,
                    234,
                    ui.inventory.view_button(),
                    true,
                    Action::Application(ApplicationAction::OpenItemEditor(member, slot)),
                );
            }
            // A click on the selected warrior's portrait cell views the warrior.
            for (index, _) in state.warriors.iter().enumerate().filter(|(index, _)| {
                *index == ui.selected
                    && ui.inspection.is_none()
                    && centurions_core::field_controls::free(state)
                    && !crate::targeting::active(state, ui)
            }) {
                controls.push(Control {
                    rect: crate::layout::roster_cell(index),
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
                placement,
                choices.map(|(item, address, choice)| {
                    (item, address, ApplicationAction::ExchangeChoice(choice))
                }),
                &mut controls,
            );
        }
        if self.questions.escape {
            draw_choice_dialog(
                assets,
                placement,
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
            let age = self
                .preferences
                .enabled(crate::preferences::pref::SCREEN_800_X_600)
                .then(|| self.help.page_age(get_time()));
            crate::help::draw_page(page, age, content, assets, &mut controls);
        }
        controls
    }

    /// The yes/no question window: keeping a created warrior, erasing a warrior, end
    /// of game, editor, help reset or quit. The escape question has its own window,
    /// drawn with the choice dialogs.
    ///
    /// Original: question
    fn draw_question(&self, state: &Session, assets: &Assets, controls: &mut Vec<Control>) {
        let creation_question = self.characters.creation.as_ref().and_then(|c| c.question());
        let keeping = creation_question.is_some();
        if keeping
            || self.questions.quit
            || self.questions.help_reset
            || self.characters.editor_question
            || self.selection.end_question
            || self.selection.erase_question
            || self.selection.remove_question.is_some()
        {
            let question = &assets.questions[if let Some(question) = creation_question {
                question - 1
            } else if self.selection.erase_question {
                2
            } else if self.selection.remove_question.is_some() {
                12
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
            }];
            let answer = |answer| {
                Action::Application(if keeping {
                    ApplicationAction::CreationAnswer(answer)
                } else if self.questions.escape {
                    ApplicationAction::EscapeAnswer(answer)
                } else if self.selection.erase_question {
                    ApplicationAction::EraseAnswer(answer)
                } else if self.selection.remove_question.is_some() {
                    ApplicationAction::RemoveAnswer(answer)
                } else if self.selection.end_question {
                    ApplicationAction::EndAnswer(answer)
                } else if self.characters.editor_question {
                    ApplicationAction::EditorAnswer(answer)
                } else if self.questions.help_reset {
                    ApplicationAction::HelpResetAnswer(answer)
                } else {
                    ApplicationAction::QuitAnswer(answer)
                })
            };
            crate::story_dialogs::question_window(
                assets,
                question,
                answer(true),
                answer(false),
                controls,
            );
        }
    }
}

/// The question strip (dialog 136) beside the information box, with the game's own strings as the
/// captions of its two halves: the left half's caption goes into the strip's first parameter and
/// the right half's into the second.
///
/// Original: question3
fn draw_choice_dialog(
    assets: &Assets,
    placement: crate::placement::Placement,
    choices: [(usize, u32, ApplicationAction); 2],
    controls: &mut Vec<Control>,
) {
    controls.clear();
    let origin = placement.question_strip();
    let caption = |half: usize| {
        choices
            .iter()
            .find(|(item, ..)| *item == half)
            .and_then(|(_, address, _)| assets.game_text.text(*address, true).ok())
            .unwrap_or_default()
    };
    let (left, right) = (caption(LEFT_HALF), caption(RIGHT_HALF));
    assets.draw_dialog_with_parameters(136, origin, &[&left, &right]);
    for (item, _, action) in choices {
        controls.push(Control {
            rect: assets.dialog_rect_at(136, item, origin),
            enabled: true,
            action: Action::Application(action),
        });
    }
}

/// The strip's item for its left half.
const LEFT_HALF: usize = 2;
/// The strip's item for its right half.
const RIGHT_HALF: usize = 1;
