//! spelltargets/showtargets/getchoice select in the map and portrait strip.
use crate::{
    assets::Assets,
    layout,
    presentation::{Action, Control, Inspection, UiState},
};
use centurions_content::{Content, word};
use centurions_core::{Command, MedicalTarget, Session};
use macroquad::prelude::*;

pub fn active(state: &Session, ui: &UiState) -> bool {
    state.medical_continuation.is_some()
        || state.outdoor_psi.is_some()
        || state.psi_selection.is_some()
        || matches!(
            ui.inspection,
            Some(Inspection::FireTargets | Inspection::PsiTargets | Inspection::Bandage)
        )
}

fn cancellation(state: &Session, ui: &UiState) -> Action {
    if state.medical_continuation.is_some() || state.psi_selection.is_some() {
        if ui.medical_cancel {
            Action::KeepMedicalTarget
        } else {
            Action::MedicalCancel
        }
    } else if state.outdoor_psi.is_some() {
        Action::Command(Command::CancelOutdoorPsi)
    } else if ui.inspection == Some(Inspection::Bandage) {
        Action::Command(Command::StopBleeding { warrior: None })
    } else {
        Action::Close
    }
}

fn body_at(state: &Session, point: (i16, i16)) -> Option<i16> {
    let combat = state.combat.as_ref()?;
    let index = centurions_content::mission::OpeningMission::index(
        i32::from(point.0 + combat.map.shift.0 + 5),
        i32::from(point.1 + combat.map.shift.1 + 5),
    )
    .ok()?;
    let body = combat.map.tiles[index].wrapping_abs();
    ((0..state.warriors.len() as i16).contains(&body)
        && state.warriors[body as usize].record().active_flag() != 0
        || (10..10 + combat.enemies.records.len() as i16).contains(&body)
            && combat.enemies.records[(body - 10) as usize]
                .record()
                .stamina()
                .unwrap_or(0)
                > 0)
    .then_some(body)
}

fn map_action(
    state: &Session,
    content: &Content,
    ui: &UiState,
    point: (i16, i16),
) -> Option<Action> {
    if state.psi_selection.is_some() {
        if state.combat.as_ref()?.attack_packet[1] != 0 {
            return Some(Action::Command(Command::FirePreparedPsi {
                x: point.0,
                y: point.1,
                rotation: ui.psi_rotation,
            }));
        }
        return Some(Action::Command(Command::SelectPsiTarget {
            x: point.0,
            y: point.1,
        }));
    }
    if let Some(pending) = &state.medical_continuation {
        let member = body_at(state, point)
            .filter(|body| (0..state.warriors.len() as i16).contains(body))?
            as usize;
        if pending.origin == centurions_core::MedicalOrigin::LearnedHeal
            || state.warriors[member].stamina().unwrap_or(0) <= 0
        {
            return None;
        }
        return Some(Action::Command(Command::SelectMedicalTarget {
            selection: MedicalTarget::LivingPoint {
                x: point.0,
                y: point.1,
            },
        }));
    }
    if ui.inspection == Some(Inspection::PsiTargets) {
        let (spell, power) = ui.psi_choice?;
        return Some(Action::Command(Command::CastPsi {
            rotation: ui.psi_rotation,
            spell,
            power,
            target: Some(point),
        }));
    }
    if ui.inspection == Some(Inspection::FireTargets) {
        return if centurions_core::combat::shooting::location_targeting(state, content) {
            Some(Action::Command(Command::ShootAt {
                x: point.0,
                y: point.1,
            }))
        } else {
            body_at(state, point).map(|body| Action::Command(Command::ShootEnemy { body }))
        };
    }
    None
}

