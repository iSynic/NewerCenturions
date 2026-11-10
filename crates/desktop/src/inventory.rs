use crate::{assets::Assets, presentation::Action};
use centurions_content::{Content, InventoryRecord};
use centurions_core::{Command, RuleError, Session};
pub use items_choice::RowGesture;
use macroquad::prelude::*;
use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
};
/// The item list rows already built for each item record and its definition.
type ItemRows = BTreeMap<Vec<u8>, Vec<(i32, String)>>;
/// The item information panel already built for an item record.
type ItemDetail = (
    Vec<u8>,
    Result<centurions_core::item_display::output::PanelPlan, String>,
);

/// The longest gap between two clicks that make a double-click, in seconds.
const DOUBLE_CLICK_SECONDS: f64 = 0.35;
/// The pause between repeats of the worn-item warning while the button stays down, in seconds.
const WARNING_REPEAT_SECONDS: f64 = 20. / 60.;

/// Whether the Items window is open: one warrior's own pack, not the trade or shop window and
/// not a crate or a battle's booty.
///
/// Original: items
pub fn in_items_window(state: &Session, ui: &crate::presentation::UiState) -> bool {
    ui.inspection == Some(crate::presentation::Inspection::Items)
        && ui.inventory.items_window(state)
        && !crate::collection::active(state, ui)
}

/// Whether warnings sit where the Items window moves them: from its opening until Done or Trade
/// leaves it, even when Trade is then refused.
///
/// Original: items, warn
pub fn in_items(state: &Session, ui: &crate::presentation::UiState) -> bool {
    in_items_window(state, ui) && ui.inventory.initems
}

