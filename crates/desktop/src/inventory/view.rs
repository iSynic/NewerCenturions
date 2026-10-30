//! The Items screen: the two item lists, their categories and controls.
use super::{Inventory, InventoryAction};
use crate::{
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_content::{Content, InventoryRecord, Warrior};
use centurions_core::Session;
use macroquad::prelude::*;
impl Inventory {
    /// Draws the Items screen: both warriors' lists with their portraits and
    /// scroll bars and the controls.
    pub fn draw(
        &self,
        content: &Content,
        state: &Session,
        assets: &Assets,
        controls: &mut Vec<Control>,
    ) {
        controls.clear();
        draw_rectangle(0., 20., 800., 580., WHITE);
        assets.panel(170, Rect::new(343., 20., 114., 387.));
        assets.panel(148, Rect::new(80., 407., 640., 73.));
        assets.panel(227, Rect::new(0., 407., 80., 73.));
        assets.panel(227, Rect::new(720., 407., 80., 73.));
        assets.panel(226, Rect::new(0., 480., 800., 120.));
        for side in 0..2 {
            let Some(warrior) = state
                .warriors
                .get(self.members[if Self::shop(state) { 0 } else { side }])
            else {
                continue;
            };
            let records = self.items(state, content, side).unwrap_or_default();
            for (row, (slot, item)) in records.iter().skip(self.scroll[side]).take(9).enumerate() {
                self.draw_item_row(content, assets, side, row, *slot, item, controls);
            }
            if side == 1 && Self::shop(state) {
                continue;
            }
            draw_owner_footer(assets, warrior, side, records.len());
        }
        for (side, layout) in SIDE_LAYOUT.iter().enumerate() {
            for (y, delta) in [(20., -1), (384., 1)] {
                let rect = Rect::new(layout.scroll, y, 16., 20.);
                crate::presentation::push_panel_control(
                    assets,
                    controls,
                    if delta < 0 { 179 } else { 183 },
                    rect,
                    true,
                    Action::Inventory(InventoryAction::Scroll { side, delta }),
                );
            }
        }
        for member in 0..if Self::shop(state) { 0 } else { 6 } {
            assets.panel(154, Rect::new(375., 20. + member as f32 * 50., 50., 50.));
        }
        let mos_names = assets
            .character_strings
            .get(&131)
            .and_then(|bytes| centurions_content::ui::string_list(bytes).ok())
            .unwrap_or_default();
        for (member, warrior) in state.warriors.iter().enumerate() {
            for side in 0..if Self::shop(state) { 1 } else { 2 } {
                let rect = Rect::new(
                    342. + side as f32 * 52. + (member % 3) as f32 * 24.,
                    408. + (member / 3) as f32 * 24.,
                    22.,
                    22.,
                );
                if let Some(image) = assets
                    .portraits
                    .get(&warrior.record().portrait().unwrap_or(0))
                {
                    crate::assets::texture_in(image, rect);
                }
                controls.push(Control {
                    rect,
                    enabled: true,
                    action: Action::Inventory(InventoryAction::Warrior { side, member }),
                });
            }
            if Self::shop(state) {
                continue;
            }
            let y = 20. + member as f32 * 50.;
            if let Some(image) = assets
                .portraits
                .get(&warrior.record().portrait().unwrap_or(0))
            {
                crate::assets::icon_in(image, vec2(383., y + 8.));
            }
            if let Ok(mos) = warrior.record().mos()
                && let Some(name) = mos_names.get(mos.wrapping_sub(1) as usize)
            {
                let name: String = name.chars().take(7).collect();
                assets.text(&name, 380., y + 47., 10, YELLOW);
            }
            for side in 0..if Self::shop(state) { 1 } else { 2 } {
                controls.push(Control {
                    rect: Rect::new(375. + side as f32 * 25., y, 25., 50.),
                    enabled: true,
                    action: Action::Inventory(InventoryAction::Warrior { side, member }),
                });
            }
        }
        if Self::shop(state) {
            assets.text(&state.party_currency.to_string(), 507., 446., 12, YELLOW);
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
                Rect::new(183., 411., 102., 40.),
                state
                    .shop_access
                    .as_ref()
                    .is_some_and(|a| a.level == state.scenario_level),
                Action::Inventory(InventoryAction::Shop),
            );
            self.draw_ammunition_button(assets, controls);
        }
        assets.panel(175, Rect::new(375., 320., 50., 37.));
        for (rect, action) in [
            (Rect::new(663., 412., 50., 39.), InventoryAction::Done),
            (Rect::new(375., 357., 50., 25.), InventoryAction::Refit(0)),
            (Rect::new(375., 382., 50., 25.), InventoryAction::Refit(1)),
            (Rect::new(375., 320., 50., 37.), InventoryAction::Stockpile),
            // CNTL 140, the info button in the PICT 148 strip.
            (
                Rect::new(609., 412., 47., 35.),
                InventoryAction::Information,
            ),
        ] {
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
                                | InventoryAction::Information
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
        self.draw_ammunition(assets, controls);
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

/// The owner's portrait, name, carried weight, capacity and item count under a list.
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
    scroll: f32,
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
        scroll: 349.,
        price: 241.,
    },
    SideLayout {
        list: 470.,
        portrait: 455.,
        name: 503.,
        edge: 720.,
        scroll: 435.,
        price: 188.,
    },
];
