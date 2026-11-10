use crate::assets::Assets;
use centurions_content::records::MapNote;
use centurions_content::{Content, mission::OpeningMission};
use centurions_core::Session;
use macroquad::prelude::*;

/// The look window a note is drawn in.
const LOOK: Rect = Rect::new(160., 60., 320., 320.);
/// The dialog that holds a note's text.
const NOTE_DIALOG: i16 = 169;
/// The static text item the note's text fills.
const NOTE_TEXT: usize = 2;

/// The note the player is reading and the labels the notes menu lists for this map.
#[derive(Default)]
pub struct Notes {
    pub open: Option<Note>,
    pub labels: Vec<String>,
    pub unknown_labels: Vec<String>,
}

/// A note the player opened from the Maps and Notes menu.
pub struct Note {
    pub record: Vec<u8>,
    pub text: String,
}
impl Note {
    /// Draws the note: its picture or map section, then its text in bold in the note dialog
    /// under the look window, which sits higher with the 800 x 600 preference.
    ///
    /// Original: showmap
    pub fn draw(
        &self,
        state: &Session,
        content: &Content,
        assets: &Assets,
        placement: crate::placement::Placement,
    ) {
        draw_rectangle(0., 20., 640., 460., BLACK);
        let view = MapNote(&self.record);
        if view.picture().unwrap_or(0) != 0 {
            draw_picture(&view, assets);
        } else {
            draw_section(&view, state, content, assets);
        }
        let origin = placement.scan_description();
        assets.draw_dialog_at(NOTE_DIALOG, origin);
        let text = assets.dialog_rect_at(NOTE_DIALOG, NOTE_TEXT, origin);
        assets.bold_wrapped(&self.text, text, 12, YELLOW);
    }
}

/// Half of a word, rounded toward zero.
fn half(value: i16) -> i16 {
    value.wrapping_add(((value as u16) >> 15) as i16) >> 1
}

/// Copies `source` of `texture` into `dest`, scaled, showing only the part inside the look
/// window.
fn plot(texture: &Texture2D, source: Rect, dest: Rect) {
    if dest.w <= 0. || dest.h <= 0. {
        return;
    }
    let Some(shown) = dest.intersect(LOOK) else {
        return;
    };
    let (sx, sy) = (source.w / dest.w, source.h / dest.h);
    draw_texture_ex(
        texture,
        shown.x,
        shown.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(shown.w, shown.h)),
            source: Some(Rect::new(
                source.x + (shown.x - dest.x) * sx,
                source.y + (shown.y - dest.y) * sy,
                shown.w * sx,
                shown.h * sy,
            )),
            ..Default::default()
        },
    );
}

/// The picture, centred in the look window. A picture rectangle with a bottom or right edge
/// gives the size drawn; only those two edges count, as the picture is placed by its size.
///
/// Original: showmap
fn draw_picture(view: &MapNote, assets: &Assets) {
    let Ok(picture) = view.picture() else {
        return;
    };
    let Some(texture) = assets.panels.get(&picture) else {
        return;
    };
    let (mut height, mut width) = (texture.height() as i16, texture.width() as i16);
    if let Ok([_, _, bottom, right]) = view.picture_rect()
        && (bottom != 0 || right != 0)
    {
        (height, width) = (bottom, right);
    }
    let left = half(320i16.wrapping_sub(width));
    let top = half(320i16.wrapping_sub(height));
    plot(
        texture,
        Rect::new(0., 0., texture.width(), texture.height()),
        Rect::new(
            LOOK.x + f32::from(left),
            LOOK.y + f32::from(top),
            f32::from(width),
            f32::from(height),
        ),
    );
}

