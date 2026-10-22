use crate::{assets::Assets, presentation::Action};
use centurions_content::{Content, InventoryRecord};
use centurions_core::{Command, RuleError, Session};
use macroquad::prelude::*;
use std::{cell::RefCell, collections::BTreeMap};
type ItemRows = BTreeMap<Vec<u8>, Vec<(i32, String)>>;
type ItemDetail = (
    Vec<u8>,
    Result<centurions_core::item_display::output::PanelPlan, String>,
);

#[derive(Clone, Debug)]
pub enum InventoryAction {
    Warrior { side: usize, member: usize },
    Item { side: usize, slot: usize },
    Scroll { side: usize, delta: i32 },
    Transfer,
    Activate,
    Refit(usize),
    Stockpile,
    Shop,
    Category(usize),
    Done,
}

pub struct Inventory {
    pub members: [usize; 2],
    pub scroll: [usize; 2],
    pub selected: Option<(usize, usize)>,
    drag: Option<(usize, usize)>,
    last_click: f64,
    category: usize,
    rows: RefCell<ItemRows>,
    detail: RefCell<Option<ItemDetail>>,
}
impl Default for Inventory {
    fn default() -> Self {
        Self {
            members: [0, 1],
            scroll: [0; 2],
            selected: None,
            drag: None,
            last_click: -1.,
            category: 0,
            rows: RefCell::new(BTreeMap::new()),
            detail: RefCell::new(None),
        }
    }
}
impl Inventory {
    pub fn context_command(&self) -> Command {
        Command::InspectInventory {
            members: self.members,
            scroll: self.scroll,
            selected: self.selected,
            shop_family: self.category,
        }
    }
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
        self.category = 0;
        self.selected = None;
        self.drag = None;
        self.rows.borrow_mut().clear();
        self.detail.borrow_mut().take();
    }
    pub fn handle(
        &mut self,
        action: InventoryAction,
        state: &Session,
        content: &Content,
        now: f64,
    ) -> Result<Option<Action>, RuleError> {
        use InventoryAction::*;
        match action {
            Warrior { side, member } => {
                if side > 1 || member >= state.warriors.len() {
                    return Err("inventory warrior outside squad".into());
                }
                self.members[side] = member;
                self.scroll[side] = 0;
                self.selected = None;
            }
            Item { side, slot } => {
                self.record(state, content, side, slot)?;
                let activate = self.selected == Some((side, slot)) && now - self.last_click < 0.35;
                self.selected = Some((side, slot));
                self.drag = Some((side, slot));
                self.last_click = now;
                if activate {
                    return self.handle(Activate, state, content, now);
                }
            }
            Scroll { side, delta } => {
                if side > 1 {
                    return Err("inventory pane unavailable".into());
                }
                let count = self.items(state, content, side)?.len();
                self.scroll[side] = (self.scroll[side] as i32 + delta)
                    .clamp(0, count.saturating_sub(9) as i32)
                    as usize;
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
                    _ if item.equipment_flag() != 0 => Command::UnequipItem { warrior, slot },
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
    pub(super) fn shop(state: &Session) -> bool {
        state
            .exploration
            .as_ref()
            .is_some_and(|p| p.presentation == Some(centurions_core::Presentation::Shop))
    }
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
            centurions_core::shop::stock(state)?
                .into_iter()
                .filter(|(_, id, _)| id.unsigned_abs() as usize / 200 == self.category)
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
                    draw_texture_ex(
                        image,
                        r.x,
                        r.y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(r.w, r.h)),
                            ..Default::default()
                        },
                    );
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

mod detail;
mod view;
