use crate::assets::Assets;
use centurions_content::{Content, word};
use centurions_core::Session;
use macroquad::prelude::*;

pub struct Note {
    pub record: Vec<u8>,
    pub text: String,
    pub scroll: f32,
}
impl Note {
    pub fn draw(&self, state: &Session, content: &Content, assets: &Assets) {
        draw_rectangle(0., 20., 640., 460., BLACK);
        let picture = word(&self.record, 66).unwrap_or(0);
        if picture != 0 {
            assets.panel(picture, Rect::new(160., 60., 320., 320.));
        } else if let Some(map) = state.map(content, i32::from(word(&self.record, 64).unwrap_or(0)))
        {
            let index = i32::from(word(&self.record, 64).unwrap_or(0));
            let saved = if state.exploration.as_ref().is_some_and(|p| p.map == index) {
                state.exploration.as_ref().map(|p| p.floor.as_slice())
            } else {
                state.visited_maps.get(&index).map(|m| m.floor.as_slice())
            };
            let tiles = saved.unwrap_or(&map.floor);
            if let Some(texture) = assets.terrain.get(&map.terrain) {
                for x in 0..90 {
                    for y in 0..90 {
                        let tile = tiles[x * 90 + y].unsigned_abs().wrapping_sub(1) as usize;
                        if tile < 300 {
                            draw_texture_ex(
                                texture,
                                140. + x as f32 * 4.,
                                40. + y as f32 * 4.,
                                WHITE,
                                DrawTextureParams {
                                    dest_size: Some(vec2(4., 4.)),
                                    source: Some(Rect::new(
                                        (tile % 20) as f32 * 32.,
                                        (tile / 20) as f32 * 32.,
                                        32.,
                                        32.,
                                    )),
                                    ..Default::default()
                                },
                            );
                        }
                    }
                }
            }
        }
        assets.wrapped_color(
            &self.text,
            Rect::new(40., 405. - self.scroll, 560., 70.),
            12,
            14.,
            YELLOW,
        );
    }
}
