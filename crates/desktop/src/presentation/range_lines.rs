//! Range lines from the warrior whose turn it is to every body: green for
//! friends, red for enemies and blue for bodies that are out of the fight.
//!
//! Reveal Friends calls showrange(0): drawbody(body, 1, 0) frames every body,
//! then 1-pixel lines are drawn straight into the window, unclipped, until a
//! click or key.
//!
//! While a warrior has the turn, centerfieldlarge also draws the same lines with
//! a 3 by 3 pen into a copy of the field and then copies the field back over the
//! view inset by 10, so only each line's last 10 pixels inside the view survive
//! as edge strokes.
use crate::layout;
use centurions_core::{Session, combat::Combat};
use macroquad::prelude::*;

/// The width of the view's border, in pixels.
const BORDER: i32 = 10;

/// How the range lines are drawn.
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

/// Original: showrange
fn draw(state: &Session, combat: &Combat, (vx, vy): (i32, i32), mode: Mode) {
    // centerfieldlarge only calls showrange while the queue's current body is
    // a warrior.
    let Some(current) = combat.active_body().filter(|body| (0..9).contains(body)) else {
        return;
    };
    let Some(&(cx, cy)) = combat.squad.pos.get(current as usize) else {
        return;
    };
    let (cx, cy) = (i32::from(cx) - vx, i32::from(cy) - vy);
    // A warrior outside the view's (0,0,9,9) cells starts its lines at (160,160).
    let start = if (0..9).contains(&cx) && (0..9).contains(&cy) {
        (cx * 32 + 16, cy * 32 + 16)
    } else {
        (160, 160)
    };
    // redColor, blueColor and the green ToolBoxInit sets up.
    let red = Color::from_rgba(0xdd, 0x08, 0x06, 255);
    let blue = Color::from_rgba(0x00, 0x00, 0xd4, 255);
    let green = Color::from_rgba(0x00, 0xff, 0x32, 255);
    // Monsters with stamina, the size class moving the end to the body centre.
    for (index, record) in combat.roster.records.iter().enumerate() {
        if record.stamina().unwrap_or(0) <= 0 {
            continue;
        }
        let Some(&(x, y)) = combat.roster.mon_pos.get(index) else {
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
    // Warriors in combat; a faction makes red, a skipped turn blue.
    for (index, warrior) in state.warriors.iter().enumerate() {
        let record = warrior.record();
        if record.in_battle() == 0 {
            continue;
        }
        let Some(&(x, y)) = combat.squad.pos.get(index) else {
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
