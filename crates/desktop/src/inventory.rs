use crate::{assets::Assets, presentation::Action};
use centurions_content::{Content, InventoryRecord};
use centurions_core::{Command, RuleError, Session};
use macroquad::prelude::*;
use std::{cell::RefCell, collections::BTreeMap};
/// The item list rows already built for each item record and its definition.
type ItemRows = BTreeMap<Vec<u8>, Vec<(i32, String)>>;
/// The item information panel already built for an item record.
type ItemDetail = (
    Vec<u8>,
    Result<centurions_core::item_display::output::PanelPlan, String>,
);

/// A choice on the Items screen: a warrior, an item, scrolling, transferring,
/// stockpiling, shopping, refitting, ammunition and item information.
#[derive(Clone, Debug)]
pub enum InventoryAction {
    Warrior {
        side: usize,
        member: usize,
    },
    Item {
        side: usize,
        slot: usize,
    },
    Scroll {
        side: usize,
        delta: i32,
    },
    Transfer,
    Activate,
    Refit(usize),
    Stockpile,
    Shop,
    Category(usize),
    Done,
    /// CNTL 140 opens showiteminfo on the selected display row.
    Information,
    CloseInformation,
    /// DLOG 149 item 1 (squad users) or item 2 (warrior item types).
    UseList(bool),
    /// CNTL 176 opens loadammoproc's MENU 1000 popup on the display row.
    Ammunition,
    /// A MENU 1000 item, numbered from 1.
    AmmunitionItem(u16),
    CloseAmmunition,
    /// moveicon's option-drag on a shop window row.
    DragInformation {
        side: usize,
        slot: usize,
    },
}

