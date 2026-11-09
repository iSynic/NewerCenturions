//! The combat control panel (PICT 153), its buttons' hit rectangles and the
//! tactical view the combat screen looks through.
use crate::{
    input,
    presentation::{Action, Control, Inspection, UiState},
};
use centurions_core::{Command, Session};
use macroquad::prelude::*;

/// The Load control's place on the combat panel.
pub(crate) const LOAD: Rect = Rect::new(78., 546., 45., 16.);

/// The combat panel's buttons that change the tactical view.
#[derive(Clone, Copy, Debug)]
pub enum ViewAction {
    Center,
    Next,
    Previous,
    Move,
    Reveal,
}

/// Which body and point the tactical view is centred on for the current turn.
#[derive(Default)]
pub struct View {
    pub body: Option<i16>,
    turn: Option<(i16, usize)>,
    point: Option<(i16, i16)>,
    /// Reveal Friends: showrange(0)'s frames and lines stay until its
    /// WaitNextEvent loop takes a mouseDown or keyDown.
    pub reveal: bool,
    pub move_target: bool,
    /// Set while the panel shows the body the player centred or stepped to: the
    /// information repaint, which reads out the range to the body.
    information: bool,
    /// The command-click hold sweeping bodies, while the button is down.
    peek: Option<Peek>,
    /// The body a released peek left in the panel, with the turn it belongs to.
    shown: Option<(i16, (i16, usize))>,
}

/// A held command-click: the body the sweep is on and whether it has moved off the
/// acting body.
struct Peek {
    current: i16,
    swept: bool,
}

impl View {
    /// The body the view is on: the player's pick for this turn, else the body whose
    /// turn it is.
    pub fn selected(&self, state: &Session) -> Option<i16> {
        let c = state.combat.as_ref()?;
        if self.turn == Some((c.combat_round, c.battle_order_index)) {
            self.body.or(c.active_body())
        } else {
            c.active_body()
        }
    }

    /// Whether the panel is repainting for a body the player centred or stepped to
    /// this turn.
    ///
    /// Original: combatupdate2
    pub fn information(&self, state: &Session) -> bool {
        self.peek.is_some()
            || self.held_body(state).is_some()
            || state.combat.as_ref().is_some_and(|c| {
                self.information && self.turn == Some((c.combat_round, c.battle_order_index))
            })
    }

    /// The body a released peek left in the panel for this turn.
    fn held_body(&self, state: &Session) -> Option<i16> {
        let c = state.combat.as_ref()?;
        self.shown
            .filter(|(_, turn)| *turn == (c.combat_round, c.battle_order_index))
            .map(|(body, _)| body)
    }

    /// The body the combat information panel shows: the one a peek is on or has left
    /// in the panel, else the one the view is on. The Items and Conditions buttons
    /// open the pop-up of this body.
    ///
    /// Original: combatinfo
    pub fn shown(&self, state: &Session) -> Option<i16> {
        if let Some(peek) = self.peek.as_ref().filter(|peek| peek.swept) {
            return Some(peek.current);
        }
        self.held_body(state).or_else(|| self.selected(state))
    }

    /// The body a held peek frames on the battlefield in place of the acting body.
    pub fn peeked(&self) -> Option<i16> {
        self.peek.as_ref().map(|peek| peek.current)
    }

    /// Whether the peek is on a body whose guarding byte is set, which shows the
    /// shield cursor.
    ///
    /// Original: combatinfo
    pub fn shield(&self, state: &Session) -> bool {
        let Some(peek) = self.peek.as_ref().filter(|peek| peek.swept) else {
            return false;
        };
        let Some(combat) = state.combat.as_ref() else {
            return false;
        };
        if (0..9).contains(&peek.current) {
            return state
                .warriors
                .get(peek.current as usize)
                .is_some_and(|warrior| warrior.record().guarding().unwrap_or(0) != 0);
        }
        combat
            .roster
            .records
            .get((peek.current - 10) as usize)
            .is_some_and(|monster| monster.record().guarding() != 0)
    }

