//! The Items window's choices: the rows, the buttons, the member strip and the keys.
use super::{Inventory, InventoryAction, items_window::ROWS};
use crate::presentation::{Action, Inspection};
use centurions_content::Content;
use centurions_core::{Command, RuleError, Session};
use macroquad::prelude::*;

/// The sound of a button the Items window presses.
const BUTTON_SOUND: i16 = 141;
/// The sound of a row click that does not wear or remove the item.
const ROW_SOUND: i16 = 83;
/// The sound of the View button.
const VIEW_SOUND: i16 = 30005;
/// The widest a double-click may wander, in pixels across and down together.
const DOUBLE_CLICK_SLOP: f32 = 5.;

/// How a click lands on the Items window's rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowGesture {
    /// A plain click; a double-click near the last one wears or removes the item.
    Click,
    /// A Command-click, which wears or removes the item.
    Command,
    /// An Option-click, which shows the item's information.
    Option,
}

impl Inventory {
    /// The warning a battle gives an owner who is not the warrior whose turn it is (warning 10),
    /// or who has no action point left (warning 17).
    fn turn_warning(state: &Session, owner: usize) -> Option<i16> {
        let combat = state.combat.as_ref()?;
        if combat.active_body() != Some(owner as i16) {
            Some(10)
        } else if state
            .warriors
            .get(owner)?
            .record()
            .action_points()
            .unwrap_or(0)
            < 1
        {
            Some(17)
        } else {
            None
        }
    }

    /// Carries out a choice in the Items window, returning the action for the game to take.
    ///
    /// Original: items
    pub(super) fn handle_items(
        &mut self,
        action: InventoryAction,
        state: &Session,
        content: &Content,
        now: f64,
    ) -> Result<Option<Action>, RuleError> {
        use InventoryAction::*;
        let owner = self.members[0];
        let count = self.items(state, content, 0)?.len();
        let row = self.display_row.max(0) as usize;
        let scroll = self.scroll[0];
        let command = |command| Ok(Some(Action::Command(command)));
        match action {
            Row { point, gesture } => self.click_row(state, content, point, gesture, now),
            Step(delta) => self.step_row(state, content, delta),
            Toggle => self.toggle(state, content),
            Warrior { member, .. } => {
                self.sounds.push(BUTTON_SOUND);
                if member >= state.warriors.len() {
                    return Err("inventory warrior outside squad".into());
                }
                self.members[0] = member;
                self.scroll[0] = 0;
                command(Command::SwitchInventoryOwner {
                    warrior: member,
                    scroll: 0,
                    quick_info: self.quick_info,
                    selected_row: self.display_row,
                })
            }
            Scroll { delta, .. } => {
                if delta > 0 && scroll >= 30 {
                    return Ok(None);
                }
                if state.globals.health_context.char_select_new == owner {
                    self.scroll[0] = if delta > 0 && scroll + ROWS < count {
                        scroll + 1
                    } else if delta < 0 {
                        scroll.saturating_sub(1)
                    } else {
                        scroll
                    };
                }
                command(Command::ScrollInventory {
                    warrior: owner,
                    scroll,
                    delta: delta.signum() as i16,
                    quick_info: self.quick_info,
                })
            }
            Use if count == 0 => Ok(None),
            Use => command(Command::PressUse {
                warrior: owner,
                scroll,
                row,
            }),
            Information => {
                if count == 0 {
                    return Ok(None);
                }
                self.sounds.push(VIEW_SOUND);
                self.information = Some((owner, scroll + row));
                self.information_item = None;
                command(Command::ShowItemInformation {
                    warrior: owner,
                    scroll,
                    row,
                    quick_info: self.quick_info,
                })
            }
            Weapon => {
                if let Some(index) = Self::turn_warning(state, owner) {
                    self.warning = Some(index);
                    return Ok(None);
                }
                Err(RuleError::Unsupported {
                    action: "items.weapon",
                    dependency: "changeweapon",
                })
            }
            Drop => {
                self.dropped = Some(count);
                command(Command::DropToCrate {
                    warrior: owner,
                    scroll,
                    row,
                    command_key: crate::application::command_modifier(),
                    protected: self.drop_protection,
                })
            }
            Join => command(Command::JoinItems {
                warrior: owner,
                scroll,
                row,
            }),
            Split => command(Command::SplitItem {
                warrior: owner,
                scroll,
                row,
            }),
            Refit(_) => {
                if state.combat.is_some() {
                    self.warning = Some(14);
                    return Ok(None);
                }
                command(Command::Reload { warrior: owner })
            }
            View => {
                self.sounds.push(BUTTON_SOUND);
                self.follow_up.push_back(Action::Application(
                    crate::application::ApplicationAction::OpenInformation(owner),
                ));
                Ok(None)
            }
            Trade => {
                if state.warriors.len() < 2 {
                    self.warning = Some(62);
                    return Ok(None);
                }
                self.initems = false;
                if state.combat.is_some() || Self::encounter(state) {
                    self.warning = Some(6);
                    return Ok(None);
                }
                self.trade = true;
                self.members = [owner, (owner + 1) % state.warriors.len()];
                self.scroll = [0; 2];
                self.selected = None;
                Ok(Some(Action::Command(self.context_command())))
            }
            Done => {
                self.initems = false;
                self.sounds.push(BUTTON_SOUND);
                self.follow_up.push_back(Action::Close);
                command(Command::CloseItems { warrior: owner })
            }
            _ => Ok(None),
        }
    }

