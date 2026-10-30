use crate::{
    layout::{self, control},
    presentation::{Action, Control, Inspection, UiState, topmost},
};
use centurions_core::{Command, Phase, Presentation, Session};
use macroquad::prelude::*;

/// The held direction keys and mouse button that walk the squad, with the delay
/// before each repeated step.
pub struct MovementInput {
    direction: (i8, i8),
    repeat_at: f64,
    refused: bool,
    pointer_held: bool,
    pub repeat_seconds: f64,
}
impl Default for MovementInput {
    /// Not moving, repeating a step every tenth of a second.
    fn default() -> Self {
        Self {
            direction: (0, 0),
            repeat_at: 0.,
            refused: false,
            pointer_held: false,
            repeat_seconds: 0.1,
        }
    }
}

impl MovementInput {
    /// Stops the held direction.
    pub fn reset(&mut self) {
        self.direction = (0, 0);
    }
    /// Stops walking after a step was refused, until the keys are released.
    pub fn refuse(&mut self) {
        self.refused = true;
        self.stop_pointer();
    }

    /// Ends a walk started with the mouse button.
    pub fn stop_pointer(&mut self) {
        self.pointer_held = false;
    }

    /// While the mouse button stays down the squad keeps stepping toward the pointer's
    /// `direction`; releasing the button, or pointing at no direction, ends the walk.
    ///
    /// Original: buttonchoice
    fn pointer_step(
        &mut self,
        direction: (i8, i8),
        pressed: bool,
        down: bool,
        now: f64,
    ) -> Option<Command> {
        if !down || direction == (0, 0) {
            self.stop_pointer();
            return None;
        }
        if pressed {
            self.pointer_held = true;
            self.direction = (0, 0);
            self.refused = false;
        }
        if self.pointer_held {
            self.step(direction, now)
        } else {
            None
        }
    }

