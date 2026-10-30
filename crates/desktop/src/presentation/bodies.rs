//! The bodies on the tactical map: each body's coloured frame and its cicn, or a
//! -3 definition's picture.
//!
//! The frame comes from PICT 401 at x 0x300 (green), 0x340 (red) or 0x380
//! (blue), 32 pixels square and 64 on an axis a size class extends; the cicn is
//! plotted over it in the body's rectangle. A frame is drawn for the body whose
//! turn it is, when the caller asks for one (Reveal Friends does), and for a
//! disabled monster or a body whose turn is skipped. A monster whose definition
//! size is -3 draws its picture instead of a cicn, transparent, with its
//! bottom-right on the cell's.
use super::UiState;
use crate::{assets::Assets, layout};
use centurions_core::{Session, combat::Combat};
use macroquad::prelude::*;

/// The combat sprite picture (PICT 401).
const SPRITES: i16 = 401;

/// Original: drawbody
pub(super) fn draw(
    state: &Session,
    combat: &Combat,
    assets: &Assets,
    ui: &UiState,
    view: (i32, i32),
) {
    let reveal = ui.combat_view.reveal && ui.playback.is_none();
    // Outside capture mode centerfieldlarge frames every view body while a
    // spell is being readied. cast clears that while it plays, so only a Ready
    // outside a cast's playback frames them.
    let in_spell = combat.in_spell > 0 && combat.capture_actor.is_none() && ui.playback.is_none();
    let flashed = ui.playback.as_ref().and_then(|playback| playback.framed);
    let current = combat.active_body();
    for (index, warrior) in state.warriors.iter().enumerate() {
        let record = warrior.record();
        // showrange refreshes warriors in battle; drawbody needs stamina.
        if record.in_battle() == 0 || record.stamina().unwrap_or(0) < 1 {
            continue;
        }
        let Some(&position) = combat.squad.pos.get(index) else {
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
                && (in_spell
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
    for (index, enemy) in combat.roster.records.iter().enumerate() {
        let monster = enemy.record();
        if enemy.stamina().unwrap_or(0) < 1 {
            continue;
        }
        let Some(&position) = combat.roster.mon_pos.get(index) else {
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
        // centerfieldlarge draws only the bodies its view cells hold;
        // showrange(0) refreshes every monster with stamina.
        if !reveal && !on_field(combat, position, body) {
            continue;
        }
        let framed = reveal || in_spell || disabled || current == Some(body);
        if framed {
            frame(assets, column, rect);
        }
        let image = monster.icon().unwrap_or(0);
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
            && let Some(texture) = assets.icons.get(&enemy_icon(monster.facing_left(), image))
        {
            plot(texture, rect);
        }
    }
}

/// A warrior's cicn: facing * 10000 + tactical icon, kept to 16 bits.
///
/// Original: drawbody
pub(super) fn warrior_icon(record: centurions_content::records::WarriorRecord<'_>) -> Option<i16> {
    let icon = record.icon().ok()?;
    Some(
        i16::from(record.horizontal_facing())
            .wrapping_mul(10000)
            .wrapping_add(icon),
    )
}

/// A monster facing left uses its cicn + 1000.
///
/// Original: drawbody
pub(super) fn enemy_icon(facing: u8, icon: i16) -> i16 {
    if facing == 1 {
        icon.wrapping_add(1000)
    } else {
        icon
    }
}

/// Whether the tactical map's cell at the body's anchor holds the body.
fn on_field(combat: &Combat, (x, y): (i16, i16), body: i16) -> bool {
    let column = i32::from(x) + i32::from(combat.field.map_shift.0) + 5;
    let row = i32::from(y) + i32::from(combat.field.map_shift.1) + 5;
    (0..90).contains(&column)
        && (0..90).contains(&row)
        && combat
            .field
            .tiles
            .get(column as usize * 90 + row as usize)
            .is_some_and(|tile| tile.wrapping_abs() == body)
}

/// The anchor cell, raised 32 for size classes 1 and 3 and widened 32 to the
/// left above class 1.
///
/// Original: drawbody
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

/// Draws the part of PICT 401 that starts at pixel `column` and is as large as `rect` into
/// `rect`.
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
    crate::assets::texture_in(texture, rect);
}