pub fn input(
    state: &Session,
    content: &Content,
    ui: &mut UiState,
    controls: &[Control],
    pointer: Option<Vec2>,
) -> Option<Action> {
    if !ui.medical_cancel
        && state.combat.is_some()
        && (state.psi_selection.is_some()
            || state.medical_continuation.is_some()
            || matches!(
                ui.inspection,
                Some(Inspection::FireTargets | Inspection::PsiTargets)
            ))
    {
        if let Some(pointer) = pointer.filter(|point| layout::MAP.contains(*point))
            && let Some(point) = crate::combat_controls::map_position(state, ui, pointer)
        {
            ui.estimate_target = body_at(state, point);
        }
        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
            return Some(Action::TargetQuantity(true));
        }
        if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
            return Some(Action::TargetQuantity(false));
        }
    }
    if ui.inspection == Some(Inspection::PsiTargets) {
        ui.psi_preview = pointer
            .filter(|point| layout::MAP.contains(*point))
            .and_then(|point| crate::combat_controls::map_position(state, ui, point));
        if is_key_pressed(KeyCode::Enter)
            && let Some((spell, _)) = ui.psi_choice
            && let Some(attacks) = state
                .exploration
                .as_ref()
                .and_then(|position| content.map(position.map))
                .and_then(|map| map.combat.as_ref())
                .map(|combat| &combat.attacks)
            && centurions_core::combat::spells::load_packet(attacks, spell)
                .is_ok_and(|packet| packet[7] != 0)
        {
            return Some(Action::RotatePsi);
        }
    }
    if is_key_pressed(KeyCode::Escape) || state.outdoor_psi.is_some() && is_key_pressed(KeyCode::A)
    {
        return Some(cancellation(state, ui));
    }
    if !ui.medical_cancel && (is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space)) {
        if state
            .psi_selection
            .as_ref()
            .is_some_and(|pending| !pending.targets.is_empty())
        {
            return Some(Action::Command(Command::FirePsiSelection));
        }
        if state
            .combat
            .as_ref()
            .is_some_and(|c| c.firearm_selection.is_some())
        {
            return Some(Action::Command(Command::FireSelectedTargets));
        }
        if centurions_core::outdoor_psi::can_finish(state) {
            return Some(Action::Command(Command::FinishOutdoorPsi));
        }
    }
    if is_mouse_button_pressed(MouseButton::Left)
        && let Some(pointer) = pointer
    {
        if let Some(control) = controls
            .iter()
            .rev()
            .find(|control| control.rect.contains(pointer))
        {
            return control.enabled.then(|| control.action.clone());
        }
        if !ui.medical_cancel
            && layout::MAP.contains(pointer)
            && let Some(point) = crate::combat_controls::map_position(state, ui, pointer)
        {
            return map_action(state, content, ui, point);
        }
    }
    None
}

/// combatchoice's showspellinfo for the chosen spell, item or firearm (0x118e1e,
/// 0x1192ca, and 0x11a622/0x11a794 after each quantity step). `power` is
/// WORD4435cc, or for a category-9 packet WORD4435be.
fn spell_info(
    state: &Session,
    content: &Content,
    ui: &UiState,
    power: i16,
) -> Option<centurions_core::SpellInfo> {
    let combat = state.combat.as_ref()?;
    let (packet, identity) = match ui.psi_choice {
        Some((spell, _)) if ui.inspection == Some(Inspection::PsiTargets) => {
            let attacks = &state
                .exploration
                .as_ref()
                .and_then(|position| content.map(position.map))?
                .combat
                .as_ref()?
                .attacks;
            (
                centurions_core::combat::spells::load_packet(attacks, spell).ok()?,
                [spell / 1000 - 1, spell % 1000 / 100 - 1, spell % 100 - 1],
            )
        }
        _ => (combat.attack_packet.clone(), combat.attack_identity),
    };
    let work = &state.native_work.ammunition_work;
    Some(centurions_core::SpellInfo::new(
        &packet,
        identity,
        if state.target_quantity.is_some() {
            state.native_work.attack_count
        } else {
            power
        },
        Some(power),
        (work.weapon.as_deref(), work.item.as_deref()),
    ))
}

fn marker(state: &Session, assets: &Assets, ui: &UiState, body: i16, rank: usize) {
    let Some(combat) = &state.combat else { return };
    let point = if body < 10 {
        combat.squad.positions.get(body as usize)
    } else {
        combat.enemies.positions.get((body - 10) as usize)
    };
    if let Some(point) = point {
        cell_marker(state, assets, ui, *point, rank);
    }
}

