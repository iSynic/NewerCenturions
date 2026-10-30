//! Viewing and target-selection actions emitted by the shared input router.
use crate::{
    application::Application,
    audio::Audio,
    presentation::{self, Action, UiState},
    status_popup, targeting,
};
use centurions_content::Content;
use centurions_core::{Command, Event, Session, apply};

/// Starts firearm target selection; false after reporting a refusal.
fn begin_fire_targets(state: &mut Session, content: &Content, ui: &mut UiState) -> bool {
    if let Err(error) = apply(Command::BeginFireTargets, state, content) {
        ui.failure(error);
        return false;
    }
    ui.targeting.estimate_target = None;
    true
}

/// Reopens the acting warrior's remembered Psi menu row and known spell.
fn restore_psi_choice(state: &Session, ui: &mut UiState) {
    let actor = crate::acting::acting_warrior(state, ui.selected);
    if let Some(choice) = &state.psi_menu_choices[actor] {
        ui.page = usize::from(choice.row).min(6);
        let school = state.warriors[actor].record().psi_school().unwrap_or(0);
        let spell = school * 1000 + (ui.page as i16 + 1) * 100 + i16::from(choice.column) + 1;
        if state.warriors[actor].knows_spell(spell).unwrap_or(false) {
            ui.psi.choice = Some((spell, ui.psi.power.clamp(1, 7)));
        }
    }
}

/// Returns false after a refused selection, which skips the rest of the frame.
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
            // showrange plays sound 0x89 once the lines are drawn.
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
                ui.psi.choice = None;
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
            if inspection == presentation::Inspection::FireTargets
                && !begin_fire_targets(state, content, ui)
            {
                return false;
            }
            ui.inspection = Some(inspection);
            ui.page = 0;
            if inspection == presentation::Inspection::Psi {
                restore_psi_choice(state, ui);
            }
        }
        Action::Open(inspection) => {
            if matches!(
                inspection,
                presentation::Inspection::Conditions | presentation::Inspection::Modifiers
            ) && application.audio.sound
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
                && let Some(body) = crate::acting::active_slot(state)
            {
                ui.selected = body as usize;
            }
            if inspection == presentation::Inspection::Psi {
                ui.psi.choice = None;
                let actor = crate::acting::acting_slot(state, ui.selected);
                if let Some(choice) = state.psi_menu_choices[actor].as_ref() {
                    ui.psi.power = choice.power.clamp(1, 7);
                }
            }
            if inspection == presentation::Inspection::FireTargets
                && !begin_fire_targets(state, content, ui)
            {
                return false;
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
                restore_psi_choice(state, ui);
            }
        }
        Action::Close => {
            if (status_popup::active(ui)
                || ui.inspection == Some(presentation::Inspection::Specials))
                && application.audio.sound
                && let Some(audio) = audio
            {
                audio.play(&[Event::Sound {
                    resource_id: if status_popup::active(ui) { 607 } else { 141 },
                }]);
            }
            ui.targeting.firearm_map = false;
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
            ui.psi.pending = None;
            ui.psi.choice = None;
            if let Some(member) = ui.return_information.take() {
                application.restore_information(member);
            }
        }
        Action::Page(change) => {
            ui.page = (ui.page as i32 + change).max(0) as usize;
            if ui.inspection == Some(presentation::Inspection::Psi) {
                ui.page = ui.page.min(6);
                ui.psi.choice = None;
            }
        }
        Action::Inventory(_) | Action::Collection(_) => unreachable!(),
        Action::TargetQuantity(increase) => match targeting::adjust(state, content, ui, increase) {
            Ok(events) => {
                if let Some(pending) = &state.psi_selection {
                    ui.psi.power = pending.power;
                    if ui.psi.choice.is_some() {
                        ui.psi.choice = Some((pending.spell, pending.power));
                        ui.psi.pending = ui.psi.choice;
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
            ui.psi.power = power.clamp(1, 7);
            if let Some(choice) = &mut ui.psi.choice {
                choice.1 = ui.psi.power;
            }
        }
        Action::ChoosePsi(spell) => {
            let power = ui.psi.power.clamp(1, 7);
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
            ui.psi.rotation = 0;
            ui.psi.preview = None;
            ui.targeting.estimate_target = None;
            ui.psi.choice = Some((spell, power));
            ui.psi.pending = Some((spell, power));
            ui.inspection = Some(presentation::Inspection::PsiTargets);
            ui.page = 0;
        }
        Action::RotatePsi => {
            ui.psi.rotation = (ui.psi.rotation + 1) % 4;
            ui.feedback
                .queue
                .push_back(centurions_core::feedback::GameFeedback::Sound { resource_id: 130 });
        }
        Action::TextEntry(open) => {
            ui.typed_entry = open;
            ui.typed_text.clear();
        }
        Action::PreviewPsi(spell) => {
            ui.psi.choice = Some((spell, ui.psi.power.clamp(1, 7)));
            ui.feedback
                .queue
                .push_back(centurions_core::feedback::GameFeedback::Sound { resource_id: 141 });
        }
        Action::MedicalCancel => ui.targeting.medical_cancel = true,
        Action::KeepMedicalTarget => ui.targeting.medical_cancel = false,
        _ => unreachable!("non-viewing action reached the viewing controller"),
    }
    true
}

/// Opens the Items screen for warrior `owner`, in a shop or the field.
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
        let quick_info = application
            .preferences
            .enabled(crate::preferences::pref::SCREEN_800_X_600);
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
