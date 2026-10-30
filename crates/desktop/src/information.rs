use crate::{
    application::ApplicationAction,
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_core::{Session, character_display::Fields};
use macroquad::prelude::*;

/// Detailed statistics (DITL 177) control that returns to the summary: item 58.
pub const DETAILED_SUMMARY: usize = 58;
/// Summary statistics (DITL 138) control that closes the window: item 72.
pub const SUMMARY_CLOSE: usize = 72;

/// A warrior's statistics window, as a summary or in detail.
pub struct WarriorInformation {
    pub member: usize,
    pub fields: Option<Fields>,
    pub attempted: bool,
    pub detailed: bool,
}
impl WarriorInformation {
    /// Opens the window on squad member `member`, showing the summary.
    pub fn new(member: usize) -> Self {
        Self {
            member,
            fields: None,
            attempted: false,
            detailed: false,
        }
    }

    /// Draws the warrior's summary (DITL 138), or the detailed statistics when they
    /// are chosen.
    pub fn draw(&self, state: &Session, assets: &Assets, controls: &mut Vec<Control>) {
        if self.detailed {
            self.draw_statistics(state, assets, controls);
            return;
        }
        let port = Rect::new(0., 20., 640., 460.);
        draw_rectangle(port.x, port.y, port.w, port.h, BLACK);
        for (index, item) in assets.information_dialog.iter().enumerate() {
            let [top, left, bottom, right] = item.rect;
            let rect = Rect::new(
                f32::from(left),
                f32::from(top) + 20.,
                f32::from(right - left),
                f32::from(bottom - top),
            );
            let Some(clip) = rect.intersect(port) else {
                continue;
            };
            if item.kind == 64 && item.data.len() == 2 {
                let id = i16::from_be_bytes(item.data[..2].try_into().unwrap());
                if let Some(texture) = assets.panels.get(&id) {
                    draw_texture_ex(
                        texture,
                        clip.x,
                        clip.y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(clip.size()),
                            source: Some(Rect::new(
                                (clip.x - rect.x) / rect.w * texture.width(),
                                (clip.y - rect.y) / rect.h * texture.height(),
                                clip.w / rect.w * texture.width(),
                                clip.h / rect.h * texture.height(),
                            )),
                            ..Default::default()
                        },
                    );
                }
            }
            let number = index + 1;
            if matches!(number, 1..=5 | 65 | 72 | 96 | 97 | 100 | 105)
                || matches!(number, 73..=78 | 84..=89 | 90..=95)
                    && number
                        - if number <= 78 {
                            73
                        } else if number <= 89 {
                            84
                        } else {
                            90
                        }
                        < state.warriors.len()
            {
                controls.push(Control {
                    rect: clip,
                    enabled: true,
                    action: Action::Application(ApplicationAction::InformationItem(number)),
                });
            }
        }
        for (index, member) in state.warriors.iter().enumerate() {
            for (number, image) in [
                (
                    73 + index,
                    assets
                        .portraits
                        .get(&member.record().portrait().unwrap_or(0)),
                ),
                (
                    90 + index,
                    assets.icons.get(&member.record().icon().unwrap_or(0)),
                ),
            ] {
                if let Some(image) = image {
                    self.image(assets, number, image);
                }
            }
        }
        let member = &state.warriors[self.member];
        if let Some(image) = assets
            .portraits
            .get(&member.record().portrait().unwrap_or(0))
        {
            self.image(assets, 68, image);
        }
        if let Some(image) = assets.icons.get(&member.record().icon().unwrap_or(0)) {
            self.image(assets, 80, image);
        }
        if let Some(fields) = &self.fields {
            for (number, (text, size)) in fields {
                if let Some(item) = assets.information_dialog.get(*number as usize - 1) {
                    let [top, left, bottom, right] = item.rect;
                    let rect = Rect::new(
                        f32::from(left),
                        f32::from(top) + 20.,
                        f32::from(right - left),
                        f32::from(bottom - top),
                    );
                    if rect.intersect(port).is_some() {
                        assets.wrapped_color(text, rect, *size, f32::from(*size) + 2., YELLOW);
                    }
                }
            }
        }
    }
    /// Draws the warrior's detailed statistics (DITL 177) and the squad's portraits.
    fn draw_statistics(&self, state: &Session, assets: &Assets, controls: &mut Vec<Control>) {
        assets.draw_dialog(177);
        for (item, (text, size)) in
            centurions_core::character_display::statistics(state, self.member).unwrap_or_default()
        {
            assets.wrapped_color(
                &text,
                assets.dialog_rect(177, item as usize),
                size,
                14.,
                YELLOW,
            );
        }
        for (index, warrior) in state.warriors.iter().enumerate() {
            if let Some(image) = assets
                .portraits
                .get(&warrior.record().portrait().unwrap_or(0))
            {
                Self::dialog_image(assets.dialog_rect(177, 78 + index), image);
            }
            controls.push(Control {
                rect: assets.dialog_rect(177, 72 + index),
                enabled: true,
                action: Action::Application(ApplicationAction::InformationItem(72 + index)),
            });
        }
        let warrior = &state.warriors[self.member];
        for (item, image) in [
            (
                86,
                assets
                    .portraits
                    .get(&warrior.record().portrait().unwrap_or(0)),
            ),
            (87, assets.icons.get(&warrior.record().icon().unwrap_or(0))),
        ] {
            if let Some(image) = image {
                Self::dialog_image(assets.dialog_rect(177, item), image);
            }
        }
        controls.push(Control {
            rect: assets.dialog_rect(177, 58),
            enabled: true,
            action: Action::Application(ApplicationAction::InformationItem(DETAILED_SUMMARY)),
        });
    }
    /// Draws `image` scaled into `rect`.
    fn dialog_image(rect: Rect, image: &Texture2D) {
        crate::assets::texture_in(image, rect);
    }
    /// Draws `image` in item `number` of the summary dialog.
    fn image(&self, assets: &Assets, number: usize, image: &Texture2D) {
        let [top, left, bottom, right] = assets.information_dialog[number - 1].rect;
        crate::assets::texture_in(
            image,
            Rect::new(
                f32::from(left),
                f32::from(top) + 20.,
                f32::from(right - left),
                f32::from(bottom - top),
            ),
        );
    }
}
