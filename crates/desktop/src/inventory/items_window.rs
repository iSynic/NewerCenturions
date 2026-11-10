//! The Items window: one warrior's pack in eleven rows, the PICT 6 buttons beneath the warrior
//! panel (DLOG 143), the member strip, the owner's portrait and name, and the detail panel in the
//! 800 by 600 window.
use super::{Inventory, InventoryAction};
use crate::{
    assets::Assets,
    layout, portrait,
    presentation::{Action, Control},
};
use centurions_content::{Content, InventoryRecord, records::CharacterInformation};
use centurions_core::Session;
use macroquad::prelude::*;

/// CNTL 152, Use.
pub(super) const USE: Rect = Rect::new(284., 341., 71., 39.);
/// CNTL 140, View: the item's information.
pub(super) const VIEW: Rect = Rect::new(355., 341., 71., 40.);
/// CNTL 176, Ammo.
pub(super) const AMMO: Rect = Rect::new(426., 341., 71., 39.);
/// CNTL 143, Weapon.
const WEAPON: Rect = Rect::new(497., 341., 72., 40.);
/// CNTL 148, Drop.
const DROP: Rect = Rect::new(569., 341., 71., 39.);
/// CNTL 150 and 151, the scroll arrows.
const SCROLL: [Rect; 2] = [
    Rect::new(294., 386., 50., 25.),
    Rect::new(294., 411., 50., 25.),
];
/// CNTL 145, the member strip.
const STRIP: Rect = Rect::new(352., 385., 48., 72.);
/// CNTL 146, the owner's portrait.
const PORTRAIT: Rect = Rect::new(408., 386., 50., 50.);
/// CNTL 198, Done.
const DONE: Rect = Rect::new(294., 438., 49., 19.);
/// CNTL 163, Trade.
const TRADE: Rect = Rect::new(466., 386., 50., 50.);
/// CNTL 161, Refit.
const REFIT: Rect = Rect::new(524., 386., 50., 50.);
/// CNTL 170 and 171, Join and Split.
const JOIN: Rect = Rect::new(581., 386., 50., 25.);
const SPLIT: Rect = Rect::new(581., 411., 50., 25.);
/// The rows the window shows.
pub(super) const ROWS: usize = 11;
/// The cicn behind the highlighted row.
const HIGHLIGHT: i16 = 174;
/// The cicn a row keeps once the highlight has left it.
const UNHIGHLIGHT: i16 = 173;
/// The cicn that marks the owner in the member strip.
const OWNER: i16 = 145;

/// A control of the Items window placed on the screen: its CNTL rectangle in the small window,
/// moved right by the 800 by 600 shift and down by the menu bar.
///
/// Original: items
pub(super) fn place(rect: Rect, shift: f32) -> Rect {
    Rect::new(rect.x + shift, rect.y + layout::MENU_BAR, rect.w, rect.h)
}

/// The highlight rectangle of display row `row`, 50 by 40 at the window's left edge.
fn highlight(row: usize) -> Rect {
    Rect::new(0., layout::MENU_BAR + 11. + row as f32 * 40., 50., 40.)
}

/// The cell member `member` has in the member strip: the first three down a column 24 apart from
/// the strip's corner, the rest in the next column 24 to the right; drawing starts the second
/// column 72 higher and the clicks 71.
///
/// Original: items
fn strip_cell(member: usize, shift: f32, click: bool) -> Rect {
    let (across, up) = match (member >= 3, click) {
        (false, _) => (0., 0.),
        (true, false) => (24., 72.),
        (true, true) => (24., 71.),
    };
    Rect::new(
        shift + 353. + across,
        layout::MENU_BAR + 386. + member as f32 * 24. - up,
        22.,
        22.,
    )
}

impl Inventory {
    /// Whether the Items window, rather than the trade window, is showing.
    pub(super) fn items_window(&self, state: &Session) -> bool {
        !self.trade && !Self::shop(state)
    }

    /// The 800 by 600 window's shift of the controls.
    pub(super) fn shift(&self) -> f32 {
        crate::placement::Placement::new(self.quick_info).origin_left as f32
    }

