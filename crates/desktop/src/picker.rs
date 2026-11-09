//! The portrait and combat icon picker dialog (DITL 170): the drawing and page rules shared by
//! the editor's picker, the warrior creation steps and the preferences' indoor squad icon.
use crate::{assets::Assets, presentation::bevel};
use centurions_content::{Content, ui};
use macroquad::prelude::*;

/// What the picker chooses.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Picker {
    /// A warrior's portrait.
    Portrait,
    /// A warrior's combat icon.
    Icon,
    /// The indoor squad icon of the preferences.
    IndoorIcon,
}

/// The warrior a picker chooses for: what the dialog prints about it and the figure it has now.
pub struct Subject {
    pub race: i16,
    pub mos: i16,
    pub gender: i16,
    pub portrait: i16,
    pub icon: i16,
}

/// The string the picker prints in label items 79 and 80 on the page of extra figures.
const EXTRA_PORTRAITS: u32 = 0x401a20;
/// The string the icon pickers print in label items 79 and 80 on the page of extra figures.
const EXTRA_ICONS: u32 = 0x4034ba;
/// The page whose last two labels are the extra figures.
const EXTRA_PAGE: i16 = 48;
/// The cicn border inside a picker cell around a 32-pixel figure.
const CELL_INSET: f32 = 6.;

/// Item `number` of the picker dialog as a rectangle, with the dialog's top left at `origin`.
pub fn item_rect(assets: &Assets, number: usize, origin: Vec2) -> Rect {
    let [top, left, bottom, right] = assets.appearance_dialog[number - 1].rect;
    Rect::new(
        origin.x + left as f32,
        origin.y + top as f32,
        (right - left) as f32,
        (bottom - top) as f32,
    )
}

