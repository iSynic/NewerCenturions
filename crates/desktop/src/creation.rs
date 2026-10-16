use crate::{
    application::ApplicationAction,
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_content::{Content, ui, word};
use centurions_core::{
    Command, Event, RuleError, Session, apply, character_display::Fields, creation::Request,
};
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Race,
    Mos,
    Gender,
    Level,
    Name,
    Portrait,
    Icon,
    Review,
    Scans,
    Keep,
}
pub struct Creation {
    pub read_only: bool,
    pub stage: Stage,
    pub draft: Session,
    pub request: Request,
    popup: bool,
    page: i16,
    pub fields: Option<Fields>,
    pub fields_attempted: bool,
}
impl Creation {
    pub fn new(state: &Session, content: &Content) -> Self {
        let mut draft = Session::new(content);
        draft.random = state.random.clone();
        draft.library_random_seed = state.library_random_seed;
        draft.library_random_draws = state.library_random_draws;
        Self {
            read_only: false,
            stage: Stage::Name,
            draft,
            request: Request {
                race: 0,
                mos: 0,
                gender: 0,
                level: 0,
                name: String::new(),
                portrait: 257,
                icon: 9257,
            },
            popup: false,
            page: 0,
            fields: None,
            fields_attempted: false,
        }
    }
    pub fn profile(
        state: &Session,
        content: &Content,
        member: usize,
        race: bool,
    ) -> Result<Self, RuleError> {
        let warrior = state
            .warriors
            .get(member)
            .ok_or("profile warrior missing")?;
        let mut view = Self::new(state, content);
        view.read_only = true;
        view.stage = if race { Stage::Race } else { Stage::Mos };
        view.request.race = word(&warrior.bytes, 0x32)?;
        view.request.mos = word(&warrior.bytes, 0x34)?;
        view.request.portrait = word(&warrior.bytes, 0x54)?;
        view.request.icon = word(&warrior.bytes, 0x56)?;
        Ok(view)
    }
    pub fn dialog(&self) -> i16 {
        match self.stage {
            Stage::Race => 190,
            Stage::Mos => 131,
            Stage::Gender => 133,
            Stage::Level => 161,
            Stage::Name => 132,
            Stage::Portrait | Stage::Icon => 170,
            _ => 128,
        }
    }
    pub fn choices(&self, content: &Content, assets: &Assets) -> Vec<(i16, String)> {
        let resource = if self.stage == Stage::Race { 129 } else { 131 };
        assets
            .character_strings
            .get(&resource)
            .and_then(|b| ui::string_list(b).ok())
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .take(30)
            .filter(|(i, name)| {
                !name.is_empty()
                    && (self.read_only
                        || self.stage == Stage::Mos
                        || content.map(0).is_some_and(|m| {
                            let at = i * 408 + 208 + self.request.mos as usize - 1;
                            m.races.get(at).is_some_and(|b| *b != 0)
                        }))
            })
            .map(|(i, name)| (i as i16 + 1, name))
            .collect()
    }
    pub fn choice(&mut self, value: i16, content: &Content) -> Result<Vec<Event>, RuleError> {
        match self.stage {
            Stage::Race => {
                if !(1..=30).contains(&value) {
                    return Err("Race choice is unavailable.".into());
                }
                let map = content.map(0).ok_or("Race profiles missing.")?;
                centurions_content::span(&map.races, (value as usize - 1) * 408, 408)?;
                self.request.race = value;
                self.request.portrait = value * 6 + 251;
                self.request.icon = self.request.portrait + 9000;
                if !self.read_only
                    && map.races[(value as usize - 1) * 408 + 208 + self.request.mos as usize - 1]
                        == 0
                {
                    return Err("Race is incompatible with MOS.".into());
                }
            }
            Stage::Mos => {
                if !(1..=30).contains(&value) {
                    return Err("MOS choice is unavailable.".into());
                }
                self.request.mos = value;
            }
            Stage::Gender if (1..=3).contains(&value) => self.request.gender = value,
            Stage::Level if [1, 3, 5, 10, 15, 20, 30].contains(&value) => {
                self.request.level = value
            }
            _ => return Err("Creation choice is unavailable.".into()),
        }
        self.popup = false;
        Ok(vec![Event::Sound { resource_id: 130 }])
    }
    pub fn item(
        &mut self,
        item: usize,
        state: &mut Session,
        content: &Content,
    ) -> Result<Vec<Event>, RuleError> {
        if matches!(self.stage, Stage::Race | Stage::Mos) && matches!(item, 57 | 58 | 74) {
            self.popup = !self.popup;
            return Ok(vec![]);
        }
        if self.stage == Stage::Gender && matches!(item, 2 | 3) {
            return self.choice(item as i16 - 1, content);
        }
        if self.stage == Stage::Level && (3..=9).contains(&item) {
            return self.choice([1, 3, 5, 10, 15, 20, 30][item - 3], content);
        }
        if matches!(self.stage, Stage::Portrait | Stage::Icon) {
            if item == 57 {
                self.page = if self.page >= 96 { 0 } else { self.page + 24 };
                return Ok(vec![Event::Sound { resource_id: 130 }]);
            }
            if (3..=32).contains(&item) {
                let id = if item < 9 {
                    item as i16 + 254 + (self.request.race - 1) * 6
                } else {
                    item as i16 + 248 + self.page
                };
                let id = if self.stage == Stage::Icon {
                    id + 9000
                } else {
                    id
                };
                if !content.resources.iter().any(|r| {
                    r.r#type == "cicn"
                        && r.id == id
                        && r.original.ends_with(if self.stage == Stage::Icon {
                            "/Tacticals"
                        } else {
                            "/Portraits"
                        })
                }) {
                    return Err("Appearance resource missing.".into());
                }
                if self.stage == Stage::Portrait {
                    self.request.portrait = id;
                } else {
                    self.request.icon = id;
                }
                return Ok(vec![Event::Sound { resource_id: 130 }]);
            }
        }
        if self.stage == Stage::Review && item == 65 {
            return self.roll(state, content);
        }
        let done = match self.stage {
            Stage::Race | Stage::Mos => matches!(item, 1 | 56),
            Stage::Gender => matches!(item, 1 | 6),
            Stage::Level => matches!(item, 1 | 29),
            Stage::Name => matches!(item, 1 | 4),
            Stage::Portrait | Stage::Icon => item < 3,
            Stage::Review => matches!(item, 1 | 66),
            _ => false,
        };
        if !done {
            return Err(RuleError::Refused { feedback: vec![] });
        }
        let (next, page) = match self.stage {
            Stage::Name if !self.request.name.trim().is_empty() => (Stage::Level, 131),
            Stage::Level if self.request.level != 0 => (Stage::Mos, 127),
            Stage::Mos if self.request.mos != 0 => (Stage::Race, 128),
            Stage::Race if self.request.race != 0 => {
                let map = content.map(0).ok_or("Race profiles missing")?;
                if word(&map.races, (self.request.race as usize - 1) * 408 + 194)? != 0 {
                    (Stage::Gender, 126)
                } else {
                    (Stage::Portrait, 129)
                }
            }
            Stage::Gender if self.request.gender != 0 => (Stage::Portrait, 129),
            Stage::Portrait => (Stage::Icon, 130),
            Stage::Icon => return self.roll(state, content),
            Stage::Review => {
                let mut events = vec![Event::Sound { resource_id: 130 }];
                if self.draft.warriors[0].psi_points()? != 0 {
                    events.extend(apply(Command::CreationScans, &mut self.draft, content)?);
                    self.stage = Stage::Scans;
                } else {
                    events.extend(self.finish(state, content)?);
                }
                return Ok(events);
            }
            _ => return Err(RuleError::Refused { feedback: vec![] }),
        };
        self.stage = next;
        self.page = 0;
        let mut events = vec![Event::Sound { resource_id: 141 }];
        if matches!(
            next,
            Stage::Mos | Stage::Gender | Stage::Level | Stage::Race
        ) {
            events.extend(apply(Command::CreationDialog, state, content)?);
        }
        if page != 0 {
            events.push(Event::Feedback(
                centurions_core::feedback::NativeFeedback::Tutorial {
                    first: page,
                    last: page,
                },
            ));
        }
        Ok(events)
    }
    pub fn finish(
        &mut self,
        state: &mut Session,
        content: &Content,
    ) -> Result<Vec<Event>, RuleError> {
        let events = apply(Command::CreationEquipment, &mut self.draft, content)?;
        state.random = self.draft.random.clone();
        state.library_random_seed = self.draft.library_random_seed;
        state.library_random_draws = self.draft.library_random_draws;
        self.stage = Stage::Keep;
        Ok(events)
    }
    fn roll(&mut self, state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
        self.draft.random = state.random.clone();
        let events = apply(
            Command::GenerateWarrior {
                request: self.request.clone(),
            },
            &mut self.draft,
            content,
        )?;
        state.random = self.draft.random.clone();
        state.library_random_seed = self.draft.library_random_seed;
        state.library_random_draws = self.draft.library_random_draws;
        self.stage = Stage::Review;
        self.fields = None;
        self.fields_attempted = false;
        Ok(events)
    }
    pub fn draw(&self, content: &Content, assets: &Assets, controls: &mut Vec<Control>) {
        if matches!(self.stage, Stage::Scans | Stage::Keep) {
            return;
        }
        let id = self.dialog();
        if id != 170 {
            assets.draw_dialog(id);
        } else {
            assets.panel(214, Rect::new(80., 80., 640., 460.));
            for item in [73, 93, 94] {
                let r = appearance_rect(assets, item);
                let picture = i16::from_be_bytes(
                    assets.appearance_dialog[item - 1].data[..2]
                        .try_into()
                        .unwrap(),
                );
                assets.panel(picture, r);
            }
        }
        let mut enabled = vec![];
        match self.stage {
            Stage::Race | Stage::Mos => {
                enabled.extend([56, 58, 74]);
                // race/caste draw the selected warrior in DITL item74.
                if let Some(image) = assets.portraits.get(&self.request.portrait) {
                    let rect = assets.dialog_rect(id, 74);
                    draw_texture_ex(
                        image,
                        rect.x,
                        rect.y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(rect.size()),
                            ..Default::default()
                        },
                    );
                }
                let chosen = if self.stage == Stage::Race {
                    self.request.race
                } else {
                    self.request.mos
                };
                if let Some((_, name)) = self
                    .choices(content, assets)
                    .iter()
                    .find(|(v, _)| *v == chosen)
                {
                    assets.wrapped_color(name, assets.dialog_rect(id, 58), 12, 14., YELLOW);
                }
                if let Some(map) = content.map(0) {
                    let (records, index, length) = if self.stage == Stage::Race {
                        (&map.races, self.request.race, 408)
                    } else {
                        (&map.profiles, self.request.mos, 576)
                    };
                    if index > 0
                        && let Ok(record) =
                            centurions_content::span(records, (index as usize - 1) * length, length)
                    {
                        let mut fields = Vec::new();
                        for (first, count, offset) in if self.stage == Stage::Race {
                            [
                                (2, 14, 74),
                                (16, 7, 60),
                                (36, 8, 0),
                                (59, 14, 16),
                                (28, 8, 44),
                            ]
                        } else {
                            [(2, 14, 110), (16, 7, 72), (59, 14, 0), (0, 0, 0), (0, 0, 0)]
                        } {
                            fields.extend((0..count).map(|i| (first + i, offset + i * 2)));
                        }
                        fields.extend(if self.stage == Stage::Race {
                            vec![
                                (44, 196),
                                (45, 198),
                                (46, 200),
                                (47, 202),
                                (50, 204),
                                (51, 206),
                            ]
                        } else {
                            vec![
                                (44, 258),
                                (45, 260),
                                (46, 266),
                                (49, 452),
                                (50, 268),
                                (51, 224),
                                (37, 264),
                                (38, 262),
                                (40, 222),
                                (90, 224),
                                (41, 234),
                                (91, 236),
                                (42, 238),
                                (92, 240),
                                (43, 242),
                                (93, 244),
                                (104, 250),
                                (106, 252),
                                (105, 246),
                                (107, 248),
                            ]
                        });
                        for (item, offset) in fields {
                            if let Ok(value) = word(record, offset) {
                                assets.wrapped_color(
                                    &value.to_string(),
                                    assets.dialog_rect(id, item),
                                    12,
                                    14.,
                                    YELLOW,
                                );
                            }
                        }
                    }
                }
            }
            Stage::Gender => {
                enabled.extend([2, 3, 6]);
                if self.request.gender > 0 {
                    let rect = assets.dialog_rect(id, self.request.gender as usize + 3);
                    assets.panel(179, rect);
                }
            }
            Stage::Level => {
                enabled.extend([3, 4, 5, 6, 7, 8, 9, 29]);
                if let Some(index) = [1, 3, 5, 10, 15, 20, 30]
                    .iter()
                    .position(|v| *v == self.request.level)
                {
                    let rect = assets.dialog_rect(id, index + 3);
                    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1., YELLOW);
                }
            }
            Stage::Name => {
                enabled.push(4);
                assets.wrapped_color(
                    &self.request.name,
                    assets.dialog_rect(id, 2),
                    18,
                    20.,
                    YELLOW,
                );
            }
            Stage::Portrait | Stage::Icon => {
                for item in 3..=32 {
                    let rid = if item < 9 {
                        item as i16 + 254 + (self.request.race - 1) * 6
                    } else {
                        item as i16 + 248 + self.page
                    };
                    let texture = if self.stage == Stage::Portrait {
                        assets.portraits.get(&rid)
                    } else {
                        assets.icons.get(&(rid + 9000))
                    };
                    if let Some(image) = texture {
                        let r = appearance_rect(assets, item);
                        draw_texture_ex(
                            image,
                            r.x,
                            r.y,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(r.size()),
                                ..Default::default()
                            },
                        );
                        enabled.push(item);
                    }
                }
                enabled.extend([2, 57]);
                let address = if self.stage == Stage::Portrait {
                    0x401a4c
                } else {
                    0x4034e2
                };
                if let Ok(title) = assets.native_text.text(address, true) {
                    assets.wrapped_color(&title, appearance_rect(assets, 71), 12, 14., YELLOW);
                }
                assets.wrapped_color(
                    &assets.appearance_dialog[71].text(),
                    appearance_rect(assets, 72),
                    12,
                    14.,
                    YELLOW,
                );
            }
            Stage::Review => {
                enabled.extend([65, 66]);
                for (item, image) in [
                    (68, assets.portraits.get(&self.request.portrait)),
                    (80, assets.icons.get(&self.request.icon)),
                ] {
                    if let Some(image) = image {
                        let rect = assets.dialog_rect(128, item);
                        draw_texture_ex(
                            image,
                            rect.x,
                            rect.y,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(rect.size()),
                                ..Default::default()
                            },
                        );
                    }
                }
                if let Some(fields) = &self.fields {
                    for (item, (value, size)) in fields {
                        if *item > 0 && *item as usize <= assets.dialogs[&128].len() {
                            assets.wrapped_color(
                                value,
                                assets.dialog_rect(128, *item as usize),
                                *size,
                                *size as f32 + 2.,
                                YELLOW,
                            );
                        }
                    }
                }
            }
            _ => {}
        }
        for item in enabled {
            let r = if id == 170 {
                appearance_rect(assets, item)
            } else {
                assets.dialog_rect(id, item)
            };
            controls.push(Control {
                rect: r,
                enabled: true,
                action: Action::Application(ApplicationAction::CreationItem(item)),
            });
        }
        if self.popup {
            let choices = self.choices(content, assets);
            let origin = assets.dialog_origin(id) + vec2(64., 40.);
            draw_rectangle(
                origin.x,
                origin.y,
                240.,
                choices.len() as f32 * 20.,
                Color::from_rgba(238, 238, 238, 255),
            );
            for (i, (value, name)) in choices.iter().enumerate() {
                let r = Rect::new(origin.x, origin.y + i as f32 * 20., 240., 20.);
                assets.menu_text(name, r.x + 4., r.y + 14., 12, BLACK);
                controls.push(Control {
                    rect: r,
                    enabled: true,
                    action: Action::Application(ApplicationAction::CreationChoice(*value)),
                });
            }
        }
    }
}
fn appearance_rect(assets: &Assets, item: usize) -> Rect {
    let [top, left, bottom, right] = assets.appearance_dialog[item - 1].rect;
    Rect::new(
        80. + left as f32,
        80. + top as f32,
        (right - left) as f32,
        (bottom - top) as f32,
    )
}
