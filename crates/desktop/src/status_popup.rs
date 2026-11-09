//! Read-only condition, modifier and creature pop-ups, as showcondition and
//! showcondition2 show them.
use crate::{
    assets::Assets,
    presentation::{Action, Control, Inspection, UiState},
};
use centurions_content::word;
use centurions_core::Session;
use macroquad::prelude::*;

/// The rows shown at once.
const VISIBLE_ROWS: usize = 30;
/// The height of each row.
const ROW_HEIGHT: f32 = 18.;

/// Whether the conditions, modifiers or creature list is open.
pub fn active(ui: &UiState) -> bool {
    matches!(
        ui.inspection,
        Some(
            Inspection::Conditions
                | Inspection::Modifiers
                | Inspection::CreatureConditions(_)
                | Inspection::CreatureItems(_)
        )
    )
}

/// Whether the open list is a creature's.
fn creature(ui: &UiState) -> bool {
    matches!(
        ui.inspection,
        Some(Inspection::CreatureConditions(_) | Inspection::CreatureItems(_))
    )
}

/// Scrolls the list with the arrow keys and the mouse wheel.
pub fn scroll(state: &Session, assets: &Assets, ui: &mut UiState) {
    let maximum = rows(state, assets, ui).len().saturating_sub(VISIBLE_ROWS);
    let direction = if is_key_pressed(KeyCode::Down) {
        1
    } else if is_key_pressed(KeyCode::Up) {
        -1
    } else {
        -mouse_wheel().1.signum() as i32
    };
    ui.page = (ui.page as i32 + direction).clamp(0, maximum as i32) as usize;
}

/// Any click, Escape or Return closes the list.
pub fn input() -> Option<Action> {
    (is_mouse_button_pressed(MouseButton::Left)
        || is_key_pressed(KeyCode::Escape)
        || is_key_pressed(KeyCode::Enter))
    .then_some(Action::Close)
}

/// One line of the list: a warrior's name or a condition, with its portrait,
/// check mark and timing.
struct Row {
    text: String,
    portrait: Option<i16>,
    icon: Option<i16>,
    checked: bool,
    timed: bool,
}

/// The lines of a creature's list: its name and icon, then one line for each
/// condition it is under or each item it carries, a carried item checked when its
/// word is negative.
///
/// Original: showcondition2
fn creature_rows(state: &Session, assets: &Assets, body: i16, items: bool) -> Vec<Row> {
    let Some(monster) = state
        .combat
        .as_ref()
        .and_then(|combat| combat.roster.records.get(usize::try_from(body - 10).ok()?))
        .map(|monster| monster.record())
    else {
        return Vec::new();
    };
    let row = |text: String, checked: bool| Row {
        text,
        portrait: None,
        icon: None,
        checked,
        timed: false,
    };
    let mut rows = vec![Row {
        icon: monster.icon().ok(),
        ..row(monster.name().unwrap_or_default(), false)
    }];
    if items {
        for slot in 0..6 {
            let item = monster.item(slot).unwrap_or(0);
            if item != 0 {
                rows.push(row(assets.name(item.wrapping_abs()), item < 0));
            }
        }
    } else {
        for index in (0..40).filter(|index| monster.condition_duration(*index) != 0) {
            rows.push(row(
                assets.conditions.get(index).cloned().unwrap_or_default(),
                false,
            ));
        }
    }
    rows
}

