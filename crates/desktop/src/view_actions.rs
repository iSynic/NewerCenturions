//! Viewing and target-selection actions emitted by the shared input router.
use crate::{
    application::Application,
    audio::Audio,
    presentation::{self, Action, UiState},
    status_popup, targeting,
};
use centurions_content::Content;
use centurions_core::{Command, Event, Session, apply};

/// False preserves the original early frame skip after a refused selection.
pub fn handle(
    action: Action,
    state: &mut Session,
    content: &Content,
    application: &mut Application,
    ui: &mut UiState,
    audio: Option<&Audio>,
) -> bool {
    match action {
        Action::CombatView(view) => {
            ui.combat_view.change(state, view);
            // showrange 0x1583e6: sound(0x89) once the lines are drawn.
            if matches!(view, crate::combat_controls::ViewAction::Reveal)
                && ui.combat_view.reveal
                && let Some(audio) = audio
            {
                audio.play(&[Event::Sound { resource_id: 0x89 }]);
            }
        }
        Action::PanCombat(x, y) => ui.combat_view.pan(state, (x, y)),
        Action::Select(i) => {
            ui.selected = i;
            ui.page = 0;
            if ui.inspection == Some(presentation::Inspection::Psi) {
                ui.psi_choice = None;
            }
        }
        Action::InformationInspection(member, inspection) => {
            if inspection == presentation::Inspection::Items
                && let Err(error) =
                    enter_items(state, content, application, ui, member, false, audio)
            {
                ui.failure(error);
                return false;
            }
            ui.selected = member;
            ui.return_information = Some(member);
            if inspection == presentation::Inspection::FireTargets {
                if let Err(error) = apply(Command::BeginFireTargets, state, content) {
                    ui.failure(error);
                    return false;
                }
                ui.estimate_target = None;
            }
            ui.inspection = Some(inspection);
            ui.page = 0;
            if inspection == presentation::Inspection::Psi {
                let actor = state
                    .combat
                    .as_ref()
                    .and_then(|arena| arena.active_body())
                    .filter(|body| (0..state.warriors.len() as i16).contains(body))
                    .map_or(ui.selected, |body| body as usize);
                if let Some(choice) = &state.psi_menu_choices[actor] {
                    ui.page = usize::from(choice.row).min(6);
                    let school = state.warriors[actor].record().psi_school().unwrap_or(0);
                    let spell =
                        school * 1000 + (ui.page as i16 + 1) * 100 + i16::from(choice.column) + 1;
                    if state.warriors[actor].knows_spell(spell).unwrap_or(false) {
                        ui.psi_choice = Some((spell, ui.psi_power.clamp(1, 7)));
                    }
                }
            }
        }
        Action::Open(inspection) => {
            if matches!(
                inspection,
                presentation::Inspection::Conditions | presentation::Inspection::Modifiers
            ) && application.sound
                && let Some(audio) = audio
            {
                audio.play(&[Event::Sound { resource_id: 609 }]);
            }
            if let Some(centurions_core::Presentation::Clinic { member, .. }) = state
                .exploration
                .as_ref()
                .and_then(|p| p.presentation.as_ref())
            {
                ui.selected = *member;
            }
            if inspection == presentation::Inspection::Items
                && state.phase == centurions_core::Phase::Combat
                && let Some(body) = state
                    .combat
                    .as_ref()
                    .and_then(|arena| arena.active_body())
                    .filter(|body| (0..6).contains(body))
            {
                ui.selected = body as usize;
            }
            if inspection == presentation::Inspection::Psi {
                ui.psi_choice = None;
                let actor = state
                    .combat
                    .as_ref()
                    .and_then(|arena| arena.active_body())
                    .filter(|body| (0..6).contains(body))
                    .map_or(ui.selected, |body| body as usize);
                if let Some(choice) = state.psi_menu_choices[actor].as_ref() {
                    ui.psi_power = choice.power.clamp(1, 7);
                }
            }
            if inspection == presentation::Inspection::FireTargets {
                if let Err(error) = apply(Command::BeginFireTargets, state, content) {
                    ui.failure(error);
                    return false;
                }
                ui.estimate_target = None;
            }
            let owner = ui.selected;
            let item_button = state.phase == centurions_core::Phase::Combat;
            if inspection == presentation::Inspection::Items
                && let Err(error) =
                    enter_items(state, content, application, ui, owner, item_button, audio)
            {
                ui.failure(error);
                return false;
            }
            ui.inspection = Some(inspection);
            ui.page = 0;
            if inspection == presentation::Inspection::Psi {
                let actor = state
                    .combat
                    .as_ref()
                    .and_then(|arena| arena.active_body())
                    .filter(|body| (0..state.warriors.len() as i16).contains(body))
                    .map_or(ui.selected, |body| body as usize);
                if let Some(choice) = &state.psi_menu_choices[actor] {
                    ui.page = usize::from(choice.row).min(6);
                    let school = state.warriors[actor].record().psi_school().unwrap_or(0);
                    let spell =
                        school * 1000 + (ui.page as i16 + 1) * 100 + i16::from(choice.column) + 1;
                    if state.warriors[actor].knows_spell(spell).unwrap_or(false) {
                        ui.psi_choice = Some((spell, ui.psi_power.clamp(1, 7)));
                    }
                }
            }
        }
        Action::Close => {
            if (status_popup::active(ui)
                || ui.inspection == Some(presentation::Inspection::Specials))
                && application.sound
                && let Some(audio) = audio
            {
                audio.play(&[Event::Sound {
                    resource_id: if status_popup::active(ui) { 607 } else { 141 },
                }]);
            }
            ui.firearm_map_target = false;
            if state
                .combat
                .as_ref()
                .is_some_and(|c| c.firearm_selection.is_some())
                || state.target_quantity.is_some()
            {
                match apply(Command::CancelFireTargets, state, content) {
                    Ok(events) => application.observe(&events),
                    Err(error) => ui.failure(error),
                }
            }
            ui.inspection = None;
            ui.psi_pending = None;
            ui.psi_choice = None;
            if let Some(member) = ui.return_information.take() {
                application.restore_information(member);
            }
        }
        Action::Page(change) => {
            ui.page = (ui.page as i32 + change).max(0) as usize;
            if ui.inspection == Some(presentation::Inspection::Psi) {
                ui.page = ui.page.min(6);
                ui.psi_choice = None;
            }
        }
        Action::Inventory(_) | Action::Collection(_) => unreachable!(),
        Action::TargetQuantity(increase) => match targeting::adjust(state, content, ui, increase) {
            Ok(events) => {
                if let Some(pending) = &state.psi_selection {
                    ui.psi_power = pending.power;
                    if ui.psi_choice.is_some() {
                        ui.psi_choice = Some((pending.spell, pending.power));
                        ui.psi_pending = ui.psi_choice;
                    }
                }
                ui.events(&events);
                application.observe(&events);
                if let Some(audio) = audio {
                    audio.play(&events);
                }
            }
            Err(error) => ui.failure(error),
        },
        Action::PsiPower(power) => {
            ui.psi_power = power.clamp(1, 7);
            if let Some(choice) = &mut ui.psi_choice {
                choice.1 = ui.psi_power;
            }
        }
        Action::ChoosePsi(spell) => {
            let power = ui.psi_power.clamp(1, 7);
            match apply(Command::BeginPsiTargeting { spell, power }, state, content) {
                Ok(events) => {
                    ui.events(&events);
                    application.observe(&events);
                }
                Err(error) => {
                    ui.failure(error);
                    return false;
                }
            }
            ui.psi_rotation = 0;
            ui.psi_preview = None;
            ui.estimate_target = None;
            ui.psi_choice = Some((spell, power));
            ui.psi_pending = Some((spell, power));
            ui.inspection = Some(presentation::Inspection::PsiTargets);
            ui.page = 0;
        }
        Action::RotatePsi => {
            ui.psi_rotation = (ui.psi_rotation + 1) % 4;
            ui.feedback
                .push_back(centurions_core::feedback::NativeFeedback::Sound { resource_id: 130 });
        }
        Action::TextEntry(open) => {
            ui.typed_entry = open;
            ui.typed_text.clear();
        }
        Action::PreviewPsi(spell) => {
            ui.psi_choice = Some((spell, ui.psi_power.clamp(1, 7)));
            ui.feedback
                .push_back(centurions_core::feedback::NativeFeedback::Sound { resource_id: 141 });
        }
        Action::MedicalCancel => ui.medical_cancel = true,
        Action::KeepMedicalTarget => ui.medical_cancel = false,
        _ => unreachable!("non-viewing action reached the viewing controller"),
    }
    true
}

fn enter_items(
    state: &mut Session,
    content: &Content,
    application: &mut Application,
    ui: &mut UiState,
    owner: usize,
    button_sound: bool,
    audio: Option<&Audio>,
) -> Result<(), centurions_core::RuleError> {
    let events = if crate::inventory::Inventory::shop(state) {
        ui.inventory.open(owner, state.warriors.len());
        apply(ui.inventory.context_command(), state, content)?
    } else {
        let quick_info = application.native_preferences.enabled(22);
        let events = match apply(
            Command::OpenInventory {
                warrior: owner,
                quick_info,
            },
            state,
            content,
        ) {
            Ok(events) => events,
            Err(error) => {
                if ui.inspection == Some(presentation::Inspection::Items) {
                    ui.inspection = None;
                }
                return Err(error);
            }
        };
        ui.inventory.open(owner, state.warriors.len());
        ui.inventory.quick_info = quick_info;
        events
    };
    ui.events(&events);
    application.observe(&events);
    if let Some(audio) = audio {
        if button_sound {
            audio.play(&[Event::Sound { resource_id: 141 }]);
        }
        audio.play(&events);
    }
    Ok(())
}
