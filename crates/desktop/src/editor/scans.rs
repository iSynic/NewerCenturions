//! Scan selection, shared with promotion.
use crate::{
    application::ApplicationAction,
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_content::Content;
use centurions_core::Command;
use macroquad::prelude::*;

/// The scan selection screen: the scans chosen, the scan being described, and
/// the questions and warnings waiting on the player.
pub struct ScanEditor {
    pub(super) selection: centurions_core::story_promotion::ScanSelection,
    pub(super) controls: Vec<u16>,
    pub(crate) question: bool,
    pub(super) description: Option<(usize, usize)>,
    pub(super) warning: bool,
}
impl ScanEditor {
    /// Draws the scan selection shown for a warrior's promotion.
    pub fn draw_promotion(
        progress: &centurions_core::story_promotion::Progress,
        content: &Content,
        assets: &Assets,
        controls: &mut Vec<Control>,
    ) {
        let view = Self {
            selection: progress.selection.clone(),
            controls: vec![],
            question: progress.question,
            description: progress.description,
            warning: progress.warning,
        };
        view.draw(content, assets, controls);
        for control in controls {
            control.action = match control.action {
                Action::Application(ApplicationAction::EditorItem(item)) => {
                    Action::Command(Command::PromotionControl { item: item as u16 })
                }
                Action::Application(ApplicationAction::ScanAnswer(yes)) => {
                    Action::Command(Command::PromotionAnswer { yes })
                }
                ref other => other.clone(),
            };
        }
    }
    /// Item `item` of the scan dialog as a rectangle in the window.
    fn rect(assets: &Assets, item: usize) -> Rect {
        let [top, left, bottom, right] = assets.scan_dialog[item - 1].rect;
        Rect::new(
            222. + f32::from(left),
            80. + f32::from(top),
            f32::from(right - left),
            f32::from(bottom - top),
        )
    }
    /// Draws the scan selection dialog and its controls.
    pub(super) fn draw(&self, content: &Content, assets: &Assets, controls: &mut Vec<Control>) {
        let bounds = Rect::new(222., 80., 356., 460.);
        draw_rectangle(bounds.x, bounds.y, bounds.w, bounds.h, BLACK);
        for item in [22, 23, 31, 35, 38, 41, 49, 50] {
            let dialog = &assets.scan_dialog[item - 1];
            let id = i16::from_be_bytes(dialog.data[..2].try_into().unwrap());
            let full = Self::rect(assets, item);
            if let Some(clipped) = full.intersect(bounds) {
                let texture = &assets.panels[&id];
                draw_texture_ex(
                    texture,
                    clipped.x,
                    clipped.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(clipped.w, clipped.h)),
                        source: Some(Rect::new(
                            (clipped.x - full.x) * texture.width() / full.w,
                            (clipped.y - full.y) * texture.height() / full.h,
                            clipped.w * texture.width() / full.w,
                            clipped.h * texture.height() / full.h,
                        )),
                        ..Default::default()
                    },
                );
            }
        }
        assets.panel(130, Self::rect(assets, 9));
        let warrior = &self.selection.warrior;
        let school = warrior.record().psi_school().unwrap_or(0);
        let portrait = warrior.record().portrait().unwrap_or(0);
        if let Some(texture) = assets.portraits.get(&portrait) {
            let rect = Self::rect(assets, 34);
            crate::assets::texture_in(texture, rect);
        }
        let rect = Self::rect(assets, 33);
        assets.text(&warrior.name, rect.x, rect.bottom() - 2., 12, YELLOW);
        let rect = Self::rect(assets, 40);
        assets.text(
            &self.selection.remaining.to_string(),
            rect.x,
            rect.bottom() - 2.,
            12,
            YELLOW,
        );
        for (tier, cost) in centurions_core::story_promotion::SCAN_COSTS
            .into_iter()
            .enumerate()
        {
            let rect = Self::rect(assets, 24 + tier);
            assets.text(&cost.to_string(), rect.x, rect.bottom() - 2., 12, YELLOW);
            let rect = Self::rect(assets, 2 + tier);
            if tier == self.selection.tier {
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1., YELLOW);
            }
            controls.push(Control {
                rect,
                enabled: true,
                action: Action::Application(ApplicationAction::EditorItem(2 + tier)),
            });
        }
        for column in 0..7 {
            let rect = Self::rect(assets, 10 + column);
            let spell = school * 1000 + (self.selection.tier as i16 + 1) * 100 + column as i16 + 1;
            let label = Self::rect(assets, 42 + column);
            assets.text(
                assets.spell_names.get(&spell).map_or("", String::as_str),
                label.x,
                label.bottom() - 2.,
                12,
                YELLOW,
            );
            if warrior.record().known_scan(self.selection.tier, column) != 0 {
                assets.panel(179, Rect::new(rect.x - 17., rect.y + 5., 9., 9.));
            }
            controls.push(Control {
                rect,
                enabled: true,
                action: Action::Application(ApplicationAction::EditorItem(10 + column)),
            });
        }
        if let Some((tier, column)) = self.description
            && let Some(map) = content.map(0)
            && let Ok(description) = map.scan_description(school, tier, column)
        {
            let mut rect = Self::rect(assets, 39);
            rect.y += 12.;
            rect.h -= 12.;
            assets.wrapped_color(&description, rect, 12, 15., YELLOW);
        }
        let rect = Self::rect(assets, 32);
        crate::presentation::push_panel_control(
            assets,
            controls,
            168,
            rect,
            true,
            Action::Application(ApplicationAction::EditorItem(32)),
        );
        if self.question || self.warning {
            controls.clear();
            let origin = vec2(221., 203.);
            draw_texture_ex(
                &assets.panels[&214],
                origin.x,
                origin.y,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(140., 167., 358., 195.)),
                    ..Default::default()
                },
            );
            assets.panel(182, Rect::new(origin.x + 22., origin.y + 23., 320., 84.));
            let text = if self.question {
                &assets.questions[16]
            } else {
                &assets.warnings[73]
            };
            assets.wrapped(
                text,
                Rect::new(origin.x + 36., origin.y + 43., 284., 65.),
                14,
                17.,
            );
            for (id, x, answer) in [(2, 66., true), (4, 208., false)] {
                if self.warning && !answer {
                    continue;
                }
                let rect = Rect::new(origin.x + x, origin.y + 127., 70., 35.);
                crate::presentation::push_panel_control(
                    assets,
                    controls,
                    if self.warning { 168 } else { id },
                    rect,
                    true,
                    Action::Application(if self.warning {
                        ApplicationAction::EditorItem(0)
                    } else {
                        ApplicationAction::ScanAnswer(answer)
                    }),
                );
            }
        }
    }
}
