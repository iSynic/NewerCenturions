//! What the desktop does after a command, chosen by the input that issued it.
use crate::{
    application::Application,
    audio::Audio,
    presentation::{self, UiState},
};
use centurions_content::Content;
use centurions_core::{Command, Event, Session, apply};

/// Returns whether automatic scheduling must stop after a failed action.
pub fn execute(
    command: Command,
    state: &mut Session,
    content: &Content,
    application: &mut Application,
    ui: &mut UiState,
    audio: Option<&Audio>,
    automatic_turn: bool,
) -> bool {
    let input = InputEffects::of(&command);
    let previous_round = state.combat.as_ref().map(|c| c.combat_round);
    let previous_session = state.combat.is_some().then(|| state.clone());
    let previous_view = crate::combat_controls::origin(state, ui);
    match apply(application.command(command), state, content) {
        Ok(mut events) => {
            present_success(
                &input,
                state,
                application,
                ui,
                audio,
                (previous_round, previous_session, previous_view),
                &mut events,
            );
            false
        }
        Err(error) => {
            if let Some(action) = input.field_control {
                ui.field_repeat = Some((action, f64::INFINITY));
            }
            ui.failure(error);
            input.enemy || automatic_turn
        }
    }
}

/// Settles the screen after a command succeeds: clears the entry and selection
/// state the command finished, queues the round notices, builds the playback of
/// the combat steps and plays the sounds.
fn present_success(
    input: &InputEffects,
    state: &Session,
    application: &mut Application,
    ui: &mut UiState,
    audio: Option<&Audio>,
    (previous_round, previous_session, previous_view): (
        Option<i16>,
        Option<Session>,
        Option<(i16, i16)>,
    ),
    events: &mut Vec<Event>,
) {
    if input.text_entry {
        ui.typed_entry = false;
        ui.typed_text.clear();
    }
    if input.collection {
        ui.collection.slot = None;
        ui.collection.drag = false;
    }
    if input.shot
        && events
            .iter()
            .any(|event| matches!(event, Event::AttackResolved { .. }))
    {
        ui.targeting.firearm_map = false;
    }
    application.prepare_combat_round(previous_round, state, events);
    if input.movement
        && !events
            .iter()
            .any(|event| matches!(event, Event::TacticalMoved { .. } | Event::Moved { .. }))
    {
        ui.movement.refuse();
    }
    if input.medical {
        ui.medical_result(state, events);
    }
    if input.psi {
        ui.inspection = None;
        ui.psi.pending = None;
        ui.psi.choice = None;
        ui.targeting.medical_cancel = false;
        if state.psi_selection.is_some() {
            ui.page = 0;
        }
    }
    if state
        .combat
        .as_ref()
        .is_some_and(|c| c.firearm_selection.is_some())
    {
        ui.inspection = Some(presentation::Inspection::FireTargets);
    } else if input.shot && ui.inspection == Some(presentation::Inspection::FireTargets) {
        ui.inspection = None;
    }
    if input.weapon {
        ui.inspection = None;
    }
    let ended = events
        .iter()
        .any(|event| matches!(event, Event::MissionEnded { .. }));
    // The field shown is the one before the command until its steps have played.
    let playback = previous_session
        .filter(|_| !ended && state.combat.is_some())
        .and_then(|session| {
            crate::combat_playback::Playback::build(
                session,
                events,
                &crate::combat_playback::Settings {
                    speed: application.speed as i16,
                    effect: state.globals.health_context.effect,
                    targeting: state
                        .combat
                        .as_ref()
                        .and_then(|combat| combat.spell_info.get(1).copied())
                        .unwrap_or(0),
                    short_walks: ui
                        .preferences
                        .enabled(crate::preferences::pref::FASTER_SCAN_CASTING),
                    batched_effects: ui
                        .preferences
                        .enabled(crate::preferences::pref::FASTER_SCAN_RESOLUTION),
                    item_value: state
                        .globals
                        .loaded_items
                        .item
                        .as_deref()
                        .and_then(|item| item.get(0x30..0x32))
                        .map_or(0, |word| i16::from_be_bytes([word[0], word[1]])),
                },
            )
        });
    let presented: &[Event] = playback
        .as_ref()
        .map_or(events.as_slice(), |(_, immediate)| immediate);
    ui.events(presented);
    application.observe(events);
    if ended {
        ui.selected = 0;
        ui.inspection = None;
        ui.effects.clear();
        ui.movement.reset();
        ui.psi.pending = None;
        ui.psi.choice = None;
    }
    if let Some(audio) = audio {
        if input.field_control.is_some() && ui.field_repeating {
            // buttonchoice plays the loop's entry sound once per held gesture.
            audio.play(presented.iter().filter(|event| {
                !matches!(
                    event,
                    Event::Sound {
                        resource_id: 6001 | 10105
                    }
                )
            }));
        } else {
            audio.play(presented.iter());
        }
    }
    if let Some((mut playback, _)) = playback {
        playback.view = previous_view;
        ui.playback = Some(playback);
    }
}

/// What the kind of command that was issued does to the screen's input state
/// when it succeeds or fails.
struct InputEffects {
    text_entry: bool,
    collection: bool,
    weapon: bool,
    enemy: bool,
    psi: bool,
    shot: bool,
    medical: bool,
    movement: bool,
    field_control: Option<centurions_core::field_controls::Action>,
}
impl InputEffects {
    /// Classifies `command` into the input effects it has.
    fn of(command: &Command) -> Self {
        let text_entry = matches!(command, Command::SubmitText { .. } | Command::CancelMenu);
        let collection = matches!(
            command,
            Command::TakeLoot { .. } | Command::StoreLoot { .. } | Command::TakeCrate { .. }
        );
        let weapon = matches!(command, Command::SelectCombatWeapon { .. });
        let enemy = matches!(command, Command::RunEnemyTurn);
        let psi = matches!(
            command,
            Command::CastPsi { .. }
                | Command::CastPsiOnWarrior { .. }
                | Command::CastEncounterPsi { .. }
                | Command::CastPartyPsi { .. }
                | Command::BeginPsiSelection { .. }
                | Command::FirePreparedPsi { .. }
                | Command::FirePreparedPsiOnWarrior { .. }
                | Command::FirePsiSelection
                | Command::CancelPsiSelection
                | Command::SelectOutdoorPsiTarget { .. }
                | Command::FinishOutdoorPsi
                | Command::CancelOutdoorPsi
        );
        let shot = matches!(
            command,
            Command::ShootEnemy { .. } | Command::ShootAt { .. } | Command::FireSelectedTargets
        );
        let medical = matches!(
            command,
            Command::BeginMedicalItem { .. }
                | Command::UseItem { .. }
                | Command::BeginLearnedHeal { .. }
                | Command::SelectMedicalTarget { .. }
                | Command::StopBleeding { .. }
        );
        let movement = matches!(command, Command::Move { .. } | Command::CombatClick { .. });
        let field_control = if let Command::FieldControl { action } = command {
            Some(*action)
        } else {
            None
        };

        Self {
            text_entry,
            collection,
            weapon,
            enemy,
            psi,
            shot,
            medical,
            movement,
            field_control,
        }
    }
}
