//! A standard vertical scroll bar (CNTL procID 16) in the platinum look: an
//! arrow box at each end and a thumb in the track when there is something to
//! scroll, a flat empty bar when there is not.
use macroquad::prelude::*;

/// The length of an arrow box and of the thumb, with their dividing lines.
const BOX: f32 = 16.;

/// Draws the scroll bar in `rect` with its thumb at `value` of `maximum`; a
/// maximum of zero leaves nothing to scroll and the bar is drawn empty.
pub(crate) fn draw(rect: Rect, value: i32, maximum: i32) {
    if maximum <= 0 {
        fill(rect, 0xf3f3f3);
        frame(rect, 0x777777);
        return;
    }
    fill(rect, 0xc0c0c0);
    let inner = Rect::new(rect.x + 1., rect.y + BOX, rect.w - 2., rect.h - 2. * BOX);
    line(inner.x, inner.y, 1., inner.h, 0x969696);
    line(inner.x, inner.y, inner.w, 1., 0x969696);
    line(inner.x + 1., inner.y + 1., 1., inner.h - 1., 0xa5a5a5);
    line(inner.x + 1., inner.y + 1., inner.w - 1., 1., 0xa5a5a5);
    line(inner.right() - 2., inner.y + 1., 1., inner.h - 1., 0xcdcdcd);
    line(inner.right() - 1., inner.y + 1., 1., inner.h - 1., 0xdadada);
    arrow_box(Rect::new(rect.x, rect.y, rect.w, BOX), true);
    arrow_box(Rect::new(rect.x, rect.bottom() - BOX, rect.w, BOX), false);
    let travel = inner.h - (BOX - 1.);
    let top = inner.y + (travel * value.clamp(0, maximum) as f32 / maximum as f32).floor();
    thumb(Rect::new(rect.x, top - 1., rect.w, BOX + 1.));
    frame(rect, 0x000000);
}

/// An arrow box: a raised face between black lines with the arrow pointing up
/// or down.
fn arrow_box(rect: Rect, up: bool) {
    fill(rect, 0x000000);
    let face = Rect::new(rect.x + 1., rect.y + 1., rect.w - 2., rect.h - 2.);
    fill(face, 0xe7e7e7);
    line(face.x, face.y, face.w - 1., 1., 0xffffff);
    line(face.x, face.y, 1., face.h - 1., 0xffffff);
    line(face.x + 1., face.bottom() - 1., face.w - 1., 1., 0xcdcdcd);
    line(face.right() - 1., face.y + 1., 1., face.h - 1., 0xcdcdcd);
    for step in 0..4 {
        let width = 2. * (step + 1) as f32;
        let row = if up { 6 + step } else { 9 - step } as f32;
        line(rect.x + 8. - width / 2., rect.y + row, width, 1., 0x000000);
    }
}

/// The thumb: a lavender face between black lines with four grip ridges.
fn thumb(rect: Rect) {
    fill(rect, 0x000000);
    let face = Rect::new(rect.x + 1., rect.y + 1., rect.w - 2., rect.h - 2.);
    fill(face, 0xb3b3ff);
    line(face.x, face.y, face.w - 1., 1., 0xdadaff);
    line(face.x, face.y, 1., face.h - 1., 0xdadaff);
    line(face.x, face.y, 1., 1., 0xf3f3f3);
    line(face.x + 1., face.bottom() - 1., face.w - 1., 1., 0x8787da);
    line(face.right() - 1., face.y + 1., 1., face.h - 1., 0x8787da);
    for ridge in 0..4 {
        let y = face.y + 3. + 2. * ridge as f32;
        line(face.x + 3., y, 7., 1., 0xdadaff);
        line(face.x + 4., y + 1., 7., 1., 0x5454b3);
    }
}

/// Fills `rect` with an RGB colour.
fn fill(rect: Rect, rgb: u32) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, colour(rgb));
}

/// A one-pixel frame round `rect`.
fn frame(rect: Rect, rgb: u32) {
    line(rect.x, rect.y, rect.w, 1., rgb);
    line(rect.x, rect.bottom() - 1., rect.w, 1., rgb);
    line(rect.x, rect.y, 1., rect.h, rgb);
    line(rect.right() - 1., rect.y, 1., rect.h, rgb);
}

/// A filled run of pixels.
fn line(x: f32, y: f32, w: f32, h: f32, rgb: u32) {
    draw_rectangle(x, y, w, h, colour(rgb));
}

/// An RGB colour from its 24-bit value.
fn colour(rgb: u32) -> Color {
    Color::from_rgba((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, 255)
}