    /// The step command for the held `direction` once the repeat delay has passed;
    /// releasing the keys clears a refused walk.
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

/// The direction the held keys point: arrow keys and the number pad.
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

/// The action for the reload key: fit the warrior's equipped missile weapon with
/// the compatible ammunition in stock in combat, the kind already loaded first and then the
/// lowest numbered; with none to fit, or outside combat, reload.
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
                inventory.into_iter().enumerate().find(|(_, item)| {
                    item.equipped() != 0 && matches!(item.category_code(), 2 | 15)
                })
            })
    {
        let candidates =
            centurions_core::ammunition::compatible_ammunition(content, &item).unwrap_or_default();
        let loaded = item.ammunition_id().unwrap_or(0);
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

/// The action this frame's input stands for. Reveal Friends, the booty screen,
/// targeting and the Items and shop screens read their own input first;
/// anything else goes to the field and combat controls, and an action that is
/// not available now is dropped.
pub fn action(
    state: &Session,
    content: &centurions_content::Content,
    ui: &mut UiState,
    controls: &[Control],
    pointer: Option<Vec2>,
) -> Option<Action> {
    if state.combat.as_ref().is_some_and(|c| c.selection_pending) {
        // The automatic resume owns this frame; no actor/camera is selected yet.
        ui.movement.reset();
        ui.movement.stop_pointer();
        return None;
    }
    if !is_mouse_button_down(MouseButton::Left) || ui.inspection.is_some() {
        ui.movement.stop_pointer();
    }
    if ui.combat_view.reveal && state.combat.is_some() {
        // The mouseDown or keyDown that ends Reveal Friends is consumed;
        // nothing else is handled while showrange waits.
        ui.movement.reset();
        let pressed = is_mouse_button_pressed(MouseButton::Left)
            || is_mouse_button_pressed(MouseButton::Right)
            || !get_keys_pressed().is_empty();
        return pressed.then_some(Action::CombatView(
            crate::combat_controls::ViewAction::Reveal,
        ));
    }
    if crate::collection::active(state, ui) && state.promotion.is_none() {
        if let Some(action) = crate::collection::input(state, ui, pointer) {
            return Some(action);
        }
        if is_mouse_button_pressed(MouseButton::Left)
            && let Some(point) = pointer
            && let Some(control) = topmost(controls, point)
        {
            if let Action::Collection(crate::collection::CollectionAction::Item(slot)) =
                control.action
                && crate::application::command_modifier()
            {
                return Some(Action::Collection(
                    crate::collection::CollectionAction::Store(slot),
                ));
            }
            if let Action::Collection(crate::collection::CollectionAction::Item(slot)) =
                control.action
                && (is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt))
            {
                return Some(Action::Collection(
                    crate::collection::CollectionAction::Information(slot),
                ));
            }
            return control.enabled.then(|| control.action.clone());
        }
        return None;
    }
    if crate::targeting::active(state, ui) {
        return crate::targeting::input(state, content, ui, controls, pointer);
    }
    if state
        .exploration
        .as_ref()
        .is_some_and(|p| p.presentation == Some(Presentation::Shop))
        || ui.inspection == Some(Inspection::Items) && !crate::collection::active(state, ui)
    {
        if is_mouse_button_pressed(MouseButton::Left)
            && (is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt))
            && let Some(point) = pointer
            && let Some(control) = topmost(controls, point)
            && let Action::Inventory(crate::inventory::InventoryAction::Item { side, slot }) =
                control.action
        {
            return Some(Action::Inventory(
                crate::inventory::InventoryAction::DragInformation { side, slot },
            ));
        }
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

/// Physical-action selection control that accepts the chosen actions (Enter): item 17.
const ENCOUNTER_ACTIONS_ACCEPT: u16 = 17;
/// Physical-action selection control that cancels the selection (Escape): item 18.
const ENCOUNTER_ACTIONS_CANCEL: u16 = 18;

/// The field buttons (Camp, Items, Search, Scan, Trade and the rest) with the
/// action each carries out.
pub fn field_buttons(state: &Session) -> Vec<(i16, Action)> {
    use centurions_core::field_controls::Action as Field;
    let mut controls = vec![
        (
            control::CAMP,
            Action::Command(Command::FieldControl {
                action: Field::Camp,
            }),
        ),
        (control::ITEMS, Action::Open(Inspection::Items)),
        (
            control::SEARCH_TOGGLE,
            Action::Command(Command::FieldControl {
                action: Field::SearchMode,
            }),
        ),
        (control::SCAN, Action::Open(Inspection::Psi)),
        (control::TRADE, Action::Open(Inspection::Items)),
        (control::ENCOUNTER, Action::Command(Command::Encounter)),
        (
            control::SEARCH,
            Action::Command(Command::FieldControl {
                action: Field::Search,
            }),
        ),
        (
            control::REST,
            Action::Command(Command::FieldControl {
                action: Field::Rest,
            }),
        ),
        (
            control::HEAL,
            Action::Command(Command::FieldControl {
                action: Field::Heal,
            }),
        ),
    ];
    if state.warriors.len() < 2 {
        controls.retain(|(id, _)| *id != control::TRADE);
    }
    controls.push(
        if state
            .field_controls
            .as_ref()
            .and_then(|c| c.clinic)
            .is_some()
        {
            (
                control::TEMPLE,
                Action::Command(Command::FieldControl {
                    action: Field::Clinic,
                }),
            )
        } else if state.shop_access.is_some() {
            (control::SHOP, Action::Command(Command::OpenShop))
        } else {
            (control::CRATE, Action::Open(Inspection::Crate))
        },
    );
    controls
}

/// Whether `action` can be chosen now for the acting warrior; during capture only
/// the capture choices and moving can.
pub fn available(state: &Session, selected: usize, action: &Action) -> bool {
    let member = crate::acting::acting_warrior(state, selected);
    let free = state
        .exploration
        .as_ref()
        .is_some_and(|position| position.presentation.is_none())
        && state.medical_continuation.is_none()
        && state.outdoor_psi.is_none()
        && (state.phase == Phase::Exploration || crate::acting::active_warrior(state).is_some());
    if state
        .combat
        .as_ref()
        .is_some_and(|c| c.capture_actor.is_some())
    {
        return matches!(
            action,
            Action::Command(
                Command::CancelCapture
                    | Command::CombatClick { .. }
                    | Command::SelectCaptureTarget { .. }
                    | Command::Move { .. }
            )
        );
    }
    match action {
        Action::Command(
            Command::BeginAutomaticTurn
            | Command::DelayTurn
            | Command::UndoCombatMove
            | Command::RequestCombatEscape
            | Command::EscapeCombat
            | Command::ToggleCombatAuto { .. }
            | Command::SelectCombatWeapon { .. },
        )
        | Action::Open(Inspection::Weapons)
        | Action::Application(crate::application::ApplicationAction::CombatInformation(_)) => {
            centurions_core::combat::controls::actor(state).is_ok()
                && (matches!(
                    action,
                    Action::Command(Command::ToggleCombatAuto { .. }) | Action::Application(_)
                ) || centurions_core::combat::runtime::controlled_warrior(state).is_none())
        }
        Action::Command(Command::BeginCapture) => {
            centurions_core::combat::capture::available(state)
        }
        Action::CombatView(_) | Action::PanCombat(..) => free && state.phase == Phase::Combat,
        Action::Open(Inspection::Psi) => {
            (free
                || state.exploration.as_ref().is_some_and(|position| {
                    matches!(position.presentation, Some(Presentation::Typed { .. }))
                }))
                && centurions_core::combat::player_psi::can_cast(
                    &state.warriors,
                    member as i16,
                    state.psi_disabled,
                )
                .unwrap_or(false)
        }
        Action::Open(Inspection::Bandage) => free && state.bandage_ready != 0,
        Action::Open(Inspection::FireTargets) => {
            free && state
                .warriors
                .get(member)
                .is_some_and(|warrior| warrior.record().toggle())
        }
        Action::Command(Command::Guard) => {
            free && state.phase == Phase::Combat
                && state
                    .combat_story
                    .as_ref()
                    .is_none_or(|story| story.waiting_child)
                && state.warriors.get(member).is_some_and(|warrior| {
                    warrior.record().in_battle() != 0
                        && warrior.stamina().unwrap_or(0) > 0
                        && warrior.record().attacks().unwrap_or(0) >= 2
                })
        }
        Action::Command(Command::FieldControl { .. }) => {
            centurions_core::field_controls::free(state)
        }
        Action::Open(Inspection::Items | Inspection::Crate) => {
            free || matches!(action, Action::Open(Inspection::Items))
                && state
                    .exploration
                    .as_ref()
                    .is_some_and(|p| matches!(p.presentation, Some(Presentation::Typed { .. })))
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

/// textbox continues past a negative scenario text ID without waiting for input.
pub fn story_action(state: &Session, pressed: bool) -> Option<Action> {
    let id = match state.exploration.as_ref()?.presentation.as_ref()? {
        Presentation::Dialogue { id, .. } => *id,
        Presentation::Messages { pages, .. } => pages.first()?.0,
        _ => return None,
    };
    (id <= 0 || pressed).then_some(Action::Command(Command::Continue))
}

/// Whether an input step consumed the frame or passes it to the next step.
enum Flow {
    Done(Option<Action>),
    Next,
}

/// Runs the game-screen input steps in order. Each step may drain typed
/// characters or poll keys, so the order is part of the behavior.
fn action_inner(
    state: &Session,
    content: &centurions_content::Content,
    ui: &mut UiState,
    controls: &[Control],
    pointer: Option<Vec2>,
) -> Option<Action> {
    if let Flow::Done(action) = shop_escape(state) {
        return action;
    }
    if let Flow::Done(action) = promotion_input(state, controls, pointer) {
        return action;
    }
    if let Flow::Done(action) = capture_input(state, ui, pointer) {
        return action;
    }
    if let Flow::Done(action) = move_target_input(state, ui, pointer) {
        return action;
    }
    if !state
        .exploration
        .as_ref()
        .and_then(|position| position.presentation.as_ref())
        .is_some_and(|presentation| matches!(presentation, Presentation::Typed { .. }))
    {
        while get_char_pressed().is_some() {}
    }
    if let Flow::Done(action) = field_repeat_input(ui, controls, pointer) {
        return action;
    }
    if is_mouse_button_pressed(MouseButton::Left)
        && let Some(p) = pointer
        && let Some(c) = topmost(controls, p)
    {
        return c.enabled.then(|| c.action.clone());
    }
    if let Flow::Done(action) = encounter_actions_input(state) {
        return action;
    }
    if let Flow::Done(action) = inspection_input(state, ui, controls) {
        return action;
    }
    if crate::application::command_modifier() {
        return None;
    }
    if let Flow::Done(action) = presentation_keys(state, content, ui) {
        return action;
    }
    if state.phase == Phase::SquadSelection {
        return None;
    }
    let active = state.combat.as_ref().and_then(|c| c.active_body());
    if active.is_some_and(|b| !crate::acting::is_warrior_body(b)) {
        return None;
    }
    let selected = active.map(|b| b as usize).unwrap_or(ui.selected);
    if let Flow::Done(action) = shortcut_keys(state, content, selected) {
        return action;
    }
    movement_input(state, ui, pointer)
}

/// Escape leaves a shop.
fn shop_escape(state: &Session) -> Flow {
    if state
        .exploration
        .as_ref()
        .is_some_and(|p| p.presentation == Some(Presentation::Shop))
        && is_key_pressed(KeyCode::Escape)
    {
        return Flow::Done(Some(Action::Command(Command::CloseShop)));
    }
    Flow::Next
}

/// Promotion takes clicks; Return answers yes to the promotion question, or accepts the message
/// that follows, and Escape answers no to the question.
fn promotion_input(state: &Session, controls: &[Control], pointer: Option<Vec2>) -> Flow {
    let Some(progress) = &state.promotion else {
        return Flow::Next;
    };
    if is_mouse_button_pressed(MouseButton::Left)
        && let Some(p) = pointer
        && let Some(c) = topmost(controls, p)
    {
        return Flow::Done(c.enabled.then(|| c.action.clone()));
    }
    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
        return Flow::Done(Some(Action::Command(if progress.question {
            Command::PromotionAnswer { yes: true }
        } else {
            Command::PromotionControl {
                item: if progress.warning { 0 } else { 32 },
            }
        })));
    }
    if is_key_pressed(KeyCode::Escape) && progress.question {
        return Flow::Done(Some(Action::Command(Command::PromotionAnswer {
            yes: false,
        })));
    }
    Flow::Done(None)
}

/// Choosing a monster to capture: Escape cancels, a click on the map picks the
/// cell, and the direction keys walk.
fn capture_input(state: &Session, ui: &mut UiState, pointer: Option<Vec2>) -> Flow {
    if !state
        .combat
        .as_ref()
        .is_some_and(|c| c.capture_actor.is_some())
    {
        return Flow::Next;
    }
    if is_key_pressed(KeyCode::Escape) {
        return Flow::Done(Some(Action::Command(Command::CancelCapture)));
    }
    if is_mouse_button_pressed(MouseButton::Left)
        && let Some(p) = pointer
        && layout::MAP.contains(p)
        && let Some((x, y)) = crate::combat_controls::map_position(state, ui, p)
    {
        return Flow::Done(Some(Action::Command(Command::CombatClick { x, y })));
    }
    Flow::Done(
        ui.movement
            .step(movement_direction(is_key_down), get_time())
            .map(Action::Command),
    )
}

/// Moving the view: Escape returns it to the active body, and a click on the map
/// centres it on that cell.
fn move_target_input(state: &Session, ui: &UiState, pointer: Option<Vec2>) -> Flow {
    if !ui.combat_view.move_target {
        return Flow::Next;
    }
    if is_key_pressed(KeyCode::Escape) {
        return Flow::Done(Some(Action::CombatView(
            crate::combat_controls::ViewAction::Center,
        )));
    }
    if is_mouse_button_pressed(MouseButton::Left)
        && let Some(p) = pointer
        && layout::MAP.contains(p)
        && let Some((vx, vy)) = crate::combat_controls::origin(state, ui)
    {
        return Flow::Done(Some(Action::PanCombat(
            vx + ((p.x - layout::MAP.x) / 32.) as i16,
            vy + ((p.y - layout::MAP.y) / 32.) as i16,
        )));
    }
    Flow::Done(None)
}

/// Holding Search, Rest or Heal repeats it; releasing clears the repeat.
fn field_repeat_input(ui: &mut UiState, controls: &[Control], pointer: Option<Vec2>) -> Flow {
    let held = pointer
        .and_then(|point| topmost(controls, point))
        .filter(|control| control.enabled && is_mouse_button_down(MouseButton::Left));
    if let Some(Control {
        action: Action::Command(Command::FieldControl { action }),
        ..
    }) = held
        && matches!(
            action,
            centurions_core::field_controls::Action::Search
                | centurions_core::field_controls::Action::Rest
                | centurions_core::field_controls::Action::Heal
        )
    {
        let now = get_time();
        if ui
            .field_repeat
            .is_none_or(|(previous, next)| previous != *action || now >= next)
        {
            ui.field_repeating = ui
                .field_repeat
                .is_some_and(|(previous, _)| previous == *action);
            ui.field_repeat = Some((*action, now + ui.movement.repeat_seconds));
            return Flow::Done(Some(Action::Command(Command::FieldControl {
                action: *action,
            })));
        }
        return Flow::Done(None);
    }
    ui.field_repeat = None;
    ui.field_repeating = false;
    Flow::Next
}

/// The physical actions choice: Return accepts the chosen actions and Escape
/// cancels.
fn encounter_actions_input(state: &Session) -> Flow {
    if state.encounter_actions.is_none() {
        return Flow::Next;
    }
    while get_char_pressed().is_some() {}
    Flow::Done(if is_key_pressed(KeyCode::Escape) {
        Some(Action::Command(Command::EncounterActionControl {
            item: ENCOUNTER_ACTIONS_CANCEL,
        }))
    } else if is_key_pressed(KeyCode::Enter) {
        Some(Action::Command(Command::EncounterActionControl {
            item: ENCOUNTER_ACTIONS_ACCEPT,
        }))
    } else {
        None
    })
}

/// The keys of the open inspection list: Return chooses the first available
/// psi, and the weapon list closes on a click and moves with Up and Down.
fn inspection_input(state: &Session, ui: &mut UiState, controls: &[Control]) -> Flow {
    if ui.inspection.is_none() {
        return Flow::Next;
    }
    if ui.inspection == Some(Inspection::Psi) && is_key_pressed(KeyCode::Enter) {
        return Flow::Done(
            controls
                .iter()
                .find(|control| {
                    control.enabled
                        && matches!(
                            control.action,
                            Action::ChoosePsi(_)
                                | Action::Command(
                                    Command::CastPsi { .. }
                                        | Command::CastPartyPsi { .. }
                                        | Command::CastEncounterPsi { .. }
                                        | Command::BeginLearnedHeal { .. }
                                        | Command::BeginPsiSelection { .. }
                                )
                        )
                })
                .map(|control| control.action.clone()),
        );
    }
    if ui.inspection == Some(Inspection::Weapons) {
        if is_mouse_button_pressed(MouseButton::Left) {
            return Flow::Done(Some(Action::Close));
        }
        let count =
            centurions_core::combat::controls::weapons(state).map_or(0, |slots| slots.len());
        let direction = if is_key_pressed(KeyCode::Down) {
            1
        } else if is_key_pressed(KeyCode::Up) {
            -1
        } else {
            -mouse_wheel().1.signum() as i32
        };
        ui.page = (ui.page as i32 + direction).clamp(0, count.saturating_sub(12) as i32) as usize;
    }
    if ui.inspection == Some(Inspection::FireTargets)
        && state
            .combat
            .as_ref()
            .is_some_and(|c| c.firearm_selection.is_some())
        && is_key_pressed(KeyCode::Enter)
    {
        return Flow::Done(Some(Action::Command(Command::FireSelectedTargets)));
    }
    Flow::Done(is_key_pressed(KeyCode::Escape).then_some(
        if ui.inspection == Some(Inspection::Bandage) {
            Action::Command(Command::StopBleeding { warrior: None })
        } else {
            Action::Close
        },
    ))
}

/// Keys for an open scenario presentation; any presentation consumes the frame.
fn presentation_keys(
    state: &Session,
    content: &centurions_content::Content,
    ui: &mut UiState,
) -> Flow {
    let Some(p) = state
        .exploration
        .as_ref()
        .and_then(|p| p.presentation.as_ref())
    else {
        return Flow::Next;
    };
    if matches!(p, Presentation::Clinic { .. }) {
        return Flow::Done(
            (is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter))
                .then_some(Action::Command(Command::ClinicControl { item: 11 })),
        );
    }
    if matches!(
        p,
        Presentation::WarriorChoice { .. } | Presentation::PartyPick { .. }
    ) {
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
                return Flow::Done(Some(Action::Command(Command::ChooseWarrior { index })));
            }
        }
        return Flow::Done(None);
    }
    if matches!(
        p,
        Presentation::Question { .. }
            | Presentation::BoatQuestion { .. }
            | Presentation::Avoidance { .. }
            | Presentation::StageChange { .. }
    ) {
        return Flow::Done(if is_key_pressed(KeyCode::Y) {
            Some(Action::Command(Command::Choose { answer: true }))
        } else if is_key_pressed(KeyCode::N) {
            Some(Action::Command(Command::Choose { answer: false }))
        } else {
            None
        });
    }
    if let Presentation::Menu { labels, .. } = p {
        for (index, key) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4]
            .into_iter()
            .enumerate()
        {
            if index < labels.len() && is_key_pressed(key) {
                return Flow::Done(Some(Action::Command(Command::SelectMenuEntry { index })));
            }
        }
        return Flow::Done(
            is_key_pressed(KeyCode::Escape).then_some(Action::Command(Command::CancelMenu)),
        );
    }
    if matches!(p, Presentation::Typed { .. }) {
        return Flow::Done(typed_entry_input(state, content, ui));
    }
    Flow::Done(
        (is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space)).then_some(
            Action::Command(
                if matches!(p, Presentation::Loot)
                    && centurions_core::needs_spell_confirmation(state)
                {
                    Command::BeginPromotion
                } else {
                    Command::Continue
                },
            ),
        ),
    )
}

