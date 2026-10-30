//! Scene presentation.
use super::{
    Action, Control, Inspection, UiState, authored_text, button, draw_clinic,
    draw_combat_information, draw_outdoor_effect, draw_story_picture, draw_survivors, hud, map,
};
use crate::{
    assets::Assets,
    layout::{self, control},
};
use centurions_content::Content;
use centurions_content::records::TypedEncounterRecord;
use centurions_core::{Command, Phase, Presentation, Session};
use macroquad::prelude::*;

/// Draws the game screen: the map, effects, story picture, squad rows, panels,
/// combat or exploration controls, and any dialog on top. Returns the controls
/// the player can click.
pub fn draw(content: &Content, state: &Session, assets: &Assets, ui: &UiState) -> Vec<Control> {
    clear_background(Color::from_rgba(41, 42, 0, 255));
    let mut controls = Vec::new();
    map(content, state, assets, ui);
    draw_outdoor_effect(state, assets, ui);
    draw_story_picture(assets, ui);
    draw_psi_target(content, state, assets, ui);
    let selected = draw_frame_panels(content, state, assets, ui, &mut controls);
    if state.phase != Phase::SquadSelection {
        if state.phase == Phase::Combat {
            combat_controls_strip(content, state, assets, ui, selected, &mut controls);
        }
        if state.phase == Phase::Exploration {
            field_controls_strip(state, assets, selected, &mut controls);
        }
        draw_story_presentation(content, state, assets, ui, &mut controls);
    }
    if draw_overlay(content, state, assets, ui, &mut controls) {
        return controls;
    }
    if let Some(progress) = &state.promotion {
        controls.clear();
        crate::editor::ScanEditor::draw_promotion(progress, content, assets, &mut controls);
    }
    if crate::targeting::active(state, ui) {
        crate::targeting::draw(content, state, assets, ui, &mut controls);
    }
    super::range_lines::draw_reveal(state, ui);
    for control in &mut controls {
        control.enabled &= crate::input::available(state, ui.selected, &control.action);
    }
    controls
}

/// The area the pending Psi spell affects, drawn under the previewed map cell.
/// An area spell takes its area size and a counted spell its power; every
/// other spell affects a single cell. A rotating spell adds its rotation, as
/// the cast does. The scan picker shows the spell's icon instead.
fn draw_psi_target(content: &Content, state: &Session, assets: &Assets, ui: &UiState) {
    if let (Some((spell, power)), Some(point), Some((vx, vy))) = (
        ui.psi.pending,
        ui.psi.preview,
        crate::combat_controls::origin(state, ui),
    ) && let Some(mission) = state.land(content)
        && let Some(tables) = mission.combat.as_ref()
        && let Ok(packet) = centurions_core::combat::spells::load_packet(&tables.attacks, spell)
    {
        let shape = match packet[1] {
            3 => i16::from(packet[31]),
            4 => power,
            _ => 1,
        } + if packet[7] != 0 {
            i16::from(ui.psi.rotation)
        } else {
            0
        };
        assets.target_pattern(
            shape,
            vec2(
                layout::MAP.x + f32::from(point.0 - 3 - vx) * 32. - 2.,
                layout::MAP.y + f32::from(point.1 - 3 - vy) * 32. - 2.,
            ),
            layout::MAP,
        );
    }
}