    /// The owner's member and the slot of the highlighted row, for the item editor.
    pub fn highlighted_slot(&self) -> (usize, usize) {
        (
            self.members[0],
            self.scroll[0] + self.display_row.max(0) as usize,
        )
    }

    /// The rectangle of the View button, which the item editor's picture covers while warriors
    /// can be edited.
    pub fn view_button(&self) -> Rect {
        place(VIEW, self.shift())
    }

    /// Draws the Items window: the white window, the eleven rows of the owner's pack over the
    /// list pattern with the highlight, PICT 6 with its buttons, the member strip, the owner's
    /// portrait and name, the warrior panel and, in the 800 by 600 window, the detail panel.
    ///
    /// Original: items, updateitems, updatecharinfo, quickinfo
    pub(super) fn draw_items_window(
        &self,
        content: &Content,
        state: &Session,
        assets: &Assets,
        class_label: bool,
        controls: &mut Vec<Control>,
    ) {
        controls.clear();
        let shift = self.shift();
        let large = self.quick_info;
        let window = vec2(0., layout::MENU_BAR);
        let height = if large { 580. } else { 460. };
        draw_rectangle(window.x, window.y, 640. + shift, height, WHITE);
        let owner = self.members[0];
        let Some(warrior) = state.warriors.get(owner) else {
            return;
        };
        let records = warrior.inventory().unwrap_or_default();
        let row = self.settled_row(records.len()).max(0) as usize;
        for display in 0..ROWS {
            if self.unhighlighted & (1 << display) != 0
                && let Some(icon) = assets.icons.get(&UNHIGHLIGHT)
            {
                crate::assets::texture_in(icon, highlight(display));
            }
            let top = layout::MENU_BAR + 15. + display as f32 * 40.;
            crate::assets::fill_pattern(
                &assets.list_background,
                Rect::new(10., top, 274. + shift, 32.),
                window,
            );
            if display == row
                && let Some(icon) = assets.icons.get(&HIGHLIGHT)
            {
                crate::assets::texture_in(icon, highlight(display));
            }
            if let Some(item) = records.get(self.scroll[0] + display) {
                self.draw_pack_row(content, assets, item, vec2(10., top));
            }
        }
        controls.push(Control {
            rect: Rect::new(0., layout::MENU_BAR + 20., 250. + shift, 440.),
            enabled: true,
            action: Action::Inventory(InventoryAction::List),
        });
        assets.panel(6, place(Rect::new(284., 341., 356., 119.), shift));
        for (rect, action) in [
            (USE, InventoryAction::Use),
            (VIEW, InventoryAction::Information),
            (AMMO, InventoryAction::Ammunition),
            (WEAPON, InventoryAction::Weapon),
            (DROP, InventoryAction::Drop),
            (SCROLL[0], InventoryAction::Scroll { side: 0, delta: -1 }),
            (SCROLL[1], InventoryAction::Scroll { side: 0, delta: 1 }),
            (PORTRAIT, InventoryAction::View),
            (DONE, InventoryAction::Done),
            (TRADE, InventoryAction::Trade),
            (REFIT, InventoryAction::Refit(0)),
            (JOIN, InventoryAction::Join),
            (SPLIT, InventoryAction::Split),
        ] {
            controls.push(Control {
                rect: place(rect, shift),
                enabled: true,
                action: Action::Inventory(action),
            });
        }
        self.draw_member_strip(state, assets, shift, controls);
        let at = vec2(shift + 417., layout::MENU_BAR + 395.);
        portrait::plot(
            assets,
            warrior,
            at,
            &portrait::Marks {
                class_label,
                conditions: Some((shift + 628., 49.)),
                automatic: state
                    .combat
                    .as_ref()
                    .is_some_and(|combat| combat.do_auto.get(owner).copied().unwrap_or(false)),
                in_combat: false,
            },
        );
        let stamina = warrior.record().stamina().unwrap_or(0);
        for (id, rect, shown) in [
            (2019, place(PORTRAIT, shift), stamina < 1),
            (2015, Rect::new(at.x, at.y, 32., 32.), stamina < -9),
        ] {
            if shown && let Some(icon) = assets.icons.get(&id) {
                crate::assets::texture_in(icon, rect);
            }
        }
        assets.text(
            &warrior.name,
            shift + 475.,
            layout::MENU_BAR + 453.,
            12,
            layout::YELLOW_COLOR,
        );
        self.draw_warrior_panel(content, state, assets, owner, shift);
        if large
            && let Some(item) = records.get(self.scroll[0] + row)
            && let Some(map) = state.land(content)
        {
            self.detail(content, map, item, false, assets);
        }
        self.draw_information(content, state, assets, controls);
        self.draw_ammunition(assets, controls);
    }

