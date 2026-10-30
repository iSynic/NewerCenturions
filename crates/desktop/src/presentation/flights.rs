//! A cast's flight over the tactical map, one delay(1) tick at a time.
//!
//! A sprite flight copies a 32-pixel cell of PICT 401 transparently at its walk
//! point. Line and oval flights draw in PenMode(srcXor), so each frame inverts
//! the pixels under it and its second drawing erases it.
use super::UiState;
use crate::{assets::Assets, combat_flight::FlightFrame, layout};
use macroquad::prelude::*;

/// The combat sprite picture (PICT 401).
const SPRITES: i16 = 401;

/// Original: cast
pub(super) fn draw(assets: &Assets, ui: &UiState, (vx, vy): (i32, i32)) {
    let Some(frame) = ui
        .playback
        .as_ref()
        .and_then(|playback| playback.flight)
        .and_then(|flight| flight.frame(get_time()))
    else {
        return;
    };
    let screen = |(x, y): (i16, i16)| {
        vec2(
            layout::MAP.x + (i32::from(x) - vx * 32) as f32,
            layout::MAP.y + (i32::from(y) - vy * 32) as f32,
        )
    };
    match frame {
        FlightFrame::Sprite { point, source } => {
            // A kind below 1 selects a source left of the GWorld, which copies nothing.
            if source.0 < 0 {
                return;
            }
            if let Some(texture) = assets.transparent_picture(SPRITES) {
                let at = screen(point);
                draw_texture_ex(
                    &texture,
                    at.x,
                    at.y,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(
                            f32::from(source.0),
                            f32::from(source.1),
                            32.,
                            32.,
                        )),
                        dest_size: Some(vec2(32., 32.)),
                        ..Default::default()
                    },
                );
            }
        }
        FlightFrame::Line { from, to, pen } => {
            if pen < 1 {
                return;
            }
            // The square pen hangs below and right of the drawn point.
            let half = f32::from(pen) / 2.;
            let (from, to) = (screen(from) + half, screen(to) + half);
            assets.inverted(|| draw_line(from.x, from.y, to.x, to.y, f32::from(pen), WHITE));
        }
        FlightFrame::Ovals(rects) => assets.inverted(|| {
            for (left, top, right, bottom) in rects.into_iter().flatten() {
                ring(screen((left, top)), screen((right, bottom)), 4.);
            }
        }),
    }
}

/// FrameOval's outline inside the rectangle, as one mesh so no pixel inverts twice.
fn ring(top_left: Vec2, bottom_right: Vec2, pen: f32) {
    let centre = (top_left + bottom_right) / 2.;
    let outer = (bottom_right - top_left) / 2.;
    if outer.x <= 0. || outer.y <= 0. {
        return;
    }
    let inner = (outer - pen).max(Vec2::ZERO);
    /// The number of straight segments that approximate the oval's outline.
    const SIDES: u16 = 32;
    let mut vertices = Vec::with_capacity(usize::from(SIDES) * 2);
    let mut indices = Vec::with_capacity(usize::from(SIDES) * 6);
    for side in 0..SIDES {
        let angle = f32::from(side) / f32::from(SIDES) * std::f32::consts::TAU;
        let direction = vec2(angle.cos(), angle.sin());
        for radius in [outer, inner] {
            let point = centre + direction * radius;
            vertices.push(Vertex::new(point.x, point.y, 0., 0., 0., WHITE));
        }
        let next = (side + 1) % SIDES;
        let (a, b, c, d) = (side * 2, side * 2 + 1, next * 2, next * 2 + 1);
        indices.extend_from_slice(&[a, b, c, b, d, c]);
    }
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}
