//! The showiteminfo DLOG 149 over the Items list, drawn from DITL 149.
use super::{Inventory, InventoryAction};
use crate::{
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_content::Content;
use centurions_core::{Session, item_display::InformationField};
use macroquad::prelude::*;
use std::collections::BTreeMap;

/// The item information fields already built for an item record.
pub(super) type ItemInformation = (Vec<u8>, BTreeMap<u16, InformationField>);
/// Centered between the panes so both lists keep clickable rows, as DLOG 149
/// opens beside the Items and second-owner windows.
const ORIGIN: Vec2 = vec2(222., 20.);

impl Inventory {
    /// Draws the item information window (DLOG 149) over the booty screen.
    pub fn draw_loot_information(
        &self,
        content: &Content,
        state: &Session,
        assets: &Assets,
        controls: &mut Vec<Control>,
    ) {
        self.draw_information(content, state, assets, controls);
    }
    /// Draws the item information window (DLOG 149) over the Items lists.
    pub(super) fn draw_information(
        &self,
        content: &Content,
        state: &Session,
        assets: &Assets,
        controls: &mut Vec<Control>,
    ) {
        let Some((member, slot)) = self.information else {
            return;
        };
        controls.retain(|control| {
            matches!(
                control.action,
                Action::Inventory(InventoryAction::Item { .. })
            )
        });
        controls.insert(
            0,
            Control {
                rect: Rect::new(0., 0., 800., 600.),
                enabled: true,
                action: Action::Inventory(InventoryAction::CloseInformation),
            },
        );
        assets.draw_dialog_at(149, ORIGIN);
        // Items 1 and 2 open showcondition modes 4 and 3.
        for (item, professions) in [(1, false), (2, true)] {
            controls.push(Control {
                rect: assets.dialog_rect_at(149, item, ORIGIN),
                enabled: true,
                action: Action::Inventory(InventoryAction::UseList(professions)),
            });
        }
        let (Some(owner), Some(warrior), Some(map)) = (
            state
                .warriors
                .get(state.globals.health_context.char_select_new),
            state.warriors.get(member),
            state.land(content),
        ) else {
            return;
        };
        let (record, merged) = match self.information_item {
            Some((id, _)) => {
                let mut record = centurions_content::InventoryRecord([0; 22]);
                record.set_id(id);
                (record, false)
            }
            None => (warrior.record().inventory_slot(slot), true),
        };
        let mut key = record.0.to_vec();
        key.push(u8::from(merged));
        key.extend(owner.bytes.iter());
        let mut cached = self.information_fields.borrow_mut();
        if cached.as_ref().is_none_or(|(old, _)| old != &key) {
            let fields = centurions_core::item_display::information(
                content,
                map,
                owner,
                (&record, merged),
                &assets.game_text,
                &assets.character_strings,
            )
            .unwrap_or_else(|error| {
                eprintln!("item information: {error}");
                BTreeMap::new()
            });
            *cached = Some((key, fields));
        }
        let Some((_, fields)) = cached.as_ref() else {
            return;
        };
        for (item, field) in fields {
            let rect = assets.dialog_rect_at(149, usize::from(*item), ORIGIN);
            match field {
                InformationField::Text { text, red } => {
                    let color = if *red { RED } else { YELLOW };
                    assets.wrapped_color(text, rect, 10, 12., color);
                }
                InformationField::Icon(id) => {
                    if let Some(image) = assets.icons.get(id) {
                        crate::assets::icon_in(image, vec2(rect.x, rect.y));
                    }
                }
            }
        }
        drop(cached);
        self.draw_use_list(content, state, assets, controls);
    }

    /// PopUpMenuSelect beside the dialog; any click closes it (the choice is unused).
    fn draw_use_list(
        &self,
        content: &Content,
        state: &Session,
        assets: &Assets,
        controls: &mut Vec<Control>,
    ) {
        let Some((professions, widened, item)) = &self.use_list else {
            return;
        };
        controls.clear();
        controls.push(Control {
            rect: Rect::new(0., 0., 800., 600.),
            enabled: true,
            action: Action::Inventory(InventoryAction::CloseInformation),
        });
        let selected = state.globals.health_context.char_select_new;
        let passes = if *widened { state.warriors.len() } else { 1 };
        let (Some(owner), Some(map)) = (state.warriors.get(selected), state.land(content)) else {
            return;
        };
        let rows = if *professions {
            centurions_core::item_display::profession_list(
                map,
                owner,
                item,
                &assets.game_text,
                &assets.character_strings,
                passes,
            )
        } else {
            centurions_core::item_display::user_list(
                content,
                map,
                &state.warriors,
                selected,
                item,
                &assets.game_text,
                passes,
            )
        };
        let rows = rows.unwrap_or_else(|error| {
            eprintln!("item use list: {error}");
            vec![]
        });
        let width = rows
            .iter()
            .map(|row| measure_text(&row.text, Some(&assets.menu_font), 12, 1.).width)
            .fold(160., f32::max)
            + 52.;
        let visible = rows.len().min(30);
        let rect = Rect::new(
            (800. - width).max(0.),
            20.,
            width.min(800.),
            visible as f32 * 18. + 8.,
        );
        let paper = Color::from_rgba(238, 238, 238, 255);
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, paper);
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1., BLACK);
        for (index, row) in rows.iter().take(visible).enumerate() {
            let y = rect.y + 4. + index as f32 * 18.;
            let image = row
                .portrait
                .and_then(|id| assets.portraits.get(&id))
                .or_else(|| row.icon.and_then(|id| assets.icons.get(&(id + 256))));
            if let Some(image) = image {
                let params = DrawTextureParams {
                    dest_size: Some(vec2(16., 16.)),
                    ..Default::default()
                };
                draw_texture_ex(image, rect.x + 17., y, WHITE, params);
            }
            if row.marked {
                assets.menu_text("\u{2713}", rect.x + 5., y + 13., 12, BLACK);
            }
            let color = if row.enabled { BLACK } else { GRAY };
            assets.menu_text(&row.text, rect.x + 38., y + 13., 12, color);
        }
    }
}
