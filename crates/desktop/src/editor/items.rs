//! The carried-item editor: its own draft, selection and modifier controls.
use crate::{
    application::ApplicationAction,
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_content::Content;
use centurions_core::{Command, Event, Session, apply};
use macroquad::prelude::*;

/// Item editor (DITL 141) control 2, which commits the draft and closes like
/// item 1 (Done).
pub const ITEM_EDITOR_CLOSE: usize = 2;

/// The carried-item editor: a draft of the session in which one item of a
/// warrior's pack is changed.
pub struct ItemEditor {
    pub draft: Session,
    member: usize,
    slot: usize,
}
impl ItemEditor {
    /// Opens the editor on item `slot` of squad member `member`'s pack.
    pub fn new(state: &Session, member: usize, slot: usize) -> Result<Self, String> {
        let count = state
            .warriors
            .get(member)
            .ok_or("Editor warrior is outside the squad.")?
            .inventory()
            .map_err(|e| e.to_string())?
            .len();
        if slot > count || slot >= 100 {
            return Err("Editor item is outside the inventory.".into());
        }
        Ok(Self {
            draft: state.clone(),
            member,
            slot,
        })
    }
    /// Carries out a click: moving to the previous or next warrior or item, or
    /// changing the item's modifiers.
    pub fn handle(&mut self, control: usize, content: &Content) -> Result<Vec<Event>, String> {
        match control {
            67 | 68 => {
                let count = self.draft.warriors.len();
                self.member = (self.member + if control == 67 { count - 1 } else { 1 }) % count;
                self.slot = 0;
            }
            69 => self.slot = self.slot.saturating_sub(1),
            70 => {
                let count = self.draft.warriors[self.member]
                    .inventory()
                    .map_err(|e| e.to_string())?
                    .len();
                self.slot = (self.slot + 1).min(count).min(99);
            }
            40..=63 => {
                return apply(
                    Command::EditCarriedItem {
                        warrior: self.member,
                        slot: self.slot,
                        control: control as u16,
                    },
                    &mut self.draft,
                    content,
                )
                .map_err(|e| e.to_string());
            }
            _ => return Err("This item editor action has not been reconstructed.".into()),
        }
        Ok(vec![])
    }
    /// Item `item` of the item editor dialog as a rectangle in the window.
    fn rect(assets: &Assets, item: usize) -> Rect {
        let [top, left, bottom, right] = assets.item_dialog[item - 1].rect;
        Rect::new(
            80. + f32::from(left),
            80. + f32::from(top),
            f32::from(right - left),
            f32::from(bottom - top),
        )
    }
    /// Draws the item editor dialog: the item's name and modifiers, and the
    /// controls.
    pub fn draw(&self, content: &Content, assets: &Assets, controls: &mut Vec<Control>) {
        clear_background(BLACK);
        assets.panel(215, crate::layout::DIALOG_FRAME);
        assets.panel(168, Self::rect(assets, 2));
        for item in 16..=27 {
            let rect = Self::rect(assets, item);
            assets.text(
                &assets.item_dialog[item - 1].text(),
                rect.x,
                rect.bottom() - 2.,
                12,
                YELLOW,
            );
        }
        let warrior = &self.draft.warriors[self.member];
        let item = warrior.record().inventory_slot(self.slot);
        for row in 0..10 {
            let rect = Self::rect(assets, 28 + row);
            assets.text(
                &item.editor_modifier(row).to_string(),
                rect.x,
                rect.bottom() - 2.,
                12,
                YELLOW,
            );
        }
        for (number, value) in [(38, item.charge()), (39, item.capacity())] {
            let rect = Self::rect(assets, number);
            assets.text(
                &value.unwrap_or(0).to_string(),
                rect.x,
                rect.bottom() - 2.,
                12,
                YELLOW,
            );
        }
        for (number, text) in [(66, assets.name(item.id())), (72, warrior.name.clone())] {
            let rect = Self::rect(assets, number);
            assets.text(&text, rect.x, rect.bottom() - 2., 12, YELLOW);
        }
        if let Ok(id) = warrior.record().portrait()
            && let Some(portrait) = assets.portraits.get(&id)
        {
            let rect = Self::rect(assets, 71);
            crate::assets::texture_in(portrait, rect);
        }
        if let Some(definition) = content.items.get(usize::from(item.id().unsigned_abs())) {
            if let Ok(id) = centurions_content::records::ItemDefinition(definition).icon()
                && let Some(icon) = assets.icons.get(&id)
            {
                let rect = Self::rect(assets, 64);
                draw_texture(icon, rect.x, rect.y, WHITE);
            }
            if let Ok(descriptor) =
                centurions_content::records::ItemDefinition(definition).description_id()
                && let Some(map) = content.map(0)
                && let Ok(text) = map.item_description(descriptor)
            {
                let mut rect = Self::rect(assets, 65);
                rect.y += 12.;
                rect.h -= 12.;
                assets.wrapped_color(&text, rect, 12, 15., YELLOW);
            }
        }
        for item in (40..=63).chain([2, 67, 68, 69, 70]) {
            controls.push(Control {
                rect: Self::rect(assets, item),
                enabled: true,
                action: Action::Application(ApplicationAction::ItemEditorItem(item)),
            });
        }
    }
}
