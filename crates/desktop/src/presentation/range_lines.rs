//! showrange 0x1580be: lines from the current warrior to every body.
//!
//! Reveal Friends (combat 0x116350) calls showrange(0): drawbody(body, 1, 0)
//! frames every body, then 1-pixel lines are drawn straight into the window
//! port 0x43879c, which has no ClipRect, until a click or key.
//!
//! When the current queue body is a warrior (0x4470b8[0x447194] in 0..9),
//! centerfieldlarge 0x10eb30..0x10ebf2 copies the field GWorld 0x4386c0 into
//! 0x4386b4, calls showrange(1) to draw the same lines there, copies the field
//! back over the view rect 0x4471b4 inset by 10, and copies 0x4386b4 to the
//! window. Only each line's last 10 pixels inside the view survive. GWorldInit
//! 0x127548 sets that GWorld's pen to 3 by 3.
use crate::layout;
use centurions_core::{Session, combat::Combat};
use macroquad::prelude::*;

const BORDER: i32 = 10;

#[derive(Clone, Copy)]
enum Mode {
    /// showrange(1) in centerfieldlarge: 3 by 3 pen, the view's 10-pixel border.
    Edge,
    /// showrange(0): 1 by 1 pen over the whole window.
    Reveal,
}

/// The edge strokes centerfieldlarge leaves around the view.
pub(super) fn draw_edges(state: &Session, combat: &Combat, view: (i32, i32)) {
    draw(state, combat, view, Mode::Edge);
}

/// Reveal Friends' lines, drawn after the panels they cross.
pub(super) fn draw_reveal(state: &Session, ui: &super::UiState) {
    if !ui.combat_view.reveal || ui.playback.is_some() {
        return;
    }
    let (Some(combat), Some((vx, vy))) = (
        state.combat.as_ref(),
        crate::combat_controls::origin(state, ui),
    ) else {
        return;
    };
    draw(state, combat, (i32::from(vx), i32::from(vy)), Mode::Reveal);
}

fn draw(state: &Session, combat: &Combat, (vx, vy): (i32, i32), mode: Mode) {
    // WORD4435d2 is the warrior whose turn it is; centerfieldlarge only calls
    // showrange while the queue's current body is a warrior.
    let Some(current) = combat.active_body().filter(|body| (0..9).contains(body)) else {
        return;
    };
    let Some(&(cx, cy)) = combat.squad.positions.get(current as usize) else {
        return;
    };
    let (cx, cy) = (i32::from(cx) - vx, i32::from(cy) - vy);
    // 0x1581a8: PtInRect against (0,0,9,9), otherwise (160,160).
    let start = if (0..9).contains(&cx) && (0..9).contains(&cy) {
        (cx * 32 + 16, cy * 32 + 16)
    } else {
        (160, 160)
    };
    // redColor, blueColor and the RGB at 0x443374 (ToolBoxInit 0x1322aa).
    let red = Color::from_rgba(0xdd, 0x08, 0x06, 255);
    let blue = Color::from_rgba(0x00, 0x00, 0xd4, 255);
    let green = Color::from_rgba(0x00, 0xff, 0x32, 255);
    // 0x158224: NPCs with stamina, the size class moving the end to the body centre.
    for (index, record) in combat.enemies.records.iter().enumerate() {
        if record.stamina().unwrap_or(0) <= 0 {
            continue;
        }
        let Some(&(x, y)) = combat.enemies.positions.get(index) else {
            continue;
        };
        let monster = record.record();
        let color = if monster.faction().unwrap_or(0) == 0 {
            green
        } else if monster.removed() || monster.turn_skip_duration() != 0 {
            blue
        } else {
            red
        };
        let size = monster.size().unwrap_or(0).wrapping_abs();
        let end = (
            (i32::from(x) - vx) * 32 + if size < 2 { 16 } else { 0 },
            (i32::from(y) - vy) * 32 + if size == 0 || size == 2 { 16 } else { 0 },
        );
        line(start, end, color, mode);
    }
    // 0x158322: warriors in combat; faction 0x10 makes red, turn skip 0x60 blue.
    for (index, warrior) in state.warriors.iter().enumerate() {
        let record = warrior.record();
        if record.active_flag() == 0 {
            continue;
        }
        let Some(&(x, y)) = combat.squad.positions.get(index) else {
            continue;
        };
        let color = if record.turn_skip_duration().unwrap_or(0) != 0 {
            blue
        } else if record.faction().unwrap_or(0) != 0 {
            red
        } else {
            green
        };
        line(
            start,
            ((i32::from(x) - vx) * 32 + 16, (i32::from(y) - vy) * 32 + 16),
            color,
            mode,
        );
    }
}

/// LineTo in patCopy: the pen rectangle at each point of the line, kept where the
/// mode's port shows it. Coordinates are relative to the window's content.
fn line((x0, y0): (i32, i32), (x1, y1): (i32, i32), color: Color, mode: Mode) {
    let (width, height) = (layout::MAP.w as i32, layout::MAP.h as i32);
    let window = (
        layout::SIZE.x as i32,
        layout::SIZE.y as i32 - layout::MAP.y as i32,
    );
    let pen = match mode {
        Mode::Edge => 3,
        Mode::Reveal => 1,
    };
    let steps = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
    for step in 0..=steps {
        let x = x0 + ((x1 - x0) * step * 2 + steps).div_euclid(steps * 2);
        let y = y0 + ((y1 - y0) * step * 2 + steps).div_euclid(steps * 2);
        for py in y..y + pen {
            for px in x..x + pen {
                let visible = match mode {
                    Mode::Edge => {
                        let inside = (0..width).contains(&px) && (0..height).contains(&py);
                        let inner = (BORDER..width - BORDER).contains(&px)
                            && (BORDER..height - BORDER).contains(&py);
                        inside && !inner
                    }
                    Mode::Reveal => (0..window.0).contains(&px) && (0..window.1).contains(&py),
                };
                if visible {
                    draw_rectangle(
                        layout::MAP.x + px as f32,
                        layout::MAP.y + py as f32,
                        1.,
                        1.,
                        color,
                    );
                }
            }
        }
    }
}