/// The Items screen's state: the two warriors shown, their scrolling, the item
/// picked up or selected, and the information and ammunition popups open.
pub struct Inventory {
    pub members: [usize; 2],
    pub scroll: [usize; 2],
    pub selected: Option<(usize, usize)>,
    pub quick_info: bool,
    /// The selected display row: zero on entry, set by a row selection and kept
    /// when the owner changes.
    display_row: i16,
    drag: Option<(usize, usize)>,
    last_click: f64,
    category: usize,
    rows: RefCell<ItemRows>,
    detail: RefCell<Option<ItemDetail>>,
    /// The open DLOG 149 member and slot.
    information: Option<(usize, usize)>,
    /// The item ID of a dialog opened with member 0xffff, and whether booty
    /// opened it over the loot grid.
    information_item: Option<(i16, bool)>,
    information_fields: RefCell<Option<information::ItemInformation>>,
    /// An open showcondition list: MOSs, widened and the ITEM work it read.
    use_list: Option<(bool, bool, Vec<u8>)>,
    /// An open ammunition popup.
    ammunition: Option<ammunition::Popup>,
}
impl Default for Inventory {
    /// The first two warriors, nothing selected.
    fn default() -> Self {
        Self {
            members: [0, 1],
            scroll: [0; 2],
            selected: None,
            quick_info: false,
            display_row: 0,
            drag: None,
            last_click: -1.,
            category: 0,
            rows: RefCell::new(BTreeMap::new()),
            detail: RefCell::new(None),
            information: None,
            information_item: None,
            information_fields: RefCell::new(None),
            use_list: None,
            ammunition: None,
        }
    }
}
impl Inventory {
    /// The command that tells the rules which warriors, scrolling and selection the
    /// Items screen is showing.
    pub fn context_command(&self) -> Command {
        Command::InspectInventory {
            members: self.members,
            scroll: self.scroll,
            selected: self.selected,
            shop_family: self.category,
        }
    }
    /// Opens the Items screen on warrior `selected` and the next one of `count`,
    /// with nothing selected.
    pub fn open(&mut self, selected: usize, count: usize) {
        self.members = [
            selected,
            if count > 1 {
                (selected + 1) % count
            } else {
                selected
            },
        ];
        self.scroll = [0; 2];
        self.display_row = 0;
        self.category = 0;
        self.selected = None;
        self.drag = None;
        self.rows.borrow_mut().clear();
        self.detail.borrow_mut().take();
        self.information = None;
        self.use_list = None;
        self.ammunition = None;
    }
    /// Carries out a choice on the Items screen, returning the action for the game
    /// to take, if any.
    pub fn handle(
        &mut self,
        action: InventoryAction,
        state: &Session,
        content: &Content,
        now: f64,
    ) -> Result<Option<Action>, RuleError> {
        use InventoryAction::*;
        if self.information.is_some() {
            return Ok(self.handle_information(action, state));
        }
        if self.ammunition.is_some() || matches!(action, Ammunition) {
            return Ok(self.handle_ammunition(action, state, content));
        }
        match action {
            Warrior { side, member } => {
                if side > 1 || member >= state.warriors.len() {
                    return Err("inventory warrior outside squad".into());
                }
                let scroll = self.scroll[side];
                self.members[side] = member;
                self.selected = None;
                if !Self::shop(state) {
                    return Ok(Some(Action::Command(Command::SwitchInventoryOwner {
                        warrior: member,
                        scroll,
                        quick_info: self.quick_info,
                        selected_row: self.display_row,
                    })));
                }
                self.scroll[side] = 0;
            }
            DragInformation { side, slot } => {
                if !Self::shop(state) {
                    return Ok(None);
                }
                return Ok(self.drag_information(state, content, side, slot));
            }
            Item { side, slot } => {
                self.record(state, content, side, slot)?;
                let activate = self.selected == Some((side, slot)) && now - self.last_click < 0.35;
                self.selected = Some((side, slot));
                self.display_row = slot.saturating_sub(self.scroll[side]) as i16;
                self.drag = Some((side, slot));
                self.last_click = now;
                if activate {
                    return self.handle(Activate, state, content, now);
                }
                if !Self::shop(state) && side == 0 && self.quick_info {
                    return Ok(Some(Action::Command(Command::HighlightInventoryRow {
                        warrior: self.members[side],
                        scroll: self.scroll[side],
                        row: self.display_row as usize,
                        quick_info: self.quick_info,
                    })));
                }
            }
            Scroll { side, delta } => {
                if side > 1 {
                    return Err("inventory pane unavailable".into());
                }
                let count = self.items(state, content, side)?.len();
                if !Self::shop(state) {
                    let before = self.scroll[side];
                    let change =
                        i16::try_from(delta).map_err(|_| "inventory scroll delta outside WORD")?;
                    if state.globals.health_context.char_select_new == self.members[side] {
                        let maximum = count.saturating_sub(11).min(30).max(before);
                        self.scroll[side] =
                            (before as i64 + i64::from(delta)).clamp(0, maximum as i64) as usize;
                    }
                    return Ok(Some(Action::Command(Command::ScrollInventory {
                        warrior: self.members[side],
                        scroll: before,
                        delta: change,
                        quick_info: self.quick_info,
                    })));
                }
                self.scroll[side] = (self.scroll[side] as i32 + delta)
                    .clamp(0, count.saturating_sub(9) as i32)
                    as usize;
            }
            Information => {
                let warrior = self.members[0];
                let row = self.display_row.max(0) as usize;
                let count = self.items(state, content, 0)?.len();
                if count == 0 {
                    return Ok(None);
                }
                self.information = Some((warrior, self.scroll[0] + row));
                self.information_item = None;
                return Ok(Some(Action::Command(Command::ShowItemInformation {
                    warrior,
                    scroll: self.scroll[0],
                    row,
                    quick_info: self.quick_info,
                })));
            }
            CloseInformation | UseList(_) | Ammunition | AmmunitionItem(_) | CloseAmmunition => {}
            Shop => return Ok(Some(Action::Command(Command::OpenShop))),
            Category(family) => {
                if family >= 5 {
                    return Err("shop category unavailable".into());
                }
                self.category = family;
                self.scroll[1] = 0;
                self.selected = None;
            }
            Done => {
                return Ok(Some(if Self::shop(state) {
                    Action::Command(Command::CloseShop)
                } else {
                    Action::Close
                }));
            }
            Refit(side) => {
                if side > 1 {
                    return Err("inventory pane unavailable".into());
                }
                return Ok(Some(Action::Command(Command::Reload {
                    warrior: self.members[side],
                })));
            }
            Transfer | Activate | Stockpile => {
                let (side, slot) = self.selected.ok_or("no inventory item selected")?;
                let item = self.record(state, content, side, slot)?;
                let warrior = self.members[side];
                let command = match action {
                    Activate if Self::shop(state) => return Ok(None),
                    Activate if centurions_core::typed_record(state, content).is_ok() => {
                        Command::UseItem { warrior, slot }
                    }
                    Transfer if Self::shop(state) => {
                        if side == 0 {
                            Command::SellItem { warrior, slot }
                        } else {
                            Command::BuyItem {
                                warrior: self.members[0],
                                stock: slot,
                            }
                        }
                    }
                    Transfer => Command::TransferItem {
                        from: warrior,
                        to: self.members[1 - side],
                        slot,
                    },
                    Stockpile if Self::shop(state) && side == 1 => Command::BuyItem {
                        warrior: self.members[0],
                        stock: slot,
                    },
                    Stockpile if item.category_code() == 26 => {
                        Command::UseAmmunitionPack { warrior, slot }
                    }
                    Stockpile => return Err(RuleError::Refused { feedback: vec![] }),
                    _ if item.equipped() != 0 => Command::UnequipItem { warrior, slot },
                    _ if (1..=20).contains(&item.category_code()) => {
                        Command::EquipItem { warrior, slot }
                    }
                    _ if item.category_code() == 26 => Command::UseAmmunitionPack { warrior, slot },
                    _ => Command::UseItem { warrior, slot },
                };
                return Ok(Some(Action::Command(command)));
            }
        }
        Ok(Some(Action::Command(self.context_command())))
    }
    /// Carries out a choice in the item information window.
    fn handle_information(&mut self, action: InventoryAction, state: &Session) -> Option<Action> {
        use centurions_core::ItemInformationList as List;
        let (member, slot) = self.information?;
        let warrior = self.members[0];
        let quick_info = self.quick_info;
        let retarget = |member, slot, list, item| {
            Some(Action::Command(Command::RetargetItemInformation {
                warrior,
                member,
                slot,
                quick_info,
                list,
                item,
            }))
        };
        let loot = self.information_item.is_some_and(|(_, loot)| loot);
        if self.use_list.take().is_some() {
            return None;
        }
        match action {
            InventoryAction::Item { .. } if loot => None,
            InventoryAction::Item { side: 0, slot } => {
                let records = state.warriors.get(warrior)?.inventory().ok()?;
                if slot >= records.len() {
                    return None;
                }
                self.information = Some((member, slot));
                if let Some((id, _)) = self.information_item.as_mut() {
                    *id = records[slot].id();
                }
                let item = self.information_item.map(|(id, _)| id);
                retarget(member, slot, List::Items, item)
            }
            InventoryAction::Item {
                side: 1,
                slot: cell,
            } if Self::shop(state) => {
                self.selected = Some((1, cell));
                if let Some((id, _)) = self.information_item.as_mut()
                    && let Ok(stock) = centurions_core::shop::stock_family(state, self.category)
                    && let Some((_, stock_id, _)) = stock.into_iter().find(|(s, ..)| *s == cell)
                {
                    *id = stock_id;
                }
                let item = self.information_item.map(|(id, _)| id);
                retarget(member, slot, List::Shop, item)
            }
            InventoryAction::Item { side: 1, slot } => {
                let member = self.members[1];
                self.information = Some((member, slot));
                self.information_item = None;
                retarget(member, slot, List::Owner, None)
            }
            InventoryAction::UseList(professions) => {
                let widened = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
                let item = state.globals.loaded_items.item.clone()?;
                self.use_list = Some((professions, widened, item));
                Some(Action::Command(Command::ListItemUsers {
                    warrior,
                    professions,
                    widened,
                }))
            }
            InventoryAction::CloseInformation => {
                self.information = None;
                self.information_item = None;
                None
            }
            _ => None,
        }
    }
    /// moveicon's option-drag opens the dialog on a shop window row instead
    /// of starting a drag.
    fn drag_information(
        &mut self,
        state: &Session,
        content: &Content,
        side: usize,
        slot: usize,
    ) -> Option<Action> {
        let warrior = self.members[0];
        let stock = side == 1;
        if stock {
            let (_, id, _) = centurions_core::shop::stock_family(state, self.category)
                .ok()?
                .into_iter()
                .find(|(cell, ..)| *cell == slot)?;
            let definition = content.items.get(id.unsigned_abs() as usize)?;
            let description = centurions_content::records::ItemDefinition(definition)
                .description_id()
                .ok()?;
            self.information = Some((warrior, 0));
            self.information_item = Some((description, false));
        } else {
            if slot >= state.warriors.get(warrior)?.inventory().ok()?.len() {
                return None;
            }
            self.information = Some((warrior, slot));
            self.information_item = None;
        }
        Some(Action::Command(Command::ShowDraggedItemInformation {
            warrior,
            slot,
            stock,
            quick_info: self.quick_info,
        }))
    }
    /// booty's option-click opens the dialog over the loot grid.
    pub fn open_loot_information(&mut self, owner: usize, id: i16) {
        self.information = Some((owner, 0));
        self.information_item = Some((id, true));
    }
    /// Whether the item information window is open over the booty screen.
    pub fn loot_information_open(&self) -> bool {
        self.information.is_some() && self.information_item.is_some_and(|(_, loot)| loot)
    }
    /// Ends a drag: an item dropped on the other warrior's list is transferred to
    /// that warrior.
    pub fn release(&mut self, pointer: Option<Vec2>) -> Option<InventoryAction> {
        let (side, slot) = self.drag.take()?;
        let pointer = pointer?;
        let destination = if pointer.x < 343. {
            Some(0)
        } else if pointer.x > 457. {
            Some(1)
        } else {
            None
        };
        if destination == Some(1 - side) && (20. ..407.).contains(&pointer.y) {
            self.selected = Some((side, slot));
            Some(InventoryAction::Transfer)
        } else {
            None
        }
    }
    /// The item record in `slot` of the warrior shown on `side`.
    pub(super) fn record(
        &self,
        state: &Session,
        content: &Content,
        side: usize,
        slot: usize,
    ) -> Result<InventoryRecord, RuleError> {
        self.items(state, content, side)?
            .into_iter()
            .find(|(index, _)| *index == slot)
            .map(|(_, item)| item)
            .ok_or_else(|| "inventory item unavailable".into())
    }
    /// Whether the shop is open.
    pub(super) fn shop(state: &Session) -> bool {
        state
            .exploration
            .as_ref()
            .is_some_and(|p| p.presentation == Some(centurions_core::Presentation::Shop))
    }
    /// The items listed on `side`: a warrior's pack, or the shop's stock on the
    /// right side in a shop.
    pub(super) fn items(
        &self,
        state: &Session,
        content: &Content,
        side: usize,
    ) -> Result<Vec<(usize, InventoryRecord)>, RuleError> {
        if side > 1 {
            return Err("inventory pane unavailable".into());
        }
        if side == 1 && Self::shop(state) {
            centurions_core::shop::stock_family(state, self.category)?
                .into_iter()
                .map(|(slot, id, _)| Ok((slot, centurions_core::shop::preview(content, id)?)))
                .collect()
        } else {
            state
                .warriors
                .get(self.members[side])
                .ok_or("inventory warrior unavailable")?
                .inventory()
                .map(|items| items.into_iter().enumerate().collect())
                .map_err(Into::into)
        }
    }
}

