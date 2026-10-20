//! Read-only showcondition154a38 pop-ups (conditions and modifiers).
use crate::{
    assets::Assets,
    presentation::{Action, Control, Inspection, UiState},
};
use centurions_content::word;
use centurions_core::Session;
use macroquad::prelude::*;

const VISIBLE_ROWS: usize = 30;
const ROW_HEIGHT: f32 = 18.;

pub fn active(ui: &UiState) -> bool {
    matches!(
        ui.inspection,
        Some(Inspection::Conditions | Inspection::Modifiers)
    )
}

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

pub fn input() -> Option<Action> {
    (is_mouse_button_pressed(MouseButton::Left)
        || is_key_pressed(KeyCode::Escape)
        || is_key_pressed(KeyCode::Enter))
    .then_some(Action::Close)
}

struct Row {
    text: String,
    portrait: Option<i16>,
    checked: bool,
    timed: bool,
}

fn rows(state: &Session, assets: &Assets, ui: &UiState) -> Vec<Row> {
    let mut rows = Vec::new();
    let modifiers = ui.inspection == Some(Inspection::Modifiers);
    let labels = assets
        .character_strings
        .get(&132)
        .and_then(|bytes| centurions_content::ui::string_list(bytes).ok())
        .unwrap_or_default();
    let duration = assets.native_text.text(0x40113c, false).unwrap_or_default();
    for warrior in &state.warriors {
        rows.push(Row {
            text: warrior.name.clone(),
            portrait: warrior.record().portrait().ok(),
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
                checked: !modifiers && value < 0,
                timed: !modifiers && value > 0,
            });
        }
    }
    rows
}

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
    // PopUpMenuSelect uses left50/top400; keep the list inside the game port.
    let rect = Rect::new(50., 420_f32.min(598. - height), width, height);
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
            draw_texture_ex(
                image,
                rect.x + 17.,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(16., 16.)),
                    ..Default::default()
                },
            );
        }
        if row.checked {
            assets.menu_text("✓", rect.x + 5., y + 13., 12, BLACK);
        } else if row.timed {
            // Native SetItemMark(-41), MacRoman215.
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
