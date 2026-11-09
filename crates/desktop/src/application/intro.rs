//! The scrolling introduction and prologue movie text.
use super::screen_state::IntroScreen;
use crate::assets::Assets;
use centurions_content::{Content, text_runs::TextRun};
use centurions_core::Session;
use macroquad::prelude::*;

impl IntroScreen {
    /// Tiles the movie background and draws the scrolled movie text over it, each
    /// run of the text in its own size and colour.
    ///
    /// Original: movie
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
        let land = state.land(content);
        let text = land
            .and_then(|map| map.movies.get(&id))
            .map(String::as_str)
            .unwrap_or("");
        let runs = land
            .and_then(|map| map.movie_styles.get(&id))
            .map_or(&[][..], Vec::as_slice);
        let mut y = rect.y + 16. + self.intro_y;
        let mut offset = 0;
        // The intro is one left-aligned text record the full width of the
        // window, scrolled in 25-pixel steps. Runs of spaces are kept, and
        // spaces at the end of a line hang past the edge. Each line is as tall
        // as its tallest run.
        for paragraph in text.replace('\r', "\n").split('\n') {
            let mut line: Vec<(char, Style)> = Vec::new();
            let mut position: usize = offset;
            for word in paragraph.split(' ') {
                let mut proposed = line.clone();
                if !line.is_empty() {
                    proposed.push((' ', Style::at(runs, position.wrapping_sub(1))));
                }
                for (index, character) in word.chars().enumerate() {
                    proposed.push((character, Style::at(runs, position + index)));
                }
                position += word.chars().count() + 1;
                let fresh: Vec<(char, Style)> = word
                    .chars()
                    .enumerate()
                    .map(|(index, character)| {
                        (
                            character,
                            Style::at(runs, position - 1 - word.chars().count() + index),
                        )
                    })
                    .collect();
                if line.iter().any(|(c, _)| !c.is_whitespace())
                    && line_width(assets, &proposed) > rect.w
                {
                    draw_line(assets, &rect, &line, &mut y);
                    line = fresh;
                } else {
                    line = proposed;
                }
            }
            if line.is_empty() {
                y += Style::at(runs, offset).height;
            } else {
                draw_line(assets, &rect, &line, &mut y);
            }
            offset += paragraph.chars().count() + 1;
        }
    }
}

/// The style of one character of a movie's text.
#[derive(Clone, Copy)]
struct Style {
    run: usize,
    size: u16,
    color: Color,
    height: f32,
}

impl Style {
    /// The style at character `offset`: that of the last run to have started,
    /// or the introduction's own when the text has no runs.
    fn at(runs: &[TextRun], offset: usize) -> Self {
        match runs.iter().rposition(|run| run.start <= offset) {
            Some(index) => {
                let run = &runs[index];
                let channel = |value: u16| (value >> 8) as u8;
                Self {
                    run: index,
                    size: run.size.max(1) as u16,
                    color: Color::from_rgba(
                        channel(run.color[0]),
                        channel(run.color[1]),
                        channel(run.color[2]),
                        255,
                    ),
                    height: f32::from(run.line_height),
                }
            }
            None => Self {
                run: usize::MAX,
                size: 24,
                color: crate::layout::INTRO_TEXT,
                height: 25.,
            },
        }
    }
}

/// The characters of `line` without the spaces that end it, grouped by style.
fn groups(line: &[(char, Style)]) -> Vec<(String, Style)> {
    let end = line
        .iter()
        .rposition(|(c, _)| *c != ' ')
        .map_or(0, |index| index + 1);
    let mut groups: Vec<(String, Style)> = Vec::new();
    for (character, style) in &line[..end] {
        match groups.last_mut() {
            Some((text, last)) if last.run == style.run => text.push(*character),
            _ => groups.push((character.to_string(), *style)),
        }
    }
    groups
}

/// How wide `line` is drawn, not counting the spaces that end it.
fn line_width(assets: &Assets, line: &[(char, Style)]) -> f32 {
    let groups = groups(line);
    groups
        .iter()
        .enumerate()
        .map(|(index, (text, style))| group_width(assets, text, style, index + 1 < groups.len()))
        .sum()
}

/// The width of a group of one style; inside a line its own ending spaces count.
fn group_width(assets: &Assets, text: &str, style: &Style, inside: bool) -> f32 {
    if inside {
        assets.text_width(&format!("{text}|"), style.size) - assets.text_width("|", style.size)
    } else {
        assets.text_width(text, style.size)
    }
}

/// Draws `line` with its baseline at `y` when it is inside `rect`, then moves
/// `y` down by the line's height.
fn draw_line(assets: &Assets, rect: &Rect, line: &[(char, Style)], y: &mut f32) {
    let height = line
        .iter()
        .map(|(_, style)| style.height)
        .fold(0., f32::max);
    if *y >= rect.y + 16. && *y <= rect.bottom() - 9. {
        let groups = groups(line);
        let mut x = rect.x;
        for (index, (text, style)) in groups.iter().enumerate() {
            assets.text(text, x, *y, style.size, style.color);
            x += group_width(assets, text, style, index + 1 < groups.len());
        }
    }
    *y += height;
}
