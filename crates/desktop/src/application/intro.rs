//! The scrolling introduction and prologue movie text.
use super::screen_state::IntroScreen;
use crate::assets::Assets;
use centurions_content::Content;
use centurions_core::Session;
use macroquad::prelude::*;

impl IntroScreen {
    /// Tiles the movie background and draws the scrolled movie text over it.
    pub(super) fn draw(&self, content: &Content, state: &Session, assets: &Assets) {
        clear_background(BLACK);
        let rect = Rect::new(160., 92., 480., 416.);
        let background = &assets.movie_background;
        for y in (0..rect.h as usize).step_by(background.height() as usize) {
            for x in (0..rect.w as usize).step_by(background.width() as usize) {
                draw_texture_ex(
                    background,
                    rect.x + x as f32,
                    rect.y + y as f32,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(
                            0.,
                            0.,
                            background.width().min(rect.w - x as f32),
                            background.height().min(rect.h - y as f32),
                        )),
                        ..Default::default()
                    },
                );
            }
        }
        let id = self.prologue.unwrap_or_else(|| {
            state
                .exploration
                .as_ref()
                .map_or(1000, |_| state.mission_stage.wrapping_add(1000))
        });
        let text = state
            .land(content)
            .and_then(|map| map.movies.get(&id))
            .map(String::as_str)
            .unwrap_or("");
        let mut y = rect.y + 16. + self.intro_y;
        // The intro is one left-aligned text record the full width of the
        // window, scrolled in 25-pixel steps. Runs of spaces are kept, and
        // spaces at the end of a line hang past the edge.
        for paragraph in text.replace('\r', "\n").split('\n') {
            let mut line = String::new();
            for word in paragraph.split(' ') {
                let proposed = if line.is_empty() {
                    word.to_owned()
                } else {
                    format!("{line} {word}")
                };
                let width = measure_text(proposed.trim_end(), Some(&assets.font), 24, 1.).width;
                if !line.trim().is_empty() && width > rect.w {
                    if y >= rect.y + 16. && y <= rect.bottom() - 9. {
                        assets.text(line.trim_end(), rect.x, y, 24, crate::layout::INTRO_TEXT);
                    }
                    y += 25.;
                    line = word.to_owned();
                } else {
                    line = proposed;
                }
            }
            if y >= rect.y + 16. && y <= rect.bottom() - 9. {
                assets.text(line.trim_end(), rect.x, y, 24, crate::layout::INTRO_TEXT);
            }
            y += 25.;
        }
    }
}
