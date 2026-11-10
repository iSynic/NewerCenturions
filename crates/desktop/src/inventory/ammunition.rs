//! loadammoproc's MENU 1000 popup, from the Items window's Ammo button (CNTL 176) or the combat
//! Load control.
use super::{Inventory, InventoryAction};
use crate::{
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_content::Content;
use centurions_core::{
    Command, Session,
    items::{AmmunitionEntry, LoadControl},
};
use macroquad::prelude::*;

/// The height of a row in the ammunition popup.
const ROW: f32 = 18.;

/// The ammunition popup (MENU 1000) open for a weapon: its choices and scrolling.
pub(super) struct Popup {
    warrior: usize,
    scroll: usize,
    row: usize,
    item: i16,
    entries: Vec<AmmunitionEntry>,
    /// The control the popup rises from.
    anchor: Rect,
    /// Opened by the combat Load control, which names the slot in `row`.
    combat: bool,
}

impl Inventory {
    /// Carries out a choice in the ammunition popup: opening it, choosing an
    /// ammunition, or closing it.
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
                            anchor: super::items_window::place(
                                super::items_window::AMMO,
                                self.shift(),
                            ),
                            combat: false,
                        });
                        None
                    }
                    // The rules report the caller's warning 3.
                    Err(_) => Some(Action::Command(Command::ChooseAmmunition {
                        warrior,
                        scroll,
                        row,
                        item: 0,
                    })),
                }
            }
            InventoryAction::CombatLoad(warrior) => {
                let LoadControl::Popup { slot } =
                    centurions_core::items::load_control(state, warrior).ok()?
                else {
                    return None;
                };
                let (item, entries) =
                    centurions_core::items::ammunition_menu(state, content, warrior, slot).ok()?;
                self.ammunition = Some(Popup {
                    warrior,
                    scroll: 0,
                    row: slot,
                    item,
                    entries,
                    anchor: crate::combat_controls::LOAD,
                    combat: true,
                });
                None
            }
            InventoryAction::AmmunitionItem(item) => {
                let popup = self.ammunition.take()?;
                if popup.combat {
                    return Some(Action::Command(Command::ChooseCombatAmmunition {
                        warrior: popup.warrior,
                        slot: popup.row,
                        item,
                    }));
                }
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
            // Closing the popup without a choice still keeps what building it loaded.
            _ => {
                let popup = self.ammunition.take()?;
                Some(Action::Command(if popup.combat {
                    Command::ChooseCombatAmmunition {
                        warrior: popup.warrior,
                        slot: popup.row,
                        item: 0,
                    }
                } else {
                    Command::ChooseAmmunition {
                        warrior: popup.warrior,
                        scroll: popup.scroll,
                        row: popup.row,
                        item: 0,
                    }
                }))
            }
        }
    }

    /// Whether the popup the combat Load control opened is showing.
    pub(crate) fn load_popup_open(&self) -> bool {
        self.ammunition.as_ref().is_some_and(|popup| popup.combat)
    }

    /// Closes the combat Load popup once its warrior no longer has the turn.
    pub(crate) fn close_stale_load_popup(&mut self, state: &Session) {
        if let Some(popup) = &self.ammunition
            && popup.combat
            && state
                .combat
                .as_ref()
                .and_then(centurions_core::combat::Combat::active_body)
                != Some(popup.warrior as i16)
        {
            self.ammunition = None;
        }
    }

    /// Draws the popup the combat Load control opened, if showing.
    pub(crate) fn draw_load_popup(&self, assets: &Assets, controls: &mut Vec<Control>) {
        if self.load_popup_open() {
            self.draw_ammunition(assets, controls);
        }
    }

    /// Draws the ammunition popup (MENU 1000) for the weapon in the selected row.
    pub(super) fn draw_ammunition(&self, assets: &Assets, controls: &mut Vec<Control>) {
        let Some(popup) = &self.ammunition else {
            return;
        };
        let items = match centurions_core::item_display::ammunition_menu_items(
            (popup.item, &popup.entries),
            &assets.game_text,
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
        let x = popup.anchor.x.min(800. - width);
        let y = (popup.anchor.y - height).max(20.);
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
