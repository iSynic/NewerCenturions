//! The booty screen after a battle, also used for a crate: seven columns and
//! four rows of loot, the party as recipients, and quickinfo for the item under
//! the pointer. Each cell is a white box with the item's cicn and showp's text.
use crate::{
    assets::Assets,
    presentation::{Action, Control, Inspection, UiState},
};
use centurions_content::{Content, InventoryRecord, word};
use centurions_core::{Command, Presentation, Session};
use macroquad::prelude::*;

/// The booty cell the player has picked up and the member whose drop menu is open.
#[derive(Default)]
pub struct Selection {
    pub slot: Option<usize>,
    pub drag: bool,
    pub drop: Option<usize>,
    pub drop_scroll: usize,
}

/// What the player does on the booty or crate screen: pick up an item, hand it to
/// a recipient, store it, open a member's drop menu, or ask about an item.
#[derive(Clone, Debug)]
pub enum CollectionAction {
    Item(usize),
    Recipient(usize),
    Store(usize),
    DropMenu(usize),
    DropItem(usize),
    CloseDrop,
    /// The option-click in booty: showiteminfo on a loot cell.
    Information(usize),
}

/// Whether the booty screen is up: after a battle's loot, or when a crate is
/// opened.
pub fn active(state: &Session, ui: &UiState) -> bool {
    ui.inspection == Some(Inspection::Crate)
        || state
            .exploration
            .as_ref()
            .is_some_and(|position| position.presentation == Some(Presentation::Loot))
}

/// The items in the cells: the crate's contents, or the battle's booty.
fn records<'a>(state: &'a Session, ui: &UiState) -> &'a [InventoryRecord] {
    if ui.inspection == Some(Inspection::Crate) {
        &state.crate_items
    } else {
        &state.booty_list
    }
}

/// Carries out a click on the booty screen: selects the item, hands it to a
/// warrior, stores it, or opens and closes the drop menu.
pub fn resolve(
    action: CollectionAction,
    state: &Session,
    content: &centurions_content::Content,
    ui: &mut UiState,
) -> Option<Action> {
    match action {
        CollectionAction::Information(slot) => {
            let id = records(state, ui).get(slot)?.id().wrapping_abs();
            if id == 0 || ui.inspection == Some(Inspection::Crate) {
                return None;
            }
            // booty picks up a 200..400 reserve item instead.
            let definition = content.items.get(id as usize)?;
            let reserve = centurions_content::records::ItemDefinition(definition)
                .reserve_ammunition_kind()
                .ok()?;
            if !((200..=400).contains(&id) && reserve != 0) {
                let owner = state.globals.health_context.char_select_new;
                ui.inventory.open_loot_information(owner, id);
            }
            Some(Action::Command(Command::ShowLootItemInformation { slot }))
        }
        CollectionAction::DropMenu(member) => {
            ui.collection.drop = Some(member);
            ui.collection.drop_scroll = 0;
            None
        }
        CollectionAction::DropItem(slot) => ui
            .collection
            .drop
            .take()
            .map(|member| Action::Drop(member, slot)),
        CollectionAction::CloseDrop => {
            ui.collection.drop = None;
            None
        }
        CollectionAction::Item(slot) => {
            ui.collection.slot = Some(slot);
            ui.collection.drag = true;
            None
        }
        CollectionAction::Recipient(warrior) => {
            ui.selected = warrior;
            ui.collection.drag = false;
            let slot = ui.collection.slot?;
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
            ui.collection.drag = false;
            (ui.inspection != Some(Inspection::Crate))
                .then_some(Action::Command(Command::StoreLoot { slot }))
        }
    }
}

