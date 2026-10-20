//! Scene presentation.
use super::{
    Action, Control, Inspection, UiState, authored_text, button, draw_clinic,
    draw_combat_information, draw_outdoor_effect, draw_survivors, hud, map,
};
use crate::{assets::Assets, layout};
use centurions_content::Content;
use centurions_core::{Command, Phase, Presentation, Session};
use macroquad::prelude::*;

pub fn draw(content: &Content, state: &Session, assets: &Assets, ui: &UiState) -> Vec<Control> {
    clear_background(Color::from_rgba(41, 42, 0, 255));
    let mut controls = Vec::new();
    map(content, state, assets, ui);
    draw_outdoor_effect(state, assets, ui);
    if let (Some((spell, power)), Some(point), Some((vx, vy))) = (
        ui.psi_pending,
        ui.psi_preview,
        crate::combat_controls::origin(state, ui),
    ) && let Some(mission) = content.map(state.exploration.as_ref().map_or(0, |p| p.map))
        && let Some(tables) = mission.combat.as_ref()
        && let Ok(packet) = centurions_core::combat::spells::load_packet(&tables.attacks, spell)
    {
        let shape = match packet[1] {
            3 => {
                i16::from(packet[31])
                    + if packet[7] != 0 {
                        i16::from(ui.psi_rotation)
                    } else {
                        0
                    }
            }
            4 => power,
            _ => 1,
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
        let points = i32::from_be_bytes(warrior.bytes[0x9d2..0x9d6].try_into().unwrap());
        assets.text(&points.to_string(), 215., 528., 14, YELLOW);
    }
    if state.phase == Phase::Combat {
        assets.panel(153, Rect::new(468., 449., 332., 89.));
        assets.panel(156, Rect::new(568., 538., 232., 50.));
    } else {
        assets.panel(176, Rect::new(468., 449., 332., 139.));
    }
    assets.panel(224, layout::INFORMATION);
    let selected = hud::roster(state, assets, ui, &mut controls);
    hud::clock(state, assets);
    hud::information(state, assets, selected);
    let mission_ready = state.phase != Phase::SquadSelection;
    let (items_rect, conditions_rect) = if state.phase == Phase::Combat {
        (layout::COMBAT_ITEMS, layout::COMBAT_CONDITIONS)
    } else {
        (assets.main_controls[&168], assets.main_controls[&169])
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
    if state.phase != Phase::SquadSelection {
        if state.phase == Phase::Combat {
            let capture =
                crate::input::available(state, selected, &Action::Command(Command::BeginCapture));
            assets.panel(if capture { 161 } else { 154 }, assets.main_controls[&301]);
            if !crate::input::available(state, selected, &Action::Open(Inspection::Psi)) {
                assets.panel(154, assets.main_controls[&300]);
            }
            let mut buttons = crate::combat_controls::buttons(state, content, selected);
            for button in &mut buttons {
                match button.action {
                    Action::Open(Inspection::Weapons) => button.rect = assets.main_controls[&166],
                    Action::Application(
                        crate::application::ApplicationAction::CombatInformation(_),
                    ) => button.rect = assets.main_controls[&167],
                    _ => {}
                }
            }
            if let Some(body) = ui.combat_view.selected(state) {
                for button in &mut buttons {
                    if let Action::Application(
                        crate::application::ApplicationAction::CombatInformation(selected),
                    ) = &mut button.action
                    {
                        *selected = body;
                    }
                }
            }
            controls.extend(buttons);
            crate::combat_controls::draw_automatic(state, assets, &mut controls);
        }
        if state.phase == Phase::Exploration {
            let context = assets.main_controls[&301];
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
                    assets.panel(154, rect);
                }
                controls.push(Control {
                    rect,
                    enabled,
                    action,
                });
            }
            if state.warriors.len() < 2 {
                assets.panel(154, assets.main_controls[&305]);
            }
        }
        if let Some(p) = state
            .exploration
            .as_ref()
            .and_then(|p| p.presentation.as_ref())
        {
            match p {
                Presentation::Messages { pages, .. } => {
                    if state.phase == Phase::Combat {
                        assets.panel(221, layout::MESSAGE);
                    }
                    controls.clear();
                    if let Some((_, text)) = pages.first() {
                        assets.wrapped(
                            &authored_text(text, &state.warriors),
                            Rect::new(12., 467., 444., 82.),
                            12,
                            16.,
                        );
                    }
                }
                Presentation::Dialogue { text, .. } => {
                    // textbox15deac replaces the combat panel with PICT221.
                    if state.phase == Phase::Combat {
                        assets.panel(221, layout::MESSAGE);
                    }
                    controls.clear();
                    assets.wrapped(
                        &authored_text(text, &state.warriors),
                        Rect::new(12., 467., 444., 82.),
                        12,
                        16.,
                    );
                }
                Presentation::Question { text, .. } | Presentation::Avoidance { text, .. } => {
                    crate::story_dialogs::question(
                        assets,
                        &authored_text(text, &state.warriors),
                        Action::Command(Command::Choose { answer: true }),
                        Action::Command(Command::Choose { answer: false }),
                        &mut controls,
                    );
                }
                Presentation::Menu { id, text, labels } => {
                    crate::story_dialogs::menu(
                        content,
                        state,
                        assets,
                        *id,
                        &authored_text(text, &state.warriors),
                        labels,
                        &mut controls,
                    );
                }
                Presentation::WarriorChoice { .. } => {
                    controls.clear();
                    crate::story_dialogs::portraits(state, assets, &mut controls, |index| {
                        (state.warriors[index].stamina().unwrap_or(0) > 0)
                            .then_some(Action::Command(Command::ChooseWarrior { index }))
                    });
                }
                Presentation::StageChange { .. } => {
                    crate::story_dialogs::question(
                        assets,
                        assets.questions.get(7).map_or("", String::as_str),
                        Action::Command(Command::Choose { answer: true }),
                        Action::Command(Command::Choose { answer: false }),
                        &mut controls,
                    );
                }
                Presentation::Prologue { .. } => {}
                Presentation::Typed { text, .. } => {
                    crate::story_dialogs::typed(
                        content,
                        state,
                        assets,
                        ui,
                        &authored_text(text, &state.warriors),
                        &mut controls,
                    );
                }
                Presentation::Loot | Presentation::Shop | Presentation::Clinic { .. } => {}
                Presentation::Survivors | Presentation::ScriptPause => {}
            }
            if !matches!(
                p,
                Presentation::Question { .. }
                    | Presentation::Dialogue { .. }
                    | Presentation::Messages { .. }
                    | Presentation::Avoidance { .. }
                    | Presentation::Menu { .. }
                    | Presentation::Typed { .. }
                    | Presentation::StageChange { .. }
                    | Presentation::WarriorChoice { .. }
                    | Presentation::Clinic { .. }
                    | Presentation::Survivors
                    | Presentation::ScriptPause
            ) {
                let keep_spells = matches!(p, Presentation::Loot)
                    && centurions_core::needs_spell_confirmation(state);
                button(
                    assets,
                    &mut controls,
                    "Continue",
                    layout::CONTINUE,
                    !state.booty_items.iter().any(|i| i.id() != 0),
                    Action::Command(if keep_spells {
                        Command::BeginPromotion
                    } else {
                        Command::Continue
                    }),
                );
            }
        }
    }
    if state
        .exploration
        .as_ref()
        .is_some_and(|p| p.presentation == Some(Presentation::Shop))
    {
        ui.inventory.draw(content, state, assets, &mut controls);
        return controls;
    }
    if crate::collection::active(state, ui) && state.promotion.is_none() {
        crate::collection::draw(content, state, assets, ui, &mut controls);
        return controls;
    }
    if let Some(Presentation::Clinic { member, percent }) = state
        .exploration
        .as_ref()
        .and_then(|p| p.presentation.as_ref())
    {
        controls.clear();
        draw_clinic(state, assets, *member, *percent, &mut controls);
        if ui.inspection.is_none() {
            return controls;
        }
    }
    if state
        .exploration
        .as_ref()
        .is_some_and(|position| position.presentation == Some(Presentation::Survivors))
    {
        controls.clear();
        draw_survivors(state, assets, &mut controls);
        return controls;
    }
    if let Some(selected) = state.encounter_actions {
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
                let label = &record[156 + index * 40..196 + index * 40];
                let text =
                    centurions_content::ui::c_string(&label[1..1 + usize::from(label[0]).min(39)]);
                assets.wrapped_color(&text, assets.dialog_rect(148, index + 19), 12, 14., YELLOW);
            }
        }
        for item in 9..=18 {
            controls.push(Control {
                rect: assets.dialog_rect(148, item),
                enabled: true,
                action: Action::Command(Command::EncounterActionControl { item: item as u16 }),
            });
        }
        return controls;
    }
    if let Some(inspection) = ui.inspection {
        if crate::status_popup::active(ui) {
            controls.clear();
            crate::status_popup::draw(state, assets, ui, &mut controls);
            return controls;
        }
        if inspection == Inspection::Specials {
            return controls;
        }
        if inspection == Inspection::Items {
            ui.inventory.draw(content, state, assets, &mut controls);
            return controls;
        }
        if inspection == Inspection::Weapons {
            controls.clear();
            crate::combat_controls::draw_weapons(state, assets, ui, &mut controls);
            return controls;
        }
        if inspection == Inspection::Psi {
            let inspected = state
                .combat
                .as_ref()
                .and_then(|c| c.active_body())
                .filter(|body| (0..state.warriors.len() as i16).contains(body))
                .map_or(ui.selected, |body| body as usize);
            crate::scan_picker::draw(content, state, assets, ui, inspected, &mut controls);
            return controls;
        }
    }
    if let Some(progress) = &state.promotion {
        controls.clear();
        crate::editor::ScanEditor::draw_promotion(progress, content, assets, &mut controls);
    }
    if crate::targeting::active(state, ui) {
        crate::targeting::draw(content, state, assets, ui, &mut controls);
    }
    for control in &mut controls {
        control.enabled &= crate::input::available(state, ui.selected, &control.action);
    }
    controls
}