    /// One row of the pack: the worn background behind the icon, the icon, the name with the
    /// charge and loaded ammunition, and the statistics.
    ///
    /// Original: updateitems, ploticon, showcharge2
    fn draw_pack_row(&self, content: &Content, assets: &Assets, item: &InventoryRecord, at: Vec2) {
        let definition = content
            .items
            .get(item.id().unsigned_abs() as usize)
            .map(|bytes| centurions_content::records::ItemDefinition(bytes));
        if item.equipped() != 0 {
            let kind = definition
                .as_ref()
                .and_then(|d| d.item_type().ok())
                .unwrap_or(0);
            let background = if kind < 0 || (23..=25).contains(&kind) {
                131
            } else {
                176
            };
            if let Some(icon) = assets.icons.get(&background) {
                crate::assets::icon_in(icon, at);
            }
        }
        if let Some(icon) = definition
            .as_ref()
            .and_then(|d| d.icon().ok())
            .and_then(|id| assets.icons.get(&id))
        {
            crate::assets::icon_in(icon, at);
        }
        let name = assets.names.get(&item.id()).map_or("", String::as_str);
        let x = at.x + 42.;
        assets.text(name, x, at.y + 17., 12, BLACK);
        if let Ok(amount) = item.charge()
            && amount > 0
        {
            let mut pen = x + measure_text(name, Some(&assets.font), 12, 1.).width;
            if let Ok(ammo) = item.ammunition_id()
                && ammo != 0
                && let Some(bytes) = content.items.get(ammo.unsigned_abs() as usize)
                && let Ok(icon) = centurions_content::records::ItemDefinition(bytes).icon()
                && let Some(image) = assets.icons.get(&icon)
            {
                crate::assets::texture_in(image, Rect::new(pen + 4., at.y + 2., 16., 16.));
                pen += 20.;
            }
            if let Ok(prefix) = assets
                .game_text
                .text(crate::game_strings::QUANTITY_PREFIX, true)
            {
                assets.text(&format!("{prefix}{amount}"), pen, at.y + 17., 12, BLUE);
            }
        }
        self.row_stats(content, item, assets, x, at.y + 30.);
    }

    /// The member strip: each member's portrait cicn at 22 pixels with the owner marked, and a
    /// click target on every member's cell inside the strip's control.
    ///
    /// Original: items
    fn draw_member_strip(
        &self,
        state: &Session,
        assets: &Assets,
        shift: f32,
        controls: &mut Vec<Control>,
    ) {
        let strip = place(STRIP, shift);
        for (member, warrior) in state.warriors.iter().enumerate() {
            let cell = strip_cell(member, shift, false);
            if let Some(image) = warrior
                .record()
                .portrait()
                .ok()
                .and_then(|id| assets.portraits.get(&id))
            {
                crate::assets::texture_in(image, cell);
            }
            if member == self.members[0]
                && let Some(mark) = assets.icons.get(&OWNER)
            {
                crate::assets::texture_in(mark, cell);
            }
            if let Some(rect) = strip_cell(member, shift, true).intersect(strip) {
                controls.push(Control {
                    rect,
                    enabled: true,
                    action: Action::Inventory(InventoryAction::Warrior { side: 0, member }),
                });
            }
        }
    }

