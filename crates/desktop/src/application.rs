use crate::presentation::Action;
use centurions_content::Content;
use centurions_core::{Event, Phase, Presentation, Session};
use macroquad::prelude::*;

/// The full-window screens: the logo, the title, the Modify Squad screen, the
/// introduction text and the game itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Logo,
    Title,
    Squad,
    Intro,
    Game,
}

/// A choice the player makes on a screen, dialog or menu outside the map; each
/// is routed to the handler that owns that screen.
#[derive(Clone, Debug)]
pub enum ApplicationAction {
    SkipLogo,
    Menu(i16, usize),
    Warrior(usize),
    Member(usize),
    Add,
    Remove,
    EraseWarrior,
    ImportWarrior,
    ViewRestrictions,
    /// Moves the roster list five rows up or down (squad dialog items 17 and 18).
    RosterScroll(bool),
    Done,
    Difficulty(i16),
    CloseIntro,
    QuitAnswer(bool),
    Help(usize),
    HelpResetAnswer(bool),
    EditorAnswer(bool),
    EditorItem(usize),
    ScanAnswer(bool),
    OpenEditor(usize),
    OpenInformation(usize),
    CombatInformation(i16),
    EscapeAnswer(bool),
    InformationItem(usize),
    AppearanceItem(usize),
    OpenItemEditor(usize, usize),
    ItemEditorItem(usize),
    Playlist(usize),
    SquadCancel,
    OrderMember(usize),
    OrderCancel,
    EndAnswer(bool),
    EndSaved,
    CreationItem(usize),
    CreationChoice(i16),
    CreationAnswer(bool),
    CreationCancel,
    PreferenceItem(usize),
    PreferenceCancel,
    CreatureItem(usize),
    CloseNote,
    ExchangeChoice(Option<bool>),
    CloseAbout,
}

/// The player's place in the game outside the rule state: the open screen and
/// dialogs, the menu bar, the roster, the preferences and the audio settings.
pub struct Application {
    about: bool,
    questions: screen_state::Questions,
    notes: crate::note::Notes,
    creature: Option<crate::creature::Creature>,
    archives: Vec<(usize, String)>,
    pub selected_scenario: Option<String>,
    scenario_names: Vec<String>,
    pub roster: Option<centurions_core::roster::Roster>,
    combat_information: bool,
    pub end_after_save: bool,
    pub preferences: crate::preferences::Preferences,
    preferences_dialog: Option<crate::preferences::Dialog>,
    pub screen: Screen,
    pub quit: bool,
    pub quit_after_save: bool,
    pub editing: bool,
    pub audio: crate::audio::Settings,
    pub speed: usize,
    pub menu_bar: menus::MenuBar,
    pub show_hover: bool,
    pub help: crate::help::Help,
    pub squad: Vec<usize>,
    /// Requests the diagnostic rule trace written by `--audit-log`.
    pub trace: bool,
    characters: screen_state::CharacterScreens,
    selection: screen_state::SquadScreen,
    intro: screen_state::IntroScreen,
}

impl Application {
    /// Opens the questions and screens the rule events ask for: the escape question, the offer
    /// to swap places with a friend or to attack one, and a viewed note; a finished mission
    /// closes them all and returns to the title.
    pub fn observe(&mut self, events: &[Event]) {
        for event in events {
            if matches!(event, Event::CombatEscapeOffered) {
                self.questions.escape = true;
            }
            if let Event::ExchangeOffered { body } = event {
                self.questions.exchange_offer = Some(*body);
                self.questions.exchange_swap = true;
            }
            if let Event::FriendAttackOffered { body } = event {
                self.questions.exchange_offer = Some(*body);
                self.questions.exchange_swap = false;
            }
            if matches!(event, Event::MissionEnded { .. }) {
                self.screen = Screen::Title;
                self.questions.exchange_offer = None;
                self.notes.open = None;
                self.creature = None;
                self.characters.information = None;
                self.characters.editor = None;
                self.characters.item_editor = None;
                self.characters.appearance = None;
                self.menu_bar.menu = None;
                self.menu_bar.submenu = None;
            }
            if let Event::NoteViewed { record, text, .. } = event {
                self.notes.open = Some(crate::note::Note {
                    record: record.clone(),
                    text: text.clone(),
                    scroll: 0.,
                });
            }
        }
    }
    /// Where a warning box is anchored: the editors place it 80 pixels in and down,
    /// every other screen just below the menu bar.
    pub fn warning_origin(&self) -> Vec2 {
        if self.characters.editor.is_some() || self.characters.item_editor.is_some() {
            vec2(80., 80.)
        } else {
            vec2(0., 20.)
        }
    }