/// The target key plots cicn 14999 + WORD4435c0 over the cell's 32-pixel square
/// (combatchoice 11b23a..11b262, 11b566..11b596); a location triple uses the cursor cell.
fn cell_marker(state: &Session, assets: &Assets, ui: &UiState, point: (i16, i16), rank: usize) {
    if let Some((vx, vy)) = crate::combat_controls::origin(state, ui) {
        let rect = Rect::new(
            (point.0 - vx) as f32 * 32.,
            20. + (point.1 - vy) as f32 * 32.,
            32.,
            32.,
        );
        if layout::MAP.contains(vec2(rect.x, rect.y))
            && let Some(image) = assets.icons.get(&(15000 + rank as i16))
        {
            draw_texture_ex(
                image,
                rect.x,
                rect.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(rect.size()),
                    ..Default::default()
                },
            );
        }
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
    if ui.medical_cancel {
        let prompt = assets.native_text.text(0x403c72, false).unwrap_or_default();
        let command = if state.psi_selection.is_some() {
            Command::CancelPsiSelection
        } else {
            Command::SelectMedicalTarget {
                selection: MedicalTarget::ConfirmedCancel,
            }
        };
        crate::story_dialogs::question(
            assets,
            &prompt,
            Action::Command(command),
            Action::KeepMedicalTarget,
            controls,
        );
        return;
    }
    if state.outdoor_psi.is_some() {
        let prompt = if state
            .outdoor_psi
            .as_ref()
            .is_some_and(|pending| pending.item.is_some())
        {
            assets.native_text.text(0x4033f6, false).unwrap_or_default()
        } else {
            assets.warnings.get(46).cloned().unwrap_or_default()
        };
        assets.wrapped_color(&prompt, Rect::new(12., 467., 444., 82.), 12, 16., YELLOW);
        crate::story_dialogs::portraits(state, assets, controls, |warrior| {
            Some(Action::Command(Command::SelectOutdoorPsiTarget { warrior }))
        });
        for (member, rank) in state
            .native_work
            .party_mask
            .iter()
            .take(state.warriors.len())
            .enumerate()
            .filter(|(_, rank)| **rank != 0)
        {
            if let Some(image) = assets.icons.get(&(2200 + *rank)) {
                draw_texture(image, 499., 29. + member as f32 * 50., WHITE);
            }
        }
        return;
    }
    if ui.inspection == Some(Inspection::Bandage) {
        crate::story_dialogs::portraits(state, assets, controls, |warrior| {
            (state.warriors[warrior].bytes[0x9de] != 0).then_some(Action::Command(
                Command::StopBleeding {
                    warrior: Some(warrior),
                },
            ))
        });
        return;
    }
    assets.panel(145, Rect::new(468., 449., 332., 89.));
    draw_estimate(state, content, assets, ui);
    for (x, increase) in [(730., false), (765., true)] {
        controls.push(Control {
            rect: Rect::new(x, 528., 28., 7.),
            enabled: true,
            action: Action::TargetQuantity(increase),
        });
    }
    let power = state
        .psi_selection
        .as_ref()
        .map(|p| p.power)
        .or_else(|| state.medical_continuation.as_ref().map(|p| p.power))
        .or_else(|| ui.psi_choice.map(|p| p.1))
        .or_else(|| centurions_core::combat::targeting::firearm_ceiling(state, content).ok());
    if let Some(power) = power
        && let Some(info) = spell_info(state, content, ui, power)
    {
        crate::presentation::draw_info(assets, &info);
    }
    let finish = if state.psi_selection.is_some() {
        Command::FirePsiSelection
    } else {
        Command::FireSelectedTargets
    };
    let finish_enabled = state
        .psi_selection
        .as_ref()
        .is_some_and(|p| !p.targets.is_empty())
        || state
            .combat
            .as_ref()
            .is_some_and(|c| c.firearm_selection.is_some());
    for (rect, enabled, action) in [
        (
            Rect::new(526., 456., 50., 18.),
            finish_enabled,
            Action::Command(finish.clone()),
        ),
        (
            Rect::new(632., 456., 47., 18.),
            finish_enabled,
            Action::Command(finish),
        ),
        (
            Rect::new(579., 456., 48., 18.),
            true,
            cancellation(state, ui),
        ),
        (
            Rect::new(526., 496., 43., 18.),
            true,
            Action::CombatView(crate::combat_controls::ViewAction::Next),
        ),
        (
            Rect::new(572., 496., 56., 18.),
            true,
            Action::CombatView(crate::combat_controls::ViewAction::Previous),
        ),
        (
            Rect::new(632., 496., 47., 18.),
            true,
            Action::CombatView(crate::combat_controls::ViewAction::Center),
        ),
    ] {
        controls.push(Control {
            rect,
            enabled,
            action,
        });
    }
    if state
        .combat
        .as_ref()
        .is_some_and(|c| c.firearm_selection.is_some())
    {
        controls.push(Control {
            rect: Rect::new(632., 476., 47., 18.),
            enabled: true,
            action: Action::Command(Command::CancelFireTargets),
        });
    }
    if state.psi_selection.is_some() {
        controls.push(Control {
            rect: Rect::new(632., 476., 47., 18.),
            enabled: finish_enabled,
            action: Action::Command(Command::ClearPsiTargets),
        });
    }
    if let Some(body) = ui.combat_view.selected(state) {
        controls.push(Control {
            rect: Rect::new(735., 449., 53., 50.),
            enabled: true,
            action: Action::Application(crate::application::ApplicationAction::CombatInformation(
                body,
            )),
        });
    }
    let portrait_spell = state
        .psi_selection
        .as_ref()
        .map(|p| (p.spell, p.power))
        .and_then(|(spell, power)| {
            let attacks = &content
                .map(state.exploration.as_ref()?.map)?
                .combat
                .as_ref()?
                .attacks;
            let packet = centurions_core::combat::spells::load_packet(attacks, spell).ok()?;
            (packet[1] == 1).then_some((spell, power))
        });
    if state.medical_continuation.is_some() || portrait_spell.is_some() {
        crate::story_dialogs::portraits(state, assets, controls, |warrior| {
            Some(Action::Command(if portrait_spell.is_some() {
                Command::FirePreparedPsiOnWarrior { warrior }
            } else {
                Command::SelectMedicalTarget {
                    selection: MedicalTarget::Portrait { warrior },
                }
            }))
        });
    }
    let targets = state
        .psi_selection
        .as_ref()
        .map(|p| &p.targets)
        .or_else(|| {
            state
                .combat
                .as_ref()?
                .firearm_selection
                .as_ref()
                .map(|p| &p.targets)
        });
    if let Some(targets) = targets {
        for (rank, body) in targets.iter().enumerate() {
            marker(state, assets, ui, *body, rank);
        }
    }
    if let Some(selection) = state
        .combat
        .as_ref()
        .and_then(|c| c.firearm_selection.as_ref())
    {
        for (rank, location) in selection.locations.iter().enumerate() {
            cell_marker(state, assets, ui, *location, rank);
        }
    }
}