/// Typing into a scenario text-entry prompt.
fn typed_entry_input(
    state: &Session,
    content: &centurions_content::Content,
    ui: &mut UiState,
) -> Option<Action> {
    if !ui.typed_entry || !crate::story_dialogs::text_entry_available(state, content) {
        while get_char_pressed().is_some() {}
        return is_key_pressed(KeyCode::Escape).then_some(Action::Command(Command::CancelMenu));
    }
    while let Some(character) = get_char_pressed() {
        if ui.typed_text.chars().count() < 40
            && centurions_content::game_text::GameText::encode(&character.to_string()).is_ok()
        {
            ui.typed_text.push(character);
        }
    }
    if is_key_pressed(KeyCode::Backspace) {
        ui.typed_text.pop();
    }
    if is_key_pressed(KeyCode::Escape) {
        return Some(Action::TextEntry(false));
    }
    is_key_pressed(KeyCode::Enter).then(|| {
        Action::Command(Command::SubmitText {
            text: std::mem::take(&mut ui.typed_text),
        })
    })
}

/// Field-control letters outdoors, refill, combat shortcuts and the Psi key.
fn shortcut_keys(state: &Session, content: &centurions_content::Content, selected: usize) -> Flow {
    if state.phase == Phase::Exploration {
        for (key, id) in [
            (KeyCode::C, control::CAMP),
            (KeyCode::I, control::ITEMS),
            (KeyCode::S, control::SCAN),
            (KeyCode::T, control::TRADE),
            (KeyCode::E, control::ENCOUNTER),
            (KeyCode::A, control::SEARCH),
            (KeyCode::R, control::REST),
            (KeyCode::H, control::HEAL),
            (KeyCode::X, control::CRATE),
            (KeyCode::G, control::SHOP),
            (KeyCode::L, control::TEMPLE),
        ] {
            if is_key_pressed(key) {
                return Flow::Done(
                    field_buttons(state)
                        .into_iter()
                        .find(|(control, _)| *control == id)
                        .map(|(_, action)| action),
                );
            }
        }
    }
    if state.phase != Phase::Combat && is_key_pressed(KeyCode::R) {
        return Flow::Done(Some(refill_action(state, content, selected)));
    }
    if state.phase == Phase::Combat {
        for key in [
            KeyCode::A,
            KeyCode::S,
            KeyCode::D,
            KeyCode::F,
            KeyCode::G,
            KeyCode::C,
            KeyCode::B,
            KeyCode::W,
            KeyCode::E,
            KeyCode::R,
            KeyCode::T,
            KeyCode::U,
            KeyCode::I,
            KeyCode::P,
            KeyCode::L,
            KeyCode::N,
            KeyCode::M,
            KeyCode::Space,
        ] {
            if is_key_pressed(key) {
                return Flow::Done(crate::combat_controls::shortcut(
                    key, state, content, selected,
                ));
            }
        }
    } else if is_key_pressed(KeyCode::P) {
        return Flow::Done(Some(Action::Open(Inspection::Psi)));
    }
    Flow::Next
}

