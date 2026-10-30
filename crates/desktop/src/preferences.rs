use crate::{
    application::ApplicationAction,
    assets::Assets,
    presentation::{Action, Control},
};
use macroquad::prelude::*;

/// Preference dialog 167 check-box items; each label is item + 26.
pub mod pref {
    /// "Faster Scan Casting": shortened scan walks.
    pub const FASTER_SCAN_CASTING: usize = 9;
    /// "Faster Scan Resolution": batched scan effects.
    pub const FASTER_SCAN_RESOLUTION: usize = 10;
    /// "Show Scan Description".
    pub const SHOW_SCAN_DESCRIPTION: usize = 11;
    /// "Drop Item Protection": dropping needs the command modifier.
    pub const DROP_ITEM_PROTECTION: usize = 12;
    /// "Ignore Next Round Message": suppresses the round message sound.
    pub const IGNORE_NEXT_ROUND_MESSAGE: usize = 13;
    /// "Ignore Bleeding Warriors".
    pub const IGNORE_BLEEDING_WARRIORS: usize = 14;
    /// "Auto Refit Items At End Of Battle".
    pub const AUTO_REFIT: usize = 15;
    /// "Manual Bandage Only In Combat".
    pub const MANUAL_BANDAGE_ONLY_IN_COMBAT: usize = 16;
    /// "Append M.O.S. To Portrait": class labels on names.
    pub const APPEND_MOS: usize = 17;
    /// "Hide The Desktop": fullscreen.
    pub const HIDE_THE_DESKTOP: usize = 20;
    /// "Use Terminator Font": the artistic font.
    pub const TERMINATOR_FONT: usize = 21;
    /// "Use 800 x 600 Screen resolution": the large window and combat field.
    pub const SCREEN_800_X_600: usize = 22;
}

/// The Preferences dialog's checkbox settings, one for each of items 9 to 22 (item 19 has no
/// checkbox), and the indoor icon.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Preferences {
    pub controls: [bool; 14],
    pub indoor_icon: i16,
}
impl Default for Preferences {
    /// The preferences a new game starts with.
    fn default() -> Self {
        // createpref's defaults, followed by its final indoor icon assignment.
        Self {
            controls: [
                true, true, true, true, false, false, true, false, true, true, false, true, true,
                true,
            ],
            indoor_icon: 9281,
        }
    }
}
impl Preferences {
    /// Whether preference item `item` is checked.
    pub fn enabled(&self, item: usize) -> bool {
        self.controls[item - 9]
    }
    /// Reads saved preferences; none saved means the defaults.
    pub fn decode(value: &serde_json::Value) -> Result<Self, String> {
        if value.is_null() {
            return Ok(Self::default());
        }
        let controls = value["controls"]
            .as_array()
            .filter(|v| v.len() == 14)
            .ok_or("Invalid preference controls.")?;
        let mut preferences = Self::default();
        for (target, value) in preferences.controls.iter_mut().zip(controls) {
            *target = value.as_bool().ok_or("Invalid preference control value.")?;
        }
        preferences.indoor_icon = value["indoor_icon"]
            .as_i64()
            .and_then(|n| i16::try_from(n).ok())
            .ok_or("Invalid indoor squad icon.")?;
        if !(9257..=9376).contains(&preferences.indoor_icon) {
            return Err("Invalid indoor squad icon preference.".into());
        }
        Ok(preferences)
    }
    /// The preferences as saved.
    pub fn encode(&self) -> serde_json::Value {
        serde_json::json!({"controls":self.controls, "indoor_icon":self.indoor_icon})
    }
}
/// The Preferences dialog (DITL 167): a draft of the choices and the one whose
/// description is shown.
pub struct Dialog {
    pub draft: Preferences,
    pub description: Option<usize>,
}
impl Dialog {
    /// Opens the dialog on a copy of `preferences`.
    pub fn new(preferences: &Preferences) -> Self {
        Self {
            draft: preferences.clone(),
            description: None,
        }
    }
    /// Carries out a click on a dialog item: toggling a checkbox and showing its
    /// description. Returns true for OK.
    pub fn item(&mut self, item: usize) -> Result<bool, centurions_core::RuleError> {
        match item {
            2 => return Ok(true),
            9..=22 if item != 19 => {
                self.draft.controls[item - 9] = !self.draft.controls[item - 9];
                self.description = Some(item - 9);
            }
            _ => return Err(centurions_core::RuleError::Refused { feedback: vec![] }),
        }
        Ok(false)
    }
    /// Draws the Preferences dialog (DITL 167): each checkbox in its on (PICT 179) or off
    /// (PICT 180) look with its label, and the description of the last one changed.
    pub fn draw(&self, assets: &Assets, controls: &mut Vec<Control>) {
        controls.clear();
        assets.draw_dialog(167);
        for item in (9..=22).filter(|i| *i != 19) {
            let rect = assets.dialog_rect(167, item);
            assets.panel(if self.draft.enabled(item) { 179 } else { 180 }, rect);
            let label = assets.dialog_rect(167, item + 26);
            controls.push(control(
                item,
                Rect::new(
                    rect.x.min(label.x),
                    rect.y.min(label.y),
                    label.right() - rect.x.min(label.x),
                    rect.h.max(label.h),
                ),
            ));
        }
        controls.push(control(2, assets.dialog_rect(167, 2)));
        if let Some(description) = self
            .description
            .and_then(|i| assets.preference_descriptions.get(i))
        {
            assets.dialog_text(description, assets.dialog_rect(167, 8));
        }
    }
}
/// A clickable control for preference dialog item `item`.
fn control(item: usize, rect: Rect) -> Control {
    Control {
        rect,
        enabled: true,
        action: Action::Application(ApplicationAction::PreferenceItem(item)),
    }
}
