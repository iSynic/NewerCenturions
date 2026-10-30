//! Map presentation.
use super::UiState;
use crate::{assets::Assets, layout};
use centurions_content::Content;
use centurions_core::Session;
use macroquad::prelude::*;

/// Draws tile `tile` (1 to 300, 20 to a row) of terrain sheet `terrain` at
/// `(x, y)`; any other tile number draws nothing.
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
/// Draws the 15 by 13 cells of the map around the party, or around the combat
/// view while a battle is on.
pub(super) fn map(content: &Content, state: &Session, assets: &Assets, ui: &UiState) {
    let Some(position) = &state.exploration else {
        return;
    };
    let Some(mission) = state.land(content) else {
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
                let index = (vx + x + i32::from(c.field.map_shift.0) + 5) as usize * 90
                    + (vy + y + i32::from(c.field.map_shift.1) + 5) as usize;
                let tile = c.field.tiles[index];
                let id = tile.wrapping_abs();
                let under = if (0..6).contains(&id) {
                    c.squad.char_under[id as usize]
                } else if id >= 10 && id < 10 + c.roster.records.len() as i16 {
                    c.roster.records[(id - 10) as usize]
                        .record()
                        .underlying_tile()
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
                    // A slot past a short squad holds no warrior to draw.
                    state
                        .warriors
                        .get(id as usize)
                        .and_then(|warrior| super::bodies::warrior_icon(warrior.record()))
                } else if id >= 10 && id < 10 + c.roster.records.len() as i16 {
                    // A -3 definition draws a picture in the bodies pass instead.
                    let monster = c.roster.records[(id - 10) as usize].record();
                    match (monster.size(), monster.icon()) {
                        (Ok(-3), _) | (_, Err(_)) => None,
                        (Ok(_), Ok(icon)) => {
                            Some(super::bodies::enemy_icon(monster.facing_left(), icon))
                        }
                        (Err(_), Ok(_)) => None,
                    }
                } else {
                    None
                };
                if let Some(texture) = icon.and_then(|i| assets.icons.get(&i)) {
                    draw_texture(texture, sx, sy, WHITE);
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
    if let Some(c) = combat {
        super::bodies::draw(state, c, assets, ui, (vx, vy));
        super::range_lines::draw_edges(state, c, (vx, vy));
    }
    if let (Some(c), Some(playback)) = (combat, &ui.playback)
        && let Some(icon) = playback.icon
    {
        result_icon(assets, c, icon, vx, vy);
    }
    if combat.is_some() {
        super::flights::draw(assets, ui, (vx, vy));
    }
    if let (Some(c), Some(playback)) = (combat, &ui.playback)
        && let Some(effect) = playback.effect
        && let Some(icon) = effect.icon(get_time())
        && let Some(texture) = assets.icons.get(&icon)
    {
        // calcbodyrect gives the body's rectangle, which PlotCIcon fills; only
        // its part in the view rect is copied.
        for body in effect.bodies() {
            if let Some((position, size)) = crate::combat_playback::Playback::body_cell(c, body) {
                super::bodies::plot_clipped(
                    texture,
                    super::bodies::cell(position, size, (vx, vy)),
                    layout::MAP,
                );
            }
        }
    }
    let party_icon = match state.globals.boat {
        0 => Some(
            if state.field_controls.as_ref().is_some_and(|c| c.camping) {
                178
            } else {
                192 + i16::from(position.facing)
            },
        ),
        _ => Some(13006 - i16::from(state.globals.boat_facing)),
    };
    if combat.is_none()
        && let Some(texture) = party_icon.and_then(|id| assets.icons.get(&id))
    {
        draw_texture(
            texture,
            (position.x - vx) as f32 * 32.,
            layout::MAP.y + (position.y - vy) as f32 * 32.,
            WHITE,
        );
    }
}

/// The result cicn in the body's 32-pixel cell, raised 16 pixels for size
/// classes 1 and 3 and moved 16 left above class 1, then the amount in bold at
/// (left + 8, top + 20) in size 10 above 9, otherwise (left + 11, top + 20) in
/// size 12. Cicns 160-163 print in blackColor and the others in whiteColor.
///
/// Original: showdamage
fn result_icon(
    assets: &Assets,
    combat: &centurions_core::combat::Combat,
    icon: crate::combat_playback::ResultIcon,
    vx: i32,
    vy: i32,
) {
    let Some(((x, y), size)) = crate::combat_playback::Playback::body_cell(combat, icon.body)
    else {
        return;
    };
    let mut top = layout::MAP.y + (i32::from(y) - vy) as f32 * 32.;
    let mut left = layout::MAP.x + (i32::from(x) - vx) as f32 * 32.;
    if size == 1 || size == 3 {
        top -= 16.;
    }
    if size > 1 {
        left -= 16.;
    }
    if let Some(texture) = assets.icons.get(&icon.icon) {
        crate::assets::icon_in(texture, vec2(left, top));
    }
    // The number is always drawn.
    let color = if (160..=163).contains(&icon.icon) {
        BLACK
    } else {
        WHITE
    };
    let (dx, text_size) = if icon.amount > 9 { (8., 10) } else { (11., 12) };
    // QuickDraw's bold repeats each glyph one pixel to the right.
    for smear in [0., 1.] {
        assets.text(
            &icon.amount.to_string(),
            left + dx + smear,
            top + 20.,
            text_size,
            color,
        );
    }
}

/// newland27's picture, centred in a 320-pixel square of the map window.
pub(super) fn draw_story_picture(assets: &Assets, ui: &UiState) {
    let Some(texture) = ui.picture.and_then(|id| assets.panels.get(&id)) else {
        return;
    };
    let x = ((320. - texture.width()) / 2.).trunc();
    let y = ((320. - texture.height()) / 2.).trunc();
    draw_texture(texture, layout::MAP.x + x, layout::MAP.y + y, WHITE);
}

/// Draws the frame of the outdoor spell effect playing now, over its map cell.
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
    crate::assets::texture_in(
        texture,
        Rect::new(
            layout::MAP.x + (i32::from(effect.x) - vx) as f32 * 32. - size / 2.,
            layout::MAP.y + (i32::from(effect.y) - vy) as f32 * 32. - size / 2.,
            size,
            size,
        ),
    );
}