/// Whether the trade window is open: two warriors' packs side by side away from a shop.
///
/// Original: seeshop, Showitems
pub fn in_trade(state: &Session, ui: &crate::presentation::UiState) -> bool {
    ui.inspection == Some(crate::presentation::Inspection::Items)
        && ui.inventory.trade
        && !Inventory::shop(state)
        && !crate::collection::active(state, ui)
}

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
    /// A command-click on an item row wears the item or takes it off.
    Wear {
        side: usize,
        slot: usize,
    },
    Refit(usize),
    Stockpile,
    Shop,
    Category(usize),
    Done,
    /// The Items window's View button (CNTL 140) opens showiteminfo on the highlighted row.
    Information,
    /// The trade window's item button (CNTL 133) leaves for the Items window.
    ItemsWindow,
    /// The field's Trade button opens the trade window.
    OpenTrade,
    /// The Items window's rows (CNTL 162), before the click is placed on a row.
    List,
    /// A click placed on the Items window's rows.
    Row {
        point: Vec2,
        gesture: RowGesture,
    },
    /// An arrow key moves the Items window's highlight by a row.
    Step(i16),
    /// The space bar wears or removes the highlighted item.
    Toggle,
    /// The Items window's Use, Weapon, Drop, Join, Split, Trade and portrait buttons.
    Use,
    Weapon,
    Drop,
    Join,
    Split,
    Trade,
    View,
    CloseInformation,
    /// DLOG 149 item 1 (squad users) or item 2 (warrior item types).
    UseList(bool),
    /// CNTL 176 opens loadammoproc's MENU 1000 popup on the display row.
    Ammunition,
    /// A MENU 1000 item, numbered from 1.
    AmmunitionItem(u16),
    CloseAmmunition,
    /// The combat Load control opens loadammoproc's popup for the warrior.
    CombatLoad(usize),
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
    /// The warning a press on a worn item has just earned, until the screen shows it.
    warning: Option<i16>,
    /// When a held press on a worn item next earns its warning (the large view warns only for a
    /// held press).
    held: Option<f64>,
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
    /// Whether the trade window shows rather than the Items window.
    pub trade: bool,
    /// Whether the next opening shows the trade window.
    trade_request: bool,
    /// Whether the Items window still counts as open for placing warnings.
    initems: bool,
    /// The display rows the highlight has left, which keep cicn 173 at their left.
    unhighlighted: u16,
    /// Where the last row click landed, for a double-click.
    last_point: Vec2,
    /// The sounds the last choice made.
    sounds: Vec<i16>,
    /// Actions the last choice leaves for the frames after it.
    follow_up: VecDeque<Action>,
    /// Preference item 12, Drop Item Protection.
    pub drop_protection: bool,
    /// The item count when Drop was pressed, until the drop's result is seen.
    dropped: Option<usize>,
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
            warning: None,
            held: None,
            last_click: -1.,
            category: 0,
            rows: RefCell::new(BTreeMap::new()),
            detail: RefCell::new(None),
            information: None,
            information_item: None,
            information_fields: RefCell::new(None),
            use_list: None,
            ammunition: None,
            trade: false,
            trade_request: false,
            initems: false,
            unhighlighted: 0,
            last_point: Vec2::ZERO,
            sounds: vec![],
            follow_up: VecDeque::new(),
            drop_protection: false,
            dropped: None,
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
        self.unhighlighted = 0;
        self.trade = std::mem::take(&mut self.trade_request);
        self.initems = !self.trade;
    }
    /// Opens the trade window, two warriors' packs side by side, on warrior `selected` and the next
    /// one of `count`.
    ///
    /// Original: seeshop
    pub fn open_trade(&mut self, selected: usize, count: usize) {
        self.trade_request = true;
        self.open(selected, count);
    }
    /// Whether the next opening shows the trade window.
    pub fn trade_requested(&self) -> bool {
        self.trade_request
    }
    /// Whether the item information window or the ammunition popup is open over the window.
    pub fn popup_open(&self) -> bool {
        self.information.is_some() || self.ammunition.is_some()
    }
    /// The sounds the last choice made, for the game to play.
    pub fn take_sounds(&mut self) -> Vec<i16> {
        std::mem::take(&mut self.sounds)
    }
    /// The next action a choice left for a later frame.
    pub fn take_follow_up(&mut self) -> Option<Action> {
        self.follow_up.pop_front()
    }
    /// After Drop, a dropped item that leaves the highlight past the last item on a row below the
    /// first moves the highlight up a row; an empty slot puts the highlight and the scrolling back
    /// at the top.
    ///
    /// Original: items
    pub fn observe(&mut self, events: &[centurions_core::Event]) {
        use centurions_core::{Event, feedback::GameFeedback};
        let Some(count) = self.dropped.take() else {
            return;
        };
        if events.contains(&Event::Sound { resource_id: 655 }) {
            if self.scroll[0] + self.display_row.max(0) as usize + 1 == count
                && self.display_row > 0
            {
                self.unhighlighted |= 1 << self.display_row;
                self.display_row -= 1;
            }
        } else if count != 0
            && !events
                .iter()
                .any(|event| matches!(event, Event::Feedback(GameFeedback::Warning { .. })))
        {
            self.unhighlighted |= 1 << self.display_row.clamp(0, items_window::ROWS as i16 - 1);
            self.display_row = 0;
            self.scroll[0] = 0;
        }
    }
    /// The highlighted row once the rows are redrawn: a row past the last item moves to the last
    /// item, or to the top row when there is none.
    ///
    /// Original: updateitems
    fn settled_row(&self, count: usize) -> i16 {
        if count as i16 <= self.display_row {
            (count as i16 - 1).max(0)
        } else {
            self.display_row
        }
    }
    /// Carries out a choice on the Items screen, returning the action for the game
    /// to take, if any. A second-owner portrait gives warning 158 while a limited stock is being
    /// taken.
    ///
    /// Original: choice
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
        if self.ammunition.is_some() || matches!(action, Ammunition | CombatLoad(_)) {
            return Ok(self.handle_ammunition(action, state, content));
        }
        match action {
            OpenTrade => {
                self.trade_request = true;
                return Ok(Some(Action::Open(crate::presentation::Inspection::Items)));
            }
            ItemsWindow => return Ok(self.leave_for_items(state)),
            _ => {}
        }
        if self.items_window(state) {
            let count = self.items(state, content, 0)?.len();
            let settled = self.settled_row(count);
            if settled != self.display_row {
                self.unhighlighted |= 1 << self.display_row.clamp(0, 10);
                self.display_row = settled;
            }
            return self.handle_items(action, state, content, now);
        }
        match action {
            Warrior { side, member } => {
                if side > 1 || member >= state.warriors.len() {
                    return Err("inventory warrior outside squad".into());
                }
                // The second owner cannot change while a limited stock is being taken.
                if side == 1 && state.globals.limited_stock != 0 {
                    self.warning = Some(158);
                    return Ok(None);
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
                let item = self.record(state, content, side, slot)?;
                let activate = self.selected == Some((side, slot))
                    && now - self.last_click < DOUBLE_CLICK_SECONDS;
                // A worn item is held in place unless the Option key starts the move.
                let worn = (side == 0 || !Self::shop(state)) && item.equipped() != 0;
                self.selected = Some((side, slot));
                self.display_row = slot.saturating_sub(self.scroll[side]) as i16;
                self.drag = (!worn).then_some((side, slot));
                self.last_click = now;
                if activate {
                    return self.handle(Activate, state, content, now);
                }
                if worn {
                    self.warn_worn(now);
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
            Information
            | CloseInformation
            | UseList(_)
            | Ammunition
            | AmmunitionItem(_)
            | CloseAmmunition
            | CombatLoad(_)
            | ItemsWindow
            | OpenTrade
            | List
            | Row { .. }
            | Step(_)
            | Toggle
            | Use
            | Weapon
            | Drop
            | Join
            | Split
            | Trade
            | View => {}
            Wear { side, slot } => {
                if side > 1 || (side == 1 && Self::shop(state)) {
                    return Ok(None);
                }
                let item = self.record(state, content, side, slot)?;
                self.selected = Some((side, slot));
                self.display_row = slot.saturating_sub(self.scroll[side]) as i16;
                let warrior = self.members[side];
                return Ok(Some(Action::Command(if item.equipped() != 0 {
                    Command::UnequipItem { warrior, slot }
                } else {
                    Command::EquipItem { warrior, slot }
                })));
            }
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
                    Activate if Self::shop(state) && side == 1 => return Ok(None),
                    Activate if Self::shop(state) => {
                        if item.equipped() != 0 {
                            Command::UnequipItem { warrior, slot }
                        } else {
                            Command::EquipItem { warrior, slot }
                        }
                    }
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
        self.held = None;
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
    /// A press on a worn item starts no move; it earns warning 60 at once, or in the large view
    /// once the button has been held for a double-click time.
    ///
    /// Original: choice
    fn warn_worn(&mut self, now: f64) {
        if self.quick_info {
            self.held = Some(now + DOUBLE_CLICK_SECONDS);
        } else {
            self.warning = Some(60);
        }
    }
    /// The warning the last choice earned, if it has not been shown yet.
    pub fn take_warning(&mut self) -> Option<i16> {
        self.warning.take()
    }
    /// Whether the button held on a worn item has earned another warning 60: one after the
    /// double-click time, then one every twenty ticks until the button is let go.
    ///
    /// Original: choice
    pub fn held_warning_due(&mut self, now: f64, pressed: bool) -> bool {
        let Some(due) = self.held else {
            return false;
        };
        if !pressed {
            self.held = None;
            return false;
        }
        if now < due {
            return false;
        }
        self.held = Some(now + WARNING_REPEAT_SECONDS);
        true
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
                let leading = crate::assets::line_height(size);
                assets.wrapped_color(&text, rectangle(rect), size, leading, color);
            }
            _ => {}
        }
    }
}

mod ammunition;
mod detail;
mod information;
mod items_choice;
mod items_window;
mod portraits;
mod view;
