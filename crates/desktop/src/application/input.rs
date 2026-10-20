//! Input workflow ownership.
use super::{Application, ApplicationAction, Screen, command_modifier};
use crate::{
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_core::Session;
use macroquad::prelude::*;

impl Application {
    pub fn input(
        &mut self,
        state: &Session,
        assets: &Assets,
        controls: &[Control],
        pointer: Option<Vec2>,
    ) -> Option<Action> {
        if self.about {
            if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) {
                return Some(Action::Application(ApplicationAction::CloseAbout));
            }
            return pointer
                .and_then(|point| {
                    controls
                        .iter()
                        .find(|control| control.enabled && control.rect.contains(point))
                })
                .filter(|_| is_mouse_button_pressed(MouseButton::Left))
                .map(|control| control.action.clone());
        }
        if self.exchange_offer.is_some() {
            if is_key_pressed(KeyCode::Escape) {
                return Some(Action::Application(ApplicationAction::ExchangeChoice(None)));
            }
            if self.exchange_swap && is_key_pressed(KeyCode::S) {
                return Some(Action::Application(ApplicationAction::ExchangeChoice(
                    Some(true),
                )));
            }
            if is_key_pressed(KeyCode::A) {
                return Some(Action::Application(ApplicationAction::ExchangeChoice(
                    Some(false),
                )));
            }
            if is_mouse_button_pressed(MouseButton::Left)
                && let Some(pointer) = pointer
            {
                return controls
                    .iter()
                    .rev()
                    .find(|control| control.enabled && control.rect.contains(pointer))
                    .map(|control| control.action.clone());
            }
            return None;
        }
        if let Some(note) = &mut self.note {
            note.scroll = (note.scroll + mouse_wheel().1 * 14.).clamp(0., 70.);
            if is_key_pressed(KeyCode::Escape)
                || is_key_pressed(KeyCode::Enter)
                || is_mouse_button_pressed(MouseButton::Left)
            {
                return Some(Action::Application(ApplicationAction::CloseNote));
            }
            return None;
        }
        if self.creature.is_some() {
            if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) {
                return Some(Action::Application(ApplicationAction::CreatureItem(4)));
            }
            if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Right) {
                return Some(Action::Application(ApplicationAction::CreatureItem(
                    if is_key_pressed(KeyCode::Left) { 2 } else { 3 },
                )));
            }
            if is_mouse_button_pressed(MouseButton::Left)
                && let Some(pointer) = pointer
            {
                return controls
                    .iter()
                    .rev()
                    .find(|c| c.enabled && c.rect.contains(pointer))
                    .map(|c| c.action.clone());
            }
            return None;
        }
        if let Some(menu) = self.menu {
            let count = self.menu_items(menu, state, assets).items.len();
            let wheel = mouse_wheel().1;
            if wheel > 0. || is_key_pressed(KeyCode::PageUp) {
                self.menu_scroll = self
                    .menu_scroll
                    .saturating_sub(if wheel > 0. { 3 } else { 20 });
            } else if wheel < 0. || is_key_pressed(KeyCode::PageDown) {
                self.menu_scroll = (self.menu_scroll + if wheel < 0. { 3 } else { 20 })
                    .min(count.saturating_sub(20));
            }
        } else {
            self.menu_scroll = 0;
        }
        if let Some(dialog) = &self.preferences_dialog {
            if is_key_pressed(KeyCode::Escape) {
                return Some(Action::Application(ApplicationAction::PreferenceCancel));
            }
            if is_key_pressed(KeyCode::Enter) {
                return Some(Action::Application(ApplicationAction::PreferenceItem(2)));
            }
            if is_mouse_button_pressed(MouseButton::Left)
                && let Some(p) = pointer
            {
                return controls
                    .iter()
                    .rev()
                    .find(|c| c.enabled && c.rect.contains(p))
                    .map(|c| c.action.clone());
            }
            let _ = dialog;
            return None;
        }
        if self.screen == Screen::Squad && self.help.page.is_none() {
            if is_key_pressed(KeyCode::Escape) {
                return Some(Action::Application(ApplicationAction::SquadCancel));
            }
            let wheel = mouse_wheel().1;
            if wheel != 0. {
                self.selection.roster_page = if wheel > 0. {
                    self.selection.roster_page.saturating_sub(1)
                } else {
                    (self.selection.roster_page + 1).min(
                        self.roster
                            .as_ref()
                            .map_or(0, |r| r.warriors.len().saturating_sub(1) / 13),
                    )
                };
            }
        }
        if self.help.page.is_some() {
            if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) {
                return Some(Action::Application(ApplicationAction::Help(3)));
            }
            if is_mouse_button_pressed(MouseButton::Left)
                && let Some(p) = pointer
            {
                return controls
                    .iter()
                    .rev()
                    .find(|control| {
                        control.rect.contains(p)
                            && matches!(
                                control.action,
                                Action::Application(ApplicationAction::Help(_))
                            )
                    })
                    .map(|control| control.action.clone());
            }
            return None;
        }
        if let Some(creation) = &mut self.characters.creation
            && self.characters.editor.is_none()
            && creation.stage != crate::creation::Stage::Keep
        {
            if is_key_pressed(KeyCode::Escape) {
                return Some(Action::Application(ApplicationAction::CreationCancel));
            }
            if creation.stage == crate::creation::Stage::Name {
                while let Some(character) = get_char_pressed() {
                    if !character.is_control() && creation.request.name.chars().count() < 30 {
                        creation.request.name.push(character);
                    }
                }
                if is_key_pressed(KeyCode::Backspace) {
                    creation.request.name.pop();
                }
            }
            if is_key_pressed(KeyCode::Enter) {
                let item = match creation.stage {
                    crate::creation::Stage::Race | crate::creation::Stage::Mos => 56,
                    crate::creation::Stage::Gender => 6,
                    crate::creation::Stage::Level => 29,
                    crate::creation::Stage::Name => 4,
                    crate::creation::Stage::Portrait | crate::creation::Stage::Icon => 2,
                    _ => 66,
                };
                return Some(Action::Application(ApplicationAction::CreationItem(item)));
            }
            if is_mouse_button_pressed(MouseButton::Left)
                && let Some(p) = pointer
            {
                return controls
                    .iter()
                    .rev()
                    .find(|c| c.enabled && c.rect.contains(p))
                    .map(|c| c.action.clone());
            }
            return None;
        }
        if self.characters.appearance.is_some() {
            if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) {
                return Some(Action::Application(ApplicationAction::AppearanceItem(2)));
            }
            if is_mouse_button_pressed(MouseButton::Left)
                && let Some(p) = pointer
            {
                return controls
                    .iter()
                    .rev()
                    .find(|c| c.enabled && c.rect.contains(p))
                    .map(|c| c.action.clone());
            }
            return None;
        }
        if self.selection.order.is_some() {
            if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::A) {
                return Some(Action::Application(ApplicationAction::OrderCancel));
            }
            if is_mouse_button_pressed(MouseButton::Left)
                && let Some(p) = pointer
            {
                return controls
                    .iter()
                    .rev()
                    .find(|c| c.enabled && c.rect.contains(p))
                    .map(|c| c.action.clone());
            }
            return None;
        }
        if self.characters.editor.is_none()
            && self.characters.item_editor.is_none()
            && let Some(information) = self.characters.information.as_ref()
        {
            if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) {
                if controls
                    .iter()
                    .any(|control| matches!(control.action, Action::Close))
                {
                    return Some(Action::Close);
                }
                return Some(Action::Application(ApplicationAction::InformationItem(
                    if information.detailed { 58 } else { 72 },
                )));
            }
            if is_mouse_button_pressed(MouseButton::Left)
                && let Some(p) = pointer
            {
                return controls
                    .iter()
                    .rev()
                    .find(|c| c.enabled && c.rect.contains(p))
                    .map(|c| c.action.clone());
            }
            return None;
        }
        if self.characters.editor.is_some() || self.characters.item_editor.is_some() {
            if self
                .characters
                .editor
                .as_ref()
                .and_then(|editor| editor.scans.as_ref())
                .is_some_and(|scan| scan.question)
            {
                if is_key_pressed(KeyCode::Y) {
                    return Some(Action::Application(ApplicationAction::ScanAnswer(true)));
                }
                if is_key_pressed(KeyCode::N) || is_key_pressed(KeyCode::Escape) {
                    return Some(Action::Application(ApplicationAction::ScanAnswer(false)));
                }
            }
            if is_key_pressed(KeyCode::Escape) {
                return Some(Action::Application(
                    if self.characters.item_editor.is_some() {
                        ApplicationAction::ItemEditorItem(2)
                    } else {
                        ApplicationAction::EditorItem(1)
                    },
                ));
            }
            if is_mouse_button_pressed(MouseButton::Left)
                && let Some(p) = pointer
            {
                return controls
                    .iter()
                    .rev()
                    .find(|c| c.enabled && c.rect.contains(p))
                    .map(|c| c.action.clone());
            }
            return None;
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
            || self.escape_question
        {
            let answer = |answer| {
                Action::Application(if keeping {
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
                })
            };
            if is_key_pressed(KeyCode::Y) && !self.escape_question
                || self.escape_question && is_key_pressed(KeyCode::E)
            {
                return Some(answer(true));
            }
            if is_key_pressed(KeyCode::N) && !self.escape_question
                || is_key_pressed(KeyCode::Escape)
                || self.escape_question && is_key_pressed(KeyCode::S)
            {
                return Some(answer(false));
            }
            if is_mouse_button_pressed(MouseButton::Left)
                && let Some(p) = pointer
            {
                return controls
                    .iter()
                    .rev()
                    .find(|control| {
                        control.rect.contains(p)
                            && matches!(
                                control.action,
                                Action::Application(
                                    ApplicationAction::QuitAnswer(_)
                                        | ApplicationAction::HelpResetAnswer(_)
                                        | ApplicationAction::EditorAnswer(_)
                                        | ApplicationAction::EndAnswer(_)
                                        | ApplicationAction::EscapeAnswer(_)
                                        | ApplicationAction::CreationAnswer(_)
                                )
                            )
                    })
                    .map(|control| control.action.clone());
            }
            return None;
        }
        if self.screen == Screen::Logo {
            return (is_mouse_button_pressed(MouseButton::Left)
                || is_mouse_button_pressed(MouseButton::Right)
                || is_mouse_button_pressed(MouseButton::Middle))
            .then_some(Action::Application(ApplicationAction::SkipLogo));
        }
        if self.screen == Screen::Intro {
            if is_mouse_button_pressed(MouseButton::Left)
                && let Some(p) = pointer
            {
                if get_time() - self.intro.intro_click < 0.5 {
                    return Some(Action::Application(ApplicationAction::CloseIntro));
                }
                self.intro.drag = Some(p);
            }
            if is_mouse_button_down(MouseButton::Left)
                && let (Some(last), Some(p)) = (self.intro.drag, pointer)
            {
                if p.y != last.y {
                    self.intro.intro_y += (p.y - last.y).signum() * 25.;
                }
                self.intro.drag = Some(p);
            }
            if is_mouse_button_released(MouseButton::Left) {
                self.intro.intro_click = get_time();
                self.intro.drag = None;
            }
            return None;
        }
        if command_modifier() {
            for key in get_keys_pressed() {
                let letter = key as u32;
                if !(65..=90).contains(&letter) {
                    continue;
                }
                let letter = letter as u8;
                if is_key_pressed(key) {
                    for id in &assets.menu_bar {
                        if let Some(index) = assets.menus[id]
                            .items
                            .iter()
                            .position(|item| item.key.to_ascii_uppercase() == letter)
                            && self.supported(*id, index + 1, state)
                        {
                            return Some(Action::Application(ApplicationAction::Menu(
                                *id,
                                index + 1,
                            )));
                        }
                    }
                }
            }
        }
        if is_key_pressed(KeyCode::Escape) {
            self.menu = None;
            self.submenu = None;
        }
        if let Some(p) = pointer {
            if p.y < 20. {
                let selected = self
                    .headings(assets)
                    .iter()
                    .find(|(_, r)| r.contains(p))
                    .map(|(id, _)| *id);
                if is_mouse_button_pressed(MouseButton::Left) {
                    self.menu = if selected == self.menu {
                        None
                    } else {
                        selected
                    };
                    self.submenu = None;
                    return None;
                }
                if self.menu.is_some() && selected.is_some() && selected != self.menu {
                    self.menu = selected;
                    self.submenu = None;
                    return None;
                }
            }
            if self.menu.is_some()
                && let Some(control) = controls
                    .iter()
                    .rev()
                    .find(|control| control.rect.contains(p))
                && let Action::Application(ApplicationAction::Menu(id, item)) = control.action
            {
                if let Some(sub) = assets.menus[&id].items[item - 1].submenu_id() {
                    self.submenu = Some(sub);
                    return None;
                }
                if is_mouse_button_pressed(MouseButton::Left) {
                    if control.enabled
                        && id == 145
                        && item >= 10
                        && (is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift))
                    {
                        return Some(Action::Application(ApplicationAction::Playlist(item - 10)));
                    }
                    return control.enabled.then(|| control.action.clone());
                }
            }
            if is_mouse_button_pressed(MouseButton::Left) {
                if self.menu.is_some() {
                    self.menu = None;
                    self.submenu = None;
                    return None;
                }
                if self.screen != Screen::Game {
                    return controls
                        .iter()
                        .rev()
                        .find(|control| control.rect.contains(p) && control.enabled)
                        .map(|control| control.action.clone());
                }
            }
        }
        None
    }
}
