use crate::presentation::Action;
use centurions_content::Content;
use centurions_core::{Event, Phase, Presentation, Session};
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Logo,
    Title,
    Squad,
    Intro,
    Game,
}

#[derive(Clone, Debug)]
pub enum ApplicationAction {
    SkipLogo,
    Menu(i16, usize),
    Warrior(usize),
    Member(usize),
    Add,
    Remove,
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

pub struct Application {
    about: bool,
    exchange_offer: Option<i16>,
    exchange_swap: bool,
    note: Option<crate::note::Note>,
    note_labels: Vec<String>,
    unknown_note_labels: Vec<String>,
    creature: Option<crate::creature::Creature>,
    archives: Vec<(usize, String)>,
    pub selected_scenario: Option<String>,
    scenario_names: Vec<String>,
    pub roster: Option<centurions_core::roster::Roster>,
    escape_question: bool,
    combat_information: bool,
    pub end_after_save: bool,
    pub native_preferences: crate::preferences::Preferences,
    preferences_dialog: Option<crate::preferences::Dialog>,
    pub screen: Screen,
    pub quit: bool,
    pub quit_after_save: bool,
    quit_question: bool,
    help_reset_question: bool,
    pub editing: bool,
    pub sound: bool,
    pub volume: usize,
    pub music_volume: usize,
    pub music_enabled: bool,
    pub music_request: Option<i16>,
    pub current_music: Option<i16>,
    pub playlist: [bool; 13],
    pub speed: usize,
    pub menu: Option<i16>,
    pub submenu: Option<i16>,
    menu_scroll: usize,
    pub show_hover: bool,
    pub help: crate::help::Help,
    pub squad: Vec<usize>,
    characters: screen_state::CharacterScreens,
    selection: screen_state::SquadScreen,
    intro: screen_state::IntroScreen,
}

impl Application {
    pub fn observe(&mut self, events: &[Event]) {
        for event in events {
            if matches!(event, Event::CombatEscapeOffered) {
                self.escape_question = true;
            }
            if let Event::ExchangeOffered { body } = event {
                self.exchange_offer = Some(*body);
                self.exchange_swap = true;
            }
            if let Event::FriendAttackOffered { body } = event {
                self.exchange_offer = Some(*body);
                self.exchange_swap = false;
            }
            if matches!(event, Event::MissionEnded { .. }) {
                self.screen = Screen::Title;
                self.exchange_offer = None;
                self.note = None;
                self.creature = None;
                self.characters.information = None;
                self.characters.editor = None;
                self.characters.item_editor = None;
                self.characters.appearance = None;
                self.menu = None;
                self.submenu = None;
            }
            if let Event::NoteViewed { record, text, .. } = event {
                self.note = Some(crate::note::Note {
                    record: record.clone(),
                    text: text.clone(),
                    scroll: 0.,
                });
            }
        }
    }
    pub fn warning_origin(&self) -> Vec2 {
        if self.characters.editor.is_some() || self.characters.item_editor.is_some() {
            vec2(80., 80.)
        } else {
            vec2(0., 20.)
        }
    }

    pub fn new(screen: Screen) -> Self {
        Self::at(screen, get_time())
    }

    pub(crate) fn at(screen: Screen, now: f64) -> Self {
        Self {
            about: false,
            exchange_offer: None,
            exchange_swap: true,
            note: None,
            note_labels: vec![],
            unknown_note_labels: vec![],
            creature: None,
            archives: vec![],
            selected_scenario: None,
            scenario_names: vec!["The Thunder of Guns".into()],
            roster: None,
            escape_question: false,
            combat_information: false,
            end_after_save: false,
            native_preferences: crate::preferences::Preferences::default(),
            preferences_dialog: None,
            screen,
            quit: false,
            quit_after_save: false,
            quit_question: false,
            help_reset_question: false,
            editing: false,
            sound: true,
            volume: 5,
            music_volume: 5,
            music_enabled: true,
            music_request: Some(20128),
            current_music: None,
            playlist: [true; 13],
            speed: 6,
            menu: None,
            submenu: None,
            menu_scroll: 0,
            show_hover: true,
            help: crate::help::Help::default(),
            squad: Vec::new(),
            characters: screen_state::CharacterScreens::new(),
            selection: screen_state::SquadScreen::new(),
            intro: screen_state::IntroScreen::new(now),
        }
    }

    pub fn sync(&mut self, state: &Session, animate: bool) -> Result<(), String> {
        let now = get_time();
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
        // updatemain 0x13bb20 requests tutorial pages 94 through 102.
        if self.screen == Screen::Game && !self.blocks_game() {
            self.help.open(94, 102)?;
        }
        Ok(())
    }

    fn open_intro(&mut self, return_to: Screen, prologue: Option<i16>) {
        self.screen = Screen::Intro;
        self.intro.intro_return = return_to;
        self.intro.prologue = prologue;
        self.intro.intro_y = 0.;
        self.intro.intro_tick = get_time();
        self.intro.intro_click = -1.;
        self.intro.drag = None;
        self.menu = None;
        self.submenu = None;
    }

    pub fn blocks_game(&self) -> bool {
        self.about
            || self.screen != Screen::Game
            || self.menu.is_some()
            || self.quit_question
            || self.help.page.is_some()
            || self.help_reset_question
            || self.characters.editor_question
            || self.characters.editor.is_some()
            || self.characters.information.is_some()
            || self.characters.appearance.is_some()
            || self.characters.item_editor.is_some()
            || self.selection.order.is_some()
            || self.selection.end_question
            || self.escape_question
            || self.characters.creation.is_some()
            || self.preferences_dialog.is_some()
            || self.creature.is_some()
            || self.note.is_some()
            || self.exchange_offer.is_some()
    }

    pub fn handle(
        &mut self,
        action: ApplicationAction,
        state: &mut Session,
        content: &Content,
    ) -> Result<(Option<Action>, Vec<Event>), centurions_core::RuleError> {
        self.scenario_names = content.scenario_names();
        self.archives = centurions_core::reference_views::archive_entries(state, content)?;
        if let Some(map) = state.map(content, state.exploration.as_ref().map_or(0, |p| p.map)) {
            self.note_labels = map.note_labels.clone();
            self.unknown_note_labels = map.unknown_note_labels.clone();
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
            self.help.settings["volume"] = self.volume.into();
            self.help.settings["music_volume"] = self.music_volume.into();
            self.help.settings["speed"] = self.speed.into();
            self.help.settings["music_enabled"] = self.music_enabled.into();
            self.help.settings["playlist"] = serde_json::json!(self.playlist);
            self.help.save()?;
        }
        self.observe(&events);
        Ok((next, events))
    }
}

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
mod lifecycle;
mod menu_actions;
mod menus;
mod screen_state;
mod settings;
mod squad;

/// Preserve original early returns that bypass preference writes and observation.
enum HandleOutcome {
    Complete(Option<Action>, Vec<Event>),
    Return(Option<Action>, Vec<Event>),
}
