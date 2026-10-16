use crate::{
    application::ApplicationAction,
    assets::Assets,
    presentation::{Action, Control},
};
use macroquad::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Preferences {
    pub controls: [bool; 14],
    pub indoor_icon: i16,
}
impl Default for Preferences {
    fn default() -> Self {
        // createpref 0x14c494, followed by its final indoor icon assignment.
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
    pub fn enabled(&self, item: usize) -> bool {
        self.controls[item - 9]
    }
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
    pub fn encode(&self) -> serde_json::Value {
        serde_json::json!({"controls":self.controls, "indoor_icon":self.indoor_icon})
    }
}
pub struct Dialog {
    pub draft: Preferences,
    pub description: Option<usize>,
}
impl Dialog {
    pub fn new(preferences: &Preferences) -> Self {
        Self {
            draft: preferences.clone(),
            description: None,
        }
    }
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
            assets.wrapped_color(description, assets.dialog_rect(167, 8), 12, 14., YELLOW);
        }
    }
}
fn control(item: usize, rect: Rect) -> Control {
    Control {
        rect,
        enabled: true,
        action: Action::Application(ApplicationAction::PreferenceItem(item)),
    }
}