/// Draws an item's information panel from the plan the rules built.
pub(crate) fn draw_detail_plan(
    plan: &centurions_core::item_display::output::PanelPlan,
    assets: &Assets,
) {
    use centurions_core::item_display::output::{BodyRect, Draw, PanelEffect};
    let mut color = YELLOW;
    let mut position = vec2(0., 0.);
    let mut size = 12;
    let rectangle = |r: &BodyRect| {
        Rect::new(
            r.left as f32,
            r.top as f32 + 20.,
            (r.right - r.left) as f32,
            (r.bottom - r.top) as f32,
        )
    };
    for effect in &plan.effects {
        match effect {
            PanelEffect::Drawing(Draw::ForeColor(value)) => {
                color = match value {
                    205 => RED,
                    273 => SKYBLUE,
                    341 => GREEN,
                    409 => BLUE,
                    33 => BLACK,
                    _ => YELLOW,
                }
            }
            PanelEffect::Drawing(Draw::TextSize(value)) => size = *value as u16,
            PanelEffect::Drawing(Draw::MoveTo {
                horizontal,
                vertical,
            }) => position = vec2(*horizontal as f32, *vertical as f32 + 20.),
            PanelEffect::Drawing(Draw::DrawString(value)) => {
                let text = centurions_content::ui::pascal_text(value.raw()).unwrap_or_default();
                assets.text(&text, position.x, position.y, size, color);
                position.x += measure_text(&text, Some(&assets.font), size, 1.).width;
            }
            PanelEffect::Drawing(Draw::DrawPicture { handle, bounds }) => {
                assets.panel(*handle as i16, rectangle(bounds))
            }
            PanelEffect::Drawing(Draw::PlotColorIcon { handle, rect }) => {
                if let Some(image) = assets.icons.get(&(*handle as i16)) {
                    let r = rectangle(rect);
                    crate::assets::texture_in(image, r);
                }
            }
            PanelEffect::TextBox {
                bytes,
                rectangle: rect,
                ..
            } => {
                let bytes = &bytes[..bytes
                    .iter()
                    .position(|byte| *byte == 0)
                    .unwrap_or(bytes.len())];
                let text = centurions_content::ui::c_string(bytes);
                assets.wrapped_color(&text, rectangle(rect), size, size as f32 + 2., color);
            }
            _ => {}
        }
    }
}

mod ammunition;
mod detail;
mod information;
mod view;
