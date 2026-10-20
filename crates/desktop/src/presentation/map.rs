//! Map presentation.
use super::UiState;
use crate::{assets::Assets, layout};
use centurions_content::{Content, word};
use centurions_core::Session;
use macroquad::prelude::*;

pub(super) fn terrain(assets: &Assets, terrain: u8, tile: i16, x: f32, y: f32) {
    let Some(texture) = assets.terrain.get(&terrain) else {
        return;
    };
    if (1..=300).contains(&tile) {
        let image = tile - 1;
        draw_texture_ex(
            texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(
                    f32::from(image % 20) * 32.,
                    f32::from(image / 20) * 32.,
                    32.,
                    32.,
                )),
                ..Default::default()
            },
        );
    }
}
pub(super) fn map(content: &Content, state: &Session, assets: &Assets, ui: &UiState) {
    let Some(position) = &state.exploration else {
        return;
    };
    let Some(mission) = content.map(position.map) else {
        return;
    };
    let combat = state.combat.as_ref();
    let (vx, vy) = combat
        .and_then(|_| crate::combat_controls::origin(state, ui))
        .map(|(x, y)| (i32::from(x), i32::from(y)))
        .unwrap_or(layout::outdoor_origin(position.x, position.y));
    for y in 0..13 {
        for x in 0..15 {
            let sx = layout::MAP.x + x as f32 * 32.;
            let sy = layout::MAP.y + y as f32 * 32.;
            if let Some(c) = combat {
                let index = (vx + x + i32::from(c.map.shift.0) + 5) as usize * 90
                    + (vy + y + i32::from(c.map.shift.1) + 5) as usize;
                let tile = c.map.tiles[index];
                let id = tile.wrapping_abs();
                let under = if (0..6).contains(&id) {
                    c.squad.under_tiles[id as usize]
                } else if id >= 10 && id < 10 + c.enemies.records.len() as i16 {
                    word(&c.enemies.records[(id - 10) as usize].bytes, 0x11e)
                        .unwrap_or(1000 + mission.default_tile)
                } else {
                    tile
                };
                terrain(
                    assets,
                    mission.terrain,
                    if under.wrapping_abs() >= 1000 {
                        under.wrapping_abs() - 1000
                    } else {
                        mission.default_tile
                    },
                    sx,
                    sy,
                );
                let icon = if tile < 0 && id >= 1000 {
                    Some(id)
                } else if (0..6).contains(&id) {
                    state.warriors[id as usize].record().tactical_icon().ok()
                } else if id >= 10 && id < 10 + c.enemies.records.len() as i16 {
                    word(&c.enemies.records[(id - 10) as usize].bytes, 0x50).ok()
                } else {
                    None
                };
                if let Some(texture) = icon.and_then(|i| assets.icons.get(&i)) {
                    draw_texture(texture, sx, sy, WHITE);
                }
                if ui.combat_view.reveal && (0..state.warriors.len() as i16).contains(&id) {
                    draw_rectangle_lines(sx + 1., sy + 1., 30., 30., 1., GREEN);
                }
                if c.active_body() == Some(id) {
                    draw_rectangle_lines(sx, sy, 32., 32., 1., YELLOW);
                }
            } else {
                let index = (vx + x) as usize * 90 + (vy + y) as usize;
                terrain(assets, mission.terrain, position.floor[index], sx, sy);
                if position.objects[index] > 0 {
                    terrain(assets, mission.terrain, position.objects[index], sx, sy);
                }
                if position.objects[index] < 0
                    && let Some(texture) = assets.icons.get(&position.objects[index].wrapping_abs())
                {
                    draw_texture(texture, sx, sy, WHITE);
                }
            }
        }
    }
    if combat.is_none()
        && let Some(texture) =
            assets.icons.get(
                &if state.field_controls.as_ref().is_some_and(|c| c.camping) {
                    178
                } else {
                    192 + i16::from(position.facing)
                },
            )
    {
        draw_texture(
            texture,
            (position.x - vx) as f32 * 32.,
            layout::MAP.y + (position.y - vy) as f32 * 32.,
            WHITE,
        );
    }
}

pub(super) fn draw_outdoor_effect(state: &Session, assets: &Assets, ui: &UiState) {
    let Some(effect) = ui.effects.front() else {
        return;
    };
    let elapsed = get_time() - effect.started;
    if !(0. ..effect.frame_seconds * 8.).contains(&elapsed) {
        return;
    }
    let frame = (elapsed / effect.frame_seconds) as i16;
    let Some(texture) = assets.icons.get(&(12000 + effect.effect * 8 + frame)) else {
        return;
    };
    let Some(position) = state.exploration.as_ref() else {
        return;
    };
    let (vx, vy) = layout::outdoor_origin(position.x, position.y);
    let size = f32::from(effect.size);
    draw_texture_ex(
        texture,
        layout::MAP.x + (i32::from(effect.x) - vx) as f32 * 32. - size / 2.,
        layout::MAP.y + (i32::from(effect.y) - vy) as f32 * 32. - size / 2.,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(size, size)),
            ..Default::default()
        },
    );
}
