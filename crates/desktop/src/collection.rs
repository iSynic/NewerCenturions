//! booty105f6c: seven columns, four rows, party recipients and quickinfo.
use crate::{
    assets::Assets,
    presentation::{Action, Control, Inspection, UiState},
};
use centurions_content::{Content, InventoryRecord, word};
use centurions_core::{Command, Presentation, Session};
use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub enum CollectionAction {
    Item(usize),
    Recipient(usize),
    Store(usize),
    DropMenu(usize),
    DropItem(usize),
    CloseDrop,
}

pub fn active(state: &Session, ui: &UiState) -> bool {
    ui.inspection == Some(Inspection::Crate)
        || state
            .exploration
            .as_ref()
            .is_some_and(|position| position.presentation == Some(Presentation::Loot))
}

fn records<'a>(state: &'a Session, ui: &UiState) -> &'a [InventoryRecord] {
    if ui.inspection == Some(Inspection::Crate) {
        &state.crate_items
    } else {
        &state.booty_items
    }
}

pub fn resolve(action: CollectionAction, state: &Session, ui: &mut UiState) -> Option<Action> {
    match action {
        CollectionAction::DropMenu(member) => {
            ui.collection_drop = Some(member);
            ui.collection_drop_scroll = 0;
            None
        }
        CollectionAction::DropItem(slot) => ui
            .collection_drop
            .take()
            .map(|member| Action::Drop(member, slot)),
        CollectionAction::CloseDrop => {
            ui.collection_drop = None;
            None
        }
        CollectionAction::Item(slot) => {
            ui.collection_slot = Some(slot);
            ui.collection_drag = true;
            None
        }
        CollectionAction::Recipient(warrior) => {
            ui.selected = warrior;
            ui.collection_drag = false;
            let slot = ui.collection_slot?;
            records(state, ui).get(slot).filter(|item| item.id() != 0)?;
            Some(Action::Command(
                if ui.inspection == Some(Inspection::Crate) {
                    Command::TakeCrate { slot, warrior }
                } else {
                    Command::TakeLoot { slot, warrior }
                },
            ))
        }
        CollectionAction::Store(slot) => {
            ui.collection_drag = false;
            (ui.inspection != Some(Inspection::Crate))
                .then_some(Action::Command(Command::StoreLoot { slot }))
        }
    }
}

pub fn input(state: &Session, ui: &mut UiState, pointer: Option<Vec2>) -> Option<Action> {
    if let Some(member) = ui.collection_drop {
        let count = state
            .warriors
            .get(member)
            .and_then(|w| w.inventory().ok())
            .map_or(0, |items| items.len());
        let delta = -mouse_wheel().1.signum() as i32;
        ui.collection_drop_scroll = (ui.collection_drop_scroll as i32 + delta)
            .clamp(0, count.saturating_sub(24) as i32) as usize;
        return is_key_pressed(KeyCode::Escape)
            .then_some(Action::Collection(CollectionAction::CloseDrop));
    }
    let count = records(state, ui)
        .iter()
        .filter(|item| item.id() != 0)
        .count();
    let pages = count.saturating_sub(28).div_ceil(7);
    let wheel = -mouse_wheel().1.signum() as i32;
    ui.page = (ui.page as i32 + wheel).clamp(0, pages as i32) as usize;
    if ui.collection_drag && is_mouse_button_released(MouseButton::Left) {
        ui.collection_drag = false;
        if let Some(point) = pointer.filter(|p| p.x >= 480. && p.y >= 20.) {
            let warrior = ((point.y - 20.) / 50.) as usize;
            if warrior < state.warriors.len() {
                return Some(Action::Collection(CollectionAction::Recipient(warrior)));
            }
        }
    }
    if is_key_pressed(KeyCode::Escape) && ui.inspection == Some(Inspection::Crate) {
        return Some(Action::Close);
    }
    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
        return if count == 0 {
            Some(done(state, ui))
        } else {
            Some(Action::Collection(CollectionAction::Recipient(ui.selected)))
        };
    }
    None
}

fn done(state: &Session, ui: &UiState) -> Action {
    if ui.inspection == Some(Inspection::Crate) {
        Action::Close
    } else {
        Action::Command(if centurions_core::needs_spell_confirmation(state) {
            Command::BeginPromotion
        } else {
            Command::Continue
        })
    }
}