/// The section of the land's floor that the note names, in cells of the note's icon size, and
/// then the note's icons. The cells are read from the floor layer followed by the object layer
/// as one run, so a section reaching past the last column or row reads on into the next.
///
/// Original: showmap
fn draw_section(view: &MapNote, state: &Session, content: &Content, assets: &Assets) {
    let (Ok(level), Ok(size), Ok(start_x), Ok(start_y)) = (
        view.level(),
        view.icon_size(),
        view.start_x(),
        view.start_y(),
    ) else {
        return;
    };
    let index = i32::from(level);
    let Some(land) = state.land_numbered(content, index) else {
        return;
    };
    let (floor, objects) = match state.exploration.as_ref().filter(|p| p.map == index) {
        Some(position) => (position.floor.as_slice(), position.objects.as_slice()),
        None => match state.visited_maps.get(&index) {
            Some(saved) => (saved.floor.as_slice(), saved.objects.as_slice()),
            None => (land.floor.as_slice(), land.objects.as_slice()),
        },
    };
    if size == 0 {
        return;
    }
    let mut cells = 320 / size;
    if cells.wrapping_mul(size) < 320 {
        cells += 1;
    }
    let sheet = assets.terrain.get(&land.terrain);
    for y in start_y..start_y.saturating_add(cells) {
        for x in start_x..start_x.saturating_add(cells) {
            let Some(sheet) = sheet else {
                break;
            };
            let position = i32::from(x) * 90 + i32::from(y);
            let Some(tile) = usize::try_from(position)
                .ok()
                .and_then(|at| floor.get(at).or_else(|| objects.get(at - floor.len())))
            else {
                continue;
            };
            let cell = Rect::new(
                LOOK.x + f32::from(x.wrapping_sub(start_x).wrapping_mul(size)),
                LOOK.y + f32::from(y.wrapping_sub(start_y).wrapping_mul(size)),
                f32::from(size),
                f32::from(size),
            );
            draw_cell(assets, sheet, land, *tile, cell);
        }
    }
    draw_marks(view, assets);
}

/// One cell of the section: a positive tile is copied from the terrain sheet; a negative tile
/// is the land's default tile with the icon of that number over it.
///
/// Original: fastplotmap
fn draw_cell(assets: &Assets, sheet: &Texture2D, land: &OpeningMission, tile: i16, cell: Rect) {
    if tile < 0 {
        draw_sheet_tile(sheet, land.default_tile, cell);
        if let Some(icon) = assets.icons.get(&tile) {
            plot(icon, Rect::new(0., 0., icon.width(), icon.height()), cell);
        }
    } else {
        draw_sheet_tile(sheet, tile, cell);
    }
}

/// Copies tile `tile` (20 to a row) of the terrain sheet into `cell`; a tile whose place lies
/// off the sheet draws nothing.
fn draw_sheet_tile(sheet: &Texture2D, tile: i16, cell: Rect) {
    let row = tile.wrapping_sub(1) / 20;
    let column = tile.wrapping_sub(row.wrapping_mul(20)).wrapping_sub(1);
    let (top, left) = (row.wrapping_shl(5), column.wrapping_shl(5));
    if top < 0
        || left < 0
        || f32::from(top) + 32. > sheet.height()
        || f32::from(left) + 32. > sheet.width()
    {
        return;
    }
    plot(
        sheet,
        Rect::new(f32::from(left), f32::from(top), 32., 32.),
        cell,
    );
}

/// The note's icons, each in the cell its column and row name. Every icon is drawn at the full
/// 32 pixels, centred on its cell, whatever the cell size.
///
/// Original: showmap
fn draw_marks(view: &MapNote, assets: &Assets) {
    let Ok(size) = view.icon_size() else {
        return;
    };
    let grow = half(32i16.wrapping_sub(size));
    for slot in 0..MapNote::MARKS {
        let Ok(mark) = view.mark(slot) else {
            continue;
        };
        if mark.icon == 0 {
            continue;
        }
        let top = mark.row.wrapping_mul(size).wrapping_sub(grow);
        let left = mark.column.wrapping_mul(size).wrapping_sub(grow);
        let extent = size.wrapping_add(grow.wrapping_mul(2));
        if let Some(icon) = assets.icons.get(&mark.icon) {
            plot(
                icon,
                Rect::new(0., 0., icon.width(), icon.height()),
                Rect::new(
                    LOOK.x + f32::from(left),
                    LOOK.y + f32::from(top),
                    f32::from(extent),
                    f32::from(extent),
                ),
            );
        }
    }
}