    /// The warrior panel (DLOG 143) at the window's right over its 'ppat' 131 background: the
    /// attributes, movement, load and capacity, the firearm and melee damage (an empty readout
    /// when below one, with cicn 2109 when a missile weapon is in hand), the accuracies, dodges
    /// and psionic resistance, the stamina and psi pools, the shields and armor, the eight
    /// resistances, and up to five of the warrior's conditions in the below-maximum colour. A
    /// value past the field's width is drawn at size 10.
    ///
    /// Original: updatecharinfo
    fn draw_warrior_panel(
        &self,
        content: &Content,
        state: &Session,
        assets: &Assets,
        owner: usize,
        shift: f32,
    ) {
        let origin = vec2(285. + shift, layout::MENU_BAR);
        assets.draw_patterned_dialog(143, origin, &assets.movie_background);
        let Some(warrior) = state.warriors.get(owner) else {
            return;
        };
        let record = warrior.record();
        let sheet = CharacterInformation(&warrior.bytes);
        let blank = assets
            .game_text
            .text(crate::game_strings::BLANK, true)
            .unwrap_or_default();
        let put = |item: usize, text: &str, size: u16, color: Color| {
            let rect = assets.dialog_rect_at(143, item, origin);
            assets.wrapped_color(text, rect, size, crate::assets::line_height(size), color);
        };
        let number = |item: usize, value: i16, size: u16| {
            put(item, &value.to_string(), size, layout::YELLOW_COLOR);
        };
        for index in 0..7 {
            number(2 + index, sheet.attribute(index).unwrap_or(0), 12);
        }
        number(28, sheet.movement_rate().unwrap_or(0), 12);
        number(9, record.load().unwrap_or(0), 12);
        number(47, record.load_max().unwrap_or(0), 12);
        for (item, firearm) in [(10, true), (29, false)] {
            let damage =
                centurions_core::items::sheet_damage(state, content, owner, firearm).unwrap_or(0);
            if damage < 1 {
                put(item, &blank, 12, layout::YELLOW_COLOR);
                if firearm
                    && record.equipped_weapon().unwrap_or(0) != 0
                    && let Some(icon) = assets.icons.get(&2109)
                {
                    let rect = assets.dialog_rect_at(143, 46, origin);
                    crate::assets::texture_in(icon, Rect::new(rect.x, rect.y, 32., 32.));
                }
            } else {
                number(item, damage, 12);
            }
        }
        for (item, value) in [
            (17, record.missile_accuracy()),
            (27, record.melee_accuracy()),
            (11, record.missile_dodge()),
            (12, record.melee_dodge()),
            (16, sheet.psionic_resistance()),
        ] {
            number(item, value.unwrap_or(0), 12);
        }
        let stamina_size = if record.stamina_max().unwrap_or(0) > 999 {
            10
        } else {
            12
        };
        number(13, record.stamina().unwrap_or(0), stamina_size);
        number(14, record.stamina_max().unwrap_or(0), stamina_size);
        let maximum_psi = record.psi_points_max().unwrap_or(0);
        if maximum_psi != 0 {
            let size = if maximum_psi > 999 { 10 } else { 12 };
            number(44, record.psi_points().unwrap_or(0), size);
            number(15, maximum_psi, size);
        } else {
            put(44, &blank, 12, layout::YELLOW_COLOR);
            put(15, &blank, 12, layout::YELLOW_COLOR);
        }
        number(45, record.shield_chance().unwrap_or(0), 12);
        let capacity = sheet.shield_capacity().unwrap_or(0);
        let size = if capacity > 999 { 10 } else { 12 };
        number(40, record.shield_points().unwrap_or(0), size);
        number(42, capacity, size);
        number(41, record.armor_rating().unwrap_or(0), 12);
        number(43, sheet.shield_cost().unwrap_or(0), 12);
        for index in 0..8 {
            let value = centurions_content::records::SavingThrow::from_index(index as i16)
                .and_then(|save| record.save(save).ok())
                .unwrap_or(0);
            let size = if !(-99..=999).contains(&value) {
                10
            } else {
                12
            };
            number(32 + index, value, size);
        }
        let names = assets
            .character_strings
            .get(&133)
            .and_then(|bytes| centurions_content::ui::string_list(bytes).ok())
            .unwrap_or_default();
        let shown = (0..40)
            .filter(|index| record.condition_duration(*index).unwrap_or(0) != 0)
            .take(5);
        for (place, index) in shown.enumerate() {
            let name = names.get(index).map_or("", String::as_str);
            put(20 + place, name, 12, layout::POOL_BELOW_MAXIMUM);
        }
    }
}
