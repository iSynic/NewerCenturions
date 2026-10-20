//! Controls presentation.
use super::{Action, Control};
use crate::assets::Assets;
use centurions_content::{ContentError, Warrior};
use macroquad::prelude::*;

pub(crate) fn authored_text(text: &str, warriors: &[Warrior]) -> String {
    let mut text = text.to_owned();
    for index in 0..warriors.len() {
        let selected = if warriors[index].stamina().unwrap_or(0) < 1 {
            (0..warriors.len())
                .rev()
                .find(|i| warriors[*i].stamina().unwrap_or(0) > 0)
                .unwrap_or(index)
        } else {
            index
        };
        text = text.replace(&format!("^{}", index + 1), &warriors[selected].name);
    }
    text
}
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
pub(super) fn field(value: Result<i16, ContentError>) -> String {
    value.map(|v| v.to_string()).unwrap_or_else(|_| "?".into())
}
