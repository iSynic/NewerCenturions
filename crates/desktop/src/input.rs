use crate::{
    layout,
    presentation::{Action, Control, Inspection, UiState, tactical_view},
};
use centurions_core::{Command, Phase, Presentation, Session};
use macroquad::prelude::*;

pub struct MovementInput {
    direction: (i8, i8),
    repeat_at: f64,
    refused: bool,
    pub repeat_seconds: f64,
}
impl Default for MovementInput {
    fn default() -> Self {
        Self {
            direction: (0, 0),
            repeat_at: 0.,
            refused: false,
            repeat_seconds: 0.1,
        }
    }
}

impl MovementInput {
    pub fn reset(&mut self) {
        self.direction = (0, 0);
    }
    pub fn refuse(&mut self) {
        self.refused = true;
    }

    fn step(&mut self, direction: (i8, i8), now: f64) -> Option<Command> {
        if direction == (0, 0) {
            self.reset();
            self.refused = false;
            return None;
        }
        if self.refused {
            return None;
        }
        if direction != self.direction || now >= self.repeat_at {
            self.direction = direction;
            // Repeat at a bounded rate without queuing moves during animation or dialogs.
            self.repeat_at = now + self.repeat_seconds;
            Some(Command::Move {
                dx: direction.0,
                dy: direction.1,
            })
        } else {
            None
        }
    }
}

fn movement_direction(down: impl Fn(KeyCode) -> bool) -> (i8, i8) {
    let right = [KeyCode::Right, KeyCode::Kp3, KeyCode::Kp6, KeyCode::Kp9]
        .into_iter()
        .any(&down);
    let left = [KeyCode::Left, KeyCode::Kp1, KeyCode::Kp4, KeyCode::Kp7]
        .into_iter()
        .any(&down);
    let below = [KeyCode::Down, KeyCode::Kp1, KeyCode::Kp2, KeyCode::Kp3]
        .into_iter()
        .any(&down);
    let above = [KeyCode::Up, KeyCode::Kp7, KeyCode::Kp8, KeyCode::Kp9]
        .into_iter()
        .any(down);
    (
        i8::from(right) - i8::from(left),
        i8::from(below) - i8::from(above),
    )
}

pub fn refill_action(
    state: &Session,
    content: &centurions_content::Content,
    warrior: usize,
) -> Action {
    if state.phase == Phase::Combat
        && let Some((slot, item)) = state
            .warriors
            .get(warrior)
            .and_then(|actor| actor.inventory().ok())
            .and_then(|inventory| {
                inventory
                    .into_iter()
                    .enumerate()
                    .find(|(_, item)| item.0[9] != 0 && matches!(item.0[10], 2 | 15))
            })
    {
        let candidates =
            centurions_core::ammunition::compatible_ammunition(content, &item).unwrap_or_default();
        let loaded = centurions_content::word(&item.0, 4).unwrap_or(0);
        if let Some(ammunition) = candidates
            .iter()
            .copied()
            .filter(|id| state.ammunition[(*id - 200) as usize] > 0)
            .min_by_key(|id| if *id == loaded { 0 } else { *id })
        {
            return Action::Command(Command::FitAmmunition {
                warrior,
                slot,
                ammunition,
            });
        }
    }
    Action::Command(Command::Reload { warrior })
}

pub fn action(
    state: &Session,
    content: &centurions_content::Content,
    ui: &mut UiState,
    controls: &[Control],
    pointer: Option<Vec2>,
) -> Option<Action> {
    if state
        .exploration
        .as_ref()
        .is_some_and(|p| p.presentation == Some(Presentation::Shop))
        || ui.inspection == Some(Inspection::Items)
            && state
                .exploration
                .as_ref()
                .is_some_and(|position| position.presentation.is_none())
    {
        if is_mouse_button_released(MouseButton::Left)
            && let Some(action) = ui.inventory.release(pointer)
        {
            return Some(Action::Inventory(action));
        }
        let delta = mouse_wheel().1;
        if delta != 0. {
            let side = usize::from(pointer.is_some_and(|point| point.x > 457.));
            return Some(Action::Inventory(
                crate::inventory::InventoryAction::Scroll {
                    side,
                    delta: if delta > 0. { -1 } else { 1 },
                },
            ));
        }
    }
    action_inner(state, content, ui, controls, pointer)
        .filter(|action| available(state, ui.selected, action))
}

