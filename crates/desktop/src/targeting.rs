//! Choosing targets on the map and in the portrait strip, as spelltargets,
//! showtargets and getchoice do.
use crate::{
    assets::Assets,
    layout,
    presentation::{Action, Control, Inspection, UiState},
};
use centurions_content::{Content, word};
use centurions_core::{Command, MedicalTarget, Session};
use macroquad::prelude::*;

/// The target the player is weighing: the hit estimate under the pointer, whether a firearm aims at
/// the map, and the medical cancel prompt.
#[derive(Default)]
pub struct Targeting {
    pub firearm_map: bool,
    pub estimate_target: Option<i16>,
    pub estimate: std::cell::RefCell<Option<EstimateCache>>,
    pub medical_cancel: bool,
}

/// Whether the player is choosing targets: for a firearm, a scan, bandaging or
/// a medical item.
pub fn active(state: &Session, ui: &UiState) -> bool {
    state.medical_continuation.is_some()
        || state.outdoor_psi.is_some()
        || state.psi_selection.is_some()
        || matches!(
            ui.inspection,
            Some(Inspection::FireTargets | Inspection::PsiTargets | Inspection::Bandage)
        )
}

/// The action Escape stands for while choosing targets.
fn cancellation(state: &Session, ui: &UiState) -> Action {
    if state.medical_continuation.is_some() || state.psi_selection.is_some() {
        if ui.targeting.medical_cancel {
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

/// The body standing on the map cell `point`: a warrior in battle or a monster
/// with stamina left.
fn body_at(state: &Session, point: (i16, i16)) -> Option<i16> {
    let combat = state.combat.as_ref()?;
    let index = centurions_content::mission::OpeningMission::index(
        i32::from(point.0 + combat.field.map_shift.0 + 5),
        i32::from(point.1 + combat.field.map_shift.1 + 5),
    )
    .ok()?;
    let body = combat.field.tiles[index].wrapping_abs();
    ((0..state.warriors.len() as i16).contains(&body)
        && state.warriors[body as usize].record().in_battle() != 0
        || (10..10 + combat.roster.records.len() as i16).contains(&body)
            && combat.roster.records[(body - 10) as usize]
                .record()
                .stamina()
                .unwrap_or(0)
                > 0)
    .then_some(body)
}

/// The action a click on map cell `point` stands for in the current targeting.
fn map_action(
    state: &Session,
    content: &Content,
    ui: &UiState,
    point: (i16, i16),
) -> Option<Action> {
    if state.psi_selection.is_some() {
        if state.combat.as_ref()?.spell_info[1] != 0 {
            return Some(Action::Command(Command::FirePreparedPsi {
                x: point.0,
                y: point.1,
                rotation: ui.psi.rotation,
            }));
        }
        return Some(Action::Command(Command::SelectPsiTarget {
            x: point.0,
            y: point.1,
        }));
    }
    if let Some(pending) = &state.medical_continuation {
        let member = body_at(state, point)
            .filter(|body| crate::acting::is_roster_warrior(state, *body))?
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
        let (spell, power) = ui.psi.choice?;
        return Some(Action::Command(Command::CastPsi {
            rotation: ui.psi.rotation,
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

/// Handles targeting: the hit estimate under the pointer, the plus and minus keys, Return or
/// Space to finish choosing and fire, Escape to cancel, and clicks on the map and controls.
pub fn input(
    state: &Session,
    content: &Content,
    ui: &mut UiState,
    controls: &[Control],
    pointer: Option<Vec2>,
) -> Option<Action> {
    if !ui.targeting.medical_cancel
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
            ui.targeting.estimate_target = body_at(state, point);
        }
        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
            return Some(Action::TargetQuantity(true));
        }
        if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
            return Some(Action::TargetQuantity(false));
        }
    }
    if ui.inspection == Some(Inspection::PsiTargets) {
        ui.psi.preview = pointer
            .filter(|point| layout::MAP.contains(*point))
            .and_then(|point| crate::combat_controls::map_position(state, ui, point));
        if is_key_pressed(KeyCode::Enter)
            && let Some((spell, _)) = ui.psi.choice
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
    if !ui.targeting.medical_cancel
        && (is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space))
    {
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
        if let Some(control) = crate::presentation::topmost(controls, pointer) {
            return control.enabled.then(|| control.action.clone());
        }
        if !ui.targeting.medical_cancel
            && layout::MAP.contains(pointer)
            && let Some(point) = crate::combat_controls::map_position(state, ui, pointer)
        {
            return map_action(state, content, ui, point);
        }
    }
    None
}

/// combatchoice's showspellinfo for the chosen spell, item or firearm, shown
/// again after each quantity step. `power` is the power level, or for a
/// category-9 spell the number of targets.
fn spell_info(
    state: &Session,
    content: &Content,
    ui: &UiState,
    power: i16,
) -> Option<centurions_core::SpellInfo> {
    let combat = state.combat.as_ref()?;
    let (packet, identity) = match ui.psi.choice {
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
        _ => (combat.spell_info.clone(), combat.cast_identity),
    };
    let loaded = &state.globals.loaded_items;
    Some(centurions_core::SpellInfo::new(
        &packet,
        identity,
        if state.target_quantity.is_some() {
            state.globals.attack_count
        } else {
            power
        },
        Some(power),
        (loaded.weapon.as_deref(), loaded.item.as_deref()),
    ))
}

/// Marks `body`'s cell as the `rank`-th target.
fn marker(state: &Session, assets: &Assets, ui: &UiState, body: i16, rank: usize) {
    let Some(combat) = &state.combat else { return };
    let point = if crate::acting::is_warrior_body(body) {
        combat.squad.pos.get(body as usize)
    } else {
        combat.roster.mon_pos.get((body - 10) as usize)
    };
    if let Some(point) = point {
        cell_marker(state, assets, ui, *point, rank);
    }
}

/// The target key plots cicn 14999 + n over the n-th target's 32-pixel cell; a
/// location triple uses the cursor cell.
///
/// Original: combatchoice
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
            crate::assets::texture_in(image, rect);
        }
    }
}

/// Draws the targeting screen: the target markers, the hit estimate, the
/// portrait strip and the controls.
pub fn draw(
    content: &Content,
    state: &Session,
    assets: &Assets,
    ui: &UiState,
    controls: &mut Vec<Control>,
) {
    controls.clear();
    if ui.targeting.medical_cancel {
        let prompt = assets
            .game_text
            .text(crate::game_strings::ABORT_CHARGE_QUESTION, false)
            .unwrap_or_default();
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
            assets
                .game_text
                .text(crate::game_strings::SELECT_TARGETS, false)
                .unwrap_or_default()
        } else {
            assets.warnings.get(46).cloned().unwrap_or_default()
        };
        assets.wrapped_color(&prompt, layout::MESSAGE_TEXT, 12, 16., YELLOW);
        crate::story_dialogs::portraits(state, assets, controls, |warrior| {
            Some(Action::Command(Command::SelectOutdoorPsiTarget { warrior }))
        });
        for (member, rank) in state
            .globals
            .party_track
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
            (state.warriors[warrior].record().bleeding() != 0).then_some(Action::Command(
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
        .or_else(|| ui.psi.choice.map(|p| p.1))
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

/// The hit estimate panel already built for the actor and body.
pub type EstimateCache = (
    Vec<u8>,
    Result<centurions_core::item_display::output::PanelPlan, String>,
);

/// Draws the hit estimate of the acting warrior against the body in view.
fn draw_estimate(state: &Session, content: &Content, assets: &Assets, ui: &UiState) {
    let Some(combat) = &state.combat else { return };
    let Some(actor) = crate::acting::active_warrior(state) else {
        return;
    };
    let body = ui
        .targeting
        .estimate_target
        .or_else(|| ui.combat_view.selected(state))
        .unwrap_or(actor);
    let mut key = vec![];
    key.extend(actor.to_be_bytes());
    key.extend(body.to_be_bytes());
    key.extend(combat.difficulty.to_be_bytes());
    let warrior = &state.warriors[actor as usize];
    key.extend(&warrior.bytes);
    if !crate::acting::is_warrior_body(body)
        && let Some(monster) = combat.roster.records.get((body - 10) as usize)
    {
        key.extend(&monster.bytes);
    }
    for loaded in [
        &state.globals.loaded_items.item,
        &state.globals.loaded_items.ammo,
        &state.globals.loaded_items.weapon,
    ]
    .into_iter()
    .flatten()
    {
        key.extend(loaded);
    }
    for at in [0xca, 0xe4, 0x126, 0x12c, 0x12e] {
        if let Ok(id) = word(&warrior.bytes, at)
            && let Some(bytes) = content.items.get(id.unsigned_abs() as usize)
        {
            key.extend(bytes);
        }
    }
    let mut cached = ui.targeting.estimate.borrow_mut();
    if cached.as_ref().is_none_or(|(previous, _)| *previous != key) {
        let plan = centurions_core::item_display::attack_estimate(
            state,
            content,
            body,
            &assets.game_text,
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

/// Raises or lowers the quantity set for the targets being chosen, keeping the
/// scan picker's power in step.
pub fn adjust(
    state: &mut Session,
    content: &Content,
    ui: &mut UiState,
    increase: bool,
) -> Result<Vec<centurions_core::Event>, centurions_core::RuleError> {
    let events =
        centurions_core::apply(Command::AdjustTargetQuantity { increase }, state, content)?;
    if let Some(pending) = &state.psi_selection {
        ui.psi.power = pending.power;
        ui.psi.choice = Some((pending.spell, pending.power));
        ui.psi.pending = ui.psi.choice;
    }
    Ok(events)
}
