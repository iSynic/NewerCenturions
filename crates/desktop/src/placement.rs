//! Where the game puts its windows. A window the game never moves sits where its resource puts
//! it, centred on the screen when the resource asks for that; the routine that opens any other
//! window moves it to a place measured from the game window, often shifted by the 800 by 600
//! window.
use crate::layout;
use macroquad::prelude::*;

/// The game window's top left on the screen and the shifts the 800 by 600 window gives the
/// controls, as the routines that move a window read them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    /// The game window's left edge on the screen.
    pub global_left: i32,
    /// The game window's top on the screen, under the menu bar.
    pub global_top: i32,
    /// How far right the 800 by 600 window moves the controls.
    pub origin_left: i32,
    /// How far down the 800 by 600 window moves the controls.
    pub origin_top: i32,
    /// The 800 by 600 preference: 1 when it is on.
    pub large: i32,
}

impl Placement {
    /// The large window fills the 800 by 600 screen and the small one the 640 by 480 screen, so
    /// the game window always starts at the screen's left edge under the menu bar; only the
    /// large window shifts the controls.
    ///
    /// Original: ToolBoxInit
    pub fn new(large: bool) -> Self {
        Self {
            global_left: 0,
            global_top: layout::MENU_BAR as i32,
            origin_left: if large { layout::ORIGIN_LEFT as i32 } else { 0 },
            origin_top: if large { layout::ORIGIN_TOP as i32 } else { 0 },
            large: i32::from(large),
        }
    }
    /// The screen point `left` and `top` from the game window's top left.
    fn at(self, left: i32, top: i32) -> Vec2 {
        vec2(
            (self.global_left + left) as f32,
            (self.global_top + top) as f32,
        )
    }
    /// How far the large window moves the lines under the field up.
    fn shift(self) -> i32 {
        self.large << 5
    }
    /// The action picker (dialog 148) stands at the right, beside the squad rows.
    ///
    /// Original: actionpicker
    pub fn action_picker(self) -> Vec2 {
        self.at(407 + self.origin_left, 4 + self.origin_top)
    }
    /// The creature page (dialog 158) moves left by half the window's shift.
    ///
    /// Original: beast
    pub fn creature_page(self) -> Vec2 {
        self.at(142 - self.origin_left / 2, 40)
    }
    /// The encounter menu (dialog 147) moves by half the window's shifts.
    ///
    /// Original: encounter
    pub fn encounter_menu(self) -> Vec2 {
        self.at(30 + self.origin_left / 2, self.origin_top / 2)
    }
    /// The typed encounter strip (dialog 153) sits under the field at the left.
    ///
    /// Original: encounter2
    pub fn typed_encounter(self) -> Vec2 {
        self.at(11, 251 + self.origin_top - self.shift())
    }
    /// The word entry (dialog 157) sits under the field at the left.
    ///
    /// Original: getword
    pub fn word_entry(self) -> Vec2 {
        self.at(30, 110 + self.origin_top)
    }
    /// The clinic (dialog 159) moves by half the large window's shifts. The small window passes
    /// the game window's top as the left and its left as the top, so the clinic sits 20 pixels
    /// in from the left edge at the very top of the screen.
    ///
    /// Original: temple
    pub fn clinic(self) -> Vec2 {
        if self.large != 0 {
            self.at(self.origin_left / 2, self.origin_top / 2)
        } else {
            vec2(self.global_top as f32, self.global_left as f32)
        }
    }
    /// The level-up notice (dialog 150) sits near the game window's top left.
    ///
    /// Original: levelup
    pub fn level_up(self) -> Vec2 {
        self.at(18, 44)
    }
    /// The question strip (dialogs 136 and 200) sits beside the information box, under the
    /// squad rows.
    ///
    /// Original: question2, question3
    pub fn question_strip(self) -> Vec2 {
        self.at(322 + self.origin_left, 301 + self.origin_top - self.shift())
    }
    /// The special abilities list (dialog 160) stands over the field.
    ///
    /// Original: showspecial
    pub fn special_abilities(self) -> Vec2 {
        self.at(150, 100)
    }
    /// The scan or note text (dialog 169) sits under the field at the left.
    ///
    /// Original: castspell, showmap
    pub fn scan_description(self) -> Vec2 {
        self.at(0, 320 + self.origin_top - self.shift())
    }
    /// The scan picker (dialog 144) stands right of the field.
    ///
    /// Original: castspell
    pub fn scan_picker(self) -> Vec2 {
        self.at(320 + self.origin_left, 0)
    }
    /// The item information window (dialog 149) stands at the right, over the second list.
    ///
    /// Original: showiteminfo
    pub fn item_information(self) -> Vec2 {
        self.at(285 + self.origin_left, 0)
    }
    /// The farewell picture (dialog 175) opens one pixel in from the game window's corner.
    ///
    /// Original: quitgame
    pub fn farewell(self) -> Vec2 {
        self.at(1, 1)
    }
    /// The logo (dialog 175) moves by half the window's shifts, one pixel further in.
    ///
    /// Original: showlogo
    pub fn logo(self) -> Vec2 {
        self.at(1 + self.origin_left / 2, 1 + self.origin_top / 2)
    }
    /// The movie window (WIND 133) takes the size of the field's view and moves to the game
    /// window's corner.
    ///
    /// Original: movie
    pub fn movie(self) -> Rect {
        let origin = self.at(0, 0);
        Rect::new(
            origin.x,
            origin.y,
            (self.origin_left + 320) as f32,
            (self.origin_top + 320 - 44 * self.large) as f32,
        )
    }
    /// The name entry (dialog 132) moves by half the window's shifts.
    ///
    /// Original: Name
    pub fn name_entry(self) -> Vec2 {
        self.at(161 + self.origin_left / 2, 185 + self.origin_top / 2)
    }
    /// The starting level choice (dialog 161) moves by half the window's shifts.
    ///
    /// Original: Startlevel
    pub fn starting_level(self) -> Vec2 {
        self.at(230 + self.origin_left / 2, 83 + self.origin_top / 2)
    }
}

/// A window whose resource asks to be centred on the main screen: centred across it and in the
/// height under the menu bar, each margin rounded down to whole pixels.
///
/// Original: GetNewDialog
pub fn centred(size: Vec2) -> Vec2 {
    let (width, height) = (size.x as i32, size.y as i32);
    let screen = (layout::SIZE.x as i32, layout::SIZE.y as i32);
    let menu_bar = layout::MENU_BAR as i32;
    vec2(
        ((screen.0 - width) / 2).max(0) as f32,
        (menu_bar + ((screen.1 - menu_bar - height) / 2).max(0)) as f32,
    )
}