    /// Starts on `screen` at the current time.
    pub fn new(screen: Screen) -> Self {
        Self::at(screen, get_time())
    }

    /// Starts on `screen` as of `now`, with no dialog open.
    pub(crate) fn at(screen: Screen, now: f64) -> Self {
        Self {
            about: false,
            questions: screen_state::Questions::new(),
            notes: crate::note::Notes::default(),
            creature: None,
            archives: vec![],
            selected_scenario: None,
            scenario_names: vec!["The Thunder of Guns".into()],
            roster: None,
            combat_information: false,
            end_after_save: false,
            preferences: crate::preferences::Preferences::default(),
            preferences_dialog: None,
            screen,
            quit: false,
            quit_after_save: false,
            editing: false,
            audio: crate::audio::Settings::default(),
            // createpref's default speed 5, an animation delay of 20.
            speed: 5,
            menu_bar: menus::MenuBar::default(),
            show_hover: true,
            help: crate::help::Help::default(),
            squad: Vec::new(),
            trace: false,
            characters: screen_state::CharacterScreens::new(),
            selection: screen_state::SquadScreen::new(),
            intro: screen_state::IntroScreen::new(now),
        }
    }

    /// Brings the screens up to date with the session: the logo gives way to the
    /// title after three seconds, the title returns when squad selection begins, a
    /// scenario prologue opens the introduction, the introduction text scrolls, and
    /// the game asks for its tutorial pages.
    pub fn sync(&mut self, state: &Session, animate: bool) -> Result<(), String> {
        let now = get_time();
        if let Some(choice) = &state.out_of_combat_walk_choice {
            self.questions.exchange_offer = Some(choice.body);
            self.questions.exchange_swap = choice.exchange;
        }
        if self.screen == Screen::Game && state.phase == Phase::SquadSelection {
            self.screen = Screen::Title;
        }
        if self.screen == Screen::Logo && now - self.intro.started >= 181. / 60. {
            self.screen = Screen::Title;
        }
        if self.screen == Screen::Game
            && let Some(Presentation::Prologue { resource_id }) = state
                .exploration
                .as_ref()
                .and_then(|p| p.presentation.as_ref())
        {
            self.open_intro(Screen::Game, Some(*resource_id));
        }
        if self.screen == Screen::Intro && animate {
            let steps = ((now - self.intro.intro_tick) * 15.).floor();
            if steps > 0. {
                self.intro.intro_y -= steps as f32;
                self.intro.intro_tick += f64::from(steps as f32) / 15.;
            }
        }
        // updatemain requests tutorial pages 94 through 102.
        if self.screen == Screen::Game && !self.blocks_game() {
            self.help.open(94, 102)?;
        }
        Ok(())
    }

    /// Opens the introduction text, returning to `return_to` when it closes; a
    /// `prologue` shows that text resource in place of the introduction.
    fn open_intro(&mut self, return_to: Screen, prologue: Option<i16>) {
        self.screen = Screen::Intro;
        self.intro.intro_return = return_to;
        self.intro.prologue = prologue;
        self.intro.intro_y = 0.;
        self.intro.intro_tick = get_time();
        self.intro.intro_click = -1.;
        self.intro.drag = None;
        self.menu_bar.menu = None;
        self.menu_bar.submenu = None;
    }

    /// Whether a dialog, menu or other screen is open over the game, so the map and
    /// its controls take no input.
    pub fn blocks_game(&self) -> bool {
        self.about
            || self.screen != Screen::Game
            || self.menu_bar.menu.is_some()
            || self.questions.quit
            || self.help.page.is_some()
            || self.questions.help_reset
            || self.characters.editor_question
            || self.characters.editor.is_some()
            || self.characters.information.is_some()
            || self.characters.appearance.is_some()
            || self.characters.item_editor.is_some()
            || self.selection.order.is_some()
            || self.selection.end_question
            || self.questions.escape
            || self.characters.creation.is_some()
            || self.preferences_dialog.is_some()
            || self.creature.is_some()
            || self.notes.open.is_some()
            || self.questions.exchange_offer.is_some()
    }

