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
    // A held command-click frames the body it sweeps over in place of the acting body.
    let current = ui.combat_view.peeked().or(combat.active_body());
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
        // In capture mode only the capturing warrior is framed.
        let framed = match combat.capture_actor {
            Some(actor) => actor == index as i16,
            None => {
                in_spell
                    || skipped
                    || current == Some(index as i16)
                    || flashed == Some(index as i16)
            }
        };
        if !(reveal || (on_field(combat, position, index as i16) && framed)) {
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
    let targets = capture_targets(state, combat, view);
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
        let framed = reveal
            || match combat.capture_actor {
                Some(_) => targets.get(index).copied().unwrap_or(false),
                None => in_spell || disabled || current == Some(body),
            };
        if framed {
            frame(assets, column, rect);
        }
        let image = monster.icon().unwrap_or(0);
        if definition == -3 {
            // In capture mode a body that fails the checks is not drawn at all.
            if combat.capture_actor.is_some() && !framed {
                continue;
            }
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

/// For each monster of the roster, whether capture mode draws it: empty outside capture mode.
/// The monsters in view are taken in roster order with a range counter that starts at 1 and
/// grows by one for each monster with a size; the monster must also pass `capture_target`.
///
/// Original: centerfieldlarge
pub(super) fn capture_targets(state: &Session, combat: &Combat, view: (i32, i32)) -> Vec<bool> {
    let Some(actor) = combat.capture_actor else {
        return Vec::new();
    };
    let mut range = 1;
    combat
        .roster
        .records
        .iter()
        .enumerate()
        .map(|(index, enemy)| {
            let Some(&position) = combat.roster.mon_pos.get(index) else {
                return false;
            };
            enemy.stamina().unwrap_or(0) >= 1
                && on_field(combat, position, index as i16 + 10)
                && in_view(position, view)
                && capture_target(
                    state,
                    combat,
                    actor,
                    (&enemy.record(), position),
                    &mut range,
                )
        })
        .collect()
}

/// Whether capture mode frames a monster in view: the range counter grows by one for each monster
/// with a size, the monster must lie within that many cells of the capturing warrior, belong to
/// the opposing faction and be a mechanical body, and a disabled body also needs the warrior's
/// second special skill.
///
/// Original: centerfieldlarge
fn capture_target(
    state: &Session,
    combat: &Combat,
    actor: i16,
    (monster, position): (&centurions_content::records::EnemyRecord<'_>, (i16, i16)),
    range: &mut i16,
) -> bool {
    if monster.size().unwrap_or(0) != 0 {
        *range += 1;
    }
    let Some(from) = combat.squad.pos.get(actor as usize).copied() else {
        return false;
    };
    if !centurions_core::combat::player_psi::in_range(from, position, *range)
        || monster.faction().unwrap_or(0) == 0
        || monster.mechanical_state() == 0
    {
        return false;
    }
    monster.mechanical_state() != 255
        || state.warriors.get(actor as usize).is_some_and(|warrior| {
            let record = warrior.record();
            record
                .spec(1)
                .unwrap_or(0)
                .wrapping_add(record.special_skill_bonus(1).unwrap_or(0))
                != 0
        })
}

/// Whether a body's cell lies in the 16 by 16 cells the field redraw visits.
///
/// Original: centerfieldlarge
fn in_view((x, y): (i16, i16), (vx, vy): (i32, i32)) -> bool {
    (0..16).contains(&(i32::from(x) - vx)) && (0..16).contains(&(i32::from(y) - vy))
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
pub(crate) fn plot_clipped(texture: &Texture2D, rect: Rect, clip: Rect) {
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