    /// Centres the view on a battlefield point for the rest of the turn.
    pub fn pan(&mut self, state: &Session, point: (i16, i16)) {
        if let Some(c) = &state.combat {
            self.turn = Some((c.combat_round, c.battle_order_index));
            self.point = Some(point);
            self.move_target = false;
            self.information = false;
            self.shown = None;
        }
    }

    /// Carries out a view button: centre on the active body, step to the next or
    /// previous body in battle order that still stands, choose a cell to move the
    /// view to, or toggle Reveal Friends.
    pub fn change(&mut self, state: &Session, action: ViewAction) {
        let Some(c) = &state.combat else { return };
        if self.turn != Some((c.combat_round, c.battle_order_index)) {
            *self = Self::default();
        }
        self.turn = Some((c.combat_round, c.battle_order_index));
        if let ViewAction::Reveal = action {
            // Reveal Friends keeps the view: showrange(0), then centerfield at
            // the current centre once a click or key arrives.
            self.reveal = !self.reveal;
            return;
        }
        self.point = None;
        self.shown = None;
        self.information = matches!(
            action,
            ViewAction::Center | ViewAction::Next | ViewAction::Previous
        );
        match action {
            ViewAction::Center => {
                self.body = c.active_body();
                self.point = None;
                self.move_target = false;
            }
            ViewAction::Move => {
                self.body = c.active_body();
                self.move_target = true;
            }
            ViewAction::Reveal => {}
            ViewAction::Next | ViewAction::Previous => {
                let step = if matches!(action, ViewAction::Next) {
                    1
                } else {
                    -1
                };
                let mut index = self
                    .body
                    .and_then(|body| c.battle_order.iter().position(|id| *id == body))
                    .unwrap_or(c.battle_order_index);
                // centerstage wraps battle order slots 1..109; bounded even if no
                // body remains.
                for _ in 0..109 {
                    index = (index as i32 - 1 + step).rem_euclid(109) as usize + 1;
                    let body = c.battle_order[index];
                    let present = if (0..state.warriors.len() as i16).contains(&body) {
                        state.warriors[body as usize].stamina().unwrap_or(0) > 0
                    } else {
                        usize::try_from(body - 10)
                            .ok()
                            .and_then(|i| c.roster.records.get(i))
                            .is_some_and(|m| m.stamina().unwrap_or(0) > 0)
                    };
                    if present {
                        self.body = Some(body);
                        break;
                    }
                }
            }
        }
    }
}

impl View {
    /// Starts a peek on the acting body; false when there is none to start from.
    fn begin_peek(&mut self, state: &Session) -> bool {
        let Some(combat) = state.combat.as_ref() else {
            return false;
        };
        let Some(acting) = combat.active_body() else {
            return false;
        };
        if combat.capture_actor.is_some() || self.reveal || self.move_target {
            return false;
        }
        self.peek = Some(Peek {
            current: acting,
            swept: false,
        });
        true
    }

    /// Moves a held peek to `body` when it differs from the body it is on.
    fn sweep_peek(&mut self, body: i16) {
        if let Some(peek) = self.peek.as_mut()
            && peek.current != body
        {
            peek.current = body;
            peek.swept = true;
        }
    }

    /// Ends the peek. The panel keeps the last body shown unless a spell is being
    /// readied, when it shows the acting warrior again.
    fn end_peek(&mut self, state: &Session) {
        let Some(peek) = self.peek.take() else { return };
        let Some(combat) = state.combat.as_ref() else {
            return;
        };
        self.shown = (peek.swept && combat.in_spell == 0).then_some((
            peek.current,
            (combat.combat_round, combat.battle_order_index),
        ));
    }
}

/// The body on the battlefield cell `point`, as the field map names it: a warrior
/// below nine or a creature from ten to a hundred and ten.
fn body_on(state: &Session, point: (i16, i16)) -> Option<i16> {
    let combat = state.combat.as_ref()?;
    let index = centurions_content::mission::OpeningMission::index(
        i32::from(point.0 + combat.field.map_shift.0 + 5),
        i32::from(point.1 + combat.field.map_shift.1 + 5),
    )
    .ok()?;
    let body = combat.field.tiles[index].wrapping_abs();
    let present = if (0..9).contains(&body) {
        state.warriors.get(body as usize).is_some()
    } else {
        (10..=110).contains(&body) && combat.roster.records.get((body - 10) as usize).is_some()
    };
    present.then_some(body)
}