    /// Carries out one application action and saves the volume, speed and music
    /// settings when the action changed them. Returns the action, if any, left for
    /// the game, and the events produced.
    pub fn handle(
        &mut self,
        action: ApplicationAction,
        state: &mut Session,
        content: &Content,
    ) -> Result<(Option<Action>, Vec<Event>), centurions_core::RuleError> {
        self.scenario_names = content.scenario_names();
        self.archives = centurions_core::reference_views::archive_entries(state, content)?;
        if let Some(map) = state.land(content) {
            self.notes.labels = map.note_labels.clone();
            self.notes.unknown_labels = map.unknown_note_labels.clone();
        }
        if self.roster.is_none() {
            self.roster = Some(centurions_core::roster::Roster::supplied(content));
        }
        let preferences_changed = matches!(
            action,
            ApplicationAction::Playlist(_)
                | ApplicationAction::Menu(134, _)
                | ApplicationAction::Menu(135, _)
                | ApplicationAction::Menu(145, _)
        );
        let outcome = match action {
            action @ (ApplicationAction::ExchangeChoice(_)
            | ApplicationAction::CloseAbout
            | ApplicationAction::EscapeAnswer(_)
            | ApplicationAction::SkipLogo
            | ApplicationAction::CloseIntro
            | ApplicationAction::QuitAnswer(_)) => self.handle_lifecycle(action, state, content)?,
            action @ (ApplicationAction::CloseNote
            | ApplicationAction::CreatureItem(_)
            | ApplicationAction::CombatInformation(_)
            | ApplicationAction::OpenInformation(_)
            | ApplicationAction::InformationItem(_)) => {
                self.handle_information(action, state, content)?
            }
            action @ (ApplicationAction::PreferenceCancel
            | ApplicationAction::PreferenceItem(_)
            | ApplicationAction::Playlist(_)
            | ApplicationAction::HelpResetAnswer(_)
            | ApplicationAction::Help(_)) => self.handle_settings(action, state, content)?,
            action @ (ApplicationAction::CreationCancel
            | ApplicationAction::CreationChoice(_)
            | ApplicationAction::CreationAnswer(_)
            | ApplicationAction::CreationItem(_)
            | ApplicationAction::AppearanceItem(_)
            | ApplicationAction::OpenItemEditor(_, _)
            | ApplicationAction::ItemEditorItem(_)
            | ApplicationAction::EditorAnswer(_)
            | ApplicationAction::OpenEditor(_)
            | ApplicationAction::EditorItem(_)
            | ApplicationAction::ScanAnswer(_)) => {
                self.handle_characters(action, state, content)?
            }
            action @ (ApplicationAction::EndAnswer(_)
            | ApplicationAction::EndSaved
            | ApplicationAction::SquadCancel
            | ApplicationAction::OrderCancel
            | ApplicationAction::OrderMember(_)
            | ApplicationAction::Difficulty(_)
            | ApplicationAction::Warrior(_)
            | ApplicationAction::Member(_)
            | ApplicationAction::Add
            | ApplicationAction::Remove
            | ApplicationAction::EraseWarrior
            | ApplicationAction::ImportWarrior
            | ApplicationAction::ViewRestrictions
            | ApplicationAction::RosterScroll(_)
            | ApplicationAction::Done) => self.handle_squad(action, state, content)?,
            action @ ApplicationAction::Menu(_, _) => {
                self.handle_menu_actions(action, state, content)?
            }
        };
        let (next, events) = match outcome {
            HandleOutcome::Complete(next, events) => (next, events),
            HandleOutcome::Return(next, events) => return Ok((next, events)),
        };
        if preferences_changed {
            self.help.settings["volume"] = self.audio.volume.into();
            self.help.settings["music_volume"] = self.audio.music_volume.into();
            self.help.settings["speed"] = self.speed.into();
            self.help.settings["music_enabled"] = self.audio.music_enabled.into();
            self.help.settings["playlist"] = serde_json::json!(self.audio.playlist);
            self.help.save()?;
        }
        self.observe(&events);
        Ok((next, events))
    }
}

/// Whether a command key (Control or Super) is held, as the Command key is for
/// the menu shortcuts.
pub fn command_modifier() -> bool {
    [
        KeyCode::LeftControl,
        KeyCode::RightControl,
        KeyCode::LeftSuper,
        KeyCode::RightSuper,
    ]
    .iter()
    .any(|key| is_key_down(*key) || is_key_pressed(*key))
}

#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../tests/unit/difficulty_state.rs"]
mod tests;

mod characters;
mod combat_feedback;
mod drawing;
mod information;
mod input;
mod intro;
mod lifecycle;
mod menu_actions;
mod menus;
mod screen_state;
mod settings;
mod squad;

/// How a handler ends the frame. An early return skips the preference writes
/// and observation, as the game's own early returns do.
enum HandleOutcome {
    Complete(Option<Action>, Vec<Event>),
    Return(Option<Action>, Vec<Event>),
}
