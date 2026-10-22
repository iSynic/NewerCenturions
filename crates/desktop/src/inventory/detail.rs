use super::Inventory;
use super::draw_detail_plan;
use crate::assets::Assets;
use centurions_content::{Content, InventoryRecord};
use macroquad::prelude::*;
impl Inventory {
    pub(crate) fn detail(
        &self,
        content: &Content,
        map: &centurions_content::mission::OpeningMission,
        item: &InventoryRecord,
        stock: bool,
        assets: &Assets,
    ) {
        let mut key = item.0.to_vec();
        key.push(u8::from(stock));
        for id in [item.id(), item.ammunition_id().unwrap_or(0)] {
            if let Some(definition) = content.items.get(id.unsigned_abs() as usize) {
                key.extend(definition);
            }
        }
        if let Ok(description) = map.item_description(item.id()) {
            key.extend(description.as_bytes());
        }
        let mut cached = self.detail.borrow_mut();
        if cached.as_ref().is_none_or(|(old, _)| old != &key) {
            let plan = centurions_core::item_display::detail(
                content,
                map,
                item,
                stock,
                &assets.native_text,
                &assets.character_strings,
            )
            .map_err(|error| {
                eprintln!("inventory detail: {error}");
                error.to_string()
            });
            *cached = Some((key, plan));
        }
        let Some((_, Ok(plan))) = cached.as_ref() else {
            return;
        };
        draw_detail_plan(plan, assets);
    }

    pub(super) fn row_stats(
        &self,
        content: &Content,
        item: &InventoryRecord,
        assets: &Assets,
        mut x: f32,
        y: f32,
    ) {
        let mut key = item.0.to_vec();
        for id in [item.id(), item.ammunition_id().unwrap_or(0)] {
            if let Some(definition) = content.items.get(id.unsigned_abs() as usize) {
                key.extend(definition);
            }
        }
        let mut rows = self.rows.borrow_mut();
        let fragments = rows.entry(key).or_insert_with(|| {
            use centurions_core::item_display::output::{Draw, PanelEffect};
            match centurions_core::item_display::row(
                content,
                item,
                &assets.native_text,
                &assets.character_strings,
            ) {
                Ok(effects) => {
                    let mut color = 33;
                    let mut fragments: Vec<(i32, String)> = vec![];
                    for effect in effects {
                        match effect {
                            PanelEffect::Drawing(Draw::ForeColor(value)) => color = value,
                            PanelEffect::Drawing(Draw::DrawString(value)) => {
                                let text = centurions_content::ui::pascal_text(value.raw())
                                    .unwrap_or_default();
                                if let Some(last) =
                                    fragments.last_mut().filter(|last| last.0 == color)
                                {
                                    last.1.push_str(&text);
                                } else {
                                    fragments.push((color, text));
                                }
                            }
                            _ => {}
                        }
                    }
                    fragments
                }
                Err(error) => {
                    eprintln!("inventory row: {error}");
                    vec![]
                }
            }
        });
        for (color, text) in fragments {
            assets.text(
                text,
                x,
                y,
                10,
                match color {
                    205 => RED,
                    409 => BLUE,
                    341 => GREEN,
                    _ => BLACK,
                },
            );
            x += measure_text(text, Some(&assets.font), 10, 1.).width;
        }
    }
}
