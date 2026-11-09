//! Controls presentation.
use super::{Action, Control};
use crate::assets::Assets;
use centurions_content::{ContentError, Warrior};
use macroquad::prelude::*;

/// Fills the scenario text's name tokens with the squad members' names; a text whose member
/// records cannot be read is shown as written.
///
/// Original: textbox
pub(crate) fn authored_text(text: &str, warriors: &[Warrior]) -> String {
    centurions_core::story::text::substitute_names(text, warriors)
        .unwrap_or_else(|_| text.to_owned())
}
/// Draws a labelled button, grey when it is not available, and adds its control.
pub(super) fn button(
    assets: &Assets,
    controls: &mut Vec<Control>,
    label: &str,
    rect: Rect,
    enabled: bool,
    action: Action,
) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::from_rgba(43, 46, 0, 245),
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.,
        Color::from_rgba(128, 127, 54, 255),
    );
    let width = measure_text(label, Some(&assets.font), 12, 1.).width;
    assets.text(
        label,
        rect.x + (rect.w - width) / 2.,
        rect.y + rect.h / 2. + 4.,
        12,
        if enabled { WHITE } else { GRAY },
    );
    controls.push(Control {
        rect,
        action,
        enabled,
    });
}
/// A statistic's value as text, or "?" when the record cannot supply it.
pub(super) fn field(value: Result<i16, ContentError>) -> String {
    value.map(|v| v.to_string()).unwrap_or_else(|_| "?".into())
}