pub fn available(state: &Session, selected: usize, action: &Action) -> bool {
    let active = state.combat.as_ref().and_then(|arena| arena.active_body());
    let member = active
        .filter(|body| (0..state.warriors.len() as i16).contains(body))
        .map(|body| body as usize)
        .unwrap_or(selected);
    let free = state
        .exploration
        .as_ref()
        .is_some_and(|position| position.presentation.is_none())
        && state.medical_continuation.is_none()
        && state.outdoor_psi.is_none()
        && (state.phase == Phase::Exploration
            || active.is_some_and(|body| (0..state.warriors.len() as i16).contains(&body)));
    match action {
        Action::Open(Inspection::Psi) => {
            (free
                || state.exploration.as_ref().is_some_and(|position| {
                    matches!(position.presentation, Some(Presentation::Typed { .. }))
                }))
                && state.warriors.get(member).is_some_and(|warrior| {
                    centurions_content::word(&warrior.bytes, 0x36).unwrap_or(0) != 0
                })
        }
        Action::Open(Inspection::Bandage) => {
            free && state.bandage_ready.is_some_and(|ready| ready != 0)
        }
        Action::Open(Inspection::FireTargets) => {
            free && state
                .warriors
                .get(member)
                .is_some_and(|warrior| warrior.bytes[0x9dd] != 0)
        }
        Action::Command(Command::Guard) => {
            free && state.phase == Phase::Combat
                && state
                    .combat_story
                    .as_ref()
                    .is_none_or(|story| story.waiting_child)
                && state.warriors.get(member).is_some_and(|warrior| {
                    warrior.bytes[0x9df] != 0
                        && warrior.stamina().unwrap_or(0) > 0
                        && centurions_content::word(&warrior.bytes, 0x40).unwrap_or(0) >= 2
                })
        }
        Action::Command(
            Command::Move { .. }
            | Command::CombatClick { .. }
            | Command::Encounter
            | Command::Reload { .. }
            | Command::FitAmmunition { .. }
            | Command::SetFirearmMode { .. }
            | Command::EndTurn,
        ) => free,
        Action::Command(Command::ChooseWarrior { index }) => *index < state.warriors.len(),
        _ => true,
    }
}