/// Handles the booty screen's mouse wheel, drag and drop onto a recipient, and
/// keys: Escape closes a crate or menu, Return gives the item to the selected warrior or, with
/// nothing left to take, leaves the screen.
pub fn input(state: &Session, ui: &mut UiState, pointer: Option<Vec2>) -> Option<Action> {
    if ui.inventory.loot_information_open() {
        return is_key_pressed(KeyCode::Escape).then_some(Action::Inventory(
            crate::inventory::InventoryAction::CloseInformation,
        ));
    }
    if let Some(member) = ui.collection.drop {
        let count = state
            .warriors
            .get(member)
            .and_then(|w| w.inventory().ok())
            .map_or(0, |items| items.len());
        let delta = -mouse_wheel().1.signum() as i32;
        ui.collection.drop_scroll = (ui.collection.drop_scroll as i32 + delta)
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
    if ui.collection.drag && is_mouse_button_released(MouseButton::Left) {
        ui.collection.drag = false;
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

/// The action that leaves the screen: closing a crate, or continuing after the
/// battle (through promotion if a warrior is to choose spells).
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

/// Draws the loot grid, the recipients, the message box, the share, credits and
/// the item under the pointer.
///
/// Original: booty
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
        // PenMode srcCopy, ForeColor whiteColor, PaintRect over the cell.
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, WHITE);
        let definition = content.items.get(item.id().unsigned_abs() as usize);
        let icon_rect = Rect::new(rect.x + 4., rect.y + 11., 32., 32.);
        if let Some(icon) = definition
            .and_then(|definition| {
                centurions_content::records::ItemDefinition(definition)
                    .icon()
                    .ok()
            })
            .and_then(|id| assets.icons.get(&id))
            .or_else(|| assets.icons.get(&6520))
        {
            crate::assets::texture_in(icon, icon_rect);
        }
        if let Some(cell) = definition.and_then(|definition| {
            centurions_content::records::loot_cell::loot_cell(definition, item).ok()
        }) {
            draw_cell(assets, &cell, icon_rect);
        }
        if ui.inspection != Some(Inspection::Crate) {
            let quality = state
                .globals
                .loot_quality
                .as_ref()
                .and_then(|quality| quality.get(*slot))
                .map_or(0, |value| *value as i8);
            draw_quality(assets, quality, icon_rect);
        }
        if ui.collection.slot == Some(*slot) {
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
        // updatepictbox in mode 1: the 32-pixel box at the row's left + 10,
        // inset by 9. plotportrait gives no condition cicns in booty's window.
        crate::portrait::draw(
            assets,
            warrior,
            vec2(499., y + 9.),
            crate::portrait::Marks {
                class_label: ui.preferences.enabled(crate::preferences::pref::APPEND_MOS),
                conditions_left: None,
                automatic: false,
            },
        );
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
    let crate_view = ui.inspection == Some(Inspection::Crate);
    // The 800 x 600 preference widens the grid.
    let wide = i16::from(
        ui.preferences
            .enabled(crate::preferences::pref::SCREEN_800_X_600),
    );
    let tally = state.booty_tally;
    let message = if crate_view {
        assets.warnings.get(151).cloned()
    } else if state.battle_after_message != 0 {
        // textbox with list -1 shows the scenario's after-battle text.
        state
            .exploration
            .as_ref()
            .and_then(|position| content.map(position.map))
            .and_then(|map| map.dialogue(state.battle_after_message).ok())
    } else {
        // STR# 3 string 0x47, 0x23 with items or credits, 0x24 when that also
        // has victory points, otherwise 0x22 with victory points.
        let mut id = 0x47;
        if !items.is_empty() || tally.credits != 0 {
            id = 0x23;
        }
        if tally.victory_points != 0 {
            id = if id == 0x23 { 0x24 } else { 0x22 };
        }
        assets.warnings.get(id - 1).cloned()
    };
    if let Some(message) = message {
        assets.dialog_text(&message, Rect::new(492., 355., 298., 64.));
    }
    // A nonzero share draws PICT 0xe8 (0xe7 at 800 x 600) with the share.
    if !crate_view && tally.share != 0 {
        assets.panel(0xe8 - wide, Rect::new(0., 459., 480., 21.));
        assets.text(&tally.share.to_string(), 125., 475., 12, YELLOW);
    }
    if crate_view {
        // PICT 0xe6 (0xe5 at 800 x 600), then the first and last visible
        // numbers and the count, size 12, unclamped.
        assets.panel(0xe6 - wide, Rect::new(0., 460., 480., 20.));
        let columns = wide * 2 + 5;
        let first = (columns * page as i16 + 1).to_string();
        let width = measure_text(&first, Some(&assets.font), 12, 1.).width;
        assets.text(&first, 90. - width, 476., 12, YELLOW);
        let last = columns * page as i16 + wide * 8 + 20;
        assets.text(&last.to_string(), 125., 476., 12, YELLOW);
        assets.text(&items.len().to_string(), 200., 476., 12, YELLOW);
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
    let scrolls = items.len() > 28;
    if scrolls {
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
    // updatebootymoney draws PICT 163 from 50 right of Done, or of the scroll
    // arrows, 640 wide and clipped by the window, then the credits in yellow at
    // its left + 20, top + 46.
    let left = if scrolls { 580. } else { 530. };
    assets.panel_part(
        163,
        Rect::new(0., 0., 800. - left, 50.),
        Rect::new(left, 430., 800. - left, 50.),
    );
    assets.text(&tally.credits.to_string(), left + 20., 476., 12, YELLOW);
    // At 800 x 600, booty's first draw shows quickinfo for record 0; pointing
    // at a cell shows that cell's.
    let shown = ui.collection.slot.or((wide != 0).then_some(0));
    if let Some(item) = shown
        .and_then(|slot| records(state, ui).get(slot))
        .filter(|item| item.id() != 0)
        && let Some(map) = state
            .exploration
            .as_ref()
            .and_then(|position| content.map(position.map))
    {
        ui.inventory.detail(content, map, item, false, assets);
    }
    if let Some(member) = ui.collection.drop {
        draw_drop_menu(state, assets, ui, member, controls);
    }
}

/// A loot cell's text, in the classic QuickDraw colors showp selects.
///
/// Original: showp
fn draw_cell(assets: &Assets, cell: &centurions_content::records::loot_cell::LootCell, icon: Rect) {
    use centurions_content::records::loot_cell::{CellAnchor, CellColor};
    if let Some(texture) = cell.caliber_icon.and_then(|id| assets.icons.get(&id)) {
        crate::assets::texture_in(
            texture,
            Rect::new(icon.right() + 1., icon.y + 14., 16., 16.),
        );
    }
    for text in &cell.texts {
        let color = match text.color {
            CellColor::Black => BLACK,
            CellColor::Red => Color::from_rgba(0xdd, 0x08, 0x06, 255),
            CellColor::Green => Color::from_rgba(0x00, 0x80, 0x11, 255),
            CellColor::Blue => Color::from_rgba(0x00, 0x00, 0xd4, 255),
        };
        let (mut x, y) = match text.anchor {
            CellAnchor::Right => (icon.right() + 1. + f32::from(text.x), icon.y),
            CellAnchor::Below => (icon.x + f32::from(text.x), icon.bottom()),
        };
        let y = y + f32::from(text.y);
        let size = u16::from(text.size);
        let mut strings = Vec::new();
        if let Some((value, signed)) = text.value {
            strings.push(if signed && value > 0 {
                format!("+{value}")
            } else {
                value.to_string()
            });
        }
        strings.extend(
            text.labels
                .iter()
                .filter_map(|address| assets.game_text.text(*address, true).ok()),
        );
        for string in strings {
            assets.text(&string, x, y, size, color);
            x += measure_text(&string, Some(&assets.font), size, 1.).width;
        }
    }
}

/// One 8x8 cicn 208 per loot-quality step, seven to a line.
///
/// Original: showp
fn draw_quality(assets: &Assets, quality: i8, icon: Rect) {
    let Some(texture) = assets
        .icons
        .get(&centurions_content::records::loot_cell::QUALITY_ICON)
    else {
        return;
    };
    for step in 0..quality.max(0) as usize {
        let (x, y) = centurions_content::records::loot_cell::quality_position(step);
        crate::assets::texture_in(
            texture,
            Rect::new(icon.x + f32::from(x), icon.y + f32::from(y), 8., 8.),
        );
    }
}

/// Draws the menu of a member's items to choose one to drop, with a check mark
/// on equipped items.
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
        .skip(ui.collection.drop_scroll)
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
        if item.equipped() != 0 {
            assets.menu_text("✓", rect.x + 4., rect.y + 13., 12, BLACK);
        }
        controls.push(Control {
            rect,
            enabled: true,
            action: Action::Collection(CollectionAction::DropItem(slot)),
        });
    }
}
