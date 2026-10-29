//! The Items ammunition button (CNTL 176, the Ammo part of PICT 6) and
//! loadammoproc's MENU 1000 popup.
use super::{Inventory, InventoryAction};
use crate::{
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_content::Content;
use centurions_core::{Command, Session, items::AmmunitionEntry};
use macroquad::prelude::*;

/// The button's place in the Items strip and its PICT 6 source.
pub(super) const BUTTON: Rect = Rect::new(287., 411., 71., 39.);
const SOURCE: Rect = Rect::new(142., 0., 71., 39.);
const ROW: f32 = 18.;

pub(super) struct Popup {
    warrior: usize,
    scroll: usize,
    row: usize,
    item: i16,
    entries: Vec<AmmunitionEntry>,
}

impl Inventory {
    pub(super) fn handle_ammunition(
        &mut self,
        action: InventoryAction,
        state: &Session,
        content: &Content,
    ) -> Option<Action> {
        match action {
            InventoryAction::Ammunition => {
                let warrior = self.members[0];
                let (scroll, row) = (self.scroll[0], self.display_row.max(0) as usize);
                if self.items(state, content, 0).ok()?.is_empty() {
                    return None;
                }
                match centurions_core::items::ammunition_menu(state, content, warrior, scroll + row)
                {
                    Ok((item, entries)) => {
                        self.ammunition = Some(Popup {
                            warrior,
                            scroll,
                            row,
                            item,
                            entries,
                        });
                        None
                    }
                    // The core command reports the caller's warning 3.
                    Err(_) => Some(Action::Command(Command::ChooseAmmunition {
                        warrior,
                        scroll,
                        row,
                        item: 0,
                    })),
                }
            }
            InventoryAction::AmmunitionItem(item) => {
                let popup = self.ammunition.take()?;
                if let Some(AmmunitionEntry::Information { description, .. }) =
                    popup.entries.get(usize::from(item).checked_sub(1)?)
                {
                    self.information = Some((popup.warrior, popup.scroll + popup.row));
                    self.information_item = Some((*description, false));
                }
                Some(Action::Command(Command::ChooseAmmunition {
                    warrior: popup.warrior,
                    scroll: popup.scroll,
                    row: popup.row,
                    item,
                }))
            }
            _ => {
                self.ammunition = None;
                None
            }
        }
    }

    pub(super) fn draw_ammunition_button(&self, assets: &Assets, controls: &mut Vec<Control>) {
        assets.panel_part(6, SOURCE, BUTTON);
        controls.push(Control {
            rect: BUTTON,
            enabled: true,
            action: Action::Inventory(InventoryAction::Ammunition),
        });
    }

    pub(super) fn draw_ammunition(&self, assets: &Assets, controls: &mut Vec<Control>) {
        let Some(popup) = &self.ammunition else {
            return;
        };
        let items = match centurions_core::item_display::ammunition_menu_items(
            (popup.item, &popup.entries),
            &assets.native_text,
            &assets.character_strings,
        ) {
            Ok(items) => items,
            Err(error) => {
                eprintln!("ammunition menu: {error}");
                return;
            }
        };
        controls.push(Control {
            rect: Rect::new(0., 0., 800., 600.),
            enabled: true,
            action: Action::Inventory(InventoryAction::CloseAmmunition),
        });
        let width = items
            .iter()
            .map(|item| measure_text(&item.text, Some(&assets.menu_font), 12, 1.).width + 40.)
            .fold(160., f32::max)
            .min(600.);
        let height = items.len() as f32 * ROW + 4.;
        let x = BUTTON.x.min(800. - width);
        let y = (BUTTON.y - height).max(20.);
        draw_rectangle(x, y, width, height, Color::from_rgba(238, 238, 238, 255));
        draw_rectangle_lines(x, y, width, height, 1., BLACK);
        for (index, item) in items.iter().enumerate() {
            let rect = Rect::new(x + 1., y + 2. + index as f32 * ROW, width - 2., ROW);
            if item.divider {
                let middle = rect.y + ROW / 2.;
                draw_line(rect.x + 5., middle, rect.right() - 5., middle, 1., GRAY);
                continue;
            }
            let color = if item.enabled { BLACK } else { GRAY };
            if item.checked {
                assets.menu_text("✓", rect.x + 4., rect.y + 13., 12, color);
            }
            assets.menu_text(&item.text, rect.x + 20., rect.y + 13., 12, color);
            controls.push(Control {
                rect,
                enabled: item.enabled,
                action: Action::Inventory(InventoryAction::AmmunitionItem(index as u16 + 1)),
            });
        }
    }
}
