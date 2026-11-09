//! The preferences' indoor squad icon picker: the icon picker dialog (DITL 170) in the mode that
//! changes only the preference.
use crate::{
    application::ApplicationAction,
    assets::Assets,
    picker::{self, Picker},
    preferences::{Preferences, Taken},
    presentation::{Action, Control},
};
use centurions_core::RuleError;
use macroquad::prelude::*;

/// The first cicn of the squad icons; the preference counts from it.
const FIRST_ICON: i16 = 9257;
/// The first cicn of the page of choices, less the item number of its first cell.
const FIRST_CHOICE: i16 = 9248;

/// The indoor squad icon picker, open on one page of 24 icons.
pub struct IconPicker {
    page: i16,
}
impl IconPicker {
    /// Opens the picker on the first page.
    pub fn new() -> Self {
        Self { page: 0 }
    }
    /// Answers a click: a cell chooses its icon as the preference, the page button turns the page
    /// and Done ends the picker. The cells of the recommended race icons and the squad members
    /// are not part of this picker.
    ///
    /// Original: iconpicture
    pub fn handle(
        &mut self,
        item: usize,
        preferences: &mut Preferences,
    ) -> Result<Taken, RuleError> {
        match item {
            0..=2 => Ok(Taken::closing(141)),
            9..=32 => {
                preferences.indoor_icon = FIRST_ICON + item as i16 - 9 + self.page;
                Ok(Taken::staying(130))
            }
            57 => {
                self.page = picker::next_page(self.page, false);
                Ok(Taken::staying(130))
            }
            _ => Err(RuleError::Refused { feedback: vec![] }),
        }
    }
    /// Draws the picker: the page of icons with its labels, the title, and the icon chosen so far.
    ///
    /// Original: iconpicture
    pub fn draw(&self, preferences: &Preferences, assets: &Assets, controls: &mut Vec<Control>) {
        let origin = vec2(0., 20.);
        draw_rectangle(0., 20., 640., 460., BLACK);
        let control = |rect, item| Control {
            rect,
            enabled: true,
            action: Action::Application(ApplicationAction::PreferenceItem(item)),
        };
        for number in 1..=assets.appearance_dialog.len() {
            let Some(rect) = picker::draw_item(assets, Picker::IndoorIcon, number, origin) else {
                continue;
            };
            if (9..=32).contains(&number) {
                let icon = assets
                    .icons
                    .get(&(FIRST_CHOICE + number as i16 + self.page));
                picker::draw_cell(assets, Picker::IndoorIcon, icon, rect, false);
                if icon.is_some() {
                    controls.push(control(rect, number));
                }
            } else if number == 45
                && let Some(icon) = assets.icons.get(&preferences.indoor_icon)
            {
                crate::assets::texture_in(icon, rect);
            } else if matches!(number, 2 | 57) {
                controls.push(control(rect, number));
            }
        }
        picker::print(
            assets,
            71,
            origin,
            &picker::title(assets, Picker::IndoorIcon),
        );
        picker::draw_texts(assets, Picker::IndoorIcon, None, self.page, origin);
        picker::draw_frames(assets, Picker::IndoorIcon, origin);
    }
}
