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
    OpenItemEditor(usize, usize),
    ItemEditorItem(usize),
}

pub struct Application {
    pub screen: Screen,
    pub quit: bool,
    pub quit_after_save: bool,
    quit_question: bool,
    help_reset_question: bool,
    editor_question: bool,
    pub editing: bool,
    editor: Option<crate::editor::WarriorEditor>,
    item_editor: Option<crate::editor::ItemEditor>,
    pub sound: bool,
    pub volume: usize,
    pub music_enabled: bool,
    pub music_request: Option<i16>,
    pub speed: usize,
    pub menu: Option<i16>,
    pub submenu: Option<i16>,
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
    pub fn new(screen: Screen) -> Self {
        Self::at(screen, get_time())
    }

    pub(crate) fn at(screen: Screen, now: f64) -> Self {
        Self {
            screen,
            quit: false,
            quit_after_save: false,
            quit_question: false,
            help_reset_question: false,
            editor_question: false,
            editing: false,
            editor: None,
            item_editor: None,
            sound: true,
            volume: 10,
            music_enabled: true,
            music_request: Some(20128),
            speed: 6,
            menu: None,
            submenu: None,
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
        if self.screen == Screen::Game && self.help.page.is_none() {
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
        self.screen != Screen::Game
            || self.menu.is_some()
            || self.quit_question
            || self.help.page.is_some()
            || self.help_reset_question
            || self.editor_question
            || self.editor.is_some()
            || self.item_editor.is_some()
    }

    pub fn handle(
        &mut self,
        action: ApplicationAction,
        state: &mut Session,
        content: &Content,
    ) -> Result<(Option<Action>, Vec<Event>), String> {
        let mut next = None;
        let mut events = Vec::new();
        match action {
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
            }
            ApplicationAction::OpenEditor(warrior) => {
                if !self.editing || self.screen != Screen::Game || state.phase != Phase::Exploration
                {
                    return Err("Warrior editing is unavailable.".into());
                }
                self.editor = Some(crate::editor::WarriorEditor::new(state, warrior)?);
            }
            ApplicationAction::EditorItem(item) => {
                let editor = self.editor.as_mut().ok_or("Warrior editor is not open.")?;
                if editor.level_information || editor.scans.is_some() {
                    events = editor.handle(item, content)?;
                    return Ok((next, events));
                }
                match item {
                    1 | 4 => {
                        if item == 1 {
                            editor.cancel_current()?;
                        }
                        editor.draft.validate(content).map_err(|e| e.to_string())?;
                        *state = editor.draft.clone();
                        self.editor = None;
                    }
                    _ => events = editor.handle(item, content)?,
                }
            }
            ApplicationAction::ScanAnswer(answer) => {
                self.editor
                    .as_mut()
                    .ok_or("Warrior editor is not open.")?
                    .scan_answer(answer, content)?;
            }
            ApplicationAction::Difficulty(level) => {
                if self.screen != Screen::Squad || !(-2..=2).contains(&level) {
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
                if self.squad.is_empty() {
                    self.screen = Screen::Title;
                } else {
                    let mut candidate = Session::new(content);
                    apply(
                        Command::SetDifficulty {
                            level: self.difficulty,
                        },
                        &mut candidate,
                        content,
                    )
                    .map_err(|error| error.to_string())?;
                    events = apply(
                        Command::SelectStarters {
                            indices: self.squad.clone(),
                        },
                        &mut candidate,
                        content,
                    )
                    .and_then(|mut events| {
                        events.extend(apply(
                            Command::EnterMission {
                                name: "The Thunder of Guns".into(),
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
                self.menu = None;
                self.submenu = None;
                match (id, item) {
                    (136, 8) => {
                        if self.editing {
                            self.editing = false;
                        } else {
                            self.editor_question = true;
                        }
                    }
                    (129, 1 | 2) => {
                        next = Some(Action::Load);
                    }
                    (129, 3 | 4) => next = Some(Action::Save),
                    (129, 6) => {
                        if self.screen == Screen::Title {
                            self.quit = true;
                        } else {
                            self.quit_question = true;
                            events.push(Event::Sound { resource_id: 20005 });
                        }
                    }
                    (130, 1) => {
                        self.difficulty = state.difficulty;
                        self.screen = Screen::Squad;
                        self.squad.clear();
                        self.selected = None;
                        self.member = None;
                        self.help.open(121, 124)?;
                        events.push(Event::Sound { resource_id: 10004 });
                    }
                    (130, 2) | (200, 1) => self.open_intro(self.screen, None),
                    (130, 5) => {}
                    (137, 4) => {}
                    (137, 5) => {
                        self.help.resume()?;
                        self.help_reset_question = true;
                    }
                    (145, 1) => self.music_enabled = !self.music_enabled,
                    (145, 10..=22) => {
                        self.music_enabled = true;
                        self.music_request = Some(20128 + item as i16 - 10);
                    }
                    (134, 1..=11) => self.speed = item,
                    (135, 2..=12) => {
                        self.volume = item - 2;
                        self.sound = self.volume != 0;
                    }
                    _ => return Err("This original menu action has not been reconstructed.".into()),
                }
            }
        }
        Ok((next, events))
    }

    fn supported(&self, id: i16, item: usize, state: &Session) -> bool {
        if self.screen == Screen::Squad
            || self.quit_question
            || self.help_reset_question
            || self.editor_question
            || self.editor.is_some()
        {
            return false;
        }
        let game = self.screen == Screen::Game && state.phase != Phase::SquadSelection;
        match (id, item) {
            (136, 8) => true,
            (129, 6) | (130, 2 | 5) | (137, 1 | 2 | 4 | 5) => true,
            (129, 1) => self.screen == Screen::Title,
            (129, 2) | (200, 1) => game && state.phase != Phase::Combat,
            (129, 3 | 4) => game && state.phase == Phase::Exploration,
            (130, 1) => state.phase == Phase::SquadSelection && self.screen == Screen::Title,
            (134, 1..=11) | (135, 2..=12) => true,
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
                assets.text(
                    "New Centurions™ Copyright ©2001, Tim Phillips / Fantasoft LLC",
                    110.,
                    535.,
                    12,
                    WHITE,
                );
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
                        .map_or(1000, |p| (p.map + 1000) as i16)
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
        if let Some(editor) = &self.item_editor {
            controls.clear();
            editor.draw(content, assets, &mut controls);
        } else if let Some(editor) = &self.editor {
            controls.clear();
            editor.draw(content, assets, &mut controls);
        } else if self.editing && self.screen == Screen::Game && state.phase == Phase::Exploration {
            if ui.inspection == Some(presentation::Inspection::Items) {
                for (row, (slot, _)) in state.warriors[ui.selected]
                    .inventory()
                    .unwrap_or_default()
                    .iter()
                    .enumerate()
                    .skip(ui.page * 8)
                    .take(8)
                    .enumerate()
                {
                    let rect = Rect::new(450., 162. + row as f32 * 39., 50., 32.);
                    assets.panel(234, rect);
                    controls.push(Control {
                        rect,
                        enabled: true,
                        action: Action::Application(ApplicationAction::OpenItemEditor(
                            ui.selected,
                            slot,
                        )),
                    });
                }
            }
            for (index, _) in state
                .warriors
                .iter()
                .enumerate()
                .filter(|_| ui.inspection.is_none())
            {
                let mut rect = crate::layout::row(index);
                rect.w = 50.;
                controls.push(Control {
                    rect,
                    enabled: true,
                    action: Action::Application(ApplicationAction::OpenEditor(index)),
                });
            }
        }
        if self.quit_question || self.help_reset_question || self.editor_question {
            let origin = vec2(221., 203.);
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
                &assets.questions[if self.editor_question {
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
                    action: Action::Application(if self.editor_question {
                        ApplicationAction::EditorAnswer(answer)
                    } else if self.help_reset_question {
                        ApplicationAction::HelpResetAnswer(answer)
                    } else {
                        ApplicationAction::QuitAnswer(answer)
                    }),
                });
            }
        }
        if (self.screen != Screen::Game || self.editor.is_some() || self.item_editor.is_some())
            && let Some(message) = ui.messages.back()
        {
            draw_rectangle(0., 567., 800., 33., BLACK);
            assets.text(message, 14., 587., 12, YELLOW);
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
        for (i, warrior) in content.warriors.iter().enumerate() {
            let rect = Rect::new(origin.x + 12., origin.y + 30. + i as f32 * 20., 155., 14.);
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
            if let Ok(id) = word(&content.warriors[*warrior].bytes, 0x54)
                && let Some(texture) = assets.portraits.get(&id)
            {
                draw_texture(texture, rect.x, rect.y, WHITE);
            }
            assets.text(
                &content.warriors[*warrior].name,
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
                enabled: true,
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
                    Some(&assets.font),
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
            assets.text(
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
                    let sub_width = self.menu_width(sub, assets);
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

    fn menu_width(&self, id: i16, assets: &Assets) -> f32 {
        assets.menus[&id]
            .items
            .iter()
            .filter(|item| !item.text.contains('\0'))
            .map(|item| measure_text(&item.text, Some(&assets.font), 12, 1.).width + 100.)
            .fold(120., f32::max)
            .min(798.)
    }

    fn dropdown(
        &self,
        id: i16,
        origin: Vec2,
        state: &Session,
        assets: &Assets,
        controls: &mut Vec<Control>,
    ) -> (Rect, Vec<(f32, Option<i16>)>) {
        let menu = &assets.menus[&id];
        let width = self.menu_width(id, assets);
        let origin = vec2(origin.x.clamp(0., 800. - width), origin.y);
        let count = menu
            .items
            .iter()
            .filter(|item| !item.text.contains('\0'))
            .count();
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
        for (index, item) in menu.items.iter().enumerate() {
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
                let text = if id == 145 && index == 0 && !self.music_enabled {
                    "Start Playing Music"
                } else {
                    &item.text
                };
                assets.text(text, rect.x + 20., y + 14., 12, color);
                if item.submenu_id().is_some() {
                    assets.text("▶", rect.right() - 18., y + 14., 12, color);
                } else if item.key.is_ascii_alphanumeric() {
                    assets.text(
                        &format!("Ctrl+{}", char::from(item.key)),
                        rect.right() - 52.,
                        y + 14.,
                        12,
                        color,
                    );
                }
                if (id == 130 && index + 1 == 5)
                    || (id == 136 && index == 7 && self.editing)
                    || (id == 134 && index + 1 == self.speed)
                    || (id == 135 && index + 1 == self.volume + 2)
                {
                    assets.text("✓", rect.x + 4., y + 14., 12, color);
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
        if self.quit_question || self.help_reset_question || self.editor_question {
            let answer = |answer| {
                Action::Application(if self.editor_question {
                    ApplicationAction::EditorAnswer(answer)
                } else if self.help_reset_question {
                    ApplicationAction::HelpResetAnswer(answer)
                } else {
                    ApplicationAction::QuitAnswer(answer)
                })
            };
            if is_key_pressed(KeyCode::Y) {
                return Some(answer(true));
            }
            if is_key_pressed(KeyCode::N) {
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
            for (key, letter) in [
                (KeyCode::L, b'L'),
                (KeyCode::R, b'R'),
                (KeyCode::S, b'S'),
                (KeyCode::F, b'F'),
                (KeyCode::Q, b'Q'),
                (KeyCode::B, b'B'),
                (KeyCode::V, b'V'),
                (KeyCode::K, b'K'),
                (KeyCode::W, b'W'),
            ] {
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
