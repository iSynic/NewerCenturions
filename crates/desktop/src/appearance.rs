use crate::{
    application::ApplicationAction,
    assets::Assets,
    picker::{self, Picker, Subject},
    presentation::{Action, Control},
};
use centurions_content::{Content, word};
use centurions_core::{Command, Event, RuleError, Session, apply};
use macroquad::prelude::*;

/// The warrior appearance picker: a draft of the session in which the current
/// warrior is given a new figure (cicn) or portrait: the six of the warrior's race, then 24
/// choices to a page.
pub struct Appearance {
    pub draft: Session,
    pub portrait: bool,
    pub page: i16,
    /// Whether the page button has turned a page since the dialog was last drawn in full.
    turned: bool,
}

impl Appearance {
    /// Opens the picker on the first page, working on a copy of the session.
    pub fn new(state: &Session, portrait: bool) -> Self {
        Self {
            draft: state.clone(),
            portrait,
            page: 0,
            turned: false,
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
    /// squad member, or turn to the next page (wrapping to the first, and from a page that lacks
    /// a portrait).
    ///
    /// Original: portrait, iconpicture
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
                self.turned = false;
                Ok(vec![Event::Sound { resource_id: 141 }])
            }
            57 => {
                let lacks = self.portrait && picker::page_lacks_portrait(content, self.page);
                self.page = picker::next_page(self.page, lacks);
                self.turned = true;
                Ok(vec![Event::Sound { resource_id: 130 }])
            }
            _ => Err(RuleError::Refused { feedback: vec![] }),
        }
    }
    /// Which picker this is.
    fn picker(&self) -> Picker {
        if self.portrait {
            Picker::Portrait
        } else {
            Picker::Icon
        }
    }
    /// Draws the appearance dialog: the page of figures or portraits and its labels, each squad
    /// member's current choice, and the selected warrior's name, gender and race. The portrait
    /// picker shows each member's portrait in the first column of figures and the icon in the
    /// second; the icon picker the other way round.
    ///
    /// Original: portrait, iconpicture
    pub fn draw(&self, assets: &Assets, controls: &mut Vec<Control>) {
        let origin = vec2(0., 20.);
        draw_rectangle(0., 20., 640., 460., BLACK);
        let actor = &self.draft.warriors[self.draft.char_select_new];
        let subject = Subject {
            race: actor.record().race().unwrap_or(0),
            mos: actor.record().mos().unwrap_or(0),
            gender: actor.record().gender().unwrap_or(0),
            portrait: actor.record().portrait().unwrap_or(0),
            icon: actor.record().icon().unwrap_or(0),
        };
        let picker = self.picker();
        for number in 1..=assets.appearance_dialog.len() {
            let Some(rect) = picker::draw_item(assets, picker, number, origin) else {
                continue;
            };
            let figures = if self.portrait {
                &assets.portraits
            } else {
                &assets.icons
            };
            if (3..=32).contains(&number) {
                let id = self.resource(number).ok();
                let image = id.and_then(|id| figures.get(&id));
                let selected = self.portrait && id == Some(subject.portrait);
                picker::draw_cell(assets, picker, image, rect, selected);
                if image.is_some() {
                    controls.push(control(rect, number));
                }
            } else if (45..=56).contains(&number) {
                let portrait = (number < 51) == self.portrait;
                if let Some(w) = self.draft.warriors.get((number - 45) % 6) {
                    let (id, table) = if portrait {
                        (w.record().portrait(), &assets.portraits)
                    } else {
                        (w.record().icon(), &assets.icons)
                    };
                    if let Some(image) = id.ok().and_then(|id| table.get(&id)) {
                        let at = if portrait {
                            crate::portrait::plot_box(Some(image), rect.point())
                        } else {
                            rect
                        };
                        crate::assets::texture_in(image, at);
                    }
                }
            }
            if matches!(number, 2 | 57)
                || (33..=38).contains(&number) && number - 33 < self.draft.warriors.len()
            {
                controls.push(control(rect, number));
            }
        }
        picker::print(assets, 71, origin, &picker::title(assets, picker));
        picker::draw_texts(assets, picker, Some(&subject), self.page, origin);
        picker::draw_frames(assets, picker, origin);
        if self.portrait {
            let leftover = if self.turned {
                33
            } else {
                self.draft.warriors.len()
            };
            picker::draw_leftover_edge(assets, origin, leftover);
        }
        for (i, warrior) in self.draft.warriors.iter().enumerate() {
            picker::print(assets, 65 + i, origin, &warrior.name);
        }
        let selected = picker::item_rect(assets, 33 + self.draft.char_select_new, origin);
        draw_rectangle_lines(selected.x, selected.y, selected.w, selected.h, 1., YELLOW);
    }
}
/// A click on item `number` of the appearance dialog.
fn control(rect: Rect, number: usize) -> Control {
    Control {
        rect,
        enabled: true,
        action: Action::Application(ApplicationAction::AppearanceItem(number)),
    }
}