/// Command-click and hold on the battlefield previews the information of each body
/// the pointer sweeps over: the body is framed, the panel shows it and a guarding
/// body brings up the shield cursor. Releasing the button ends the preview. Returns
/// whether the preview owns this frame's input.
///
/// Original: combatinfo
pub fn peek(state: &Session, ui: &mut UiState, pointer: Option<Vec2>) -> bool {
    if state.combat.is_none() {
        ui.combat_view.peek = None;
        return false;
    }
    if ui.combat_view.peek.is_some() {
        if !is_mouse_button_down(MouseButton::Left) {
            ui.combat_view.end_peek(state);
            return true;
        }
    } else {
        let allowed = ui.inspection.is_none() || crate::targeting::active(state, ui);
        if !(is_mouse_button_pressed(MouseButton::Left)
            && crate::application::command_modifier()
            && allowed
            && pointer.is_some_and(|point| crate::layout::MAP.contains(point))
            && ui.combat_view.begin_peek(state))
        {
            return false;
        }
    }
    if let Some(point) = pointer.filter(|point| crate::layout::MAP.contains(*point))
        && let Some(cell) = map_position(state, ui, point)
        && let Some(body) = body_on(state, cell)
    {
        ui.combat_view.sweep_peek(body);
    }
    true
}

/// Draws the shield cursor in place of the pointer while a peek is on a guarding
/// body, and the cursor that counts the targets still to pick while targets are chosen; the
/// sword cursor is the pointer itself.
///
/// Original: combatinfo, updatespell
pub fn draw_peek_cursor(
    state: &Session,
    assets: &crate::assets::Assets,
    ui: &UiState,
    pointer: Option<Vec2>,
) {
    let counting = ui
        .targeting
        .count_cursor
        .take()
        .and_then(|id| assets.count_cursors.get(&id));
    let shield = assets
        .shield_cursor
        .as_ref()
        .filter(|_| ui.combat_view.shield(state))
        .or(counting);
    show_mouse(shield.is_none());
    if let (Some((texture, hotspot)), Some(point)) = (shield, pointer) {
        draw_texture(
            texture,
            point.x - f32::from(hotspot.0),
            point.y - f32::from(hotspot.1),
            WHITE,
        );
    }
}

/// The battlefield cell at the view's top left: the playback's view while it
/// runs, otherwise centred on the panned point or the selected body.
pub fn origin(state: &Session, ui: &UiState) -> Option<(i16, i16)> {
    if let Some(origin) = ui.playback.as_ref().and_then(|playback| playback.view) {
        return Some(origin);
    }
    let live = state.combat.as_ref()?;
    let body = ui.combat_view.selected(state)?;
    let c = live;
    let point = if ui.combat_view.turn == Some((live.combat_round, live.battle_order_index))
        && let Some(point) = ui.combat_view.point
    {
        point
    } else if crate::acting::is_warrior_body(body) {
        *c.squad.pos.get(body as usize)?
    } else {
        *c.roster.mon_pos.get((body - 10) as usize)?
    };
    Some(centered(c, point))
}

/// The view origin that puts a tactical position at the view's centre cell.
pub fn centered(c: &centurions_core::combat::Combat, point: (i16, i16)) -> (i16, i16) {
    let shift = (c.field.map_shift.0 + 5, c.field.map_shift.1 + 5);
    (
        (point.0 - 7).clamp(-shift.0, 75 - shift.0),
        (point.1 - 6).clamp(-shift.1, 77 - shift.1),
    )
}

