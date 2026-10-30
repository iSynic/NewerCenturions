use crate::{
    application::ApplicationAction,
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_content::{Content, ui, word};
use centurions_core::{Command, Event, RuleError, Session, apply};
use macroquad::prelude::*;

/// The warrior appearance picker: a draft of the session in which the current
/// warrior is given a new figure (cicn) or portrait: the six of the warrior's race, then 24
/// choices to a page.
pub struct Appearance {
    pub draft: Session,
    pub portrait: bool,
    pub page: i16,
}

impl Appearance {
    /// Opens the picker on the first page, working on a copy of the session.
    pub fn new(state: &Session, portrait: bool) -> Self {
        Self {
            draft: state.clone(),
            portrait,
            page: 0,
        }
    }
    /// The cicn or portrait resource shown in picker cell `item`: for the first six cells, one
    /// of the warrior's race; for the others, one from the current page.
    fn resource(&self, item: usize) -> Result<i16, RuleError> {
        let first = if self.portrait { 254 } else { 9254 };
        let offset = if item < 9 {
            (word(&self.draft.warriors[self.draft.char_select_new].bytes, 0x32)? - 1) * 6
        } else {
            self.page - 6
        };
        Ok(first + item as i16 + offset)
    }
    /// Answers a click in the picker: choose a figure or portrait, switch to another
    /// squad member, or turn to the next page (wrapping to the first).
    pub fn handle(&mut self, item: usize, content: &Content) -> Result<Vec<Event>, RuleError> {
        match item {
            3..=32 => apply(
                Command::SetWarriorAppearance {
                    warrior: self.draft.char_select_new,
                    portrait: self.portrait,
                    resource: self.resource(item)?,
                },
                &mut self.draft,
                content,
            ),
            33..=38 if item - 33 < self.draft.warriors.len() => {
                self.draft.char_select_new = item - 33;
                Ok(vec![Event::Sound { resource_id: 141 }])
            }
            57 => {
                let missing = self.portrait
                    && (9..=32).any(|item| {
                        self.resource(item).is_ok_and(|id| {
                            !content.resources.iter().any(|r| {
                                r.original.ends_with("/Portraits")
                                    && r.r#type == "cicn"
                                    && r.id == id
                            })
                        })
                    });
                self.page = if missing || self.page > 95 {
                    0
                } else {
                    self.page + 24
                };
                Ok(vec![Event::Sound { resource_id: 130 }])
            }
            _ => Err(RuleError::Refused { feedback: vec![] }),
        }
    }
    /// Draws the appearance dialog: the page of figures or portraits, each squad
    /// member's current choice, and the selected warrior's name, gender and race.
    pub fn draw(&self, assets: &Assets, controls: &mut Vec<Control>) {
        draw_rectangle(0., 20., 640., 460., BLACK);
        for (i, item) in assets.appearance_dialog.iter().enumerate() {
            let number = i + 1;
            let rect = rect(item);
            if rect.bottom() > 480. {
                continue;
            }
            if item.kind == 64 && item.data.len() == 2 {
                assets.panel(i16::from_be_bytes(item.data[..2].try_into().unwrap()), rect);
            }
            if item.kind == 8 && !matches!(number, 71 | 74..=80) {
                assets.dialog_text(&item.text(), rect);
            }
            let image = if (3..=32).contains(&number) {
                self.resource(number).ok().and_then(|id| {
                    if self.portrait {
                        assets.portraits.get(&id)
                    } else {
                        assets.icons.get(&id)
                    }
                })
            } else if (45..=56).contains(&number) {
                let (member, portrait) = if number < 51 {
                    (number - 45, false)
                } else {
                    (number - 51, true)
                };
                self.draft.warriors.get(member).and_then(|w| {
                    word(&w.bytes, if portrait { 0x54 } else { 0x56 })
                        .ok()
                        .and_then(|id| {
                            if portrait {
                                assets.portraits.get(&id)
                            } else {
                                assets.icons.get(&id)
                            }
                        })
                })
            } else {
                None
            };
            if let Some(image) = image {
                crate::assets::texture_in(image, rect);
            }
            if matches!(number, 2 | 57)
                || (3..=32).contains(&number) && image.is_some()
                || (33..=38).contains(&number) && number - 33 < self.draft.warriors.len()
            {
                controls.push(Control {
                    rect,
                    enabled: true,
                    action: Action::Application(ApplicationAction::AppearanceItem(number)),
                });
            }
        }
        let actor = &self.draft.warriors[self.draft.char_select_new];
        let title = assets
            .game_text
            .text(if self.portrait { 0x401a4c } else { 0x4034e2 }, true)
            .unwrap_or_default();
        text(assets, 71, &title);
        let gender = match actor.record().gender().unwrap_or(0) {
            1 => 0x401a46,
            2 => 0x401a3e,
            _ => 0x401a38,
        };
        text(
            assets,
            74,
            &assets.game_text.text(gender, true).unwrap_or_default(),
        );
        for (item, table, offset) in [(75, 129, 0x32), (76, 131, 0x34)] {
            if let Some(strings) = assets
                .character_strings
                .get(&table)
                .and_then(|b| ui::string_list(b).ok())
                && let Some(value) = word(&actor.bytes, offset)
                    .ok()
                    .and_then(|i| usize::try_from(i - 1).ok())
                    .and_then(|i| strings.get(i))
            {
                text(assets, item, value);
            }
        }
        for (i, warrior) in self.draft.warriors.iter().enumerate() {
            text(assets, 65 + i, &warrior.name);
        }
        let selected = rect(&assets.appearance_dialog[32 + self.draft.char_select_new]);
        draw_rectangle_lines(selected.x, selected.y, selected.w, selected.h, 1., YELLOW);
    }
}
/// The dialog item's rectangle, moved down below the menu bar.
fn rect(item: &ui::DialogItem) -> Rect {
    let [top, left, bottom, right] = item.rect;
    Rect::new(
        left as f32,
        top as f32 + 20.,
        (right - left) as f32,
        (bottom - top) as f32,
    )
}
/// Prints `value` in item `item` of the appearance dialog.
fn text(assets: &Assets, item: usize, value: &str) {
    if let Some(field) = assets.appearance_dialog.get(item - 1) {
        assets.dialog_text(value, rect(field));
    }
}
