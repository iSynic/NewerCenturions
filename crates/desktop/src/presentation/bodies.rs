//! drawbody 0x120c14: a body's state frame and a -3 definition's picture.
//!
//! The frame is copied (srcCopy) from sprite GWorld 0x4386c8, which GWorldInit
//! 0x1277ae fills with PICT 401, at x 0x300 (green), 0x340 (red) or 0x380 (blue),
//! 32 pixels square and 64 on an axis a size class extends. The cicn is then
//! plotted over it in the body's rectangle. A frame is drawn for the queue's
//! current body (0x120edc), when the caller passes a positive mode (showrange's
//! Reveal Friends refresh passes 1), and for a disabled NPC or a body whose turn
//! is skipped. An NPC whose definition word +0x10 is -3 draws PICT +0x50 instead
//! of a cicn, its bottom-right on the cell's (0x120fa2..0x1210c0, mode 0x24).
use super::UiState;
use crate::{assets::Assets, layout};
use centurions_core::{Session, combat::Combat};
use macroquad::prelude::*;

const SPRITES: i16 = 401;

pub(super) fn draw(
    state: &Session,
    combat: &Combat,
    assets: &Assets,
    ui: &UiState,
    view: (i32, i32),
) {
    let reveal = ui.combat_view.reveal && ui.playback.is_none();
    // centerfieldlarge 0x10eaca..0x10eafa outside capture mode: drawbody(body,
    // WORD438760 - BYTE435661 - BYTE435882, 1) for every view body. cast holds
    // both BYTEs at 1 (0x10d54a, 0x10d632 until 0x10e66c), so only a Ready
    // outside a cast's playback frames them.
    let ready = combat.ready > 0 && combat.capture_actor.is_none() && ui.playback.is_none();
    let flashed = ui.playback.as_ref().and_then(|playback| playback.framed);
    let current = combat.active_body();
    for (index, warrior) in state.warriors.iter().enumerate() {
        let record = warrior.record();
        // showrange 0x158150 refreshes warriors with +0x9df; drawbody needs stamina.
        if record.active_flag() == 0 || record.stamina().unwrap_or(0) < 1 {
            continue;
        }
        let Some(&position) = combat.squad.positions.get(index) else {
            continue;
        };
        let skipped = record.turn_skip_duration().unwrap_or(0) != 0;
        let column = if skipped {
            0x380
        } else if record.faction().unwrap_or(0) != 0 {
            0x340
        } else {
            0x300
        };
        if !(reveal
            || (on_field(combat, position, index as i16)
                && (ready
                    || skipped
                    || current == Some(index as i16)
                    || flashed == Some(index as i16))))
        {
            continue;
        }
        let rect = cell(position, 0, view);
        frame(assets, column, rect);
        if let Some(icon) = warrior_icon(record)
            && let Some(texture) = assets.icons.get(&icon)
        {
            plot(texture, rect);
        }
    }
    for (index, enemy) in combat.enemies.records.iter().enumerate() {
        let monster = enemy.record();
        if enemy.stamina().unwrap_or(0) < 1 {
            continue;
        }
        let Some(&position) = combat.enemies.positions.get(index) else {
            continue;
        };
        let body = index as i16 + 10;
        let disabled = monster.removed() || monster.turn_skip_duration() != 0;
        let column = if disabled {
            0x380
        } else if monster.faction().unwrap_or(0) != 0 {
            0x340
        } else {
            0x300
        };
        let definition = monster.size().unwrap_or(0);
        let rect = cell(position, definition.wrapping_abs(), view);
        // centerfieldlarge 0x10e8a8..0x10eb08 draws only the bodies its view
        // cells hold; showrange(0) refreshes every NPC with stamina.
        if !reveal && !on_field(combat, position, body) {
            continue;
        }
        let framed = reveal || ready || disabled || current == Some(body);
        if framed {
            frame(assets, column, rect);
        }
        let image = monster.tactical_icon().unwrap_or(0);
        if definition == -3 {
            // The picture's bottom-right sits on the anchor cell's.
            if let Some(texture) = assets.transparent_picture(image) {
                let right = layout::MAP.x + ((position.0 as i32 - view.0) * 32 + 32) as f32;
                let bottom = layout::MAP.y + ((position.1 as i32 - view.1) * 32 + 32) as f32;
                draw_texture(
                    &texture,
                    right - texture.width(),
                    bottom - texture.height(),
                    WHITE,
                );
            }
        } else if framed
            && let Some(texture) = assets.icons.get(&enemy_icon(monster.facing_flag(), image))
        {
            plot(texture, rect);
        }
    }
}