/// Keyboard and held-pointer movement, then a map click.
fn movement_input(state: &Session, ui: &mut UiState, pointer: Option<Vec2>) -> Option<Action> {
    let direction = movement_direction(is_key_down);
    if direction != (0, 0) {
        ui.movement.stop_pointer();
    } else if state.phase == Phase::Exploration
        && is_mouse_button_down(MouseButton::Left)
        && let Some(point) = pointer.filter(|point| layout::MAP.contains(*point))
    {
        let x = ((point.x - layout::MAP.x) / 32.) as i32;
        let y = ((point.y - layout::MAP.y) / 32.) as i32;
        let direction = match outdoor_move(state, x, y) {
            Some(Action::Command(Command::Move { dx, dy })) => (dx, dy),
            _ => (0, 0),
        };
        return ui
            .movement
            .pointer_step(
                direction,
                is_mouse_button_pressed(MouseButton::Left),
                true,
                get_time(),
            )
            .map(Action::Command);
    } else {
        ui.movement.stop_pointer();
    }
    if let Some(command) = ui.movement.step(direction, get_time()) {
        return Some(Action::Command(command));
    }
    if is_mouse_button_pressed(MouseButton::Left)
        && let Some(p) = pointer
        && layout::MAP.contains(p)
    {
        let x = ((p.x - layout::MAP.x) / 32.) as i32;
        let y = ((p.y - layout::MAP.y) / 32.) as i32;
        if state.combat.is_some() {
            let (x, y) = crate::combat_controls::map_position(state, ui, p)?;
            return Some(Action::Command(Command::CombatClick { x, y }));
        } else {
            return outdoor_move(state, x, y);
        }
    }
    None
}
/// The step toward the clicked outdoor cell, if it is not the party's own.
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