/// The combat panel's button controls (PICT 153), enabled for the acting
/// warrior's current choices.
pub fn buttons(state: &Session, selected: usize) -> Vec<Control> {
    let actor = state
        .combat
        .as_ref()
        .and_then(|c| c.active_body())
        .filter(|i| (0..state.warriors.len() as i16).contains(i))
        .map_or(selected, |i| i as usize);
    let firearm = state.warriors[actor].record().toggle();
    // The controls sit 20 pixels lower here than in the game's window (combat's
    // Guard at 526, 436 is drawn at 526, 456).
    let entries = [
        (
            Rect::new(526., 456., 46., 15.),
            Action::Command(Command::Guard),
        ),
        (
            Rect::new(579., 456., 46., 15.),
            Action::Command(Command::DelayTurn),
        ),
        (
            Rect::new(632., 456., 47., 15.),
            Action::Command(Command::EndTurn),
        ),
        (
            Rect::new(684., 456., 41., 15.),
            Action::Command(Command::SetFirearmMode {
                warrior: actor,
                enabled: !firearm,
            }),
        ),
        (
            Rect::new(526., 476., 99., 15.),
            Action::CombatView(ViewAction::Reveal),
        ),
        (
            Rect::new(632., 476., 46., 15.),
            Action::CombatView(ViewAction::Move),
        ),
        (
            Rect::new(684., 476., 41., 15.),
            Action::Open(Inspection::FireTargets),
        ),
        (
            Rect::new(526., 496., 51., 15.),
            Action::Command(Command::RequestCombatEscape),
        ),
        (
            Rect::new(584., 496., 41., 15.),
            Action::Command(Command::UndoCombatMove),
        ),
        (
            Rect::new(632., 496., 47., 15.),
            Action::CombatView(ViewAction::Center),
        ),
        (
            Rect::new(526., 516., 57., 15.),
            Action::Open(Inspection::Bandage),
        ),
        (
            Rect::new(590., 516., 35., 15.),
            Action::CombatView(ViewAction::Next),
        ),
        (
            Rect::new(632., 516., 47., 15.),
            Action::CombatView(ViewAction::Previous),
        ),
        (
            Rect::new(468., 449., 50., 39.),
            Action::Command(Command::BeginAutomaticTurn),
        ),
        (
            Rect::new(468., 488., 50., 50.),
            Action::Open(Inspection::Items),
        ),
        (
            Rect::new(468., 538., 50., 50.),
            Action::Open(Inspection::Psi),
        ),
        (
            Rect::new(518., 538., 50., 50.),
            Action::Command(Command::BeginCapture),
        ),
        (
            Rect::new(45., 526., 16., 34.),
            Action::Command(Command::SetFirearmMode {
                warrior: actor,
                enabled: !firearm,
            }),
        ),
        (LOAD, input::refill_action(state, actor)),
        (
            Rect::new(125., 546., 178., 16.),
            Action::Open(Inspection::Weapons),
        ),
        (
            Rect::new(7., 455., 66., 66.),
            Action::Application(crate::application::ApplicationAction::CombatInformation(
                state
                    .combat
                    .as_ref()
                    .and_then(|c| c.active_body())
                    .unwrap_or(actor as i16),
            )),
        ),
    ];
    entries
        .into_iter()
        .map(|(rect, action)| Control {
            rect,
            enabled: input::available(state, actor, &action),
            action,
        })
        .collect()
}

/// The action a combat key press stands for (G, D, F, W and so on, one for
/// each panel button), if its button is available.
pub fn shortcut(key: KeyCode, state: &Session, selected: usize) -> Option<Action> {
    let index = match key {
        KeyCode::G => 0,
        KeyCode::D => 1,
        KeyCode::F | KeyCode::Space => 2,
        KeyCode::W => 3,
        KeyCode::R => 4,
        KeyCode::M => 5,
        KeyCode::T => 6,
        KeyCode::E => 7,
        KeyCode::U => 8,
        KeyCode::C => 9,
        KeyCode::B => 10,
        KeyCode::N => 11,
        KeyCode::P => 12,
        KeyCode::A => 13,
        KeyCode::I => 14,
        KeyCode::S => 15,
        KeyCode::L => 18,
        _ => return None,
    };
    buttons(state, selected)
        .get(index)
        .map(|c| c.action.clone())
}

