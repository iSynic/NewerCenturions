//! Menus workflow ownership.
use super::{Application, ApplicationAction, Screen};
use crate::{
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_core::{Phase, Session};
use macroquad::prelude::*;

impl Application {
    pub(super) fn supported(&self, id: i16, item: usize, state: &Session) -> bool {
        if self.about
            || self.screen == Screen::Squad
            || self.quit_question
            || self.help_reset_question
            || self.characters.editor_question
            || self.characters.editor.is_some()
            || self.characters.information.is_some()
            || self.characters.appearance.is_some()
            || self.characters.item_editor.is_some()
            || self.help.page.is_some()
            || self.selection.order.is_some()
            || self.selection.end_question
            || self.escape_question
            || self.preferences_dialog.is_some()
            || self.creature.is_some()
            || self.note.is_some()
        {
            return false;
        }
        if self.characters.creation.is_some() {
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

    pub(super) fn draw_menus(&self, state: &Session, assets: &Assets, controls: &mut Vec<Control>) {
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

    pub(super) fn menu_width(&self, id: i16, state: &Session, assets: &Assets) -> f32 {
        self.menu_items(id, state, assets)
            .items
            .iter()
            .filter(|item| !item.text.contains('\0'))
            .map(|item| measure_text(&item.text, Some(&assets.menu_font), 12, 1.).width + 100.)
            .fold(120., f32::max)
            .min(798.)
    }

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
}
