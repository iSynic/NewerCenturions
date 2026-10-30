//! Routes each frame's input to the open dialog, the menu bar or the screen.
use super::{Application, ApplicationAction, Screen, command_modifier};
use crate::{
    assets::Assets,
    presentation::{Action, Control, HitOrder, clicked, topmost},
};
use centurions_core::Session;
use macroquad::prelude::*;

/// `Some` when a modal handler consumed the frame, carrying its action if any.
type Handled = Option<Option<Action>>;

/// A left click on the topmost enabled control, as most modal dialogs accept it.
fn enabled_click(controls: &[Control], pointer: Option<Vec2>) -> Option<Action> {
    if is_mouse_button_pressed(MouseButton::Left)
        && let Some(p) = pointer
    {
        return clicked(controls, p, HitOrder::Topmost, |c| c.enabled);
    }
    None
}

impl Application {
    /// Routes this frame's input through the modal handlers in priority order,
    /// then the menu bar.
    pub fn input(
        &mut self,
        state: &Session,
        assets: &Assets,
        controls: &[Control],
        pointer: Option<Vec2>,
    ) -> Option<Action> {
        if let Some(action) = self.about_input(controls, pointer) {
            return action;
        }
        if let Some(action) = self.exchange_input(controls, pointer) {
            return action;
        }
        if let Some(action) = self.note_input() {
            return action;
        }
        if let Some(action) = self.creature_input(controls, pointer) {
            return action;
        }
        self.scroll_menu(state, assets);
        if let Some(action) = self.preferences_input(controls, pointer) {
            return action;
        }
        if let Some(action) = self.squad_input() {
            return action;
        }
        if let Some(action) = self.help_input(controls, pointer) {
            return action;
        }
        if let Some(action) = self.creation_input(controls, pointer) {
            return action;
        }
        if let Some(action) = self.appearance_input(controls, pointer) {
            return action;
        }
        if let Some(action) = self.order_input(controls, pointer) {
            return action;
        }
        if let Some(action) = self.information_input(controls, pointer) {
            return action;
        }
        if let Some(action) = self.editor_input(controls, pointer) {
            return action;
        }
        if let Some(action) = self.question_input(controls, pointer) {
            return action;
        }
        if let Some(action) = self.logo_input() {
            return action;
        }
        if let Some(action) = self.intro_input(pointer) {
            return action;
        }
        self.menu_bar_input(state, assets, controls, pointer)
    }