/// The panels around the map, the roster, clock and information panes, and
/// the Items and Conditions controls. Returns the warrior the roster shows.
fn draw_frame_panels(
    content: &Content,
    state: &Session,
    assets: &Assets,
    ui: &UiState,
    controls: &mut Vec<Control>,
) -> usize {
    if state.phase == Phase::Exploration {
        assets.panel(199, Rect::new(480., 428., 320., 21.));
    } else {
        assets.panel(197, Rect::new(480., 429., 320., 19.));
    }
    let outdoor_idle = state.phase == Phase::Exploration
        && state
            .exploration
            .as_ref()
            .is_some_and(|p| p.presentation.is_none());
    assets.panel(if outdoor_idle { 222 } else { 220 }, layout::MESSAGE);
    if outdoor_idle && let Some(warrior) = state.warriors.get(ui.selected) {
        let points = warrior.record().upgrade_points();
        assets.text(&points.to_string(), 215., 528., 14, YELLOW);
    }
    if state.phase == Phase::Combat {
        assets.panel(153, Rect::new(468., 449., 332., 89.));
        assets.panel(156, Rect::new(568., 538., 232., 50.));
    } else {
        assets.panel(176, Rect::new(468., 449., 332., 139.));
    }
    assets.panel(224, layout::INFORMATION);
    let selected = hud::roster(state, assets, ui, controls);
    hud::clock(state, assets);
    hud::information(state, assets, selected);
    let mission_ready = state.phase != Phase::SquadSelection;
    let (items_rect, conditions_rect) = if state.phase == Phase::Combat {
        (layout::COMBAT_ITEMS, layout::COMBAT_CONDITIONS)
    } else {
        (
            assets.main_controls[&control::SHOW_ITEMS],
            assets.main_controls[&control::SHOW_CONDITIONS],
        )
    };
    if state.phase == Phase::Combat {
        draw_combat_information(
            content,
            state,
            assets,
            ui.combat_view.selected(state).unwrap_or(selected as i16),
        );
    }
    for (rect, inspection) in [
        (items_rect, Inspection::Items),
        (conditions_rect, Inspection::Conditions),
    ] {
        let action = Action::Open(inspection);
        controls.push(Control {
            rect,
            enabled: mission_ready && crate::input::available(state, selected, &action),
            action,
        });
    }
    selected
}

/// The combat control strip: capture and Psi availability, the turn buttons,
/// automatic-mode controls and the playing spell's panel.
fn combat_controls_strip(
    content: &Content,
    state: &Session,
    assets: &Assets,
    ui: &UiState,
    selected: usize,
    controls: &mut Vec<Control>,
) {
    let capture = crate::input::available(state, selected, &Action::Command(Command::BeginCapture));
    assets.panel(
        if capture {
            161
        } else {
            layout::DISABLED_CONTROL_PICT
        },
        assets.main_controls[&control::REPAIR],
    );
    if !crate::input::available(state, selected, &Action::Open(Inspection::Psi)) {
        assets.panel(
            layout::DISABLED_CONTROL_PICT,
            assets.main_controls[&control::SCAN],
        );
    }
    let mut buttons = crate::combat_controls::buttons(state, content, selected);
    for button in &mut buttons {
        match button.action {
            Action::Open(Inspection::Weapons) => {
                button.rect = assets.main_controls[&control::CHANGE_WEAPON]
            }
            Action::Application(crate::application::ApplicationAction::CombatInformation(_)) => {
                button.rect = assets.main_controls[&control::MONSTER_INFO]
            }
            _ => {}
        }
    }
    if let Some(body) = ui.combat_view.selected(state) {
        for button in &mut buttons {
            if let Action::Application(crate::application::ApplicationAction::CombatInformation(
                selected,
            )) = &mut button.action
            {
                *selected = body;
            }
        }
    }
    // combat and combatchoice call downbutton(1) over the clicked button's hit
    // rectangle and upbutton(1) once its action ends.
    if let Some(point) = ui.held_pointer
        && let Some(button) = buttons
            .iter()
            .find(|button| button.enabled && button.rect.contains(point))
    {
        super::bevel::draw_pressed(assets, button.rect);
    }
    controls.extend(buttons);
    crate::combat_controls::draw_automatic(state, assets, controls);
    if let Some(playback) = &ui.playback {
        super::spell_panel::draw(assets, playback);
    }
}

/// The outdoor control strip: the context button (clinic, shop or crate)
/// and the field buttons, greyed when unavailable.
fn field_controls_strip(
    state: &Session,
    assets: &Assets,
    selected: usize,
    controls: &mut Vec<Control>,
) {
    let context = assets.main_controls[&control::REPAIR];
    if state
        .field_controls
        .as_ref()
        .and_then(|c| c.clinic)
        .is_some()
    {
        assets.panel(200, context);
    } else if state.shop_access.is_some() {
        assets.panel(225, context);
    } else {
        assets.panel(208, context);
    }
    for (id, action) in crate::input::field_buttons(state) {
        let rect = assets.main_controls[&id];
        let enabled = crate::input::available(state, selected, &action);
        if !enabled {
            assets.panel(layout::DISABLED_CONTROL_PICT, rect);
        }
        controls.push(Control {
            rect,
            enabled,
            action,
        });
    }
    if state.warriors.len() < 2 {
        assets.panel(
            layout::DISABLED_CONTROL_PICT,
            assets.main_controls[&control::TRADE],
        );
    }
}

