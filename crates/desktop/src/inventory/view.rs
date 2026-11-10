//! The Items screen: the two item lists, their categories and controls.
use super::{Inventory, InventoryAction, portraits::STOCKPILE};
use crate::{
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_content::{Content, InventoryRecord, Warrior};
use centurions_core::Session;
use macroquad::prelude::*;
impl Inventory {
    /// Draws the Items window, or the trade or shop window: both warriors' lists with their
    /// portraits and scroll bars and the controls, each where the window's controls and pictures
    /// put it. `class_label` is the Append M.O.S. To Portrait preference.
    ///
    /// Original: Showitems, change, Display
    pub fn draw(
        &self,
        content: &Content,
        state: &Session,
        assets: &Assets,
        class_label: bool,
        controls: &mut Vec<Control>,
    ) {
        if self.items_window(state) {
            self.draw_items_window(content, state, assets, class_label, controls);
            return;
        }
        controls.clear();
        draw_rectangle(0., 20., 800., 580., WHITE);
        assets.panel(170, Rect::new(343., 20., 114., 387.));
        assets.panel(148, Rect::new(80., 407., 640., 73.));
        assets.panel(227, Rect::new(0., 407., 80., 73.));
        assets.panel(227, Rect::new(720., 407., 80., 73.));
        assets.panel(226, Rect::new(0., 480., 800., 120.));
        let mut counts = [0; 2];
        for (side, count) in counts.iter_mut().enumerate() {
            let Some(warrior) = state
                .warriors
                .get(self.members[if Self::shop(state) { 0 } else { side }])
            else {
                continue;
            };
            let records = self.items(state, content, side).unwrap_or_default();
            *count = records.len();
            for (row, (slot, item)) in records.iter().skip(self.scroll[side]).take(9).enumerate() {
                self.draw_item_row(content, assets, side, row, *slot, item, controls);
            }
            if side == 1 && Self::shop(state) {
                continue;
            }
            draw_owner_footer(assets, warrior, side, records.len());
        }
        for (side, layout) in SIDE_LAYOUT.iter().enumerate() {
            let bar = layout.scroll;
            crate::presentation::scroll_bar::draw(
                bar,
                self.scroll[side] as i32,
                counts[side].saturating_sub(9) as i32,
            );
            for (y, delta) in [(bar.y, -1), (bar.bottom() - 16., 1)] {
                controls.push(Control {
                    rect: Rect::new(bar.x, y, bar.w, 16.),
                    enabled: true,
                    action: Action::Inventory(InventoryAction::Scroll { side, delta }),
                });
            }
        }
        if !Self::shop(state) {
            Self::draw_column(state, assets, class_label);
        }
        self.draw_credits(state, assets);
        if Self::shop(state) {
            if let Some((side, slot)) = self.selected
                && let Ok(item) = self.record(state, content, side, slot)
                && let Ok(price) =
                    centurions_core::shop::price(state, content, item.id(), side == 0)
            {
                assets.text(
                    &price.to_string(),
                    SIDE_LAYOUT[side].price,
                    447.,
                    12,
                    YELLOW,
                );
            }
            // CNTL 131, 130, 132, 134, 135 over the authored PICT 170 categories.
            for (family, (y, height)) in [
                (20., 50.),
                (69., 50.),
                (120., 137.),
                (257., 49.),
                (307., 49.),
            ]
            .into_iter()
            .enumerate()
            {
                controls.push(Control {
                    rect: Rect::new(375., y, 50., height),
                    enabled: true,
                    action: Action::Inventory(InventoryAction::Category(family)),
                });
            }
        } else {
            crate::presentation::push_panel_control(
                assets,
                controls,
                202,
                ENTER_SHOP,
                state
                    .shop_access
                    .as_ref()
                    .is_some_and(|a| a.level == state.scenario_level),
                Action::Inventory(InventoryAction::Shop),
            );
        }
        self.draw_strips(state, assets, controls);
        let stockpile = (!Self::shop(state)).then_some((STOCKPILE, InventoryAction::Stockpile));
        for (rect, action) in [
            (DONE, InventoryAction::Done),
            (REFIT[0], InventoryAction::Refit(0)),
            (REFIT[1], InventoryAction::Refit(1)),
            (ITEMS_WINDOW, InventoryAction::ItemsWindow),
        ]
        .into_iter()
        .chain(stockpile)
        {
            controls.push(Control {
                rect,
                enabled: if Self::shop(state) && matches!(action, InventoryAction::Refit(1)) {
                    false
                } else {
                    self.selected.is_some()
                        || matches!(
                            action,
                            InventoryAction::Done
                                | InventoryAction::Refit(_)
                                | InventoryAction::ItemsWindow
                        )
                },
                action: Action::Inventory(action),
            });
        }
        if let Some((side, slot)) = self.selected
            && let Ok(item) = self.record(state, content, side, slot)
            && let Some(map) = state.land(content)
        {
            self.detail(content, map, &item, side == 1 && Self::shop(state), assets);
        }
        self.draw_information(content, state, assets, controls);
    }

    /// One item row: its icon, name, quantity with loaded ammunition, and stats.
    #[allow(clippy::too_many_arguments)]
    fn draw_item_row(
        &self,
        content: &Content,
        assets: &Assets,
        side: usize,
        row: usize,
        slot: usize,
        item: &InventoryRecord,
        controls: &mut Vec<Control>,
    ) {
        let rect = Rect::new(SIDE_LAYOUT[side].list, 36. + row as f32 * 40., 325., 39.);
        if self.selected == Some((side, slot)) {
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                Color::from_rgba(220, 230, 225, 255),
            );
        }
        if let Some(definition) = content.items.get(item.id().unsigned_abs() as usize)
            && let Ok(id) = centurions_content::records::ItemDefinition(definition).icon()
            && let Some(icon) = assets.icons.get(&id)
        {
            crate::assets::icon_in(icon, vec2(rect.x, rect.y));
        }
        let name = assets.names.get(&item.id()).map_or("", String::as_str);
        assets.text(name, rect.x + 42., rect.y + 17., 12, BLACK);
        if let Ok(amount) = item.charge()
            && amount > 0
        {
            let mut x = rect.x + 42. + measure_text(name, Some(&assets.font), 12, 1.).width;
            if let Ok(ammo) = item.ammunition_id()
                && ammo != 0
                && let Some(definition) = content.items.get(ammo.unsigned_abs() as usize)
                && let Ok(icon) = centurions_content::records::ItemDefinition(definition).icon()
                && let Some(image) = assets.icons.get(&icon)
            {
                crate::assets::texture_in(image, Rect::new(x + 4., rect.y, 16., 16.));
                x += 22.;
            }
            if let Ok(prefix) = assets
                .game_text
                .text(crate::game_strings::QUANTITY_PREFIX, true)
            {
                assets.text(&format!("{prefix}{amount}"), x, rect.y + 17., 12, BLUE);
            }
        }
        self.row_stats(content, item, assets, rect.x + 42., rect.y + 30.);
        controls.push(Control {
            rect,
            enabled: true,
            action: Action::Inventory(InventoryAction::Item { side, slot }),
        });
    }
}

