use macroquad::prelude::*;

/// The 800 by 600 window.
pub const SIZE: Vec2 = Vec2::new(800., 600.);
/// The area the map is drawn in.
pub const MAP: Rect = Rect::new(0., 20., 480., 416.);
/// The left edge of the squad rows beside the map.
pub const ROW_X: f32 = 488.;
/// The top of the first squad row.
pub const ROW_Y: f32 = 22.;
/// The height of each squad row.
pub const ROW_HEIGHT: f32 = 50.;
/// The information box under the squad rows.
pub const INFORMATION: Rect = Rect::new(488., 324., 308., 104.);
/// The message box under the map.
pub const MESSAGE: Rect = Rect::new(0., 449., 468., 139.);
/// The Items button during combat; outside combat the main window's own
/// control is used.
pub const COMBAT_ITEMS: Rect = Rect::new(234., 480., 66., 16.);
/// The Conditions button during combat; outside combat the main window's own
/// control is used.
pub const COMBAT_CONDITIONS: Rect = Rect::new(154., 480., 72., 16.);
/// The Continue button inside the message box.
pub const CONTINUE: Rect = Rect::new(330., 558., 126., 25.);
/// Wrapped message text inside the message box.
pub const MESSAGE_TEXT: Rect = Rect::new(12., 467., 444., 82.);
/// The centred 640 x 480 dialog and splash frame.
pub const DIALOG_FRAME: Rect = Rect::new(80., 80., 640., 460.);
/// PICT 154, drawn over a control that is unavailable.
pub const DISABLED_CONTROL_PICT: i16 = 154;
/// Mission introduction text colour.
pub const INTRO_TEXT: Color = Color::from_rgba(252, 243, 15, 255);
/// cyanColor, used for the squad screen's unselected roster names.
pub const CYAN: Color = Color::from_rgba(0x02, 0xab, 0xea, 255);
/// RGB 0x3800/0xb300/0xf900, used for a pool below its maximum.
pub const POOL_BELOW_MAXIMUM: Color = Color::from_rgba(0x38, 0xb3, 0xf9, 255);

/// Main-window CNTL resource ids (resource names in comments) whose adjusted
/// rectangles `Assets::main_controls` holds; field input uses them as button identities.
pub mod control {
    /// "Search Toggle".
    pub const SEARCH_TOGGLE: i16 = 142;
    /// "main camp".
    pub const CAMP: i16 = 154;
    /// "main items".
    pub const ITEMS: i16 = 155;
    /// "Combat Change Weapon".
    pub const CHANGE_WEAPON: i16 = 166;
    /// "Combat monster info".
    pub const MONSTER_INFO: i16 = 167;
    /// "ShowMain Items".
    pub const SHOW_ITEMS: i16 = 168;
    /// "ShowMain Conditions".
    pub const SHOW_CONDITIONS: i16 = 169;
    /// "Auto Pos 1"; members 2..6 follow at 191..195.
    pub const AUTO_POSITION: i16 = 190;
    /// "main scan".
    pub const SCAN: i16 = 300;
    /// "main repair": the shared context slot (Capture in combat; clinic, shop or
    /// crate artwork in the field).
    pub const REPAIR: i16 = 301;
    /// "main temple".
    pub const TEMPLE: i16 = 302;
    /// "main crate".
    pub const CRATE: i16 = 303;
    /// "main shop".
    pub const SHOP: i16 = 304;
    /// "main trade".
    pub const TRADE: i16 = 305;
    /// "main encounter".
    pub const ENCOUNTER: i16 = 306;
    /// "main search".
    pub const SEARCH: i16 = 307;
    /// "main rest".
    pub const REST: i16 = 308;
    /// "main heal".
    pub const HEAL: i16 = 309;
}

/// The window's 800 by 600 picture scaled to fit, and where it sits.
#[derive(Clone, Copy)]
pub struct Canvas {
    pub scale: f32,
    pub offset: Vec2,
}
impl Canvas {
    /// Fits the picture in a window of this size: whole-number scales above 1,
    /// shrunk below.
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
    /// The pointer's place in the picture, if it is over it.
    pub fn pointer(&self, point: Vec2) -> Option<Vec2> {
        let logical = (point - self.offset) / self.scale;
        Rect::new(0., 0., SIZE.x, SIZE.y)
            .contains(logical)
            .then_some(logical)
    }
    /// Draws the picture scaled into the window.
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
/// The rectangle of squad row `index`.
pub fn row(index: usize) -> Rect {
    Rect::new(ROW_X, ROW_Y + index as f32 * ROW_HEIGHT, 308., ROW_HEIGHT)
}

/// The outdoor view origin: the party sits in cell 8, and the origin stops at
/// 75 across and 76 down.
///
/// Original: centerpictlarge
pub fn outdoor_origin(x: i32, y: i32) -> (i32, i32) {
    ((x - 8).clamp(0, 75), (y - 8).clamp(0, 76))
}
