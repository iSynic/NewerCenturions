use crate::{
    application::ApplicationAction,
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_content::word;
use macroquad::prelude::*;

/// A monster or ally record viewed on the creature page.
pub struct Creature {
    pub index: usize,
    pub ally: bool,
    pub record: Vec<u8>,
    pub description: String,
}

impl Creature {
    /// Draws the creature page (dialog 158): the creature's name, description,
    /// picture and statistics, and the buttons.
    pub fn draw(&self, assets: &Assets, controls: &mut Vec<Control>) {
        assets.draw_dialog(158);
        for number in [1, 2, 3, 4] {
            controls.push(Control {
                rect: assets.dialog_rect(158, number),
                enabled: true,
                action: Action::Application(ApplicationAction::CreatureItem(number)),
            });
        }
        let r = &self.record;
        let field = |number, text: String| {
            assets.wrapped_color(&text, assets.dialog_rect(158, number), 10, 12., YELLOW);
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
        assets.panel(142, assets.dialog_rect(158, 37));
        if let Some(texture) = assets.icons.get(&icon) {
            let rect = assets.dialog_rect(158, 37);
            crate::assets::texture_in(texture, rect);
        }
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
                self.mark(assets, 29 + i, r[168 + i] != 0);
            }
            for i in 1..8 {
                self.mark(assets, 38 + i, (r[139 + i] as i8) < 0);
                self.mark(assets, 45 + i, (r[139 + i] as i8) > 100);
            }
        }
    }
    /// Draws a check box in its on (PICT 179) or off (PICT 180) look at dialog item `number`.
    fn mark(&self, assets: &Assets, number: usize, checked: bool) {
        assets.panel(
            if checked { 179 } else { 180 },
            assets.dialog_rect(158, number),
        );
    }
}
