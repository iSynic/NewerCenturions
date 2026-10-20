//! PICT153 and the normal-combat hit rectangles in combat114378.
use crate::{
    input,
    presentation::{Action, Control, Inspection, UiState},
};
use centurions_core::{Command, Session};
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum ViewAction {
    Center,
    Next,
    Previous,
    Move,
    Reveal,
}

#[derive(Default)]
pub struct View {
    pub body: Option<i16>,
    turn: Option<(i16, usize)>,
    point: Option<(i16, i16)>,
    pub reveal: bool,
    pub move_target: bool,
}

impl View {
    pub fn selected(&self, state: &Session) -> Option<i16> {
        let c = state.combat.as_ref()?;
        if self.turn == Some((c.round, c.cursor)) {
            self.body.or(c.active_body())
        } else {
            c.active_body()
        }
    }

    pub fn pan(&mut self, state: &Session, point: (i16, i16)) {
        if let Some(c) = &state.combat {
            self.turn = Some((c.round, c.cursor));
            self.point = Some(point);
            self.move_target = false;
        }
    }

    pub fn change(&mut self, state: &Session, action: ViewAction) {
        let Some(c) = &state.combat else { return };
        if self.turn != Some((c.round, c.cursor)) {
            *self = Self::default();
        }
        self.turn = Some((c.round, c.cursor));
        self.point = None;
        match action {
            ViewAction::Center => {
                self.body = c.active_body();
                self.point = None;
                self.move_target = false;
            }
            ViewAction::Move => {
                self.body = c.active_body();
                self.move_target = true;
            }
            ViewAction::Reveal => self.reveal = !self.reveal,
            ViewAction::Next | ViewAction::Previous => {
                let step = if matches!(action, ViewAction::Next) {
                    1
                } else {
                    -1
                };
                let mut index = self
                    .body
                    .and_then(|body| c.order.iter().position(|id| *id == body))
                    .unwrap_or(c.cursor);
                // centerstage110644 wraps slots1..109; bounded even if no body remains.
                for _ in 0..109 {
                    index = (index as i32 - 1 + step).rem_euclid(109) as usize + 1;
                    let body = c.order[index];
                    let present = if (0..state.warriors.len() as i16).contains(&body) {
                        state.warriors[body as usize].stamina().unwrap_or(0) > 0
                    } else {
                        usize::try_from(body - 10)
                            .ok()
                            .and_then(|i| c.enemies.records.get(i))
                            .is_some_and(|m| m.stamina().unwrap_or(0) > 0)
                    };
                    if present {
                        self.body = Some(body);
                        break;
                    }
                }
            }
        }
    }
}

pub fn origin(state: &Session, ui: &UiState) -> Option<(i16, i16)> {
    let c = state.combat.as_ref()?;
    let body = ui.combat_view.selected(state)?;
    let point = if ui.combat_view.turn == Some((c.round, c.cursor))
        && let Some(point) = ui.combat_view.point
    {
        point
    } else if body < 10 {
        *c.squad.positions.get(body as usize)?
    } else {
        *c.enemies.positions.get((body - 10) as usize)?
    };
    let shift = (c.map.shift.0 + 5, c.map.shift.1 + 5);
    Some((
        (point.0 - 7).clamp(-shift.0, 75 - shift.0),
        (point.1 - 6).clamp(-shift.1, 77 - shift.1),
    ))
}

pub fn buttons(
    state: &Session,
    content: &centurions_content::Content,
    selected: usize,
) -> Vec<Control> {
    let actor = state
        .combat
        .as_ref()
        .and_then(|c| c.active_body())
        .filter(|i| (0..state.warriors.len() as i16).contains(i))
        .map_or(selected, |i| i as usize);
    let firearm = state.warriors[actor].bytes[0x9dd] != 0;
    let entries = [
        (
            Rect::new(526., 456., 46., 15.),
            Action::Command(Command::Guard),
        ),
        (
            Rect::new(579., 456., 46., 15.),
            Action::Command(Command::DelayTurn),
        ),
        (
            Rect::new(632., 456., 47., 15.),
            Action::Command(Command::EndTurn),
        ),
        (
            Rect::new(684., 456., 41., 15.),
            Action::Command(Command::SetFirearmMode {
                warrior: actor,
                enabled: !firearm,
            }),
        ),
        (
            Rect::new(526., 476., 99., 15.),
            Action::CombatView(ViewAction::Reveal),
        ),
        (
            Rect::new(632., 476., 46., 15.),
            Action::CombatView(ViewAction::Move),
        ),
        (
            Rect::new(684., 476., 41., 15.),
            Action::Open(Inspection::FireTargets),
        ),
        (
            Rect::new(526., 496., 51., 15.),
            Action::Command(Command::RequestCombatEscape),
        ),
        (
            Rect::new(584., 496., 41., 15.),
            Action::Command(Command::UndoCombatMove),
        ),
        (
            Rect::new(632., 496., 47., 15.),
            Action::CombatView(ViewAction::Center),
        ),
        (
            Rect::new(526., 516., 57., 15.),
            Action::Open(Inspection::Bandage),
        ),
        (
            Rect::new(590., 516., 35., 15.),
            Action::CombatView(ViewAction::Next),
        ),
        (
            Rect::new(632., 516., 47., 15.),
            Action::CombatView(ViewAction::Previous),
        ),
        (
            Rect::new(468., 449., 50., 39.),
            Action::Command(Command::BeginAutomaticTurn),
        ),
        (
            Rect::new(468., 488., 50., 50.),
            Action::Open(Inspection::Items),
        ),
        (
            Rect::new(468., 538., 50., 50.),
            Action::Open(Inspection::Psi),
        ),
        (
            Rect::new(518., 538., 50., 50.),
            Action::Command(Command::BeginCapture),
        ),
        (
            Rect::new(45., 526., 16., 34.),
            Action::Command(Command::SetFirearmMode {
                warrior: actor,
                enabled: !firearm,
            }),
        ),
        (
            Rect::new(78., 546., 45., 16.),
            input::refill_action(state, content, actor),
        ),
        (
            Rect::new(125., 546., 178., 16.),
            Action::Open(Inspection::Weapons),
        ),
        (
            Rect::new(7., 455., 66., 66.),
            Action::Application(crate::application::ApplicationAction::CombatInformation(
                state
                    .combat
                    .as_ref()
                    .and_then(|c| c.active_body())
                    .unwrap_or(actor as i16),
            )),
        ),
    ];
    entries
        .into_iter()
        .map(|(rect, action)| Control {
            rect,
            enabled: input::available(state, actor, &action),
            action,
        })
        .collect()
}

