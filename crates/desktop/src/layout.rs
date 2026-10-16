use macroquad::prelude::*;

pub const SIZE: Vec2 = Vec2::new(800., 600.);
pub const MAP: Rect = Rect::new(0., 20., 480., 416.);
pub const ROW_X: f32 = 488.;
pub const ROW_Y: f32 = 22.;
pub const ROW_HEIGHT: f32 = 50.;
pub const INFORMATION: Rect = Rect::new(488., 324., 308., 104.);
pub const MESSAGE: Rect = Rect::new(0., 449., 468., 139.);
pub const COMBAT_ITEMS: Rect = Rect::new(234., 480., 66., 16.);
pub const COMBAT_CONDITIONS: Rect = Rect::new(154., 480., 72., 16.);
pub const CONTINUE: Rect = Rect::new(330., 558., 126., 25.);

#[derive(Clone, Copy)]
pub struct Canvas {
    pub scale: f32,
    pub offset: Vec2,
}
impl Canvas {
    pub fn new(width: f32, height: f32) -> Self {
        let fit = (width / SIZE.x).min(height / SIZE.y);
        let scale = if fit < 1. { fit.max(0.1) } else { fit.floor() };
        Self {
            scale,
            offset: vec2(
                (width - SIZE.x * scale) / 2.,
                (height - SIZE.y * scale) / 2.,
            ),
        }
    }
    pub fn pointer(&self, point: Vec2) -> Option<Vec2> {
        let logical = (point - self.offset) / self.scale;
        Rect::new(0., 0., SIZE.x, SIZE.y)
            .contains(logical)
            .then_some(logical)
    }
    pub fn draw(&self, target: &RenderTarget) {
        set_default_camera();
        clear_background(BLACK);
        draw_texture_ex(
            &target.texture,
            self.offset.x,
            self.offset.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(SIZE * self.scale),
                flip_y: true,
                ..Default::default()
            },
        );
    }
}
pub fn row(index: usize) -> Rect {
    Rect::new(ROW_X, ROW_Y + index as f32 * ROW_HEIGHT, 308., ROW_HEIGHT)
}

/// centerpictlarge 0x10f1c4 uses cell8 and native origin limits75/76.
pub fn outdoor_origin(x: i32, y: i32) -> (i32, i32) {
    ((x - 8).clamp(0, 75), (y - 8).clamp(0, 76))
}