fn action_inner(
    state: &Session,
    content: &centurions_content::Content,
    ui: &mut UiState,
    controls: &[Control],
    pointer: Option<Vec2>,
) -> Option<Action> {
    if state
        .exploration
        .as_ref()
        .is_some_and(|p| p.presentation == Some(Presentation::Shop))
        && is_key_pressed(KeyCode::Escape)
    {
        return Some(Action::Command(Command::CloseShop));
    }
    if let Some(progress) = &state.promotion {
        if is_mouse_button_pressed(MouseButton::Left)
            && let Some(p) = pointer
            && let Some(c) = controls.iter().rev().find(|c| c.rect.contains(p))
        {
            return c.enabled.then(|| c.action.clone());
        }
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
            return Some(Action::Command(if progress.question {
                Command::PromotionAnswer { yes: true }
            } else {
                Command::PromotionControl {
                    item: if progress.warning { 0 } else { 32 },
                }
            }));
        }
        if is_key_pressed(KeyCode::Escape) && progress.question {
            return Some(Action::Command(Command::PromotionAnswer { yes: false }));
        }
        return None;
    }
    if state.outdoor_psi.is_some() {
        if is_mouse_button_pressed(MouseButton::Left)
            && let Some(p) = pointer
            && let Some(c) = controls.iter().rev().find(|c| c.rect.contains(p))
        {
            return c.enabled.then(|| c.action.clone());
        }
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::A) {
            return Some(Action::Command(Command::CancelOutdoorPsi));
        }
        if is_key_pressed(KeyCode::Space) && state.party_mask.iter().any(|rank| *rank > 0) {
            return Some(Action::Command(Command::FinishOutdoorPsi));
        }
        return None;
    }
    if state.psi_selection.is_some() {
        if is_mouse_button_pressed(MouseButton::Left)
            && let Some(p) = pointer
            && let Some(c) = controls.iter().rev().find(|c| c.rect.contains(p))
        {
            return c.enabled.then(|| c.action.clone());
        }
        if is_key_pressed(KeyCode::Enter) && !ui.medical_cancel {
            return Some(Action::Command(Command::FirePsiSelection));
        }
        if is_key_pressed(KeyCode::Escape) {
            return Some(if ui.medical_cancel {
                Action::KeepMedicalTarget
            } else {
                Action::MedicalCancel
            });
        }
        return None;
    }
    if state.medical_continuation.is_some() {
        if is_mouse_button_pressed(MouseButton::Left)
            && let Some(p) = pointer
            && let Some(c) = controls.iter().rev().find(|c| c.rect.contains(p))
        {
            return c.enabled.then(|| c.action.clone());
        }
        if is_key_pressed(KeyCode::Escape) {
            return Some(if ui.medical_cancel {
                Action::KeepMedicalTarget
            } else {
                Action::MedicalCancel
            });
        }
        return None;
    }
    if ui.firearm_map_target {
        if is_key_pressed(KeyCode::Escape) {
            return Some(Action::Close);
        }
        if is_mouse_button_pressed(MouseButton::Left)
            && let Some(p) = pointer
            && layout::MAP.contains(p)
            && let Some((vx, vy)) = tactical_view(state)
        {
            return Some(Action::Command(Command::ShootAt {
                x: vx + ((p.x - layout::MAP.x) / 32.) as i16,
                y: vy + ((p.y - layout::MAP.y) / 32.) as i16,
            }));
        }
        return None;
    }
    if let Some((spell, power)) = ui.psi_pending {
        ui.psi_preview = pointer.filter(|p| layout::MAP.contains(*p)).and_then(|p| {
            tactical_view(state).map(|(vx, vy)| {
                (
                    vx + ((p.x - layout::MAP.x) / 32.) as i16,
                    vy + ((p.y - layout::MAP.y) / 32.) as i16,
                )
            })
        });
        if is_key_pressed(KeyCode::Enter)
            && content
                .map(state.exploration.as_ref().map_or(0, |p| p.map))
                .and_then(|m| m.combat.as_ref())
                .and_then(|t| centurions_core::combat::spells::load_packet(&t.attacks, spell).ok())
                .is_some_and(|p| p[7] != 0)
        {
            return Some(Action::RotatePsi);
        }
        if is_key_pressed(KeyCode::Escape) {
            return Some(Action::Close);
        }
        if is_mouse_button_pressed(MouseButton::Left)
            && let Some(p) = pointer
            && layout::MAP.contains(p)
            && let Some((vx, vy)) = tactical_view(state)
        {
            return Some(Action::Command(Command::CastPsi {
                rotation: ui.psi_rotation,
                spell,
                power,
                target: Some((
                    vx + ((p.x - layout::MAP.x) / 32.) as i16,
                    vy + ((p.y - layout::MAP.y) / 32.) as i16,
                )),
            }));
        }
        return None;
    }
    if !state
        .exploration
        .as_ref()
        .and_then(|position| position.presentation.as_ref())
        .is_some_and(|presentation| matches!(presentation, Presentation::Typed { .. }))
    {
        while get_char_pressed().is_some() {}
    }
    if is_mouse_button_pressed(MouseButton::Left)
        && let Some(p) = pointer
        && let Some(c) = controls.iter().rev().find(|c| c.rect.contains(p))
    {
        return c.enabled.then(|| c.action.clone());
    }
    if state.encounter_actions.is_some() {
        while get_char_pressed().is_some() {}
        return if is_key_pressed(KeyCode::Escape) {
            Some(Action::Command(Command::EncounterActionControl {
                item: 18,
            }))
        } else if is_key_pressed(KeyCode::Enter) {
            Some(Action::Command(Command::EncounterActionControl {
                item: 17,
            }))
        } else {
            None
        };
    }
    if ui.inspection.is_some() {
        if ui.inspection == Some(Inspection::FireTargets)
            && state
                .combat
                .as_ref()
                .is_some_and(|c| c.firearm_selection.is_some())
            && is_key_pressed(KeyCode::Enter)
        {
            return Some(Action::Command(Command::FireSelectedTargets));
        }
        return is_key_pressed(KeyCode::Escape).then_some(
            if ui.inspection == Some(Inspection::Bandage) {
                Action::Command(Command::StopBleeding { warrior: None })
            } else {
                Action::Close
            },
        );
    }
    if crate::application::command_modifier() {
        return None;
    }
    if let Some(p) = state
        .exploration
        .as_ref()
        .and_then(|p| p.presentation.as_ref())
    {
        if matches!(p, Presentation::Clinic { .. }) {
            return (is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter))
                .then_some(Action::Command(Command::ClinicControl { item: 11 }));
        }
        if matches!(p, Presentation::WarriorChoice { .. }) {
            for (index, key) in [
                KeyCode::Key1,
                KeyCode::Key2,
                KeyCode::Key3,
                KeyCode::Key4,
                KeyCode::Key5,
                KeyCode::Key6,
            ]
            .into_iter()
            .enumerate()
            {
                if is_key_pressed(key) {
                    return Some(Action::Command(Command::ChooseWarrior { index }));
                }
            }
            return None;
        }
        if matches!(
            p,
            Presentation::Question { .. }
                | Presentation::Avoidance { .. }
                | Presentation::StageChange { .. }
        ) {
            return if is_key_pressed(KeyCode::Y) {
                Some(Action::Command(Command::Choose { answer: true }))
            } else if is_key_pressed(KeyCode::N) {
                Some(Action::Command(Command::Choose { answer: false }))
            } else {
                None
            };
        }
        if let Presentation::Menu { labels, .. } = p {
            for (index, key) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4]
                .into_iter()
                .enumerate()
            {
                if index < labels.len() && is_key_pressed(key) {
                    return Some(Action::Command(Command::SelectMenuEntry { index }));
                }
            }
            return is_key_pressed(KeyCode::Escape).then_some(Action::Command(Command::CancelMenu));
        }
        if matches!(p, Presentation::Typed { .. }) {
            while let Some(character) = get_char_pressed() {
                if !character.is_control() && ui.typed_text.len() < 40 {
                    ui.typed_text.push(character);
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                ui.typed_text.pop();
            }
            if is_key_pressed(KeyCode::Escape) {
                ui.typed_text.clear();
                return Some(Action::Command(Command::CancelMenu));
            }
            return is_key_pressed(KeyCode::Enter).then(|| {
                Action::Command(Command::SubmitText {
                    text: std::mem::take(&mut ui.typed_text),
                })
            });
        }
        return (is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space)).then_some(
            Action::Command(
                if matches!(p, Presentation::Loot)
                    && centurions_core::needs_spell_confirmation(state)
                {
                    Command::BeginPromotion
                } else {
                    Command::Continue
                },
            ),
        );
    }
    if state.phase == Phase::SquadSelection {
        return None;
    }
    let active = state.combat.as_ref().and_then(|c| c.active_body());
    if active.is_some_and(|b| b >= 10) {
        return None;
    }
    let selected = active.map(|b| b as usize).unwrap_or(ui.selected);
    if state.phase == Phase::Exploration && is_key_pressed(KeyCode::E) {
        return Some(Action::Command(Command::Encounter));
    }
    if is_key_pressed(KeyCode::R) {
        return Some(refill_action(state, content, selected));
    }
    if is_key_pressed(KeyCode::P) {
        return Some(Action::Open(Inspection::Psi));
    }
    if state.phase == Phase::Combat {
        if is_key_pressed(KeyCode::B) {
            return Some(if state.bandage_ready.is_some_and(|ready| ready != 0) {
                Action::Open(Inspection::Bandage)
            } else {
                Action::Command(Command::StopBleeding { warrior: None })
            });
        }
        if is_key_pressed(KeyCode::F) {
            return Some(Action::Command(Command::SetFirearmMode {
                warrior: selected,
                enabled: state.warriors[selected].bytes[0x9dd] == 0,
            }));
        }
        if is_key_pressed(KeyCode::G) {
            return Some(Action::Command(Command::Guard));
        }
        if is_key_pressed(KeyCode::Space) {
            return Some(Action::Command(Command::EndTurn));
        }
    }
    if let Some(command) = ui
        .movement
        .step(movement_direction(is_key_down), get_time())
    {
        return Some(Action::Command(command));
    }
    if is_mouse_button_pressed(MouseButton::Left)
        && let Some(p) = pointer
        && layout::MAP.contains(p)
    {
        let x = ((p.x - layout::MAP.x) / 32.) as i32;
        let y = ((p.y - layout::MAP.y) / 32.) as i32;
        if state.combat.is_some() {
            let (vx, vy) = tactical_view(state).unwrap();
            let x = vx + x as i16;
            let y = vy + y as i16;
            return Some(Action::Command(Command::CombatClick { x, y }));
        } else {
            return outdoor_move(state, x, y);
        }
    }
    None
}
fn outdoor_move(state: &Session, x: i32, y: i32) -> Option<Action> {
    let position = state.exploration.as_ref()?;
    let (vx, vy) = crate::layout::outdoor_origin(position.x, position.y);
    let dx = (x + vx - position.x).signum() as i8;
    let dy = (y + vy - position.y).signum() as i8;
    (dx != 0 || dy != 0).then_some(Action::Command(Command::Move { dx, dy }))
}
#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../tests/unit/input_reference.rs"]
mod tests;
