//! The 1-pixel bevel downbutton and upbutton draw around a button in PenMode 0
//! with a 1 by 1 pen. Each draws MoveTo(left, bottom - 1), LineTo(left, top),
//! LineTo(right - 1, top) in one pixel pattern, then MoveTo(right - 1, top + 1),
//! LineTo(right - 1, bottom - 1), LineTo(left + 1, bottom - 1) in the other.
//! downbutton starts with the dark pen and upbutton with the light pen, so a
//! pressed button looks sunken.
use crate::assets::Assets;
use centurions_content::graphics::Image;
use macroquad::prelude::*;

/// downbutton's look for a combat panel button held under the pointer.
///
/// Original: downbutton
pub(super) fn draw_pressed(assets: &Assets, rect: Rect) {
    let [light, dark] = &assets.button_pens;
    let (left, top) = (rect.x as i32, rect.y as i32);
    let (right, bottom) = (left + rect.w as i32, top + rect.h as i32);
    let first: Vec<(i32, i32)> = (top..bottom)
        .map(|y| (left, y))
        .chain((left + 1..right).map(|x| (x, top)))
        .collect();
    let second: Vec<(i32, i32)> = (top + 1..bottom)
        .map(|y| (right - 1, y))
        .chain((left + 1..right - 1).map(|x| (x, bottom - 1)))
        .collect();
    stroke(dark.as_ref(), &first);
    stroke(light.as_ref(), &second);
}

/// A pixel pattern's colour at each pixel, the pattern anchored at the origin.
fn stroke(pattern: Option<&Image>, pixels: &[(i32, i32)]) {
    let Some(pattern) = pattern.filter(|p| p.width > 0 && p.height > 0) else {
        return;
    };
    let (width, height) = (i32::from(pattern.width), i32::from(pattern.height));
    for &(x, y) in pixels {
        let index = (y.rem_euclid(height) * width + x.rem_euclid(width)) as usize * 4;
        if let Some(rgba) = pattern.rgba.get(index..index + 4) {
            draw_rectangle(
                x as f32,
                y as f32,
                1.,
                1.,
                Color::from_rgba(rgba[0], rgba[1], rgba[2], rgba[3]),
            );
        }
    }
}