    /// The About box closes on Escape, Return or a click on its button.
    fn about_input(&self, controls: &[Control], pointer: Option<Vec2>) -> Handled {
        if !self.about {
            return None;
        }
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) {
            return Some(Some(Action::Application(ApplicationAction::CloseAbout)));
        }
        Some(
            pointer
                .and_then(|point| clicked(controls, point, HitOrder::First, |c| c.enabled))
                .filter(|_| is_mouse_button_pressed(MouseButton::Left)),
        )
    }

    /// The occupied-cell question: Escape declines, S swaps places when swapping is offered, A
    /// attacks the occupant, and a click answers.
    fn exchange_input(&self, controls: &[Control], pointer: Option<Vec2>) -> Handled {
        self.questions.exchange_offer.as_ref()?;
        if is_key_pressed(KeyCode::Escape) {
            return Some(Some(Action::Application(
                ApplicationAction::ExchangeChoice(None),
            )));
        }
        if self.questions.exchange_swap && is_key_pressed(KeyCode::S) {
            return Some(Some(Action::Application(
                ApplicationAction::ExchangeChoice(Some(true)),
            )));
        }
        if is_key_pressed(KeyCode::A) {
            return Some(Some(Action::Application(
                ApplicationAction::ExchangeChoice(Some(false)),
            )));
        }
        Some(enabled_click(controls, pointer))
    }

    /// A viewed note scrolls with the mouse wheel and closes on Escape, Return or a
    /// click.
    fn note_input(&mut self) -> Handled {
        let note = self.notes.open.as_mut()?;
        note.scroll = (note.scroll + mouse_wheel().1 * 14.).clamp(0., 70.);
        if is_key_pressed(KeyCode::Escape)
            || is_key_pressed(KeyCode::Enter)
            || is_mouse_button_pressed(MouseButton::Left)
        {
            return Some(Some(Action::Application(ApplicationAction::CloseNote)));
        }
        Some(None)
    }

    /// The creature page: Escape and Return close it, the arrow keys step between
    /// creatures, and a click goes to its buttons.
    fn creature_input(&self, controls: &[Control], pointer: Option<Vec2>) -> Handled {
        self.creature.as_ref()?;
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) {
            return Some(Some(Action::Application(ApplicationAction::CreatureItem(
                4,
            ))));
        }
        if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Right) {
            return Some(Some(Action::Application(ApplicationAction::CreatureItem(
                if is_key_pressed(KeyCode::Left) { 2 } else { 3 },
            ))));
        }
        Some(enabled_click(controls, pointer))
    }

    /// Scrolls an open menu by wheel or page keys; a closed menu resets its scroll.
    fn scroll_menu(&mut self, state: &Session, assets: &Assets) {
        if let Some(menu) = self.menu_bar.menu {
            let count = self.menu_items(menu, state, assets).items.len();
            let wheel = mouse_wheel().1;
            if wheel > 0. || is_key_pressed(KeyCode::PageUp) {
                self.menu_bar.scroll =
                    self.menu_bar
                        .scroll
                        .saturating_sub(if wheel > 0. { 3 } else { 20 });
            } else if wheel < 0. || is_key_pressed(KeyCode::PageDown) {
                self.menu_bar.scroll = (self.menu_bar.scroll + if wheel < 0. { 3 } else { 20 })
                    .min(count.saturating_sub(20));
            }
        } else {
            self.menu_bar.scroll = 0;
        }
    }

    /// The preferences dialog: Escape cancels, Return accepts, and clicks go to its
    /// choices.
    fn preferences_input(&self, controls: &[Control], pointer: Option<Vec2>) -> Handled {
        self.preferences_dialog.as_ref()?;
        if is_key_pressed(KeyCode::Escape) {
            return Some(Some(Action::Application(
                ApplicationAction::PreferenceCancel,
            )));
        }
        if is_key_pressed(KeyCode::Enter) {
            return Some(Some(Action::Application(
                ApplicationAction::PreferenceItem(2),
            )));
        }
        Some(enabled_click(controls, pointer))
    }

    /// The squad screen's cancel key and roster wheel; anything else falls through.
    fn squad_input(&self) -> Handled {
        if self.screen == Screen::Squad && self.help.page.is_none() {
            if is_key_pressed(KeyCode::Escape) {
                return Some(Some(Action::Application(ApplicationAction::SquadCancel)));
            }
            let wheel = mouse_wheel().1;
            if wheel != 0. {
                return Some(Some(Action::Application(ApplicationAction::RosterScroll(
                    wheel < 0.,
                ))));
            }
        }
        None
    }

    /// The popup help: Escape and Return finish it and a click goes to its buttons.
    fn help_input(&self, controls: &[Control], pointer: Option<Vec2>) -> Handled {
        self.help.page.as_ref()?;
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) {
            return Some(Some(Action::Application(ApplicationAction::Help(
                crate::help::DONE,
            ))));
        }
        if is_mouse_button_pressed(MouseButton::Left)
            && let Some(p) = pointer
        {
            return Some(clicked(controls, p, HitOrder::Topmost, |control| {
                matches!(
                    control.action,
                    Action::Application(ApplicationAction::Help(_))
                )
            }));
        }
        Some(None)
    }

    /// The new warrior screens: Escape cancels, typing fills in the name, Return presses the
    /// stage's Done, and clicks answer the current stage.
    fn creation_input(&mut self, controls: &[Control], pointer: Option<Vec2>) -> Handled {
        let editing = self.characters.editor.is_some();
        let creation = self.characters.creation.as_mut()?;
        if editing || creation.stage == crate::creation::Stage::Keep {
            return None;
        }
        if is_key_pressed(KeyCode::Escape) {
            return Some(Some(Action::Application(ApplicationAction::CreationCancel)));
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
                crate::creation::Stage::Race | crate::creation::Stage::Mos => {
                    crate::creation::RACE_DONE
                }
                crate::creation::Stage::Gender => crate::creation::GENDER_DONE,
                crate::creation::Stage::Level => crate::creation::LEVEL_DONE,
                crate::creation::Stage::Name => crate::creation::NAME_DONE,
                crate::creation::Stage::Portrait | crate::creation::Stage::Icon => {
                    crate::creation::APPEARANCE_DONE
                }
                _ => crate::creation::REVIEW_DONE,
            };
            return Some(Some(Action::Application(ApplicationAction::CreationItem(
                item,
            ))));
        }
        Some(enabled_click(controls, pointer))
    }

    /// The appearance picker closes on Escape or Return and otherwise takes clicks.
    fn appearance_input(&self, controls: &[Control], pointer: Option<Vec2>) -> Handled {
        self.characters.appearance.as_ref()?;
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) {
            return Some(Some(Action::Application(
                ApplicationAction::AppearanceItem(2),
            )));
        }
        Some(enabled_click(controls, pointer))
    }

    /// The marching order dialog: Escape or A cancels, a click picks a member.
    fn order_input(&self, controls: &[Control], pointer: Option<Vec2>) -> Handled {
        self.selection.order.as_ref()?;
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::A) {
            return Some(Some(Action::Application(ApplicationAction::OrderCancel)));
        }
        Some(enabled_click(controls, pointer))
    }

    /// The warrior information screen: Escape or Return closes it, or from the detailed
    /// statistics returns to the summary; clicks go to its buttons.
    fn information_input(&self, controls: &[Control], pointer: Option<Vec2>) -> Handled {
        if self.characters.editor.is_some() || self.characters.item_editor.is_some() {
            return None;
        }
        let information = self.characters.information.as_ref()?;
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) {
            if controls
                .iter()
                .any(|control| matches!(control.action, Action::Close))
            {
                return Some(Some(Action::Close));
            }
            return Some(Some(Action::Application(
                ApplicationAction::InformationItem(if information.detailed {
                    crate::information::DETAILED_SUMMARY
                } else {
                    crate::information::SUMMARY_CLOSE
                }),
            )));
        }
        Some(enabled_click(controls, pointer))
    }

    /// The warrior and item editors take their keyboard and click input; an open
    /// scan picker takes it first.
    fn editor_input(&self, controls: &[Control], pointer: Option<Vec2>) -> Handled {
        if self.characters.editor.is_none() && self.characters.item_editor.is_none() {
            return None;
        }
        if self
            .characters
            .editor
            .as_ref()
            .and_then(|editor| editor.scans.as_ref())
            .is_some_and(|scan| scan.question)
        {
            if is_key_pressed(KeyCode::Y) {
                return Some(Some(Action::Application(ApplicationAction::ScanAnswer(
                    true,
                ))));
            }
            if is_key_pressed(KeyCode::N) || is_key_pressed(KeyCode::Escape) {
                return Some(Some(Action::Application(ApplicationAction::ScanAnswer(
                    false,
                ))));
            }
        }
        if is_key_pressed(KeyCode::Escape) {
            return Some(Some(Action::Application(
                if self.characters.item_editor.is_some() {
                    ApplicationAction::ItemEditorItem(crate::editor::ITEM_EDITOR_CLOSE)
                } else {
                    ApplicationAction::EditorItem(1)
                },
            )));
        }
        Some(enabled_click(controls, pointer))
    }

    /// The yes/no questions: keeping a created warrior, quit, help reset, editor,
    /// end of game and escape. The yes/no questions answer only to Y and N; the
    /// escape question answers to E and S, and Escape stays.
    ///
    /// Original: question, question3
    fn question_input(&self, controls: &[Control], pointer: Option<Vec2>) -> Handled {
        let keeping = self
            .characters
            .creation
            .as_ref()
            .is_some_and(|c| c.stage == crate::creation::Stage::Keep);
        if !(keeping
            || self.questions.quit
            || self.questions.help_reset
            || self.characters.editor_question
            || self.selection.end_question
            || self.questions.escape)
        {
            return None;
        }
        let answer = |answer| {
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
            })
        };
        if is_key_pressed(KeyCode::Y) && !self.questions.escape
            || self.questions.escape && is_key_pressed(KeyCode::E)
        {
            return Some(Some(answer(true)));
        }
        if is_key_pressed(KeyCode::N) && !self.questions.escape
            || self.questions.escape
                && (is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Escape))
        {
            return Some(Some(answer(false)));
        }
        if is_mouse_button_pressed(MouseButton::Left)
            && let Some(p) = pointer
        {
            return Some(clicked(controls, p, HitOrder::Topmost, |control| {
                matches!(
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
            }));
        }
        Some(None)
    }

    /// Any mouse button skips the logo.
    fn logo_input(&self) -> Handled {
        if self.screen != Screen::Logo {
            return None;
        }
        Some(
            (is_mouse_button_pressed(MouseButton::Left)
                || is_mouse_button_pressed(MouseButton::Right)
                || is_mouse_button_pressed(MouseButton::Middle))
            .then_some(Action::Application(ApplicationAction::SkipLogo)),
        )
    }

    /// Dragging scrolls the introduction; a quick second click closes it.
    fn intro_input(&mut self, pointer: Option<Vec2>) -> Handled {
        if self.screen != Screen::Intro {
            return None;
        }
        if is_mouse_button_pressed(MouseButton::Left)
            && let Some(p) = pointer
        {
            if get_time() - self.intro.intro_click < 0.5 {
                return Some(Some(Action::Application(ApplicationAction::CloseIntro)));
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
        Some(None)
    }

    /// Menu shortcuts, menu-bar headings, open menu items and screen controls.
    fn menu_bar_input(
        &mut self,
        state: &Session,
        assets: &Assets,
        controls: &[Control],
        pointer: Option<Vec2>,
    ) -> Option<Action> {
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
            self.menu_bar.menu = None;
            self.menu_bar.submenu = None;
        }
        if let Some(p) = pointer {
            if p.y < 20. {
                let selected = self
                    .headings(assets)
                    .iter()
                    .find(|(_, r)| r.contains(p))
                    .map(|(id, _)| *id);
                if is_mouse_button_pressed(MouseButton::Left) {
                    self.menu_bar.menu = if selected == self.menu_bar.menu {
                        None
                    } else {
                        selected
                    };
                    self.menu_bar.submenu = None;
                    return None;
                }
                if self.menu_bar.menu.is_some()
                    && selected.is_some()
                    && selected != self.menu_bar.menu
                {
                    self.menu_bar.menu = selected;
                    self.menu_bar.submenu = None;
                    return None;
                }
            }
            if self.menu_bar.menu.is_some()
                && let Some(control) = topmost(controls, p)
                && let Action::Application(ApplicationAction::Menu(id, item)) = control.action
            {
                if let Some(sub) = assets.menus[&id].items[item - 1].submenu_id() {
                    self.menu_bar.submenu = Some(sub);
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
                if self.menu_bar.menu.is_some() {
                    self.menu_bar.menu = None;
                    self.menu_bar.submenu = None;
                    return None;
                }
                if self.screen != Screen::Game {
                    return clicked(controls, p, HitOrder::Topmost, |c| c.enabled);
                }
            }
        }
        None
    }
}