/// The open scenario presentation in the message area, followed by a
/// Continue button for the presentations that wait on one.
fn draw_story_presentation(
    content: &Content,
    state: &Session,
    assets: &Assets,
    ui: &UiState,
    controls: &mut Vec<Control>,
) {
    let Some(p) = state
        .exploration
        .as_ref()
        .and_then(|p| p.presentation.as_ref())
    else {
        return;
    };
    let needs_continue = match p {
        Presentation::Messages { pages, .. } => {
            if state.phase == Phase::Combat {
                assets.panel(221, layout::MESSAGE);
            }
            controls.clear();
            if let Some((_, text)) = pages.first() {
                assets.wrapped(
                    &authored_text(text, &state.warriors),
                    layout::MESSAGE_TEXT,
                    12,
                    16.,
                );
            }
            false
        }
        Presentation::Dialogue { text, .. } => {
            // textbox replaces the combat panel with PICT 221.
            if state.phase == Phase::Combat {
                assets.panel(221, layout::MESSAGE);
            }
            controls.clear();
            assets.wrapped(
                &authored_text(text, &state.warriors),
                layout::MESSAGE_TEXT,
                12,
                16.,
            );
            false
        }
        Presentation::Question { text, .. } | Presentation::Avoidance { text, .. } => {
            crate::story_dialogs::question(
                assets,
                &authored_text(text, &state.warriors),
                Action::Command(Command::Choose { answer: true }),
                Action::Command(Command::Choose { answer: false }),
                controls,
            );
            false
        }
        Presentation::BoatQuestion { boarding } => {
            crate::story_dialogs::question(
                assets,
                &assets.questions[if *boarding { 14 } else { 13 }],
                Action::Command(Command::Choose { answer: true }),
                Action::Command(Command::Choose { answer: false }),
                controls,
            );
            false
        }
        Presentation::Menu { id, text, labels } => {
            crate::story_dialogs::menu(
                content,
                state,
                assets,
                *id,
                &authored_text(text, &state.warriors),
                labels,
                controls,
            );
            false
        }
        Presentation::WarriorChoice { .. } => {
            controls.clear();
            crate::story_dialogs::portraits(state, assets, controls, |index| {
                (state.warriors[index].stamina().unwrap_or(0) > 0)
                    .then_some(Action::Command(Command::ChooseWarrior { index }))
            });
            false
        }
        Presentation::PartyPick { unconscious, .. } => {
            controls.clear();
            crate::story_dialogs::portraits(state, assets, controls, |index| {
                (*unconscious || state.warriors[index].stamina().unwrap_or(0) > 0)
                    .then_some(Action::Command(Command::ChooseWarrior { index }))
            });
            // getchoice draws each pick's countdown number, cicn 2200 + n.
            for (member, pick) in state.globals.party_track.iter().enumerate() {
                if let Some(icon) = (*pick > 0)
                    .then(|| assets.icons.get(&(2200 + pick)))
                    .flatten()
                {
                    draw_texture(icon, 499., 29. + member as f32 * 50., WHITE);
                }
            }
            false
        }
        Presentation::StageChange { .. } => {
            crate::story_dialogs::question(
                assets,
                assets.questions.get(7).map_or("", String::as_str),
                Action::Command(Command::Choose { answer: true }),
                Action::Command(Command::Choose { answer: false }),
                controls,
            );
            false
        }
        Presentation::Prologue { .. } => true,
        Presentation::Typed { text, .. } => {
            crate::story_dialogs::typed(
                content,
                state,
                assets,
                ui,
                &authored_text(text, &state.warriors),
                controls,
            );
            false
        }
        Presentation::Loot | Presentation::Shop => true,
        Presentation::Clinic { .. } => false,
        Presentation::Survivors | Presentation::ScriptPause => false,
        Presentation::ClinicEntry { .. } => true,
    };
    if needs_continue {
        let keep_spells =
            matches!(p, Presentation::Loot) && centurions_core::needs_spell_confirmation(state);
        button(
            assets,
            controls,
            "Continue",
            layout::CONTINUE,
            !state.booty_list.iter().any(|i| i.id() != 0),
            Action::Command(if keep_spells {
                Command::BeginPromotion
            } else {
                Command::Continue
            }),
        );
    }
}