    /// Whether an encounter called the Items window, which then cannot trade.
    fn encounter(state: &Session) -> bool {
        state.exploration.as_ref().is_some_and(|p| {
            matches!(
                p.presentation,
                Some(centurions_core::Presentation::Typed { .. })
            )
        })
    }

    /// A click on the rows: the row under the point, kept within the pack, highlighted, with its
    /// detail in the 800 by 600 window. A click below the last row scrolls down instead, and on an
    /// empty pack scrolled down it scrolls up. A Command-click or a double-click near the last
    /// click on the left of the rows wears or removes the item; an Option-click shows its
    /// information; any other click sounds the row click.
    ///
    /// Original: items
    fn click_row(
        &mut self,
        state: &Session,
        content: &Content,
        point: Vec2,
        gesture: RowGesture,
        now: f64,
    ) -> Result<Option<Action>, RuleError> {
        let window_y = point.y - crate::layout::MENU_BAR;
        let mut row = ((window_y - 11.) / 40.).trunc() as i16;
        if gesture == RowGesture::Option {
            self.display_row = row.clamp(0, ROWS as i16 - 1);
            return self.handle_items(InventoryAction::Information, state, content, now);
        }
        let count = self.items(state, content, 0)?.len() as i16;
        let near = (point - self.last_point).abs();
        let double = now - self.last_click < super::DOUBLE_CLICK_SECONDS
            && near.x + near.y < DOUBLE_CLICK_SLOP;
        self.last_click = now;
        self.last_point = point;
        let wear = point.x < self.shift() + 240.
            && (gesture == RowGesture::Command || (gesture == RowGesture::Click && double));
        self.unhighlighted |= 1 << self.display_row.clamp(0, ROWS as i16 - 1);
        if row >= count {
            row = count - 1;
        }
        if row < 0 {
            self.display_row = 0;
            if self.scroll[0] != 0 {
                self.sounds.push(ROW_SOUND);
                return self.handle_items(
                    InventoryAction::Scroll { side: 0, delta: -1 },
                    state,
                    content,
                    now,
                );
            }
            row = 0;
        } else if row > ROWS as i16 - 1 {
            self.display_row = ROWS as i16 - 1;
            self.sounds.push(ROW_SOUND);
            return self.handle_items(
                InventoryAction::Scroll { side: 0, delta: 1 },
                state,
                content,
                now,
            );
        }
        self.display_row = row;
        if wear {
            self.follow_up
                .push_back(Action::Inventory(InventoryAction::Toggle));
        } else {
            self.sounds.push(ROW_SOUND);
        }
        Ok(Some(Action::Command(Command::HighlightInventoryRow {
            warrior: self.members[0],
            scroll: self.scroll[0],
            row: row as usize,
            quick_info: self.quick_info,
        })))
    }