/// The lines of the conditions or modifiers list for the squad, as showcondition
/// lists them, or of the creature list.
fn rows(state: &Session, assets: &Assets, ui: &UiState) -> Vec<Row> {
    match ui.inspection {
        Some(Inspection::CreatureConditions(body)) => {
            return creature_rows(state, assets, body, false);
        }
        Some(Inspection::CreatureItems(body)) => return creature_rows(state, assets, body, true),
        _ => {}
    }
    let mut rows = Vec::new();
    let modifiers = ui.inspection == Some(Inspection::Modifiers);
    let labels = assets
        .character_strings
        .get(&132)
        .and_then(|bytes| centurions_content::ui::string_list(bytes).ok())
        .unwrap_or_default();
    let duration = assets
        .game_text
        .text(crate::game_strings::DURATION_LABEL, false)
        .unwrap_or_default();
    for warrior in &state.warriors {
        rows.push(Row {
            text: warrior.name.clone(),
            portrait: warrior.record().portrait().ok(),
            icon: None,
            checked: false,
            timed: false,
        });
        let names = if modifiers {
            &labels
        } else {
            &assets.conditions
        };
        for (index, label) in names
            .iter()
            .take(if modifiers { 12 } else { 40 })
            .enumerate()
        {
            let offset = if modifiers { 0xae } else { 0x5e };
            let value = word(&warrior.bytes, offset + index * 2).unwrap_or(0);
            if value == 0 || label.is_empty() {
                continue;
            }
            let text = if modifiers {
                format!("{value:+} {label}")
            } else if value > 0 {
                format!("{label}{duration}{value}")
            } else {
                label.clone()
            };
            rows.push(Row {
                text,
                portrait: None,
                icon: None,
                checked: !modifiers && value < 0,
                timed: !modifiers && value > 0,
            });
        }
    }
    rows
}

/// Draws the list in a pop-up, a page at a time, with a control that closes it.
pub fn draw(state: &Session, assets: &Assets, ui: &UiState, controls: &mut Vec<Control>) {
    let rows = rows(state, assets, ui);
    let first = ui.page.min(rows.len().saturating_sub(VISIBLE_ROWS));
    let visible = rows.len().min(VISIBLE_ROWS);
    let width = (rows
        .iter()
        .map(|row| measure_text(&row.text, Some(&assets.menu_font), 12, 1.).width)
        .fold(160., f32::max)
        + 52.)
        .min(744.);
    let height = visible as f32 * ROW_HEIGHT + 8.;
    // PopUpMenuSelect opens at left 50, top 400, or at left 40 for a creature; keep
    // the list inside the game port.
    let left = if creature(ui) { 40. } else { 50. };
    let rect = Rect::new(left, 420_f32.min(598. - height), width, height);
    draw_rectangle(rect.x + 3., rect.y + 3., rect.w, rect.h, BLACK);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::from_rgba(238, 238, 238, 255),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1., BLACK);
    for (index, row) in rows.iter().skip(first).take(visible).enumerate() {
        let y = rect.y + 4. + index as f32 * ROW_HEIGHT;
        if let Some(image) = row.portrait.and_then(|id| assets.portraits.get(&id)) {
            crate::assets::texture_in(image, Rect::new(rect.x + 17., y, 16., 16.));
        }
        if let Some(image) = row.icon.and_then(|id| assets.icons.get(&id)) {
            crate::assets::texture_in(image, Rect::new(rect.x + 17., y, 16., 16.));
        }
        if row.checked {
            assets.menu_text("✓", rect.x + 5., y + 13., 12, BLACK);
        } else if row.timed {
            // SetItemMark(-41), MacRoman 215.
            assets.menu_text("◊", rect.x + 5., y + 13., 12, BLACK);
        }
        assets.menu_text(&row.text, rect.x + 38., y + 13., 12, BLACK);
    }
    let center = rect.x + rect.w / 2.;
    if first > 0 {
        draw_triangle(
            vec2(center, rect.y + 1.),
            vec2(center - 3., rect.y + 4.),
            vec2(center + 3., rect.y + 4.),
            BLACK,
        );
    }
    if first + visible < rows.len() {
        draw_triangle(
            vec2(center, rect.bottom() - 1.),
            vec2(center - 3., rect.bottom() - 4.),
            vec2(center + 3., rect.bottom() - 4.),
            BLACK,
        );
    }
    controls.push(Control {
        rect,
        enabled: true,
        action: Action::Close,
    });
}
