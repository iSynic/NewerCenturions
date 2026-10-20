//! Inventory view using original resources and controls.
use super::{Inventory, InventoryAction};
use crate::{
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_content::Content;
use centurions_core::Session;
use macroquad::prelude::*;
impl Inventory {
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
            let x = if side == 0 { 10. } else { 470. };
            let records = self.items(state, content, side).unwrap_or_default();
            for (row, (slot, item)) in records.iter().skip(self.scroll[side]).take(9).enumerate() {
                let rect = Rect::new(x, 36. + row as f32 * 40., 325., 39.);
                if self.selected == Some((side, *slot)) {
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
                    draw_texture_ex(
                        icon,
                        rect.x,
                        rect.y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(32., 32.)),
                            ..Default::default()
                        },
                    );
                }
                let name = assets.names.get(&item.id()).map_or("", String::as_str);
                assets.text(name, rect.x + 42., rect.y + 17., 12, BLACK);
                if let Ok(amount) = item.quantity()
                    && amount > 0
                {
                    let mut x = rect.x + 42. + measure_text(name, Some(&assets.font), 12, 1.).width;
                    if let Ok(ammo) = item.ammunition_id()
                        && ammo != 0
                        && let Some(definition) = content.items.get(ammo.unsigned_abs() as usize)
                        && let Ok(icon) =
                            centurions_content::records::ItemDefinition(definition).icon()
                        && let Some(image) = assets.icons.get(&icon)
                    {
                        draw_texture_ex(
                            image,
                            x + 4.,
                            rect.y,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(16., 16.)),
                                ..Default::default()
                            },
                        );
                        x += 22.;
                    }
                    if let Ok(prefix) = assets.native_text.text(0x401152, true) {
                        assets.text(&format!("{prefix}{amount}"), x, rect.y + 17., 12, BLUE);
                    }
                }
                self.row_stats(content, item, assets, rect.x + 42., rect.y + 30.);
                controls.push(Control {
                    rect,
                    enabled: true,
                    action: Action::Inventory(InventoryAction::Item { side, slot: *slot }),
                });
            }
            if side == 1 && Self::shop(state) {
                continue;
            }
            let portrait = warrior.record().portrait().unwrap_or(0);
            if let Some(image) = assets.portraits.get(&portrait) {
                draw_texture_ex(
                    image,
                    if side == 0 { 97. } else { 455. },
                    422.,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(32., 32.)),
                        ..Default::default()
                    },
                );
            }
            assets.text(
                &warrior.name,
                if side == 0 { 151. } else { 503. },
                471.,
                12,
                YELLOW,
            );
            let edge = if side == 0 { 0. } else { 720. };
            assets.text(
                &warrior.record().carried_weight().unwrap_or(0).to_string(),
                edge + 8.,
                445.,
                12,
                YELLOW,
            );
            assets.text(
                &warrior.record().weight_capacity().unwrap_or(0).to_string(),
                edge + 42.,
                445.,
                12,
                YELLOW,
            );
            assets.text(&records.len().to_string(), edge + 49., 475., 12, YELLOW);
        }
        for side in 0..2 {
            for (y, delta) in [(20., -1), (384., 1)] {
                let rect = Rect::new(if side == 0 { 349. } else { 435. }, y, 16., 20.);
                assets.panel(if delta < 0 { 179 } else { 183 }, rect);
                controls.push(Control {
                    rect,
                    enabled: true,
                    action: Action::Inventory(InventoryAction::Scroll { side, delta }),
                });
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
                    draw_texture_ex(
                        image,
                        rect.x,
                        rect.y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(rect.w, rect.h)),
                            ..Default::default()
                        },
                    );
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
                draw_texture_ex(
                    image,
                    383.,
                    y + 8.,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(32., 32.)),
                        ..Default::default()
                    },
                );
            }
            if let Ok(mos) = warrior.record().profession()
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
                    if side == 0 { 241. } else { 188. },
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
            assets.panel(202, Rect::new(183., 411., 102., 40.));
            controls.push(Control {
                rect: Rect::new(183., 411., 102., 40.),
                enabled: state
                    .shop_access
                    .as_ref()
                    .is_some_and(|a| a.level == state.scenario_level),
                action: Action::Inventory(InventoryAction::Shop),
            });
        }
        assets.panel(175, Rect::new(375., 320., 50., 37.));
        for (rect, action) in [
            (Rect::new(663., 412., 50., 39.), InventoryAction::Done),
            (Rect::new(375., 357., 50., 25.), InventoryAction::Refit(0)),
            (Rect::new(375., 382., 50., 25.), InventoryAction::Refit(1)),
            (Rect::new(375., 320., 50., 37.), InventoryAction::Stockpile),
        ] {
            controls.push(Control {
                rect,
                enabled: if Self::shop(state) && matches!(action, InventoryAction::Refit(1)) {
                    false
                } else {
                    self.selected.is_some()
                        || matches!(action, InventoryAction::Done | InventoryAction::Refit(_))
                },
                action: Action::Inventory(action),
            });
        }
        if let Some((side, slot)) = self.selected
            && let Ok(item) = self.record(state, content, side, slot)
            && let Some(map) = content.map(
                state
                    .exploration
                    .as_ref()
                    .map_or(0, |position| position.map),
            )
        {
            self.detail(content, map, &item, side == 1 && Self::shop(state), assets);
        }
    }
}