    /// The arrow keys move the highlight a row, scrolling at the ends.
    ///
    /// Original: items
    fn step_row(
        &mut self,
        state: &Session,
        content: &Content,
        delta: i16,
    ) -> Result<Option<Action>, RuleError> {
        let count = self.items(state, content, 0)?.len() as i16;
        let row = self.display_row.wrapping_add(delta);
        self.sounds.push(ROW_SOUND);
        let scroll = |delta| InventoryAction::Scroll { side: 0, delta };
        if row < 0 {
            if self.scroll[0] == 0 {
                return Ok(None);
            }
            return self.handle_items(scroll(-1), state, content, get_time());
        }
        if row > ROWS as i16 - 1 {
            return self.handle_items(scroll(1), state, content, get_time());
        }
        if row >= count {
            return Ok(None);
        }
        self.unhighlighted |= 1 << self.display_row.clamp(0, ROWS as i16 - 1);
        self.display_row = row;
        Ok(Some(Action::Command(Command::HighlightInventoryRow {
            warrior: self.members[0],
            scroll: self.scroll[0],
            row: row as usize,
            quick_info: self.quick_info,
        })))
    }

    /// Wears the highlighted item or takes it off; in a battle wearing needs an action point of
    /// the warrior whose turn it is (warning 17).
    ///
    /// Original: items, wear, removeitem
    pub(super) fn toggle(
        &mut self,
        state: &Session,
        content: &Content,
    ) -> Result<Option<Action>, RuleError> {
        let warrior = self.members[0];
        let slot = self.scroll[0] + self.display_row.max(0) as usize;
        let Some((_, item)) = self
            .items(state, content, 0)?
            .into_iter()
            .find(|(index, _)| *index == slot)
        else {
            return Ok(None);
        };
        if item.equipped() != 0 {
            return Ok(Some(Action::Command(Command::UnequipItem {
                warrior,
                slot,
            })));
        }
        if let Some(combat) = state.combat.as_ref()
            && combat
                .active_body()
                .and_then(|body| state.warriors.get(body as usize))
                .is_some_and(|actor| actor.record().action_points().unwrap_or(0) < 1)
        {
            self.warning = Some(17);
            return Ok(None);
        }
        Ok(Some(Action::Command(Command::EquipItem { warrior, slot })))
    }

    /// The choice the Items window's keys make: Return closes it, the space bar and Enter wear or
    /// remove the highlighted item, the arrows and keypad 8 and 2 move the highlight, and the
    /// letters press their buttons.
    ///
    /// Original: items
    pub fn items_key(&self) -> Option<InventoryAction> {
        use InventoryAction::*;
        [
            (KeyCode::Enter, Done),
            (KeyCode::Space, Toggle),
            (KeyCode::KpEnter, Toggle),
            (KeyCode::Up, Step(-1)),
            (KeyCode::Kp8, Step(-1)),
            (KeyCode::Down, Step(1)),
            (KeyCode::Kp2, Step(1)),
            (KeyCode::U, Use),
            (KeyCode::V, Information),
            (KeyCode::D, Drop),
            (KeyCode::R, Refit(0)),
            (KeyCode::T, Trade),
            (KeyCode::J, Join),
            (KeyCode::S, Split),
        ]
        .into_iter()
        .find(|(key, _)| is_key_pressed(*key))
        .map(|(_, action)| action)
    }

    /// Opens the Items window for the trade window's left warrior: a shop closes first.
    ///
    /// Original: choice, buttonchoice
    pub(super) fn leave_for_items(&mut self, state: &Session) -> Option<Action> {
        self.sounds.push(BUTTON_SOUND);
        self.trade = false;
        let owner = self.members[0];
        self.follow_up.push_back(Action::Select(owner));
        self.follow_up.push_back(Action::Open(Inspection::Items));
        Self::shop(state).then_some(Action::Command(Command::CloseShop))
    }
}