pub fn shortcut(
    key: KeyCode,
    state: &Session,
    content: &centurions_content::Content,
    selected: usize,
) -> Option<Action> {
    let index = match key {
        KeyCode::G => 0,
        KeyCode::D => 1,
        KeyCode::F | KeyCode::Space => 2,
        KeyCode::W => 3,
        KeyCode::R => 4,
        KeyCode::M => 5,
        KeyCode::T => 6,
        KeyCode::E => 7,
        KeyCode::U => 8,
        KeyCode::C => 9,
        KeyCode::B => 10,
        KeyCode::N => 11,
        KeyCode::P => 12,
        KeyCode::A => 13,
        KeyCode::I => 14,
        KeyCode::S => 15,
        KeyCode::L => 18,
        _ => return None,
    };
    buttons(state, content, selected)
        .get(index)
        .map(|c| c.action.clone())
}

/// Original CNTL190..195 retain their own automation flags between turns.
pub fn draw_automatic(
    state: &Session,
    assets: &crate::assets::Assets,
    controls: &mut Vec<Control>,
) {
    let Some(c) = &state.combat else { return };
    for member in 0..state.warriors.len() {
        let rect = assets.main_controls[&(190 + member as i16)];
        assets.panel(if c.automatic[member] { 179 } else { 180 }, rect);
        let action = Action::Command(Command::ToggleCombatAuto { warrior: member });
        controls.push(Control {
            rect,
            enabled: input::available(state, member, &action),
            action,
        });
    }
}

pub fn draw_weapons(
    state: &Session,
    assets: &crate::assets::Assets,
    ui: &UiState,
    controls: &mut Vec<Control>,
) {
    let Ok(member) = centurions_core::combat::controls::actor(state) else {
        return;
    };
    let Ok(slots) = centurions_core::combat::controls::weapons(state) else {
        return;
    };
    let inventory = state.warriors[member].inventory().unwrap_or_default();
    let mut rows = Vec::new();
    let mut category = None;
    for slot in slots.into_iter().skip(ui.page).take(12) {
        let item = &inventory[slot];
        let kind = (item.category_code() as i8).unsigned_abs();
        if category != Some(kind) {
            let address = if kind == 15 { 0x403d28 } else { 0x403cfc };
            rows.push((
                assets.native_text.text(address, true).unwrap_or_default(),
                None,
                false,
            ));
            category = Some(kind);
        }
        let amount = i16::from_be_bytes([item.0[2], item.0[3]]);
        let mut label = assets.name(item.id());
        if kind == 15 || amount > 0 {
            let prefix = assets
                .native_text
                .text(if kind == 15 { 0x403d22 } else { 0x403cfa }, false)
                .unwrap_or_default();
            let suffix = assets
                .native_text
                .text(if kind == 15 { 0x403d1c } else { 0x403cf4 }, false)
                .unwrap_or_default();
            label.push_str(&format!("{prefix}{amount}{suffix}"));
        }
        rows.push((label, Some(slot), item.equipment_flag() != 0));
    }
    let width = rows
        .iter()
        .map(|(label, _, _)| measure_text(label, Some(&assets.menu_font), 12, 1.).width)
        .fold(180., f32::max)
        + 30.;
    let height = rows.len() as f32 * 18. + 8.;
    let rect = Rect::new(
        (726. - width).max(0.),
        (456. - height).max(20.),
        width,
        height,
    );
    draw_rectangle(rect.x + 3., rect.y + 3., rect.w, rect.h, BLACK);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::from_rgba(238, 238, 238, 255),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1., BLACK);
    for (row, (label, slot, checked)) in rows.iter().enumerate() {
        let row_rect = Rect::new(
            rect.x + 1.,
            rect.y + 4. + row as f32 * 18.,
            rect.w - 2.,
            18.,
        );
        assets.menu_text(
            label,
            row_rect.x + 23.,
            row_rect.y + 13.,
            12,
            if slot.is_some() { BLACK } else { GRAY },
        );
        if *checked {
            assets.menu_text("✓", row_rect.x + 5., row_rect.y + 13., 12, BLACK);
        }
        if let Some(slot) = slot {
            controls.push(Control {
                rect: row_rect,
                enabled: true,
                action: Action::Command(Command::SelectCombatWeapon { slot: *slot }),
            });
        }
    }
}

/// Converts an in-map logical point using the same visible origin as targeting.
pub fn map_position(state: &Session, ui: &UiState, pointer: Vec2) -> Option<(i16, i16)> {
    let (x, y) = origin(state, ui)?;
    Some((
        x + ((pointer.x - crate::layout::MAP.x) / 32.) as i16,
        y + ((pointer.y - crate::layout::MAP.y) / 32.) as i16,
    ))
}