pub type EstimateCache = (
    Vec<u8>,
    Result<centurions_core::item_display::output::PanelPlan, String>,
);

fn draw_estimate(state: &Session, content: &Content, assets: &Assets, ui: &UiState) {
    let Some(combat) = &state.combat else { return };
    let Some(actor) = combat
        .active_body()
        .filter(|body| (0..state.warriors.len() as i16).contains(body))
    else {
        return;
    };
    let body = ui
        .estimate_target
        .or_else(|| ui.combat_view.selected(state))
        .unwrap_or(actor);
    let mut key = vec![];
    key.extend(actor.to_be_bytes());
    key.extend(body.to_be_bytes());
    key.extend(combat.difficulty.to_be_bytes());
    let warrior = &state.warriors[actor as usize];
    key.extend(&warrior.bytes);
    if body >= 10
        && let Some(enemy) = combat.enemies.records.get((body - 10) as usize)
    {
        key.extend(&enemy.bytes);
    }
    for work in [
        &state.native_work.ammunition_work.item,
        &state.native_work.ammunition_work.ammo,
        &state.native_work.ammunition_work.weapon,
    ]
    .into_iter()
    .flatten()
    {
        key.extend(work);
    }
    for at in [0xca, 0xe4, 0x126, 0x12c, 0x12e] {
        if let Ok(id) = word(&warrior.bytes, at)
            && let Some(bytes) = content.items.get(id.unsigned_abs() as usize)
        {
            key.extend(bytes);
        }
    }
    let mut cached = ui.estimate.borrow_mut();
    if cached.as_ref().is_none_or(|(previous, _)| *previous != key) {
        let plan = centurions_core::item_display::attack_estimate(
            state,
            content,
            body,
            &assets.native_text,
            &assets.character_strings,
        )
        .map_err(|error| {
            eprintln!("attack estimate: {error}");
            error.to_string()
        });
        *cached = Some((key, plan));
    }
    if let Some((_, Ok(plan))) = cached.as_ref() {
        crate::inventory::draw_detail_plan(plan, assets);
    }
}

pub fn adjust(
    state: &mut Session,
    content: &Content,
    ui: &mut UiState,
    increase: bool,
) -> Result<Vec<centurions_core::Event>, centurions_core::RuleError> {
    let events =
        centurions_core::apply(Command::AdjustTargetQuantity { increase }, state, content)?;
    if let Some(pending) = &state.psi_selection {
        ui.psi_power = pending.power;
        ui.psi_choice = Some((pending.spell, pending.power));
        ui.psi_pending = ui.psi_choice;
    }
    Ok(events)
}
