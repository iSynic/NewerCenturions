//! The popup-help window drawn over the current screen.
use crate::{
    application::ApplicationAction,
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_content::{Content, records::TutorialRectangles};
use macroquad::prelude::*;

/// The picture that lights the highlighted rectangles.
const FLASH_PICTURE: i16 = 196;
/// How long a highlight stays lit, in ticks of a sixtieth of a second.
const LIT_TICKS: f64 = 3.;

/// The rectangles of tutorial `page`, moved down by the menu bar, with the ticks the
/// page waits after lighting each: ten when the other rectangle's top is zero and
/// otherwise five. A rectangle whose top is zero once moved is not lit.
///
/// Original: flashrect
fn highlights(page: usize, content: &Content) -> Vec<(Rect, f64)> {
    let Some(table) = content
        .scenario
        .as_ref()
        .and_then(|scenario| scenario.maps.first())
        .and_then(|map| map.tutorial_rectangles.as_deref())
    else {
        return Vec::new();
    };
    let Ok(Some(mut rectangles)) = TutorialRectangles(table).page(page) else {
        return Vec::new();
    };
    for rectangle in &mut rectangles {
        rectangle[0] = rectangle[0].wrapping_add(crate::layout::MENU_BAR as i16);
        rectangle[2] = rectangle[2].wrapping_add(crate::layout::MENU_BAR as i16);
    }
    let waits = [rectangles[1][0], rectangles[0][0]].map(|other| if other == 0 { 10. } else { 5. });
    rectangles
        .iter()
        .zip(waits)
        .filter(|(rectangle, _)| rectangle[0] != 0)
        .map(|(rectangle, wait)| {
            let [top, left, bottom, right] = rectangle.map(f32::from);
            (Rect::new(left, top, right - left, bottom - top), wait)
        })
        .collect()
}

/// Lights each highlight of the page in turn, restoring the screen under it after the
/// pause that follows, over and over for as long as the page is open.
///
/// Original: flashrect
fn flash(page: usize, age: f64, content: &Content, assets: &Assets) {
    let highlights = highlights(page, content);
    let cycle: f64 = highlights.iter().map(|(_, wait)| LIT_TICKS + wait).sum();
    if cycle == 0. {
        return;
    }
    let mut ticks = age * 60. % cycle;
    for (rectangle, wait) in highlights {
        if ticks < LIT_TICKS {
            if let Some(texture) = assets.transparent_picture(FLASH_PICTURE) {
                crate::assets::texture_in(&texture, rectangle);
            }
            return;
        }
        ticks -= LIT_TICKS + wait;
        if ticks < 0. {
            return;
        }
    }
}

/// Draws tutorial `page` with its Done, page and reset strips; with the large window
/// preference on, `age` is the seconds the page has been up and the page's highlights
/// flash by it.
pub fn draw(
    page: usize,
    age: Option<f64>,
    content: &Content,
    assets: &Assets,
    controls: &mut Vec<Control>,
) {
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
    if let Some(age) = age {
        flash(page, age, content, assets);
    }
}
