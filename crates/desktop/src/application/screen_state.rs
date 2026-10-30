//! The state each separate screen keeps while it is open.
use super::Screen;
use macroquad::prelude::Vec2;

/// The warrior screens that can be open: editor, information, appearance, item
/// editor and new warrior creation.
pub(super) struct CharacterScreens {
    pub(super) editor_question: bool,
    pub(super) editor: Option<crate::editor::WarriorEditor>,
    pub(super) information: Option<crate::information::WarriorInformation>,
    pub(super) appearance: Option<crate::appearance::Appearance>,
    pub(super) item_editor: Option<crate::editor::ItemEditor>,
    pub(super) creation: Option<crate::creation::Creation>,
}
impl CharacterScreens {
    /// All character screens closed.
    pub(super) fn new() -> Self {
        Self {
            editor_question: false,
            editor: None,
            information: None,
            appearance: None,
            item_editor: None,
            creation: None,
        }
    }
}

/// The Modify Squad screen's choices: roster scroll, marching order, selected
/// rows and difficulty.
pub(super) struct SquadScreen {
    pub(super) modify_squad: bool,
    pub(super) roster_top: usize,
    pub(super) order: Option<Vec<usize>>,
    pub(super) end_question: bool,
    pub(super) selected: Option<usize>,
    pub(super) member: Option<usize>,
    pub(super) difficulty: i16,
}
impl SquadScreen {
    /// Roster at the top, nothing selected, difficulty zero.
    pub(super) fn new() -> Self {
        Self {
            modify_squad: false,
            roster_top: 0,
            order: None,
            end_question: false,
            selected: None,
            member: None,
            difficulty: 0,
        }
    }
}

/// The introduction text's scroll, click and drag, and the screen it returns to.
pub(super) struct IntroScreen {
    pub(super) started: f64,
    pub(super) intro_y: f32,
    pub(super) intro_tick: f64,
    pub(super) intro_click: f64,
    pub(super) drag: Option<Vec2>,
    pub(super) intro_return: Screen,
    pub(super) prologue: Option<i16>,
}
impl IntroScreen {
    /// Starts the introduction clock at `now`, returning to the title.
    pub(super) fn new(now: f64) -> Self {
        Self {
            started: now,
            intro_y: 0.,
            intro_tick: now,
            intro_click: -1.,
            drag: None,
            intro_return: Screen::Title,
            prologue: None,
        }
    }
}

/// The yes-or-no questions waiting on the player's answer.
pub(super) struct Questions {
    pub(super) quit: bool,
    pub(super) help_reset: bool,
    pub(super) escape: bool,
    pub(super) exchange_offer: Option<i16>,
    pub(super) exchange_swap: bool,
}
impl Questions {
    /// No question waiting.
    pub(super) fn new() -> Self {
        Self {
            quit: false,
            help_reset: false,
            escape: false,
            exchange_offer: None,
            exchange_swap: true,
        }
    }
}
