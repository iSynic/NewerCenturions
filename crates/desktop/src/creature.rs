use crate::{
    application::ApplicationAction,
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_content::word;
use macroquad::prelude::*;
use std::cell::Cell;

/// A monster or ally record viewed on the creature page.
pub struct Creature {
    pub index: usize,
    pub ally: bool,
    pub record: Vec<u8>,
    pub description: String,
    /// The icon rectangle the page starts from: the one the creature page before it
    /// left behind, or the picture frame (dialog item 37) on a page opened afresh.
    pub icon_start: Option<Rect>,
    /// The icon rectangle this page leaves for the next page it is turned to.
    pub icon_next: Cell<Option<Rect>>,
}

impl Creature {
    /// Draws the creature page (dialog 158): the creature's name, description,
    /// picture and statistics, and the buttons. The picture is drawn narrower for
    /// the tall and wide sizes, inside its full-size frame. The icon rectangle is
    /// one shared rectangle: each page insets the one the page before it left, and
    /// the check boxes of a monster page leave the rectangle of the last box.
    ///
    /// Original: beast
    pub fn draw(
        &self,
        assets: &Assets,
        placement: crate::placement::Placement,
        controls: &mut Vec<Control>,
    ) {
        let origin = placement.creature_page();
        let item = |number: usize| assets.dialog_rect_at(158, number, origin);
        assets.draw_dialog_at(158, origin);
        for number in [1, 2, 3, 4] {
            controls.push(Control {
                rect: item(number),
                enabled: true,
                action: Action::Application(ApplicationAction::CreatureItem(number)),
            });
        }
        let r = &self.record;
        let field = |number, text: String| {
            let rect = item(number);
            assets.wrapped_color(&text, rect, 10, crate::assets::line_height(10), YELLOW);
        };
        field(
            7,
            centurions_content::combat::monster_name(r).unwrap_or_default(),
        );
        field(28, self.description.clone());
        for (number, offset) in [(24, 10), (38, 0), (26, 8), (23, 6), (25, 86)] {
            field(number, word(r, offset).unwrap_or(0).to_string());
        }
        let icon = word(r, 80).unwrap_or(0);
        assets.panel(142, item(37));
        let size = word(r, 16).unwrap_or(0);
        let across = if matches!(size, 0 | 2) { 16. } else { 0. };
        let down = if size.unsigned_abs() < 2 { 16. } else { 0. };
        let start = self.icon_start.unwrap_or_else(|| item(37));
        if let Some(texture) = assets.icons.get(&icon) {
            crate::assets::texture_in(texture, inset(start, across, down));
        }
        self.icon_next.set(Some(if self.ally {
            inset(start, across, down)
        } else {
            item(45)
        }));
        if word(r, 78).unwrap_or(0) != 0 {
            for (number, address) in [(8, 0x40406a), (9, 0x404062), (10, 0x40405a)] {
                field(
                    number,
                    assets.game_text.text(address, true).unwrap_or_default(),
                );
            }
        } else {
            for i in 0..5 {
                let at = 147 + i * 4;
                if r[at] != 0 {
                    let delta = word(r, 64).unwrap_or(0);
                    field(
                        13 + i,
                        (i16::from(r[at] as i8).wrapping_add(delta)).to_string(),
                    );
                    field(
                        18 + i,
                        (i16::from(r[at + 1] as i8).wrapping_add(delta)).to_string(),
                    );
                    let index = i16::from(r[at + 2] as i8).wrapping_sub(99);
                    field(
                        8 + i,
                        assets
                            .reference_strings
                            .get(&128)
                            .and_then(|s| s.get(index as usize))
                            .cloned()
                            .unwrap_or_default(),
                    );
                }
            }
        }
        let mut special = assets
            .game_text
            .text(crate::game_strings::NO_SPECIAL_ATTACKS, true)
            .unwrap_or_default();
        for i in 0..5 {
            let value = r[150 + i * 4] as i8;
            if value != 0 {
                special = assets
                    .reference_strings
                    .get(&137)
                    .and_then(|s| s.get(value as usize))
                    .cloned()
                    .unwrap_or_default();
            }
        }
        field(27, special);
        if !self.ally {
            for i in 0..8 {
                self.mark(assets, &item, 29 + i, r[168 + i] != 0);
            }
            for i in 1..8 {
                self.mark(assets, &item, 38 + i, (r[139 + i] as i8) < 0);
                self.mark(assets, &item, 45 + i, (r[139 + i] as i8) > 100);
            }
        }
    }
    /// Draws a check box in its on (PICT 179) or off (PICT 180) look at dialog item `number`.
    fn mark(&self, assets: &Assets, item: &impl Fn(usize) -> Rect, number: usize, checked: bool) {
        assets.panel(if checked { 179 } else { 180 }, item(number));
    }
}

/// The rectangle drawn in from its left and right edges by `across` and from its
/// top and bottom edges by `down`.
fn inset(mut rect: Rect, across: f32, down: f32) -> Rect {
    rect.x += across;
    rect.w -= 2. * across;
    rect.y += down;
    rect.h -= 2. * down;
    rect
}
