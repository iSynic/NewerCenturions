use crate::{
    application::ApplicationAction,
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_content::records::{ProfileTable, RACE_PROFILE_BYTES};
use centurions_content::{Content, ui, word};
use centurions_core::{
    Command, Event, RuleError, Session, apply, character_display::Fields, creation::Request,
};
use macroquad::prelude::*;

/// Done button of the race (DITL 190) and M.O.S. (DITL 131) dialogs: item 56.
pub const RACE_DONE: usize = 56;
/// Done button of the gender dialog (DITL 133): item 6.
pub const GENDER_DONE: usize = 6;
/// Done button of the level dialog (DITL 161): item 29.
pub const LEVEL_DONE: usize = 29;
/// Done button of the name dialog (DITL 132): item 4.
pub const NAME_DONE: usize = 4;
/// Done control of the portrait and icon dialog (DITL 170): item 2.
pub const APPEARANCE_DONE: usize = 2;
/// Done control of the review dialog (DITL 128): item 66.
pub const REVIEW_DONE: usize = 66;

/// The steps of making a new warrior, each with its own dialog and taken in this order: name,
/// level, M.O.S., race, gender (only for a race that offers a choice), portrait, icon and
/// review, then the scan choice, and last keeping the finished warrior.
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
/// A new warrior being made on a draft of the session, with the choices made so
/// far; `read_only` shows a warrior's race or M.O.S. profile without changing
/// anything.
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
    /// Starts a new warrior at the name step, on a fresh draft that shares the
    /// session's random state.
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
    /// Shows the race or M.O.S. profile of squad member `member` as the same dialog
    /// used to choose them, read only.
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
        view.request.race = warrior.record().race()?;
        view.request.mos = warrior.record().profession()?;
        view.request.portrait = warrior.record().portrait()?;
        view.request.icon = warrior.record().icon()?;
        Ok(view)
    }
    /// The dialog resource for the current step.
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
    /// The race (STR# 129) or M.O.S. (STR# 131) names that can be chosen, with their
    /// numbers; a race is offered only if it allows the chosen M.O.S.
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
                            let at = i * RACE_PROFILE_BYTES + 208 + self.request.mos as usize - 1;
                            m.races.get(at).is_some_and(|b| *b != 0)
                        }))
            })
            .map(|(i, name)| (i as i16 + 1, name))
            .collect()
    }
    /// Takes a choice at the current step (race, M.O.S., gender or level).
    pub fn choice(&mut self, value: i16, content: &Content) -> Result<Vec<Event>, RuleError> {
        match self.stage {
            Stage::Race => {
                if !(1..=30).contains(&value) {
                    return Err("Race choice is unavailable.".into());
                }
                let map = content.map(0).ok_or("Race profiles missing.")?;
                ProfileTable::races(&map.races).row(value as usize - 1)?;
                self.request.race = value;
                self.request.portrait = value * 6 + 251;
                self.request.icon = self.request.portrait + 9000;
                if !self.read_only
                    && map.races[(value as usize - 1) * RACE_PROFILE_BYTES
                        + 208
                        + self.request.mos as usize
                        - 1]
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
    /// Carries out a click on a dialog item of the current step.
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
            Stage::Race | Stage::Mos => matches!(item, 1 | RACE_DONE),
            Stage::Gender => matches!(item, 1 | GENDER_DONE),
            Stage::Level => matches!(item, 1 | LEVEL_DONE),
            Stage::Name => matches!(item, 1 | NAME_DONE),
            Stage::Portrait | Stage::Icon => item < 3,
            Stage::Review => matches!(item, 1 | REVIEW_DONE),
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
                if word(
                    &map.races,
                    (self.request.race as usize - 1) * RACE_PROFILE_BYTES + 194,
                )? != 0
                {
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
                centurions_core::feedback::GameFeedback::Tutorial {
                    first: page,
                    last: page,
                },
            ));
        }
        Ok(events)
    }
    /// Gives the new warrior its starting equipment, returns the random state to the
    /// session and keeps the warrior.
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
    /// Generates the warrior's record from the choices made, using the session's
    /// random state.
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
    /// Draws the dialog for the current step with its choices and controls.
    pub fn draw(&self, content: &Content, assets: &Assets, controls: &mut Vec<Control>) {
        if matches!(self.stage, Stage::Scans | Stage::Keep) {
            return;
        }
        let id = self.dialog();
        if id != 170 {
            assets.draw_dialog(id);
        } else {
            assets.panel(214, crate::layout::DIALOG_FRAME);
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
                self.draw_record_fields(content, assets, id);
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
                        crate::assets::texture_in(image, r);
                        enabled.push(item);
                    }
                }
                enabled.extend([2, 57]);
                let address = if self.stage == Stage::Portrait {
                    0x401a4c
                } else {
                    0x4034e2
                };
                if let Ok(title) = assets.game_text.text(address, true) {
                    assets.dialog_text(&title, appearance_rect(assets, 71));
                }
                assets.dialog_text(
                    &assets.appearance_dialog[71].text(),
                    appearance_rect(assets, 72),
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
                        crate::assets::texture_in(image, rect);
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

    /// Race or MOS: the warrior's portrait, the chosen name and the chosen
    /// row's fields in their dialog items.
    fn draw_record_fields(&self, content: &Content, assets: &Assets, id: i16) {
        if let Some(image) = assets.portraits.get(&self.request.portrait) {
            let rect = assets.dialog_rect(id, 74);
            crate::assets::texture_in(image, rect);
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
            assets.dialog_text(name, assets.dialog_rect(id, 58));
        }
        if let Some(map) = content.map(0) {
            let (table, index, runs, singles) = if self.stage == Stage::Race {
                (
                    ProfileTable::races(&map.races),
                    self.request.race,
                    &RACE_FIELD_RUNS[..],
                    &RACE_FIELDS[..],
                )
            } else {
                (
                    ProfileTable::professions(&map.profiles),
                    self.request.mos,
                    &PROFILE_FIELD_RUNS[..],
                    &PROFILE_FIELDS[..],
                )
            };
            if index > 0
                && let Ok(record) = table.row(index as usize - 1)
            {
                let mut fields = Vec::new();
                for &(first, count, offset) in runs {
                    fields.extend((0..count).map(|i| (first + i, offset + i * 2)));
                }
                fields.extend_from_slice(singles);
                for (item, offset) in fields {
                    if let Ok(value) = word(record, offset) {
                        assets.dialog_text(&value.to_string(), assets.dialog_rect(id, item));
                    }
                }
            }
        }
    }
}
/// Item `item` of the appearance dialog as a rectangle in the window.
fn appearance_rect(assets: &Assets, item: usize) -> Rect {
    let [top, left, bottom, right] = assets.appearance_dialog[item - 1].rect;
    Rect::new(
        80. + left as f32,
        80. + top as f32,
        (right - left) as f32,
        (bottom - top) as f32,
    )
}

/// Race dialog runs of 16-bit fields: (first DITL item, count, first record offset).
const RACE_FIELD_RUNS: [(usize, usize, usize); 5] = [
    (2, 14, 74),
    (16, 7, 60),
    (36, 8, 0),
    (59, 14, 16),
    (28, 8, 44),
];
/// Race dialog single 16-bit fields: (DITL item, record offset).
const RACE_FIELDS: [(usize, usize); 6] = [
    (44, 196),
    (45, 198),
    (46, 200),
    (47, 202),
    (50, 204),
    (51, 206),
];
/// MOS dialog runs of 16-bit fields: (first DITL item, count, first record offset).
const PROFILE_FIELD_RUNS: [(usize, usize, usize); 3] = [(2, 14, 110), (16, 7, 72), (59, 14, 0)];
/// MOS dialog single 16-bit fields: (DITL item, record offset).
const PROFILE_FIELDS: [(usize, usize); 20] = [
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
];
