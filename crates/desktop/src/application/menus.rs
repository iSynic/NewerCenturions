//! The menu bar: its MENU resources, which items are available and how they draw.
use super::{Application, ApplicationAction, Screen};
use crate::{
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_core::{Phase, Session};
use macroquad::prelude::*;

/// The menu the player has pulled down, its open submenu and how far its items are scrolled.
#[derive(Default)]
pub struct MenuBar {
    pub menu: Option<i16>,
    pub submenu: Option<i16>,
    pub(super) scroll: usize,
}

/// MENU resource ids of the menu bar, and their item numbers (labels in comments).
/// `supported` and `handle_menu_actions` key their parallel tables on these pairs.
pub(super) mod menu {
    /// MENU 128, the Apple menu.
    pub const APPLE: i16 = 128;
    /// MENU 129, the Game menu.
    pub const GAME: i16 = 129;
    /// MENU 130, the Mission menu: its three commands and a separator, then one item per
    /// scenario.
    pub const MISSION: i16 = 130;
    /// MENU 132, the saved games found in the archives.
    pub const ARCHIVES: i16 = 132;
    /// MENU 134, the game speed choices.
    pub const GAME_SPEED: i16 = 134;
    /// MENU 135, the sound effect and music volume choices.
    pub const VOLUME: i16 = 135;
    /// MENU 136, the Character menu.
    pub const CHARACTER: i16 = 136;
    /// MENU 137, the Preferences menu.
    pub const PREFERENCES: i16 = 137;
    /// MENU 145, the music choices.
    pub const MUSIC: i16 = 145;
    /// MENU 146, the allies held over from the last battle.
    pub const ALLIES: i16 = 146;
    /// MENU 200, the Maps and Notes menu: the briefing, then the notes found so far.
    pub const MAPS_NOTES: i16 = 200;

    pub mod apple {
        /// "About New Centurions…".
        pub const ABOUT: usize = 1;
    }
    pub mod game {
        /// "Load A Saved Game".
        pub const LOAD: usize = 1;
        /// "Revert To A Previous Game".
        pub const REVERT: usize = 2;
        /// "Save Current Game".
        pub const SAVE: usize = 3;
        /// "Fast Save".
        pub const FAST_SAVE: usize = 4;
        /// "Quit".
        pub const QUIT: usize = 6;
    }
    pub mod mission {
        /// "Begin New Mission".
        pub const BEGIN: usize = 1;
        /// "View Current Mission Briefing".
        pub const BRIEFING: usize = 2;
        /// "End This Mission".
        pub const END: usize = 3;
        /// The first scenario title; scenarios run through item 254.
        pub const FIRST_SCENARIO: usize = 5;
        /// The last item that names a scenario.
        pub const LAST_SCENARIO: usize = 254;
    }
    pub mod game_speed {
        /// "Lets Open Her Up!" (speed 1).
        pub const FASTEST: usize = 1;
        /// "ZZzzZZzzZzZz" (speed 11).
        pub const SLOWEST: usize = 11;
    }
    pub mod volume {
        /// "Let's Hunt Wabbits." (sound volume 0).
        pub const SOUND_SILENT: usize = 2;
        /// "Wake the Neighbors!" (sound volume 10).
        pub const SOUND_LOUDEST: usize = 12;
        /// "Let me listen to my hard drive spin." (music volume 0).
        pub const MUSIC_SILENT: usize = 16;
        /// "Damage my eardrums, I'm an impetuous youth!" (music volume 10).
        pub const MUSIC_LOUDEST: usize = 26;
    }
    pub mod character {
        /// "Squad Order".
        pub const SQUAD_ORDER: usize = 1;
        /// "Change Warrior Portrait".
        pub const CHANGE_PORTRAIT: usize = 2;
        /// "Change Warrior Icon".
        pub const CHANGE_ICON: usize = 3;
        /// "Generate New Warrior".
        pub const GENERATE_WARRIOR: usize = 5;
        /// "Modify Squad".
        pub const MODIFY_SQUAD: usize = 7;
        /// "Warrior Edtior (DEMO)", retitled Activate/Deactivate Warrior Edit.
        pub const WARRIOR_EDIT: usize = 8;
    }
    pub mod preferences {
        /// "Sound Volume" (the first item).
        pub const SOUND_VOLUME: usize = 1;
        /// "Set Preferences".
        pub const SET_PREFERENCES: usize = 3;
        /// "Refresh Screen".
        pub const REFRESH_SCREEN: usize = 4;
        /// "Start Popup Help" (the last supported item).
        pub const START_POPUP_HELP: usize = 5;
    }
    pub mod music {
        /// "Stop Playing Music".
        pub const STOP: usize = 1;
        /// "New Centurions Theme"; titles run through "Energy Overload" (22).
        pub const FIRST_TITLE: usize = 10;
        /// The last item that names a music title.
        pub const LAST_TITLE: usize = 22;
    }
    pub mod maps_notes {
        /// "Mission Briefing".
        pub const BRIEFING: usize = 1;
        /// The first appended note; notes run through item 22.
        pub const FIRST_NOTE: usize = 4;
        /// The last note item.
        pub const LAST_NOTE: usize = 22;
    }
}

impl Application {
    /// Whether menu item `item` of menu `id` can be chosen now; none can while a
    /// dialog is open.
    pub(super) fn supported(&self, id: i16, item: usize, state: &Session) -> bool {
        if self.about
            || self.screen == Screen::Squad
            || self.questions.quit
            || self.questions.help_reset
            || self.characters.editor_question
            || self.characters.editor.is_some()
            || self.characters.information.is_some()
            || self.characters.appearance.is_some()
            || self.characters.item_editor.is_some()
            || self.help.page.is_some()
            || self.selection.order.is_some()
            || self.selection.end_question
            || self.questions.escape
            || self.preferences_dialog.is_some()
            || self.creature.is_some()
            || self.notes.open.is_some()
        {
            return false;
        }
        if self.characters.creation.is_some() {
            return false;
        }
        use menu::{
            apple, character, game, game_speed, maps_notes, mission, music, preferences, volume,
        };
        let game = self.screen == Screen::Game && state.phase != Phase::SquadSelection;
        let exploring = game
            && state.phase == Phase::Exploration
            && state.medical_continuation.is_none()
            && state
                .exploration
                .as_ref()
                .is_some_and(|p| p.presentation.is_none());
        match (id, item) {
            (menu::APPLE, apple::ABOUT) => true,
            (menu::MAPS_NOTES, maps_notes::FIRST_NOTE..=maps_notes::LAST_NOTE) => {
                game && state.notes[item - maps_notes::FIRST_NOTE] != 0
            }
            (menu::ARCHIVES, _) => game && item > 0 && item <= self.archives.len(),
            (menu::ALLIES, _) => game && item > 0 && item <= state.held_over,
            (menu::CHARACTER, character::WARRIOR_EDIT) => true,
            (menu::CHARACTER, character::GENERATE_WARRIOR) => {
                self.screen == Screen::Title && state.phase == Phase::SquadSelection
            }
            (menu::CHARACTER, character::CHANGE_PORTRAIT | character::CHANGE_ICON) => {
                exploring && !state.warriors.is_empty()
            }
            (menu::MISSION, mission::END) => game && state.medical_continuation.is_none(),
            (menu::CHARACTER, character::SQUAD_ORDER | character::MODIFY_SQUAD) => {
                exploring
                    && state
                        .exploration
                        .as_ref()
                        .is_some_and(|p| p.script.is_none())
            }
            (menu::GAME, game::QUIT)
            | (menu::MISSION, mission::BRIEFING)
            | (menu::PREFERENCES, preferences::SOUND_VOLUME..=preferences::START_POPUP_HELP) => {
                true
            }
            (menu::MISSION, mission::FIRST_SCENARIO..=mission::LAST_SCENARIO) => {
                self.screen == Screen::Title
                    && state.phase == Phase::SquadSelection
                    && item - mission::FIRST_SCENARIO < self.scenario_names.len()
            }
            (menu::GAME, game::LOAD) => self.screen == Screen::Title,
            (menu::GAME, game::REVERT) | (menu::MAPS_NOTES, maps_notes::BRIEFING) => {
                game && state.phase != Phase::Combat
            }
            (menu::GAME, game::SAVE | game::FAST_SAVE) => exploring,
            (menu::MISSION, mission::BEGIN) => {
                state.phase == Phase::SquadSelection && self.screen == Screen::Title
            }
            (menu::GAME_SPEED, game_speed::FASTEST..=game_speed::SLOWEST)
            | (
                menu::VOLUME,
                volume::SOUND_SILENT..=volume::SOUND_LOUDEST
                | volume::MUSIC_SILENT..=volume::MUSIC_LOUDEST,
            ) => true,
            (menu::MUSIC, music::STOP | music::FIRST_TITLE..=music::LAST_TITLE) => true,
            _ => false,
        }
    }

    /// The menu bar's titles laid out left to right, each with its rectangle.
    pub(super) fn headings(&self, assets: &Assets) -> Vec<(i16, Rect)> {
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

    /// Draws the menu bar across the top of the window and the pulled-down menu, and
    /// adds a control for each choosable item.
    pub(super) fn draw_menus(&self, state: &Session, assets: &Assets, controls: &mut Vec<Control>) {
        draw_rectangle(0., 0., 800., 20., Color::from_rgba(238, 238, 238, 255));
        for (id, rect) in self.headings(assets) {
            if self.menu_bar.menu == Some(id) {
                draw_rectangle(rect.x, rect.y, rect.w, rect.h, DARKBLUE);
            }
            assets.menu_text(
                assets.menu_title(id).unwrap_or(""),
                rect.x + 8.,
                14.,
                12,
                if self.menu_bar.menu == Some(id) {
                    WHITE
                } else {
                    BLACK
                },
            );
            if self.menu_bar.menu == Some(id) {
                let (bounds, rows) = self.dropdown(id, vec2(rect.x, 20.), state, assets, controls);
                if let Some(sub) = self.menu_bar.submenu
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

    /// The width of a pulled-down menu: wide enough for its longest item.
    pub(super) fn menu_width(&self, id: i16, state: &Session, assets: &Assets) -> f32 {
        self.menu_items(id, state, assets)
            .items
            .iter()
            .filter(|item| !item.text.contains('\0'))
            .map(|item| measure_text(&item.text, Some(&assets.menu_font), 12, 1.).width + 100.)
            .fold(120., f32::max)
            .min(798.)
    }

    /// The items of menu `id`, with the scenario names, archive names, held-over allies or note
    /// labels filled in where the menu lists them; a note not yet found shows its unknown label.
    pub(super) fn menu_items(
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
                (0..state.held_over)
                    .map(|index| {
                        centurions_content::combat::monster_name(
                            &state.holdover[index * 288..(index + 1) * 288],
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
                        self.notes.labels.get(index)
                    } else {
                        self.notes.unknown_labels.get(index)
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

    /// Draws the pulled-down menu at `origin`, kept inside the window, and returns
    /// its rectangle and item rows.
    pub(super) fn dropdown(
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
        let start = if self.menu_bar.menu == Some(id) {
            self.menu_bar.scroll.min(total.saturating_sub(1))
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
                    assets
                        .game_text
                        .text(crate::game_strings::DEACTIVATE_WARRIOR_EDIT, true)
                        .unwrap_or_default()
                } else if id == 145 && index == 0 && !self.audio.music_enabled {
                    assets
                        .game_text
                        .text(crate::game_strings::START_MUSIC, true)
                        .unwrap_or_default()
                } else if id == 145 && index == 3 && self.audio.current_music.is_some() {
                    self.audio
                        .current_music
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
                    || (id == 145 && index >= 9 && self.audio.playlist[index - 9])
                    || (id == 134 && index + 1 == self.speed)
                    || (id == 135 && index + 1 == self.audio.volume + 2)
                    || (id == 135 && index + 1 == self.audio.music_volume + 16)
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
}