/// drawbody 0x120dee..0x120e0a: GetCIcon(BYTE 0x9e0 * 10000 + WORD 0x56), the sum
/// kept to a WORD.
pub(super) fn warrior_icon(record: centurions_content::records::WarriorRecord<'_>) -> Option<i16> {
    let icon = record.tactical_icon().ok()?;
    Some(
        i16::from(record.horizontal_facing())
            .wrapping_mul(10000)
            .wrapping_add(icon),
    )
}

/// 0x120cdc..0x120d04: facing flag +0x107 == 1 selects cicn +0x50 + 1000.
pub(super) fn enemy_icon(facing: u8, icon: i16) -> i16 {
    if facing == 1 {
        icon.wrapping_add(1000)
    } else {
        icon
    }
}

/// Whether the tactical map's cell at the body's anchor holds the body.
fn on_field(combat: &Combat, (x, y): (i16, i16), body: i16) -> bool {
    let column = i32::from(x) + i32::from(combat.map.shift.0) + 5;
    let row = i32::from(y) + i32::from(combat.map.shift.1) + 5;
    (0..90).contains(&column)
        && (0..90).contains(&row)
        && combat
            .map
            .tiles
            .get(column as usize * 90 + row as usize)
            .is_some_and(|tile| tile.wrapping_abs() == body)
}

/// 0x120d1e..0x120ece: the anchor cell, raised 32 for size classes 1 and 3 and
/// widened 32 to the left above class 1.
pub(super) fn cell(position: (i16, i16), size: i16, (vx, vy): (i32, i32)) -> Rect {
    let mut rect = Rect::new(
        layout::MAP.x + ((i32::from(position.0) - vx) * 32) as f32,
        layout::MAP.y + ((i32::from(position.1) - vy) * 32) as f32,
        32.,
        32.,
    );
    if size == 1 || size == 3 {
        rect.y -= 32.;
        rect.h += 32.;
    }
    if size > 1 {
        rect.x -= 32.;
        rect.w += 32.;
    }
    rect
}

fn frame(assets: &Assets, column: i16, rect: Rect) {
    assets.panel_part(
        SPRITES,
        Rect::new(f32::from(column), 0., rect.w, rect.h),
        rect,
    );
}

/// PlotCIcon into `rect`, showing only the part inside `clip`.
pub(super) fn plot_clipped(texture: &Texture2D, rect: Rect, clip: Rect) {
    let Some(shown) = rect.intersect(clip) else {
        return;
    };
    let (sx, sy) = (texture.width() / rect.w, texture.height() / rect.h);
    draw_texture_ex(
        texture,
        shown.x,
        shown.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(shown.w, shown.h)),
            source: Some(Rect::new(
                (shown.x - rect.x) * sx,
                (shown.y - rect.y) * sy,
                shown.w * sx,
                shown.h * sy,
            )),
            ..Default::default()
        },
    );
}

/// PlotCIcon scales the cicn to the rectangle.
fn plot(texture: &Texture2D, rect: Rect) {
    draw_texture_ex(
        texture,
        rect.x,
        rect.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(rect.w, rect.h)),
            ..Default::default()
        },
    );
}
