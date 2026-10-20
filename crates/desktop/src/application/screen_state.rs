//! Private ownership of independent desktop workflows.
use super::Screen;
use macroquad::prelude::Vec2;

pub(super) struct CharacterScreens {
    pub(super) editor_question: bool,
    pub(super) editor: Option<crate::editor::WarriorEditor>,
    pub(super) information: Option<crate::information::WarriorInformation>,
    pub(super) appearance: Option<crate::appearance::Appearance>,
    pub(super) item_editor: Option<crate::editor::ItemEditor>,
    pub(super) creation: Option<crate::creation::Creation>,
}
impl CharacterScreens {
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

pub(super) struct SquadScreen {
    pub(super) modify_squad: bool,
    pub(super) roster_page: usize,
    pub(super) order: Option<Vec<usize>>,
    pub(super) end_question: bool,
    pub(super) selected: Option<usize>,
    pub(super) member: Option<usize>,
    pub(super) difficulty: i16,
}
impl SquadScreen {
    pub(super) fn new() -> Self {
        Self {
            modify_squad: false,
            roster_page: 0,
            order: None,
            end_question: false,
            selected: None,
            member: None,
            difficulty: 0,
        }
    }
}

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