impl Inventory {
    /// Draws the party's credits in the PICT 148 strip, in yellow at 12
    /// points: the pooled total, which away from a shop is the pool and what
    /// every member carries.
    ///
    /// Original: Display
    fn draw_credits(&self, state: &Session, assets: &Assets) {
        let mut credits = state.party_currency;
        if !Self::shop(state) {
            for warrior in &state.warriors {
                credits = credits
                    .wrapping_add(i32::from(warrior.record().carried_credits().unwrap_or(0)));
            }
        }
        assets.text(&credits.to_string(), 505., 447., 12, YELLOW);
    }
}

/// CNTL 144, Enter Shop, covered by PICT 202 away from a shop.
const ENTER_SHOP: Rect = Rect::new(183., 411., 102., 40.);
/// CNTL 141, Done.
const DONE: Rect = Rect::new(663., 412., 50., 39.);
/// CNTL 133, the item button in the PICT 148 strip, which leaves for the Items window.
const ITEMS_WINDOW: Rect = Rect::new(600., 412., 59., 39.);
/// CNTL 138 and 139, the two Refit buttons in PICT 170.
const REFIT: [Rect; 2] = [
    Rect::new(375., 357., 50., 25.),
    Rect::new(375., 382., 50., 25.),
];

/// The owner's portrait, name, carried weight, capacity and item count under a list.
///
/// Original: updateshopwings
fn draw_owner_footer(assets: &Assets, warrior: &Warrior, side: usize, item_count: usize) {
    let layout = &SIDE_LAYOUT[side];
    let portrait = warrior.record().portrait().unwrap_or(0);
    if let Some(image) = assets.portraits.get(&portrait) {
        crate::assets::icon_in(image, vec2(layout.portrait, 422.));
    }
    assets.text(&warrior.name, layout.name, 471., 12, YELLOW);
    let edge = layout.edge;
    assets.text(
        &warrior.record().load().unwrap_or(0).to_string(),
        edge + 8.,
        445.,
        12,
        YELLOW,
    );
    assets.text(
        &warrior.record().load_max().unwrap_or(0).to_string(),
        edge + 42.,
        445.,
        12,
        YELLOW,
    );
    assets.text(&item_count.to_string(), edge + 49., 475., 12, YELLOW);
}

/// Horizontal positions that differ between the left (0) and right (1) lists.
struct SideLayout {
    list: f32,
    portrait: f32,
    name: f32,
    /// Left edge of the weight and count panel.
    edge: f32,
    /// The list's scroll bar: CNTL 128 on the left, 129 on the right.
    scroll: Rect,
    price: f32,
}

/// Where each side of the Items screen puts its list, portrait, name, weight
/// and count panel, scroll bar and price column.
const SIDE_LAYOUT: [SideLayout; 2] = [
    SideLayout {
        list: 10.,
        portrait: 97.,
        name: 151.,
        edge: 0.,
        scroll: Rect::new(352., 28., 16., 371.),
        price: 241.,
    },
    SideLayout {
        list: 470.,
        portrait: 455.,
        name: 503.,
        edge: 720.,
        scroll: Rect::new(434., 28., 16., 371.),
        price: 188.,
    },
];
