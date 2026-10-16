use crate::{
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_content::{Content, InventoryRecord, word};
use centurions_core::{Command, RuleError, Session};
use macroquad::prelude::*;
use std::{cell::RefCell, collections::BTreeMap};
type ItemRows = BTreeMap<Vec<u8>, Vec<(i32, String)>>;
type ItemDetail = (
    Vec<u8>,
    Result<centurions_core::recovered_detail_ui::DetailPlan, String>,
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
                    Stockpile if item.0[10] == 26 => Command::UseAmmunitionPack { warrior, slot },
                    Stockpile => return Err(RuleError::Refused { feedback: vec![] }),
                    _ if item.0[9] != 0 => Command::UnequipItem { warrior, slot },
                    _ if (1..=20).contains(&item.0[10]) => Command::EquipItem { warrior, slot },
                    _ if item.0[10] == 26 => Command::UseAmmunitionPack { warrior, slot },
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
    fn record(
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
    fn shop(state: &Session) -> bool {
        state
            .exploration
            .as_ref()
            .is_some_and(|p| p.presentation == Some(centurions_core::Presentation::Shop))
    }
    fn items(
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
    pub fn draw(
        &self,
        content: &Content,
        state: &Session,
        assets: &Assets,
        controls: &mut Vec<Control>,
    ) {
        controls.clear();
        draw_rectangle(0., 20., 800., 580., WHITE);
        assets.panel(170, Rect::new(343., 20., 114., 387.));
        assets.panel(148, Rect::new(80., 407., 640., 73.));
        assets.panel(227, Rect::new(0., 407., 80., 73.));
        assets.panel(227, Rect::new(720., 407., 80., 73.));
        assets.panel(226, Rect::new(0., 480., 800., 120.));
        for side in 0..2 {
            let Some(warrior) = state
                .warriors
                .get(self.members[if Self::shop(state) { 0 } else { side }])
            else {
                continue;
            };
            let x = if side == 0 { 10. } else { 470. };
            let records = self.items(state, content, side).unwrap_or_default();
            for (row, (slot, item)) in records.iter().skip(self.scroll[side]).take(9).enumerate() {
                let rect = Rect::new(x, 36. + row as f32 * 40., 325., 39.);
                if self.selected == Some((side, *slot)) {
                    draw_rectangle(
                        rect.x,
                        rect.y,
                        rect.w,
                        rect.h,
                        Color::from_rgba(220, 230, 225, 255),
                    );
                }
                if let Some(definition) = content.items.get(item.id().unsigned_abs() as usize)
                    && let Ok(id) = word(definition, 10)
                    && let Some(icon) = assets.icons.get(&id)
                {
                    draw_texture_ex(
                        icon,
                        rect.x,
                        rect.y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(32., 32.)),
                            ..Default::default()
                        },
                    );
                }
                let name = assets.names.get(&item.id()).map_or("", String::as_str);
                assets.text(name, rect.x + 42., rect.y + 17., 12, BLACK);
                if let Ok(amount) = word(&item.0, 2)
                    && amount > 0
                {
                    let mut x = rect.x + 42. + measure_text(name, Some(&assets.font), 12, 1.).width;
                    if let Ok(ammo) = word(&item.0, 4)
                        && ammo != 0
                        && let Some(definition) = content.items.get(ammo.unsigned_abs() as usize)
                        && let Ok(icon) = word(definition, 10)
                        && let Some(image) = assets.icons.get(&icon)
                    {
                        draw_texture_ex(
                            image,
                            x + 4.,
                            rect.y,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(16., 16.)),
                                ..Default::default()
                            },
                        );
                        x += 22.;
                    }
                    if let Ok(prefix) = assets.native_text.text(0x401152, true) {
                        assets.text(&format!("{prefix}{amount}"), x, rect.y + 17., 12, BLUE);
                    }
                }
                self.row_stats(content, item, assets, rect.x + 42., rect.y + 30.);
                controls.push(Control {
                    rect,
                    enabled: true,
                    action: Action::Inventory(InventoryAction::Item { side, slot: *slot }),
                });
            }
            if side == 1 && Self::shop(state) {
                continue;
            }
            let portrait = word(&warrior.bytes, 0x54).unwrap_or(0);
            if let Some(image) = assets.portraits.get(&portrait) {
                draw_texture_ex(
                    image,
                    if side == 0 { 97. } else { 455. },
                    422.,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(32., 32.)),
                        ..Default::default()
                    },
                );
            }
            assets.text(
                &warrior.name,
                if side == 0 { 151. } else { 503. },
                471.,
                12,
                YELLOW,
            );
            let edge = if side == 0 { 0. } else { 720. };
            assets.text(
                &word(&warrior.bytes, 0x9d6).unwrap_or(0).to_string(),
                edge + 8.,
                445.,
                12,
                YELLOW,
            );
            assets.text(
                &word(&warrior.bytes, 0x9d8).unwrap_or(0).to_string(),
                edge + 42.,
                445.,
                12,
                YELLOW,
            );
            assets.text(&records.len().to_string(), edge + 49., 475., 12, YELLOW);
        }
        for side in 0..2 {
            for (y, delta) in [(20., -1), (384., 1)] {
                let rect = Rect::new(if side == 0 { 349. } else { 435. }, y, 16., 20.);
                assets.panel(if delta < 0 { 179 } else { 183 }, rect);
                controls.push(Control {
                    rect,
                    enabled: true,
                    action: Action::Inventory(InventoryAction::Scroll { side, delta }),
                });
            }
        }
        for member in 0..if Self::shop(state) { 0 } else { 6 } {
            assets.panel(154, Rect::new(375., 20. + member as f32 * 50., 50., 50.));
        }
        let mos_names = assets
            .character_strings
            .get(&131)
            .and_then(|bytes| centurions_content::ui::string_list(bytes).ok())
            .unwrap_or_default();
        for (member, warrior) in state.warriors.iter().enumerate() {
            for side in 0..if Self::shop(state) { 1 } else { 2 } {
                let rect = Rect::new(
                    342. + side as f32 * 52. + (member % 3) as f32 * 24.,
                    408. + (member / 3) as f32 * 24.,
                    22.,
                    22.,
                );
                if let Some(image) = assets
                    .portraits
                    .get(&word(&warrior.bytes, 0x54).unwrap_or(0))
                {
                    draw_texture_ex(
                        image,
                        rect.x,
                        rect.y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(rect.w, rect.h)),
                            ..Default::default()
                        },
                    );
                }
                controls.push(Control {
                    rect,
                    enabled: true,
                    action: Action::Inventory(InventoryAction::Warrior { side, member }),
                });
            }
            if Self::shop(state) {
                continue;
            }
            let y = 20. + member as f32 * 50.;
            if let Some(image) = assets
                .portraits
                .get(&word(&warrior.bytes, 0x54).unwrap_or(0))
            {
                draw_texture_ex(
                    image,
                    383.,
                    y + 8.,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(32., 32.)),
                        ..Default::default()
                    },
                );
            }
            if let Ok(mos) = word(&warrior.bytes, 0x34)
                && let Some(name) = mos_names.get(mos.wrapping_sub(1) as usize)
            {
                let name: String = name.chars().take(7).collect();
                assets.text(&name, 380., y + 47., 10, YELLOW);
            }
            for side in 0..if Self::shop(state) { 1 } else { 2 } {
                controls.push(Control {
                    rect: Rect::new(375. + side as f32 * 25., y, 25., 50.),
                    enabled: true,
                    action: Action::Inventory(InventoryAction::Warrior { side, member }),
                });
            }
        }
        if Self::shop(state) {
            assets.text(&state.party_currency.to_string(), 507., 446., 12, YELLOW);
            if let Some((side, slot)) = self.selected
                && let Ok(item) = self.record(state, content, side, slot)
                && let Ok(price) =
                    centurions_core::shop::price(state, content, item.id(), side == 0)
            {
                assets.text(
                    &price.to_string(),
                    if side == 0 { 241. } else { 188. },
                    447.,
                    12,
                    YELLOW,
                );
            }
            // CNTL 131, 130, 132, 134, 135 over the authored PICT 170 categories.
            for (family, (y, height)) in [
                (20., 50.),
                (69., 50.),
                (120., 137.),
                (257., 49.),
                (307., 49.),
            ]
            .into_iter()
            .enumerate()
            {
                controls.push(Control {
                    rect: Rect::new(375., y, 50., height),
                    enabled: true,
                    action: Action::Inventory(InventoryAction::Category(family)),
                });
            }
        } else {
            assets.panel(202, Rect::new(183., 411., 102., 40.));
            controls.push(Control {
                rect: Rect::new(183., 411., 102., 40.),
                enabled: state
                    .shop_access
                    .as_ref()
                    .is_some_and(|a| a.level == state.scenario_level),
                action: Action::Inventory(InventoryAction::Shop),
            });
        }
        assets.panel(175, Rect::new(375., 320., 50., 37.));
        for (rect, action) in [
            (Rect::new(663., 412., 50., 39.), InventoryAction::Done),
            (Rect::new(375., 357., 50., 25.), InventoryAction::Refit(0)),
            (Rect::new(375., 382., 50., 25.), InventoryAction::Refit(1)),
            (Rect::new(375., 320., 50., 37.), InventoryAction::Stockpile),
        ] {
            controls.push(Control {
                rect,
                enabled: if Self::shop(state) && matches!(action, InventoryAction::Refit(1)) {
                    false
                } else {
                    self.selected.is_some()
                        || matches!(action, InventoryAction::Done | InventoryAction::Refit(_))
                },
                action: Action::Inventory(action),
            });
        }
        if let Some((side, slot)) = self.selected
            && let Ok(item) = self.record(state, content, side, slot)
            && let Some(map) = content.map(
                state
                    .exploration
                    .as_ref()
                    .map_or(0, |position| position.map),
            )
        {
            self.detail(content, map, &item, side == 1 && Self::shop(state), assets);
        }
    }
    fn detail(
        &self,
        content: &Content,
        map: &centurions_content::mission::OpeningMission,
        item: &InventoryRecord,
        stock: bool,
        assets: &Assets,
    ) {
        use centurions_core::{
            recovered_detail_ui::DetailEffect, recovered_drawing::DrawingEffect,
        };
        let mut key = item.0.to_vec();
        key.push(u8::from(stock));
        for id in [item.id(), word(&item.0, 4).unwrap_or(0)] {
            if let Some(definition) = content.items.get(id.unsigned_abs() as usize) {
                key.extend(definition);
            }
        }
        if let Ok(description) = map.item_description(item.id()) {
            key.extend(description.as_bytes());
        }
        let mut cached = self.detail.borrow_mut();
        if cached.as_ref().is_none_or(|(old, _)| old != &key) {
            let plan = centurions_core::item_display::detail(
                content,
                map,
                item,
                stock,
                &assets.native_text,
                &assets.character_strings,
            )
            .map_err(|error| {
                eprintln!("inventory detail: {error}");
                error.to_string()
            });
            *cached = Some((key, plan));
        }
        let Some((_, Ok(plan))) = cached.as_ref() else {
            return;
        };
        let mut color = YELLOW;
        let mut position = vec2(0., 0.);
        let mut size = 12;
        let rectangle = |r: &centurions_core::recovered_geometry::BodyRect| {
            Rect::new(
                r.left as f32,
                r.top as f32 + 20.,
                (r.right - r.left) as f32,
                (r.bottom - r.top) as f32,
            )
        };
        for effect in &plan.effects {
            match effect {
                DetailEffect::Drawing(DrawingEffect::ForeColor(value)) => {
                    color = match value {
                        205 => RED,
                        273 => SKYBLUE,
                        341 => GREEN,
                        409 => BLUE,
                        33 => BLACK,
                        _ => YELLOW,
                    }
                }
                DetailEffect::Drawing(DrawingEffect::TextSize(value)) => size = *value as u16,
                DetailEffect::Drawing(DrawingEffect::MoveTo {
                    horizontal,
                    vertical,
                }) => position = vec2(*horizontal as f32, *vertical as f32 + 20.),
                DetailEffect::Drawing(DrawingEffect::DrawString(value)) => {
                    let text = centurions_content::ui::pascal_text(value.raw()).unwrap_or_default();
                    assets.text(&text, position.x, position.y, size, color);
                    position.x += measure_text(&text, Some(&assets.font), size, 1.).width;
                }
                DetailEffect::Drawing(DrawingEffect::DrawPicture { handle, bounds }) => {
                    assets.panel(*handle as i16, rectangle(bounds))
                }
                DetailEffect::Drawing(DrawingEffect::PlotColorIcon { handle, rect }) => {
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
                DetailEffect::TextBox {
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
    fn row_stats(
        &self,
        content: &Content,
        item: &InventoryRecord,
        assets: &Assets,
        mut x: f32,
        y: f32,
    ) {
        let mut key = item.0.to_vec();
        for id in [item.id(), word(&item.0, 4).unwrap_or(0)] {
            if let Some(definition) = content.items.get(id.unsigned_abs() as usize) {
                key.extend(definition);
            }
        }
        let mut rows = self.rows.borrow_mut();
        let fragments = rows.entry(key).or_insert_with(|| {
            use centurions_core::{
                recovered_drawing::DrawingEffect, recovered_item_ui::ItemUiEffect,
            };
            match centurions_core::item_display::row(
                content,
                item,
                &assets.native_text,
                &assets.character_strings,
            ) {
                Ok(effects) => {
                    let mut color = 33;
                    let mut fragments: Vec<(i32, String)> = vec![];
                    for effect in effects {
                        match effect {
                            ItemUiEffect::Drawing(DrawingEffect::ForeColor(value)) => color = value,
                            ItemUiEffect::Drawing(DrawingEffect::DrawString(value)) => {
                                let text = centurions_content::ui::pascal_text(value.raw())
                                    .unwrap_or_default();
                                if let Some(last) =
                                    fragments.last_mut().filter(|last| last.0 == color)
                                {
                                    last.1.push_str(&text);
                                } else {
                                    fragments.push((color, text));
                                }
                            }
                            _ => {}
                        }
                    }
                    fragments
                }
                Err(error) => {
                    eprintln!("inventory row: {error}");
                    vec![]
                }
            }
        });
        for (color, text) in fragments {
            assets.text(
                text,
                x,
                y,
                10,
                match color {
                    205 => RED,
                    409 => BLUE,
                    341 => GREEN,
                    _ => BLACK,
                },
            );
            x += measure_text(text, Some(&assets.font), 10, 1.).width;
        }
    }
}