/// The Auto boxes, CNTL 190..195, keep each warrior's automatic flag between
/// turns.
pub fn draw_automatic(
    state: &Session,
    assets: &crate::assets::Assets,
    controls: &mut Vec<Control>,
) {
    let Some(c) = &state.combat else { return };
    for member in 0..state.warriors.len() {
        let rect = assets.main_controls[&(crate::layout::control::AUTO_POSITION + member as i16)];
        assets.panel(if c.do_auto[member] { 179 } else { 180 }, rect);
        let action = Action::Command(Command::ToggleCombatAuto { warrior: member });
        controls.push(Control {
            rect,
            enabled: input::available(state, member, &action),
            action,
        });
    }
}

/// Draws up to 12 of the acting warrior's weapons from the current page under
/// their category headings, with ammunition counts.
pub fn draw_weapons(
    state: &Session,
    assets: &crate::assets::Assets,
    ui: &UiState,
    controls: &mut Vec<Control>,
) {
    let Ok(member) = centurions_core::combat::controls::actor(state) else {
        return;
    };
    let Ok(slots) = centurions_core::combat::controls::weapons(state) else {
        return;
    };
    let inventory = state.warriors[member].inventory().unwrap_or_default();
    let mut rows = Vec::new();
    let mut category = None;
    for slot in slots.into_iter().skip(ui.page).take(12) {
        let item = &inventory[slot];
        let kind = (item.category_code() as i8).unsigned_abs();
        if category != Some(kind) {
            let address = if kind == 15 { 0x403d28 } else { 0x403cfc };
            rows.push((
                assets.game_text.text(address, true).unwrap_or_default(),
                None,
                false,
            ));
            category = Some(kind);
        }
        let amount = i16::from_be_bytes([item.0[2], item.0[3]]);
        let mut label = assets.name(item.id());
        if kind == 15 || amount > 0 {
            let prefix = assets
                .game_text
                .text(if kind == 15 { 0x403d22 } else { 0x403cfa }, false)
                .unwrap_or_default();
            let suffix = assets
                .game_text
                .text(if kind == 15 { 0x403d1c } else { 0x403cf4 }, false)
                .unwrap_or_default();
            label.push_str(&format!("{prefix}{amount}{suffix}"));
        }
        rows.push((label, Some(slot), item.equipped() != 0));
    }
    let width = rows
        .iter()
        .map(|(label, _, _)| measure_text(label, Some(&assets.menu_font), 12, 1.).width)
        .fold(180., f32::max)
        + 30.;
    let height = rows.len() as f32 * 18. + 8.;
    let rect = Rect::new(
        (726. - width).max(0.),
        (456. - height).max(20.),
        width,
        height,
    );
    draw_rectangle(rect.x + 3., rect.y + 3., rect.w, rect.h, BLACK);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::from_rgba(238, 238, 238, 255),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1., BLACK);
    for (row, (label, slot, checked)) in rows.iter().enumerate() {
        let row_rect = Rect::new(
            rect.x + 1.,
            rect.y + 4. + row as f32 * 18.,
            rect.w - 2.,
            18.,
        );
        assets.menu_text(
            label,
            row_rect.x + 23.,
            row_rect.y + 13.,
            12,
            if slot.is_some() { BLACK } else { GRAY },
        );
        if *checked {
            assets.menu_text("✓", row_rect.x + 5., row_rect.y + 13., 12, BLACK);
        }
        if let Some(slot) = slot {
            controls.push(Control {
                rect: row_rect,
                enabled: true,
                action: Action::Command(Command::SelectCombatWeapon { slot: *slot }),
            });
        }
    }
}

/// Converts an in-map logical point using the same visible origin as targeting.
pub fn map_position(state: &Session, ui: &UiState, pointer: Vec2) -> Option<(i16, i16)> {
    let (x, y) = origin(state, ui)?;
    Some((
        x + ((pointer.x - crate::layout::MAP.x) / 32.) as i16,
        y + ((pointer.y - crate::layout::MAP.y) / 32.) as i16,
    ))
}