/// A screen that replaces the game's controls: shop, collection, clinic,
/// survivors, encounter actions or an inspection page. Returns true when the
/// frame's controls are complete.
fn draw_overlay(
    content: &Content,
    state: &Session,
    assets: &Assets,
    ui: &UiState,
    controls: &mut Vec<Control>,
) -> bool {
    if state
        .exploration
        .as_ref()
        .is_some_and(|p| p.presentation == Some(Presentation::Shop))
    {
        ui.inventory.draw(content, state, assets, controls);
        return true;
    }
    if crate::collection::active(state, ui) && state.promotion.is_none() {
        crate::collection::draw(content, state, assets, ui, controls);
        if ui.inventory.loot_information_open() {
            ui.inventory
                .draw_loot_information(content, state, assets, controls);
        }
        return true;
    }
    if let Some(Presentation::Clinic { member, percent }) = state
        .exploration
        .as_ref()
        .and_then(|p| p.presentation.as_ref())
    {
        controls.clear();
        draw_clinic(state, assets, *member, *percent, controls);
        if ui.inspection.is_none() {
            return true;
        }
    }
    if state
        .exploration
        .as_ref()
        .is_some_and(|position| position.presentation == Some(Presentation::Survivors))
    {
        controls.clear();
        draw_survivors(state, assets, controls);
        return true;
    }
    if let Some(selected) = state.encounter_actions {
        draw_encounter_actions(content, state, assets, selected, controls);
        return true;
    }
    if let Some(inspection) = ui.inspection {
        if crate::status_popup::active(ui) {
            controls.clear();
            crate::status_popup::draw(state, assets, ui, controls);
            return true;
        }
        if inspection == Inspection::Specials {
            return true;
        }
        if inspection == Inspection::Items {
            ui.inventory.draw(content, state, assets, controls);
            return true;
        }
        if inspection == Inspection::Weapons {
            controls.clear();
            crate::combat_controls::draw_weapons(state, assets, ui, controls);
            return true;
        }
        if inspection == Inspection::Psi {
            let inspected = crate::acting::acting_warrior(state, ui.selected);
            crate::scan_picker::draw(content, state, assets, ui, inspected, controls);
            return true;
        }
    }
    false
}

/// Dialog 148, the physical-action selection with its checkboxes.
fn draw_encounter_actions(
    content: &Content,
    state: &Session,
    assets: &Assets,
    selected: [bool; 8],
    controls: &mut Vec<Control>,
) {
    controls.clear();
    assets.draw_dialog(148);
    if let Ok(record) = centurions_core::typed_record(state, content) {
        for (index, chosen) in selected.into_iter().enumerate() {
            assets.panel(156, assets.dialog_rect(148, index + 9));
            let checkbox = assets.dialog_rect(148, index + 1);
            assets.panel(179, checkbox);
            if chosen {
                draw_rectangle(
                    checkbox.x + 2.,
                    checkbox.y + 2.,
                    checkbox.w - 4.,
                    checkbox.h - 4.,
                    YELLOW,
                );
            }
            let label = TypedEncounterRecord(&record).action_label(index);
            let text =
                centurions_content::ui::c_string(&label[1..1 + usize::from(label[0]).min(39)]);
            assets.dialog_text(&text, assets.dialog_rect(148, index + 19));
        }
    }
    for item in 9..=18 {
        controls.push(Control {
            rect: assets.dialog_rect(148, item),
            enabled: true,
            action: Action::Command(Command::EncounterActionControl { item: item as u16 }),
        });
    }
}
