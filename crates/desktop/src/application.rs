use crate::{
    assets::Assets,
    presentation::{self, Action, Control, UiState},
};
use centurions_content::{Content, word};
use centurions_core::{Command, Event, Phase, Presentation, Session, apply};
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
    modify_squad: bool,
    roster_page: usize,
    order: Option<Vec<usize>>,
    end_question: bool,
    escape_question: bool,
    combat_information: bool,
    pub end_after_save: bool,
    creation: Option<crate::creation::Creation>,
    pub native_preferences: crate::preferences::Preferences,
    preferences_dialog: Option<crate::preferences::Dialog>,
    pub screen: Screen,
    pub quit: bool,
    pub quit_after_save: bool,
    quit_question: bool,
    help_reset_question: bool,
    editor_question: bool,
    pub editing: bool,
    editor: Option<crate::editor::WarriorEditor>,
    information: Option<crate::information::WarriorInformation>,
    appearance: Option<crate::appearance::Appearance>,
    item_editor: Option<crate::editor::ItemEditor>,
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
    selected: Option<usize>,
    member: Option<usize>,
    pub squad: Vec<usize>,
    difficulty: i16,
    started: f64,
    intro_y: f32,
    intro_tick: f64,
    intro_click: f64,
    drag: Option<Vec2>,
    intro_return: Screen,
    prologue: Option<i16>,
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
                self.information = None;
                self.editor = None;
                self.item_editor = None;
                self.appearance = None;
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
    pub fn restore_information(&mut self, member: usize) {
        self.information = Some(crate::information::WarriorInformation::new(member));
    }
    pub fn window_mode(&mut self, fullscreen: bool) {
        self.native_preferences.controls[20 - 9] = fullscreen;
        if let Some(dialog) = &mut self.preferences_dialog {
            dialog.draft.controls[20 - 9] = fullscreen;
        }
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
    fn warriors<'a>(&'a self, content: &'a Content) -> &'a [centurions_content::Warrior] {
        self.roster
            .as_ref()
            .map_or(&content.warriors, |r| &r.warriors)
    }
    fn finish_mission(
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
        self.end_question = false;
        self.end_after_save = false;
        self.screen = Screen::Title;
        Ok(())
    }
    pub fn warning_origin(&self) -> Vec2 {
        if self.editor.is_some() || self.item_editor.is_some() {
            vec2(80., 80.)
        } else {
            vec2(0., 20.)
        }
    }
    fn finish_creation_scans(
        &mut self,
        state: &mut Session,
        content: &Content,
    ) -> Result<Vec<Event>, centurions_core::RuleError> {
        if let Some(creation) = &mut self.creation
            && creation.stage == crate::creation::Stage::Scans
            && let Some(editor) = &self.editor
            && editor.scans.is_none()
        {
            creation.draft = editor.draft.clone();
            let events = creation.finish(state, content)?;
            self.editor = None;
            return Ok(events);
        }
        Ok(vec![])
    }
    pub fn prepare_information(
        &mut self,
        state: &Session,
        assets: &Assets,
    ) -> Result<(), centurions_core::RuleError> {
        if let Some(creation) = &mut self.creation
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
        if let Some(info) = &mut self.information
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
    pub fn preferences(&mut self, path: std::path::PathBuf) -> Result<(), String> {
        self.help.preferences(path)?;
        let settings = &self.help.settings;
        self.native_preferences = crate::preferences::Preferences::decode(&settings["native"])?;
        self.volume = settings["volume"]
            .as_u64()
            .filter(|v| *v <= 10)
            .unwrap_or(5) as usize;
        self.sound = self.volume != 0;
        self.music_volume = settings["music_volume"]
            .as_u64()
            .filter(|v| *v <= 10)
            .map_or(self.volume, |v| v as usize);
        self.speed = settings["speed"]
            .as_u64()
            .filter(|v| (1..=11).contains(v))
            .unwrap_or(6) as usize;
        self.music_enabled = settings["music_enabled"].as_bool().unwrap_or(true);
        if let Some(values) = settings["playlist"].as_array().filter(|v| v.len() == 13) {
            for (target, value) in self.playlist.iter_mut().zip(values) {
                *target = value.as_bool().unwrap_or(true);
            }
        }
        Ok(())
    }
    pub fn command(&self, command: Command) -> Command {
        Command::WithPreferences {
            command: Box::new(command),
            preferences: centurions_core::CommandPreferences {
                auto_refit: self.native_preferences.enabled(15),
                suppress_round_message_sound: self.native_preferences.enabled(13),
                animation_delay: self.speed as i16 * 5 - 5,
            },
        }
    }
    pub fn combat_notices(&self, previous_round: Option<i16>, state: &Session) -> Vec<Event> {
        let Some(combat) = &state.combat else {
            return vec![];
        };
        if previous_round.is_none_or(|round| round == combat.round) {
            return vec![];
        }
        use centurions_core::feedback::NativeFeedback;
        let mut events = vec![];
        if !self.native_preferences.enabled(14)
            && state.warriors.iter().any(|w| w.bytes[0x9de] != 0)
        {
            events.push(Event::Feedback(NativeFeedback::Flash {
                address: 0x403708,
                x: 30,
                y: 100,
                wait: 120,
                sound: 10122,
            }));
            events.push(Event::Feedback(NativeFeedback::Tutorial {
                first: 118,
                last: 118,
            }));
        }
        if !self.native_preferences.enabled(13) {
            events.push(Event::Feedback(NativeFeedback::Flash {
                address: 0x4036f2,
                x: 30,
                y: 100,
                wait: -1,
                sound: 0,
            }));
        }
        // getup1263ca..1263f4 plays the round sound twice, then closes the flash.
        for _ in 0..2 {
            events.push(Event::Feedback(NativeFeedback::Sound { resource_id: -139 }));
        }
        if !self.native_preferences.enabled(13) {
            events.push(Event::Feedback(NativeFeedback::Flash {
                address: 0x4036f1,
                x: 30,
                y: 100,
                wait: -1,
                sound: -139,
            }));
        }
        events
    }
    /// Preserve round sound order while presenting the paired nonblocking native flash.
    pub fn prepare_combat_round(
        &self,
        previous: Option<i16>,
        state: &Session,
        events: &mut Vec<Event>,
    ) {
        let notices = self.combat_notices(previous, state);
        if notices.is_empty() {
            return;
        }
        let Some(start) = events
            .iter()
            .position(|event| matches!(event, Event::Sound { resource_id: -139 }))
        else {
            return;
        };
        let mut tail = events.split_off(start).into_iter().peekable();
        let sounds = if self.native_preferences.enabled(13) {
            2
        } else {
            3
        };
        while let Some(event) = tail.next() {
            if matches!(event, Event::Sound { resource_id: -139 }) {
                events.extend(self.combat_notices(previous, state));
                for _ in 1..sounds {
                    if tail
                        .peek()
                        .is_some_and(|e| matches!(e, Event::Sound { resource_id: -139 }))
                    {
                        tail.next();
                    } else {
                        break;
                    }
                }
            } else if let Event::Sound { resource_id } = event {
                events.push(Event::Feedback(
                    centurions_core::feedback::NativeFeedback::Sound { resource_id },
                ));
            } else {
                events.push(event);
            }
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
            modify_squad: false,
            roster_page: 0,
            order: None,
            end_question: false,
            escape_question: false,
            combat_information: false,
            end_after_save: false,
            creation: None,
            native_preferences: crate::preferences::Preferences::default(),
            preferences_dialog: None,
            screen,
            quit: false,
            quit_after_save: false,
            quit_question: false,
            help_reset_question: false,
            editor_question: false,
            editing: false,
            editor: None,
            information: None,
            appearance: None,
            item_editor: None,
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
            selected: None,
            member: None,
            squad: Vec::new(),
            difficulty: 0,
            started: now,
            intro_y: 0.,
            intro_tick: now,
            intro_click: -1.,
            drag: None,
            intro_return: Screen::Title,
            prologue: None,
        }
    }

    pub fn sync(&mut self, state: &Session, animate: bool) -> Result<(), String> {
        let now = get_time();
        if self.screen == Screen::Game && state.phase == Phase::SquadSelection {
            self.screen = Screen::Title;
        }
        if self.screen == Screen::Logo && now - self.started >= 181. / 60. {
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
            let steps = ((now - self.intro_tick) * 15.).floor();
            if steps > 0. {
                self.intro_y -= steps as f32;
                self.intro_tick += f64::from(steps as f32) / 15.;
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
        self.intro_return = return_to;
        self.prologue = prologue;
        self.intro_y = 0.;
        self.intro_tick = get_time();
        self.intro_click = -1.;
        self.drag = None;
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
            || self.editor_question
            || self.editor.is_some()
            || self.information.is_some()
            || self.appearance.is_some()
            || self.item_editor.is_some()
            || self.order.is_some()
            || self.end_question
            || self.escape_question
            || self.creation.is_some()
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
        let mut next = None;
        let mut events = Vec::new();
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
        match action {
            ApplicationAction::ExchangeChoice(choice) => {
                let body = self
                    .exchange_offer
                    .ok_or("no occupied-cell choice is open")?;
                if let Some(exchange) = choice {
                    events = apply(
                        if exchange {
                            Command::SwapCombatants { body }
                        } else {
                            Command::AttackFriend { body }
                        },
                        state,
                        content,
                    )?;
                }
                self.exchange_offer = None;
            }
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
                        return self.handle(
                            ApplicationAction::CombatInformation(next as i16 + 10),
                            state,
                            content,
                        );
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
            ApplicationAction::CloseAbout => self.about = false,
            ApplicationAction::PreferenceCancel => self.preferences_dialog = None,
            ApplicationAction::PreferenceItem(item) => {
                let dialog = self
                    .preferences_dialog
                    .as_mut()
                    .ok_or("Preferences are not open.")?;
                if dialog.item(item)? {
                    let settings = self.help.settings.clone();
                    self.help.settings["native"] = dialog.draft.encode();
                    if let Err(error) = self.help.save() {
                        self.help.settings = settings;
                        return Err(error.into());
                    }
                    self.native_preferences = dialog.draft.clone();
                    self.preferences_dialog = None;
                }
                events.push(Event::Sound { resource_id: 141 });
            }
            ApplicationAction::CreationCancel => {
                self.creation = None;
                if self.information.is_some() {
                    return Ok((next, events));
                }
                self.editor = None;
            }
            ApplicationAction::CreationChoice(choice) => {
                events = self
                    .creation
                    .as_mut()
                    .ok_or("Creation is not open.")?
                    .choice(choice, content)?;
            }
            ApplicationAction::CreationAnswer(answer) => {
                let creation = self.creation.as_ref().ok_or("Creation is not open.")?;
                if creation.stage != crate::creation::Stage::Keep {
                    return Err("Creation confirmation is not open.".into());
                }
                if answer {
                    self.roster
                        .as_mut()
                        .unwrap()
                        .store(&creation.draft.warriors)?;
                }
                self.creation = None;
            }
            ApplicationAction::CreationItem(item) => {
                if self
                    .creation
                    .as_ref()
                    .is_some_and(|creation| creation.read_only)
                {
                    if matches!(item, 1 | 2 | 56) {
                        self.creation = None;
                    } else if matches!(item, 57 | 58 | 74) {
                        events = self.creation.as_mut().unwrap().item(item, state, content)?;
                    }
                    return Ok((next, events));
                }
                let creation = self.creation.as_mut().ok_or("Creation is not open.")?;
                events = creation.item(item, state, content)?;
                if creation.stage == crate::creation::Stage::Scans {
                    let mut editor = crate::editor::WarriorEditor::new(&creation.draft, 0)?;
                    editor.free = false;
                    events.extend(editor.handle(39, content)?);
                    self.editor = Some(editor);
                }
            }
            ApplicationAction::EndAnswer(answer) => {
                if !self.end_question {
                    return Err("Mission ending question is not open.".into());
                }
                if state.phase == Phase::Combat && !answer {
                    self.end_question = false;
                } else if answer {
                    self.finish_mission(state, content)?;
                } else {
                    self.end_question = false;
                    self.end_after_save = true;
                    next = Some(Action::Save);
                }
            }
            ApplicationAction::EndSaved => self.finish_mission(state, content)?,
            ApplicationAction::SquadCancel => {
                self.screen = if self.modify_squad {
                    Screen::Game
                } else {
                    Screen::Title
                };
                self.modify_squad = false;
            }
            ApplicationAction::OrderCancel => self.order = None,
            ApplicationAction::OrderMember(member) => {
                let order = self.order.as_mut().ok_or("Squad ordering is not open.")?;
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
                    self.order = None;
                }
            }
            ApplicationAction::Playlist(index) => {
                if !self.supported(145, index + 10, state) {
                    return Err(centurions_core::RuleError::Refused { feedback: vec![] });
                }
                let entry = self.playlist.get_mut(index).ok_or("music playlist index")?;
                *entry = !*entry;
                self.menu = None;
                self.submenu = None;
            }
            ApplicationAction::AppearanceItem(item) => {
                let editor = self
                    .appearance
                    .as_mut()
                    .ok_or("Appearance dialog is not open.")?;
                if item < 3 {
                    editor.draft.validate(content)?;
                    *state = editor.draft.clone();
                    self.appearance = None;
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
                self.item_editor = Some(crate::editor::ItemEditor::new(state, warrior, slot)?);
            }
            ApplicationAction::ItemEditorItem(item) => {
                let editor = self
                    .item_editor
                    .as_mut()
                    .ok_or("Item editor is not open.")?;
                if item < 3 {
                    editor.draft.validate(content).map_err(|e| e.to_string())?;
                    *state = editor.draft.clone();
                    self.item_editor = None;
                } else {
                    events = editor.handle(item, content)?;
                }
            }
            ApplicationAction::EditorAnswer(answer) => {
                self.editor_question = false;
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
            ApplicationAction::EscapeAnswer(answer) => {
                self.escape_question = false;
                if answer {
                    events = apply(self.command(Command::EscapeCombat), state, content)?;
                }
            }
            ApplicationAction::CombatInformation(body) => {
                events = apply(Command::ViewCombatBody { body }, state, content)?;
                if body < 10 {
                    self.information =
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
                self.information = Some(crate::information::WarriorInformation::new(warrior));
            }
            ApplicationAction::InformationItem(item) => {
                let member = self
                    .information
                    .as_ref()
                    .ok_or("Warrior information is not open.")?
                    .member;
                if self.information.as_ref().unwrap().detailed {
                    let info = self.information.as_mut().unwrap();
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
                    return Ok((next, events));
                }
                match item {
                    1 | 72 => self.information = None,
                    2..=5 => {
                        let inspection = match item {
                            2 => crate::presentation::Inspection::Conditions,
                            3 => crate::presentation::Inspection::Items,
                            4 => crate::presentation::Inspection::Specials,
                            _ => crate::presentation::Inspection::Modifiers,
                        };
                        if item == 3 {
                            self.information = None;
                        } else {
                            events.push(Event::Sound {
                                resource_id: if item == 4 { 647 } else { 609 },
                            });
                        }
                        next = Some(Action::InformationInspection(member, inspection));
                    }
                    96 | 97 => {
                        self.creation = Some(crate::creation::Creation::profile(
                            state,
                            content,
                            member,
                            item == 97,
                        )?);
                    }
                    100 => {
                        self.information.as_mut().unwrap().detailed = true;
                        let mut presentation_random = state.random.clone();
                        events.push(Event::Sound {
                            resource_id: presentation_random.bound(2).wrapping_add(200),
                        });
                    }
                    65 | 105 => {
                        let mut editor = crate::editor::WarriorEditor::new(state, member)?;
                        editor.free = self.editing;
                        self.editor = Some(editor);
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
                        self.information =
                            Some(crate::information::WarriorInformation::new(warrior));
                    }
                    _ => return Err("Character information action is not connected.".into()),
                }
            }
            ApplicationAction::OpenEditor(warrior) => {
                if self.screen != Screen::Game || state.phase != Phase::Exploration {
                    return Err("Warrior editing is unavailable.".into());
                }
                let mut editor = crate::editor::WarriorEditor::new(state, warrior)?;
                editor.free = self.editing;
                self.editor = Some(editor);
            }
            ApplicationAction::EditorItem(item) => {
                let editor = self.editor.as_mut().ok_or("Warrior editor is not open.")?;
                if editor.level_information || editor.scans.is_some() {
                    events = editor.handle(item, content)?;
                    events.extend(self.finish_creation_scans(state, content)?);
                    return Ok((next, events));
                }
                match item {
                    1 | 4 => {
                        if item == 1 {
                            editor.cancel_current()?;
                        }
                        editor.draft.validate(content).map_err(|e| e.to_string())?;
                        if let Some(creation) = &mut self.creation {
                            creation.draft = editor.draft.clone();
                            creation.stage = crate::creation::Stage::Keep;
                            state.library_random_seed = creation.draft.library_random_seed;
                            state.library_random_draws = creation.draft.library_random_draws;
                        } else {
                            *state = editor.draft.clone();
                        }
                        self.editor = None;
                        if let Some(info) = &mut self.information {
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
                self.editor
                    .as_mut()
                    .ok_or("Warrior editor is not open.")?
                    .scan_answer(answer, content)?;
                events.extend(self.finish_creation_scans(state, content)?);
            }
            ApplicationAction::Difficulty(level) => {
                if self.screen != Screen::Squad || self.modify_squad || !(-2..=2).contains(&level) {
                    return Err("Difficulty selection is unavailable.".into());
                }
                self.difficulty = level;
                events.push(Event::Sound { resource_id: 144 });
            }
            ApplicationAction::HelpResetAnswer(answer) => {
                self.help_reset_question = false;
                if answer {
                    self.help.reset()?;
                }
            }
            ApplicationAction::Help(item) => {
                let resource_id = self.help.action(item)?;
                if resource_id != 0 {
                    events.push(Event::Sound { resource_id });
                }
            }
            ApplicationAction::SkipLogo => self.screen = Screen::Title,
            ApplicationAction::Warrior(i) => {
                self.selected = Some(i);
                self.member = None;
            }
            ApplicationAction::Member(i) => self.member = Some(i),
            ApplicationAction::Add => {
                if let Some(i) = self.selected
                    && self.squad.len() < 6
                    && !self.squad.contains(&i)
                {
                    self.squad.push(i);
                    self.member = Some(self.squad.len() - 1);
                    events.push(Event::Sound { resource_id: 141 });
                }
            }
            ApplicationAction::Remove => {
                if let Some(i) = self.member.take()
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
                                if self.modify_squad {
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
                if self.modify_squad {
                    if warriors.is_empty() {
                        return Err(centurions_core::RuleError::Refused {
                            feedback: vec![centurions_core::feedback::NativeFeedback::Sound {
                                resource_id: 6000,
                            }],
                        });
                    }
                    events = apply(Command::ReplaceSquad { warriors }, state, content)?;
                    self.modify_squad = false;
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
                            level: self.difficulty,
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
            ApplicationAction::CloseIntro => {
                self.screen = self.intro_return;
                if self.prologue.take().is_some() {
                    next = Some(Action::Command(Command::Continue));
                }
            }
            ApplicationAction::QuitAnswer(answer) => {
                self.quit_question = false;
                if answer {
                    self.quit = true;
                } else if state.phase != Phase::Combat {
                    self.quit_after_save = true;
                    next = Some(Action::Save);
                }
            }
            ApplicationAction::Menu(id, item) => {
                if !self.supported(id, item, state) {
                    return Err(centurions_core::RuleError::Refused { feedback: vec![] });
                }
                self.menu = None;
                self.submenu = None;
                match (id, item) {
                    (128, 1) => {
                        self.about = true;
                        events.extend([
                            Event::Sound { resource_id: 10107 },
                            Event::Sound { resource_id: 30000 },
                            Event::Delay { ticks: 20 },
                            Event::Sound { resource_id: 10141 },
                        ]);
                    }
                    (200, 4..=22) => {
                        events = apply(Command::ViewNote { index: item - 4 }, state, content)?
                    }
                    (132 | 146, _) => {
                        let index = if id == 146 {
                            item - 1
                        } else {
                            self.archives
                                .get(item - 1)
                                .ok_or("archive selection unavailable")?
                                .0
                        };
                        events = apply(
                            Command::ViewCreature {
                                index,
                                ally: id == 146,
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
                        events.push(Event::Sound { resource_id: 201 });
                    }
                    (136, 5) => {
                        self.creation = Some(crate::creation::Creation::new(state, content));
                        events = apply(Command::CreationDialog, state, content)?;
                        events.push(Event::Feedback(
                            centurions_core::feedback::NativeFeedback::Tutorial {
                                first: 125,
                                last: 125,
                            },
                        ));
                    }
                    (136, 1) => {
                        self.order = Some(Vec::new());
                        events.push(Event::Feedback(
                            centurions_core::feedback::NativeFeedback::Warning { index: 33 },
                        ));
                        events.push(Event::Feedback(
                            centurions_core::feedback::NativeFeedback::Tutorial {
                                first: 105,
                                last: 105,
                            },
                        ));
                    }
                    (136, 7) => {
                        self.roster
                            .as_mut()
                            .unwrap()
                            .store(&centurions_core::roster::mission_records(state, content)?)?;
                        self.squad = state
                            .warriors
                            .iter()
                            .map(|w| {
                                self.warriors(content)
                                    .iter()
                                    .position(|r| r.original_sha256 == w.original_sha256)
                                    .ok_or("Squad warrior missing from roster.")
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        self.modify_squad = true;
                        self.screen = Screen::Squad;
                        self.selected = None;
                        self.member = None;
                    }
                    (130, 3) => self.end_question = true,
                    (136, 2 | 3) => {
                        self.appearance =
                            Some(crate::appearance::Appearance::new(state, item == 2));
                        events.push(Event::Feedback(
                            centurions_core::feedback::NativeFeedback::Tutorial {
                                first: if item == 2 { 129 } else { 130 },
                                last: if item == 2 { 129 } else { 130 },
                            },
                        ));
                    }
                    (136, 8) => {
                        if self.editing {
                            self.editing = false;
                            events.push(Event::Feedback(
                                centurions_core::feedback::NativeFeedback::Flash {
                                    address: 0x403686,
                                    x: 50,
                                    y: 50,
                                    wait: 0,
                                    sound: 5000,
                                },
                            ));
                        } else {
                            self.editor_question = true;
                        }
                    }
                    (129, 1 | 2) => {
                        next = Some(Action::Load);
                    }
                    (129, 3) => next = Some(Action::Save),
                    (129, 4) => next = Some(Action::FastSave),
                    (129, 6) => {
                        if self.screen == Screen::Title {
                            self.quit = true;
                        } else {
                            self.quit_question = true;
                            events.push(Event::Sound { resource_id: 20005 });
                        }
                    }
                    (130, 1) => {
                        self.modify_squad = false;
                        self.roster_page = 0;
                        self.difficulty = state.difficulty;
                        self.screen = Screen::Squad;
                        self.squad.clear();
                        self.selected = None;
                        self.member = None;
                        self.help.open(121, 124)?;
                        events.push(Event::Sound { resource_id: 10004 });
                    }
                    (130, 2) | (200, 1) => self.open_intro(self.screen, None),
                    (130, 5..=254) => {
                        self.selected_scenario = Some(
                            self.scenario_names
                                .get(item - 5)
                                .ok_or("scenario menu entry missing")?
                                .clone(),
                        );
                    }
                    (137, 3) => {
                        self.preferences_dialog =
                            Some(crate::preferences::Dialog::new(&self.native_preferences))
                    }
                    (137, 4) => {}
                    (137, 5) => {
                        self.help.resume()?;
                        self.help_reset_question = true;
                    }
                    (145, 1) => self.music_enabled = !self.music_enabled,
                    (145, 10..=22) => {
                        if self.playlist.iter().any(|enabled| *enabled) {
                            self.music_enabled = true;
                            self.music_request = Some(20128 + item as i16 - 10);
                        } else {
                            self.music_enabled = false;
                            self.music_request = None;
                            events.push(Event::Feedback(
                                centurions_core::feedback::NativeFeedback::Warning { index: 155 },
                            ));
                        }
                    }
                    (134, 1..=11) => self.speed = item,
                    (135, 2..=12) => {
                        self.volume = item - 2;
                        self.sound = self.volume != 0;
                        events.push(Event::Sound { resource_id: 642 });
                    }
                    (135, 16..=26) => self.music_volume = item - 16,
                    _ => return Err("This original menu action has not been reconstructed.".into()),
                }
            }
        }
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

    fn supported(&self, id: i16, item: usize, state: &Session) -> bool {
        if self.about
            || self.screen == Screen::Squad
            || self.quit_question
            || self.help_reset_question
            || self.editor_question
            || self.editor.is_some()
            || self.information.is_some()
            || self.appearance.is_some()
            || self.item_editor.is_some()
            || self.help.page.is_some()
            || self.order.is_some()
            || self.end_question
            || self.escape_question
            || self.preferences_dialog.is_some()
            || self.creature.is_some()
            || self.note.is_some()
        {
            return false;
        }
        if self.creation.is_some() {
            return false;
        }
        let game = self.screen == Screen::Game && state.phase != Phase::SquadSelection;
        let exploring = game
            && state.phase == Phase::Exploration
            && state.medical_continuation.is_none()
            && state
                .exploration
                .as_ref()
                .is_some_and(|p| p.presentation.is_none());
        match (id, item) {
            (128, 1) => true,
            (200, 4..=22) => game && state.notes[item - 4] != 0,
            (132, _) => game && item > 0 && item <= self.archives.len(),
            (146, _) => game && item > 0 && item <= state.ally_count,
            (136, 8) => true,
            (136, 5) => self.screen == Screen::Title && state.phase == Phase::SquadSelection,
            (136, 2 | 3) => exploring && !state.warriors.is_empty(),
            (130, 3) => game && state.medical_continuation.is_none(),
            (136, 1 | 7) => {
                exploring
                    && state
                        .exploration
                        .as_ref()
                        .is_some_and(|p| p.script.is_none())
            }
            (129, 6) | (130, 2) | (137, 1..=5) => true,
            (130, 5..=254) => {
                self.screen == Screen::Title
                    && state.phase == Phase::SquadSelection
                    && item - 5 < self.scenario_names.len()
            }
            (129, 1) => self.screen == Screen::Title,
            (129, 2) | (200, 1) => game && state.phase != Phase::Combat,
            (129, 3 | 4) => exploring,
            (130, 1) => state.phase == Phase::SquadSelection && self.screen == Screen::Title,
            (134, 1..=11) | (135, 2..=12 | 16..=26) => true,
            (145, 1 | 10..=22) => true,
            _ => false,
        }
    }

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
                let id = self.prologue.unwrap_or_else(|| {
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
                let mut y = rect.y + 16. + self.intro_y;
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
        } else if let Some(creation) = &self.creation
            && self.editor.is_none()
            && creation.stage != crate::creation::Stage::Keep
        {
            controls.clear();
            creation.draw(content, assets, &mut controls);
        } else if let Some(editor) = &self.appearance {
            controls.clear();
            editor.draw(assets, &mut controls);
        } else if let Some(editor) = &self.item_editor {
            controls.clear();
            editor.draw(content, assets, &mut controls);
        } else if let Some(editor) = &self.editor {
            controls.clear();
            editor.draw(content, assets, &mut controls);
        } else if let Some(info) = &self.information {
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
        if let Some(order) = &self.order {
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
            .creation
            .as_ref()
            .is_some_and(|c| c.stage == crate::creation::Stage::Keep);
        if keeping
            || self.quit_question
            || self.help_reset_question
            || self.editor_question
            || self.end_question
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
                } else if self.end_question {
                    if state.phase == Phase::Combat { 9 } else { 0 }
                } else if self.editor_question {
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
                    } else if self.end_question {
                        ApplicationAction::EndAnswer(answer)
                    } else if self.editor_question {
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

    fn draw_squad(&self, content: &Content, assets: &Assets, controls: &mut Vec<Control>) {
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
            .skip(self.roster_page * 13)
            .take(13)
        {
            let row = i - self.roster_page * 13;
            let rect = Rect::new(origin.x + 12., origin.y + 30. + row as f32 * 20., 155., 14.);
            if self.selected == Some(i) {
                draw_rectangle(rect.x, rect.y, rect.w, rect.h, DARKBLUE);
            }
            assets.text(&warrior.name, rect.x, rect.y + 12., 12, WHITE);
            assets.text(
                &word(&warrior.bytes, 0x3a).unwrap_or(0).to_string(),
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
            if let Ok(id) = word(&self.warriors(content)[*warrior].bytes, 0x54)
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
            if self.member == Some(i) {
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1., YELLOW);
            }
            controls.push(Control {
                rect,
                enabled: true,
                action: Action::Application(ApplicationAction::Member(i)),
            });
        }
        let add = self.selected.is_some_and(|i| !self.squad.contains(&i)) && self.squad.len() < 6;
        for (id, y, enabled, action) in [
            (160, 22., add, ApplicationAction::Add),
            (7, 72., self.member.is_some(), ApplicationAction::Remove),
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
                enabled: !self.modify_squad,
                action: Action::Application(ApplicationAction::Difficulty(i as i16 - 2)),
            });
        }
        assets.panel(
            179,
            Rect::new(
                origin.x + 241.,
                origin.y + 364. + f32::from(self.difficulty + 2) * 16.,
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

    fn headings(&self, assets: &Assets) -> Vec<(i16, Rect)> {
        let mut x = 8.;
        assets
            .menu_bar
            .iter()
            .map(|id| {
                let width = measure_text(
                    assets.menu_title(*id).unwrap_or(""),
                    Some(&assets.menu_font),
                    12,
                    1.,
                )
                .width
                    + 16.;
                let rect = Rect::new(x, 0., width, 20.);
                x += width;
                (*id, rect)
            })
            .collect()
    }

    fn draw_menus(&self, state: &Session, assets: &Assets, controls: &mut Vec<Control>) {
        draw_rectangle(0., 0., 800., 20., Color::from_rgba(238, 238, 238, 255));
        for (id, rect) in self.headings(assets) {
            if self.menu == Some(id) {
                draw_rectangle(rect.x, rect.y, rect.w, rect.h, DARKBLUE);
            }
            assets.menu_text(
                assets.menu_title(id).unwrap_or(""),
                rect.x + 8.,
                14.,
                12,
                if self.menu == Some(id) { WHITE } else { BLACK },
            );
            if self.menu == Some(id) {
                let (bounds, rows) = self.dropdown(id, vec2(rect.x, 20.), state, assets, controls);
                if let Some(sub) = self.submenu
                    && let Some(row) = rows.iter().find(|(_, child)| *child == Some(sub))
                {
                    let sub_width = self.menu_width(sub, state, assets);
                    let x = if bounds.right() + sub_width > 800. {
                        bounds.x - sub_width
                    } else {
                        bounds.right()
                    };
                    self.dropdown(sub, vec2(x, row.0), state, assets, controls);
                }
            }
        }
    }

    fn menu_width(&self, id: i16, state: &Session, assets: &Assets) -> f32 {
        self.menu_items(id, state, assets)
            .items
            .iter()
            .filter(|item| !item.text.contains('\0'))
            .map(|item| measure_text(&item.text, Some(&assets.menu_font), 12, 1.).width + 100.)
            .fold(120., f32::max)
            .min(798.)
    }

    fn menu_items(
        &self,
        id: i16,
        state: &Session,
        assets: &Assets,
    ) -> centurions_content::ui::Menu {
        let mut menu = assets.menus[&id].clone();
        if id == 130 {
            menu.items.truncate(4);
            menu.items.extend(self.scenario_names.iter().map(|name| {
                centurions_content::ui::MenuItem {
                    text: name.clone(),
                    icon: 0,
                    key: 0,
                    mark: 0,
                    style: 0,
                }
            }));
        }
        if matches!(id, 132 | 146) {
            let names: Vec<_> = if id == 132 {
                self.archives.iter().map(|(_, name)| name.clone()).collect()
            } else {
                (0..state.ally_count)
                    .map(|index| {
                        centurions_content::combat::monster_name(
                            &state.allies[index * 288..(index + 1) * 288],
                        )
                        .unwrap_or_default()
                    })
                    .collect()
            };
            menu.items = names
                .into_iter()
                .map(|text| centurions_content::ui::MenuItem {
                    text,
                    icon: 0,
                    key: 0,
                    mark: 0,
                    style: 0,
                })
                .collect();
        }
        if id == 200 {
            menu.items.truncate(3);
            menu.items.extend((0..19).map(|index| {
                centurions_content::ui::MenuItem {
                    text: if state.notes[index] != 0 {
                        self.note_labels.get(index)
                    } else {
                        self.unknown_note_labels.get(index)
                    }
                    .cloned()
                    .unwrap_or_default(),
                    icon: 0,
                    key: 0,
                    mark: 0,
                    style: 0,
                }
            }));
        }
        menu
    }

    fn dropdown(
        &self,
        id: i16,
        origin: Vec2,
        state: &Session,
        assets: &Assets,
        controls: &mut Vec<Control>,
    ) -> (Rect, Vec<(f32, Option<i16>)>) {
        let menu = self.menu_items(id, state, assets);
        let width = self.menu_width(id, state, assets);
        let origin = vec2(origin.x.clamp(0., 800. - width), origin.y);
        let total = menu
            .items
            .iter()
            .filter(|item| !item.text.contains('\0'))
            .count();
        let start = if self.menu == Some(id) {
            self.menu_scroll.min(total.saturating_sub(1))
        } else {
            0
        };
        let visible = ((480. - origin.y - 4.) / 20.).floor().max(1.) as usize;
        let count = total.saturating_sub(start).min(visible);
        draw_rectangle(
            origin.x,
            origin.y,
            width,
            count as f32 * 20. + 4.,
            Color::from_rgba(238, 238, 238, 255),
        );
        draw_rectangle_lines(
            origin.x,
            origin.y,
            width,
            count as f32 * 20. + 4.,
            1.,
            BLACK,
        );
        let mut y = origin.y + 2.;
        let mut rows = Vec::new();
        for (index, item) in menu.items.iter().enumerate().skip(start).take(count) {
            if item.text.contains('\0') {
                continue;
            }
            let rect = Rect::new(origin.x + 1., y, width - 2., 20.);
            if item.is_separator() {
                draw_line(rect.x + 5., y + 10., rect.right() - 5., y + 10., 1., GRAY);
            } else {
                let enabled = self.supported(id, index + 1, state)
                    && (id != 145
                        || index < 9
                        || assets.music_ids.contains(&(20128 + index as i16 - 9)));
                let hovered = self.show_hover
                    && crate::layout::Canvas::new(screen_width(), screen_height())
                        .pointer(vec2(mouse_position().0, mouse_position().1))
                        .is_some_and(|p| rect.contains(p));
                if enabled && hovered {
                    draw_rectangle(rect.x, rect.y, rect.w, rect.h, DARKBLUE);
                }
                let color = if !enabled {
                    GRAY
                } else if hovered {
                    WHITE
                } else {
                    BLACK
                };
                let text = if id == 136 && index == 7 && self.editing {
                    assets.native_text.text(0x403658, true).unwrap_or_default()
                } else if id == 145 && index == 0 && !self.music_enabled {
                    assets.native_text.text(0x40355e, true).unwrap_or_default()
                } else if id == 145 && index == 3 && self.current_music.is_some() {
                    self.current_music
                        .and_then(|id| usize::try_from(id - 20128).ok())
                        .and_then(|index| menu.items.get(index + 9))
                        .map_or_else(|| item.text.clone(), |track| track.text.clone())
                } else {
                    item.text.clone()
                };
                assets.menu_text(&text, rect.x + 20., y + 14., 12, color);
                if item.submenu_id().is_some() {
                    assets.menu_text("▶", rect.right() - 18., y + 14., 12, color);
                } else if item.key.is_ascii_alphanumeric() {
                    assets.menu_text(
                        &format!(
                            "{}{}",
                            if cfg!(target_os = "macos") {
                                "⌘"
                            } else {
                                "Ctrl+"
                            },
                            char::from(item.key)
                        ),
                        rect.right() - 52.,
                        y + 14.,
                        12,
                        color,
                    );
                }
                if (id == 130
                    && index >= 4
                    && self
                        .selected_scenario
                        .as_ref()
                        .map_or(index == 4, |name| name == &item.text))
                    || (id == 145 && index >= 9 && self.playlist[index - 9])
                    || (id == 134 && index + 1 == self.speed)
                    || (id == 135 && index + 1 == self.volume + 2)
                    || (id == 135 && index + 1 == self.music_volume + 16)
                {
                    assets.menu_text("✓", rect.x + 4., y + 14., 12, color);
                }
                controls.push(Control {
                    rect,
                    enabled,
                    action: Action::Application(ApplicationAction::Menu(id, index + 1)),
                });
            }
            rows.push((y, item.submenu_id()));
            y += 20.;
        }
        (
            Rect::new(origin.x, origin.y, width, count as f32 * 20. + 4.),
            rows,
        )
    }

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
                self.roster_page = if wheel > 0. {
                    self.roster_page.saturating_sub(1)
                } else {
                    (self.roster_page + 1).min(
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
        if let Some(creation) = &mut self.creation
            && self.editor.is_none()
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
        if self.appearance.is_some() {
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
        if self.order.is_some() {
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
        if self.editor.is_none()
            && self.item_editor.is_none()
            && let Some(information) = self.information.as_ref()
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
        if self.editor.is_some() || self.item_editor.is_some() {
            if self
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
                return Some(Action::Application(if self.item_editor.is_some() {
                    ApplicationAction::ItemEditorItem(2)
                } else {
                    ApplicationAction::EditorItem(1)
                }));
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
            .creation
            .as_ref()
            .is_some_and(|c| c.stage == crate::creation::Stage::Keep);
        if keeping
            || self.quit_question
            || self.help_reset_question
            || self.editor_question
            || self.end_question
            || self.escape_question
        {
            let answer = |answer| {
                Action::Application(if keeping {
                    ApplicationAction::CreationAnswer(answer)
                } else if self.escape_question {
                    ApplicationAction::EscapeAnswer(answer)
                } else if self.end_question {
                    ApplicationAction::EndAnswer(answer)
                } else if self.editor_question {
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
                if get_time() - self.intro_click < 0.5 {
                    return Some(Action::Application(ApplicationAction::CloseIntro));
                }
                self.drag = Some(p);
            }
            if is_mouse_button_down(MouseButton::Left)
                && let (Some(last), Some(p)) = (self.drag, pointer)
            {
                if p.y != last.y {
                    self.intro_y += (p.y - last.y).signum() * 25.;
                }
                self.drag = Some(p);
            }
            if is_mouse_button_released(MouseButton::Left) {
                self.intro_click = get_time();
                self.drag = None;
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
