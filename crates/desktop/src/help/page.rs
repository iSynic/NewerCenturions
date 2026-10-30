//! The popup-help window drawn over the current screen.
use crate::{
    application::ApplicationAction,
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_content::Content;
use macroquad::prelude::*;

/// Draws tutorial `page` with its Done, page and reset strips.
pub fn draw(page: usize, content: &Content, assets: &Assets, controls: &mut Vec<Control>) {
    let origin = vec2(5., 25.);
    draw_texture_ex(
        &assets.panels[&214],
        origin.x,
        origin.y,
        WHITE,
        DrawTextureParams {
            source: Some(Rect::new(12., 18., 320., 200.)),
            ..Default::default()
        },
    );
    draw_rectangle_lines(origin.x, origin.y, 320., 200., 1., MAGENTA);
    assets.panel(221, Rect::new(origin.x + 6., origin.y + 7., 308., 139.));
    let text = content
        .tutorials
        .get(page)
        .map(String::as_str)
        .unwrap_or("");
    let mut y = origin.y + 30.;
    let mut line = String::new();
    // Tutorial text has no line breaks, so wrapping goes word by word.
    for word in text.split_whitespace() {
        let proposed = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if measure_text(&proposed, Some(&assets.font), 14, 1.).width > 286. && !line.is_empty() {
            draw_text_ex(
                &line,
                origin.x + 17.,
                y,
                TextParams {
                    font: Some(&assets.font),
                    font_size: 14,
                    color: YELLOW,
                    ..Default::default()
                },
            );
            y += 16.;
            line.clear();
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    draw_text_ex(
        &line,
        origin.x + 17.,
        y,
        TextParams {
            font: Some(&assets.font),
            font_size: 14,
            color: YELLOW,
            ..Default::default()
        },
    );
    for (id, rect, items) in [
        (
            168,
            Rect::new(196., 146., 50., 50.),
            vec![(super::DONE, 0., 50.)],
        ),
        (
            142,
            Rect::new(264., 146., 50., 50.),
            vec![(4, 0., 25.), (5, 25., 25.)],
        ),
        (180, Rect::new(6., 152., 170., 31.), vec![(6, 0., 170.)]),
    ] {
        let rect = Rect::new(origin.x + rect.x, origin.y + rect.y, rect.w, rect.h);
        assets.panel(id, rect);
        for (item, x, width) in items {
            controls.push(Control {
                rect: Rect::new(rect.x + x, rect.y, width, rect.h),
                enabled: true,
                action: Action::Application(ApplicationAction::Help(item)),
            });
        }
    }
}