/// Whether one of the 24 portraits of `page` is missing, which sends the page button back to the
/// first page.
///
/// Original: portrait
pub fn page_lacks_portrait(content: &Content, page: i16) -> bool {
    (9..=32).any(|item| {
        let id = item + 248 + page;
        !content
            .resources
            .iter()
            .any(|r| r.original.ends_with("/Portraits") && r.r#type == "cicn" && r.id == id)
    })
}

/// The page the page button turns to: the next of five pages of 24, then the first. A page with a
/// missing portrait turns to the first at once.
///
/// Original: portrait, iconpicture
pub fn next_page(page: i16, lacks_figure: bool) -> i16 {
    if lacks_figure || page > 95 {
        0
    } else {
        page + 24
    }
}

/// The title the picker shows.
///
/// Original: portrait, iconpicture
pub fn title(assets: &Assets, picker: Picker) -> String {
    let address = match picker {
        Picker::Portrait => 0x401a4c,
        Picker::Icon => 0x4034e2,
        Picker::IndoorIcon => 0x401a02,
    };
    assets.game_text.text(address, true).unwrap_or_default()
}

/// Draws item `number` of the picker dialog as the dialog does: a picture, or static text other
/// than the title and the lists of labels, which the picker prints itself; the indoor squad icon
/// picker clears the caption. Returns the item's rectangle, or nothing for an item below the
/// dialog.
///
/// Original: portrait, iconpicture
pub fn draw_item(assets: &Assets, picker: Picker, number: usize, origin: Vec2) -> Option<Rect> {
    let item = &assets.appearance_dialog[number - 1];
    let rect = item_rect(assets, number, origin);
    if rect.bottom() > 480. {
        return None;
    }
    if item.kind == 64 && item.data.len() == 2 {
        assets.panel(i16::from_be_bytes(item.data[..2].try_into().unwrap()), rect);
    }
    let caption = number == 72 && picker == Picker::IndoorIcon;
    if item.kind == 8 && !matches!(number, 71 | 74..=80) && !caption {
        assets.dialog_text(&item.text(), rect);
    }
    Some(rect)
}

/// Prints `value` in item `number` of the picker dialog.
pub fn print(assets: &Assets, number: usize, origin: Vec2, value: &str) {
    if number <= assets.appearance_dialog.len() {
        assets.dialog_text(value, item_rect(assets, number, origin));
    }
}

/// Prints the warrior's gender, race and M.O.S. in items 74 to 76 (the indoor squad icon has no
/// warrior) and the labels of the page in items 77 to 80: the race names of the page's group, and
/// on the page of extra figures the two last labels name them.
///
/// Original: portrait, iconpicture
pub fn draw_texts(
    assets: &Assets,
    picker: Picker,
    subject: Option<&Subject>,
    page: i16,
    origin: Vec2,
) {
    let strings = |table: i16| {
        assets
            .character_strings
            .get(&table)
            .and_then(|bytes| ui::string_list(bytes).ok())
            .unwrap_or_default()
    };
    if let Some(subject) = subject.filter(|_| picker != Picker::IndoorIcon) {
        let gender = match (picker == Picker::Portrait, subject.gender) {
            (true, 1) => 0x401a46,
            (true, 2) => 0x401a3e,
            (true, _) => 0x401a38,
            (false, 1) => 0x4034dc,
            (false, 2) => 0x4034d4,
            (false, _) => 0x4034ce,
        };
        print(
            assets,
            74,
            origin,
            &assets.game_text.text(gender, true).unwrap_or_default(),
        );
        for (item, table, index) in [(75, 129, subject.race), (76, 131, subject.mos)] {
            if let Some(value) = usize::try_from(index - 1)
                .ok()
                .and_then(|index| strings(table).get(index).cloned())
            {
                print(assets, item, origin, &value);
            }
        }
    }
    let races = strings(129);
    for item in 77..=80 {
        let entry = item - 76 + (page / 24).max(0) as usize * 4;
        print(
            assets,
            item,
            origin,
            races.get(entry - 1).map_or("", String::as_str),
        );
    }
    if page == EXTRA_PAGE {
        let extra = assets
            .game_text
            .text(
                if picker == Picker::Portrait {
                    EXTRA_PORTRAITS
                } else {
                    EXTRA_ICONS
                },
                true,
            )
            .unwrap_or_default();
        for item in [79, 80] {
            print(assets, item, origin, &extra);
        }
    }
}

/// Draws the sunken frames around the picker's lists of choices; the portrait picker also frames
/// the last two name items.
///
/// Original: portrait, iconpicture
pub fn draw_frames(assets: &Assets, picker: Picker, origin: Vec2) {
    let mut frames: Vec<usize> = (95..=99).collect();
    if picker == Picker::Portrait {
        frames.extend([69, 70]);
    }
    for item in frames {
        bevel::draw_edge(assets, item_rect(assets, item, origin), false);
    }
}

/// Draws the sunken edge the portrait picker leaves on a leftover item: its label loop reuses the
/// last value of the cell counter as the item. After a page has been turned that is item 33; before,
/// it is the number of squad members in the editor, whose item then lies in the first cells, on
/// the Done button or outside the window, and 9 in warrior creation, where the cell is drawn again
/// afterwards. An item outside the window is not drawn.
///
/// Original: portrait
pub fn draw_leftover_edge(assets: &Assets, origin: Vec2, item: usize) {
    if (1..=assets.appearance_dialog.len()).contains(&item) && item != 9 {
        let rect = item_rect(assets, item, origin);
        if rect.bottom() <= 480. {
            bevel::draw_edge(assets, rect, false);
        }
    }
}

/// Draws one choice cell: a raised edge and the figure, a 32-pixel one inside the cell's border.
/// The portrait picker draws the edge over the portrait and the icon pickers under the icon; a
/// selected portrait is sunken.
///
/// Original: portrait, iconpicture
pub fn draw_cell(
    assets: &Assets,
    picker: Picker,
    texture: Option<&Texture2D>,
    cell: Rect,
    selected: bool,
) {
    let figure = |texture: &Texture2D| {
        let rect = if texture.height() == 32. {
            Rect::new(
                cell.x + CELL_INSET,
                cell.y + CELL_INSET,
                cell.w - 2. * CELL_INSET,
                cell.h - 2. * CELL_INSET,
            )
        } else {
            cell
        };
        crate::assets::texture_in(texture, rect);
    };
    if picker != Picker::Portrait {
        bevel::draw_edge(assets, cell, true);
    }
    if let Some(texture) = texture {
        figure(texture);
    }
    if picker == Picker::Portrait {
        bevel::draw_edge(assets, cell, true);
        if selected {
            bevel::draw_edge(assets, cell, false);
        }
    }
}
