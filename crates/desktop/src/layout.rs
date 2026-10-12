use macroquad::prelude::*;

pub const SIZE: Vec2 = Vec2::new(800., 600.);
pub const MAP: Rect = Rect::new(0., 20., 480., 416.);
pub const ROW_X: f32 = 488.;
pub const ROW_Y: f32 = 22.;
pub const ROW_HEIGHT: f32 = 50.;
pub const INFORMATION: Rect = Rect::new(488., 324., 308., 104.);
pub const MESSAGE: Rect = Rect::new(0., 449., 468., 139.);
pub const ITEMS: Rect = Rect::new(709., 453., 85., 22.);
pub const CONDITIONS: Rect = Rect::new(709., 477., 85., 22.);
pub const COMBAT_ITEMS: Rect = Rect::new(233., 475., 69., 22.);
pub const COMBAT_CONDITIONS: Rect = Rect::new(154., 475., 75., 22.);
pub const FIREARM: Rect = Rect::new(480., 540., 70., 48.);
pub const RELOAD: Rect = Rect::new(552., 540., 70., 48.);
pub const END_TURN: Rect = Rect::new(624., 540., 78., 48.);
pub const GUARD: Rect = Rect::new(710., 540., 84., 48.);
pub const CONTINUE: Rect = Rect::new(330., 558., 126., 25.);
pub const INSPECT: Rect = Rect::new(32., 42., 736., 520.);
pub const CLOSE: Rect = Rect::new(668., 52., 84., 24.);

#[derive(Clone, Copy)]
pub struct Canvas {
    pub scale: f32,
    pub offset: Vec2,
}
impl Canvas {
    pub fn new(width: f32, height: f32) -> Self {
        let scale = (width / SIZE.x).min(height / SIZE.y).floor().max(1.);
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