pub fn draw(
    content: &Content,
    state: &Session,
    assets: &Assets,
    ui: &UiState,
    controls: &mut Vec<Control>,
) {
    controls.clear();
    draw_rectangle(0., 20., 800., 580., BLACK);
    assets.panel(207, Rect::new(0., 20., 480., 460.));
    assets.panel(226, Rect::new(0., 480., 800., 120.));
    let items: Vec<_> = records(state, ui)
        .iter()
        .enumerate()
        .filter(|(_, item)| item.id() != 0)
        .collect();
    let page = ui.page.min(items.len().saturating_sub(28).div_ceil(7));
    for (cell, (slot, item)) in items.iter().skip(page * 7).take(28).enumerate() {
        let rect = Rect::new(
            4. + (cell % 7) as f32 * 68.,
            24. + (cell / 7) as f32 * 102.,
            64.,
            100.,
        );
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, WHITE);
        if let Some(icon) = content
            .items
            .get(item.id().unsigned_abs() as usize)
            .and_then(|definition| word(definition, 10).ok())
            .and_then(|id| assets.icons.get(&id))
            .or_else(|| assets.icons.get(&6520))
        {
            draw_texture_ex(
                icon,
                rect.x + 4.,
                rect.y + 11.,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(32., 32.)),
                    ..Default::default()
                },
            );
        }
        let quantity = word(&item.0, 2).unwrap_or(0);
        if quantity != 0 {
            assets.text(&quantity.to_string(), rect.x + 4., rect.y + 57., 10, BLACK);
        }
        if ui.collection_slot == Some(*slot) {
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1., YELLOW);
        }
        controls.push(Control {
            rect,
            enabled: true,
            action: Action::Collection(CollectionAction::Item(*slot)),
        });
    }
    for (member, warrior) in state.warriors.iter().enumerate() {
        let y = 20. + member as f32 * 50.;
        assets.panel(184, Rect::new(540., y, 260., 50.));
        if let Some(texture) = word(&warrior.bytes, 0x54)
            .ok()
            .and_then(|id| assets.portraits.get(&id))
        {
            draw_texture_ex(
                texture,
                490.,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(50., 50.)),
                    ..Default::default()
                },
            );
        }
        assets.text(&warrior.name, 549., y + 20., 12, YELLOW);
        for (offset, x) in [(0x3c, 628.), (0x9d6, 698.), (0x9d8, 752.)] {
            assets.text(
                &word(&warrior.bytes, offset).unwrap_or(0).to_string(),
                x,
                y + 44.,
                12,
                YELLOW,
            );
        }
        assets.text(
            &warrior
                .inventory()
                .map_or(0, |items| items.len())
                .to_string(),
            775.,
            y + 20.,
            12,
            YELLOW,
        );
        let rect = Rect::new(480., y, 320., 50.);
        if ui.selected == member {
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1., YELLOW);
        }
        controls.push(Control {
            rect,
            enabled: true,
            action: Action::Collection(CollectionAction::Recipient(member)),
        });
        controls.push(Control {
            rect: Rect::new(547., y + 29., 35., 16.),
            enabled: true,
            action: Action::Collection(CollectionAction::DropMenu(member)),
        });
    }
    assets.panel(198, Rect::new(480., 320., 320., 110.));
    let message = assets
        .warnings
        .get(if ui.inspection == Some(Inspection::Crate) {
            151
        } else {
            34
        });
    if let Some(message) = message {
        assets.wrapped_color(message, Rect::new(492., 355., 298., 64.), 12, 14., YELLOW);
    }
    if ui.inspection == Some(Inspection::Crate) {
        assets.panel(230, Rect::new(0., 459., 480., 21.));
        for (x, value) in [
            (48., (page * 7 + 1).min(items.len())),
            (150., (page * 7 + 28).min(items.len())),
            (235., items.len()),
        ] {
            assets.text(&value.to_string(), x, 476., 12, YELLOW);
        }
    }
    let rect = Rect::new(480., 430., 50., 50.);
    assets.panel(168, rect);
    let enabled = ui.inspection == Some(Inspection::Crate) || items.is_empty();
    if !enabled {
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0., 0., 0., 0.45));
    }
    controls.push(Control {
        rect,
        enabled,
        action: done(state, ui),
    });
    if items.len() > 28 {
        assets.panel(132, Rect::new(530., 430., 50., 50.));
        for (y, delta, enabled) in [
            (430., -1, page > 0),
            (455., 1, page < items.len().saturating_sub(28).div_ceil(7)),
        ] {
            controls.push(Control {
                rect: Rect::new(530., y, 50., 25.),
                enabled,
                action: Action::Page(delta),
            });
        }
    }
    if let Some(item) = ui
        .collection_slot
        .and_then(|slot| records(state, ui).get(slot))
        .filter(|item| item.id() != 0)
        && let Some(map) = state
            .exploration
            .as_ref()
            .and_then(|position| content.map(position.map))
    {
        ui.inventory.detail(content, map, item, false, assets);
    }
    if let Some(member) = ui.collection_drop {
        draw_drop_menu(state, assets, ui, member, controls);
    }
}

fn draw_drop_menu(
    state: &Session,
    assets: &Assets,
    ui: &UiState,
    member: usize,
    controls: &mut Vec<Control>,
) {
    let Some(warrior) = state.warriors.get(member) else {
        return;
    };
    let items = warrior.inventory().unwrap_or_default();
    let width = items
        .iter()
        .map(|item| measure_text(assets.name(item.id()), Some(&assets.menu_font), 12, 1.).width)
        .fold(160., f32::max)
        .min(430.)
        + 26.;
    let height = items.len().min(24) as f32 * 18. + 8.;
    let rect = Rect::new(
        547_f32.min(800. - width),
        (49. + member as f32 * 50.).min(598. - height),
        width,
        height,
    );
    controls.clear();
    controls.push(Control {
        rect: Rect::new(0., 20., 800., 580.),
        enabled: true,
        action: Action::Collection(CollectionAction::CloseDrop),
    });
    draw_rectangle(rect.x + 3., rect.y + 3., rect.w, rect.h, BLACK);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::from_rgba(238, 238, 238, 255),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1., BLACK);
    for (row, (slot, item)) in items
        .iter()
        .enumerate()
        .skip(ui.collection_drop_scroll)
        .take(24)
        .enumerate()
    {
        let rect = Rect::new(
            rect.x + 1.,
            rect.y + 4. + row as f32 * 18.,
            rect.w - 2.,
            18.,
        );
        assets.menu_text(
            &assets.name(item.id()),
            rect.x + 21.,
            rect.y + 13.,
            12,
            BLACK,
        );
        if item.0[9] != 0 {
            assets.menu_text("✓", rect.x + 4., rect.y + 13., 12, BLACK);
        }
        controls.push(Control {
            rect,
            enabled: true,
            action: Action::Collection(CollectionAction::DropItem(slot)),
        });
    }
}
