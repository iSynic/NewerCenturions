use crate::{
    application::ApplicationAction,
    assets::Assets,
    game_strings,
    picker::{self, Picker, Subject},
    placement::Placement,
    presentation::{Action, Control},
};
use centurions_content::records::{ProfileTable, RACE_PROFILE_BYTES};
use centurions_content::{Content, ui, word};
use centurions_core::{
    Command, Event, RuleError, Session, apply,
    character_display::Fields,
    creation::Request,
    feedback::{ERROR_SOUND, GameFeedback},
    item_display::signed_or_blank,
    roster::Roster,
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

/// The most pixels a warrior's name may take in the name dialog's 15-point text.
const NAME_WIDTH_LIMIT: f32 = 133.;
/// The most characters a warrior's name may have.
const NAME_LENGTH_LIMIT: usize = 28;
/// Warning 44: the name is too long.
const NAME_TOO_LONG_WARNING: i16 = 44;
/// Question 11: whether to cancel the new warrior.
const CANCEL_QUESTION: usize = 11;
/// Question 4: whether to replace the kept warrior of the same name.
const REPLACE_QUESTION: usize = 4;
/// Question 2: whether to keep the finished warrior.
const KEEP_QUESTION: usize = 2;
/// The starting levels of the level dialog (DITL 161), on items 3 to 9.
const LEVELS: [i16; 7] = [1, 3, 5, 10, 15, 20, 30];

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
/// A yes/no question raised over a step of warrior creation.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Ask {
    /// Whether to cancel making the new warrior.
    Cancel,
    /// Whether the new warrior replaces the kept one of the same name.
    Replace,
}
/// What a yes or no to a question of warrior creation leads to.
pub enum Answered {
    /// The new warrior is abandoned.
    Cancelled,
    /// Creation goes on; these are the events of the answer.
    Continue(Vec<Event>),
}
/// A new warrior being made on a draft of the session, with the choices made so
/// far; `read_only` shows a warrior's race or M.O.S. profile without changing
/// anything.
pub struct Creation {
    pub read_only: bool,
    pub stage: Stage,
    pub draft: Session,
    pub request: Request,
    /// Where the open race or M.O.S. list has its top left: the point the player clicked.
    popup: Option<Vec2>,
    /// Where the player last clicked in the dialog.
    pub click: Vec2,
    /// The gender whose box the last choice cleared, which shows the empty box.
    cleared_gender: i16,
    page: i16,
    /// Whether the page button has turned a page since the step opened.
    turned: bool,
    pub fields: Option<Fields>,
    pub fields_attempted: bool,
    /// The question waiting for an answer, if any.
    pub ask: Option<Ask>,
    /// The width of the typed name in the name dialog's text, in pixels.
    pub name_width: f32,
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
            popup: None,
            click: Vec2::ZERO,
            cleared_gender: 0,
            page: 0,
            turned: false,
            fields: None,
            fields_attempted: false,
            ask: None,
            name_width: 0.,
        }
    }
    /// Shows the race or M.O.S. profile of squad member `member` as the same dialog
    /// used to choose them, read only. The caller draws its opening sound through the
    /// creation command, as choosing does.
    ///
    /// Original: Race, Caste
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
        view.request.mos = warrior.record().mos()?;
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
    /// The race (STR# 129) or M.O.S. (STR# 131) names that are listed, with their numbers.
    pub fn choices(&self, assets: &Assets) -> Vec<(i16, String)> {
        let resource = if self.stage == Stage::Race { 129 } else { 131 };
        assets
            .character_strings
            .get(&resource)
            .and_then(|b| ui::string_list(b).ok())
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .take(30)
            .filter(|(_, name)| !name.is_empty())
            .map(|(i, name)| (i as i16 + 1, name))
            .collect()
    }
    /// Whether a listed race can be chosen: it must allow the chosen M.O.S. A profile that is only
    /// shown offers every race.
    ///
    /// Original: Race
    fn selectable(&self, content: &Content, value: i16) -> bool {
        self.read_only
            || self.stage == Stage::Mos
            || content.map(0).is_some_and(|m| {
                let at =
                    (value as usize - 1) * RACE_PROFILE_BYTES + 208 + self.request.mos as usize - 1;
                m.races.get(at).is_some_and(|b| *b != 0)
            })
    }
    /// The portrait shown beside a listed race or M.O.S.: a race shows the portrait picture of
    /// its number, an M.O.S. its own portrait.
    ///
    /// Original: Race, Caste
    fn listed_portrait(&self, content: &Content, value: i16) -> Option<i16> {
        let map = content.map(0)?;
        if self.stage == Stage::Race {
            let row = ProfileTable::races(&map.races)
                .row(value as usize - 1)
                .ok()?;
            Some(word(row, 334).ok()?.wrapping_mul(6).wrapping_add(251))
        } else {
            let row = ProfileTable::mos_profiles(&map.profiles)
                .row(value as usize - 1)
                .ok()?;
            word(row, 450).ok()
        }
    }
    /// Takes a choice at the current step (race, M.O.S., gender or level). Choosing a race or an
    /// M.O.S. from the list is silent; the gender and level buttons click.
    ///
    /// Original: Race, Caste, Gender, Startlevel
    pub fn choice(&mut self, value: i16, content: &Content) -> Result<Vec<Event>, RuleError> {
        let mut sound = None;
        match self.stage {
            Stage::Race => {
                if !(1..=30).contains(&value) {
                    return Err("Race choice is unavailable.".into());
                }
                let map = content.map(0).ok_or("Race profiles missing.")?;
                ProfileTable::races(&map.races).row(value as usize - 1)?;
                if !self.selectable(content, value) {
                    return Err("Race is incompatible with MOS.".into());
                }
                self.request.race = value;
                self.request.portrait = value * 6 + 251;
                self.request.icon = self.request.portrait + 9000;
            }
            Stage::Mos => {
                if !(1..=30).contains(&value) {
                    return Err("MOS choice is unavailable.".into());
                }
                let map = content.map(0).ok_or("MOS profiles missing.")?;
                let row = ProfileTable::mos_profiles(&map.profiles).row(value as usize - 1)?;
                self.request.portrait = word(row, 450)?;
                self.request.mos = value;
            }
            Stage::Gender if (1..=3).contains(&value) => {
                // The first choice clears the male box, as if male had been ticked before.
                self.cleared_gender = self.request.gender.max(1);
                self.request.gender = value;
                sound = Some(141);
            }
            Stage::Level if LEVELS.contains(&value) => {
                self.request.level = value;
                sound = Some(130);
            }
            _ => return Err("Creation choice is unavailable.".into()),
        }
        self.popup = None;
        Ok(sound
            .map(|resource_id| Event::Sound { resource_id })
            .into_iter()
            .collect())
    }
    /// The tutorial pages the step raises when it opens; each shows once.
    ///
    /// Original: Name, Startlevel, Caste, Race, Gender, portrait, icon_picture
    fn tutorials(stage: Stage) -> &'static [usize] {
        match stage {
            Stage::Name => &[125],
            Stage::Level => &[131],
            Stage::Mos => &[127, 132],
            Stage::Race => &[128, 133],
            Stage::Gender => &[126],
            Stage::Portrait => &[129],
            Stage::Icon => &[130],
            _ => &[],
        }
    }
    /// The tutorial pages a step raises as events.
    ///
    /// Original: Name, Startlevel, Caste, Race, Gender
    pub fn tutorial_events(stage: Stage) -> Vec<Event> {
        Self::tutorials(stage)
            .iter()
            .map(|page| {
                Event::Feedback(GameFeedback::Tutorial {
                    first: *page,
                    last: *page,
                })
            })
            .collect()
    }
    /// The number of the question waiting for an answer, if any.
    pub fn question(&self) -> Option<usize> {
        match (self.stage, self.ask) {
            (Stage::Keep, _) => Some(KEEP_QUESTION),
            (_, Some(Ask::Cancel)) => Some(CANCEL_QUESTION),
            (_, Some(Ask::Replace)) => Some(REPLACE_QUESTION),
            _ => None,
        }
    }
    /// Carries out a click on a dialog item of the current step.
    ///
    /// Original: Name, Startlevel, Gender, Caste, Race
    pub fn item(
        &mut self,
        item: usize,
        state: &mut Session,
        content: &Content,
        roster: &mut Roster,
    ) -> Result<Vec<Event>, RuleError> {
        if matches!(self.stage, Stage::Race | Stage::Mos) && matches!(item, 57 | 58 | 74) {
            self.popup = if self.popup.is_some() {
                None
            } else {
                Some(self.click)
            };
            return Ok(vec![]);
        }
        if self.stage == Stage::Gender && matches!(item, 2 | 3) {
            return self.choice(item as i16 - 1, content);
        }
        if self.stage == Stage::Level && (3..=9).contains(&item) {
            return self.choice(LEVELS[item - 3], content);
        }
        if matches!(self.stage, Stage::Portrait | Stage::Icon) {
            if item == 57 {
                let lacks = self.stage == Stage::Portrait
                    && picker::page_lacks_portrait(content, self.page);
                self.page = picker::next_page(self.page, lacks);
                self.turned = true;
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
        let (next, sound) = match self.stage {
            Stage::Name => return self.accept_name(roster, state, content),
            Stage::Level if self.request.level != 0 => {
                // The name is listed with the chosen item's number less two as its level word.
                let item = LEVELS
                    .iter()
                    .position(|level| *level == self.request.level)
                    .ok_or("Creation level is unavailable.")?
                    + 3;
                roster.start_level(&self.request.name, item as i16 - 2)?;
                (Stage::Mos, Some(6001))
            }
            Stage::Mos if self.request.mos != 0 => (Stage::Race, Some(214)),
            Stage::Race if self.request.race != 0 => {
                if !self.read_only {
                    self.request.portrait = self.request.race * 6 + 251;
                    self.request.icon = self.request.portrait + 9000;
                }
                let map = content.map(0).ok_or("Race profiles missing")?;
                if word(
                    &map.races,
                    (self.request.race as usize - 1) * RACE_PROFILE_BYTES + 194,
                )? != 0
                {
                    (Stage::Gender, Some(214))
                } else {
                    (Stage::Portrait, Some(214))
                }
            }
            Stage::Gender if self.request.gender != 0 => (Stage::Portrait, None),
            Stage::Level | Stage::Gender => {
                return Err(RuleError::Refused {
                    feedback: vec![GameFeedback::Sound {
                        resource_id: ERROR_SOUND,
                    }],
                });
            }
            Stage::Portrait => (Stage::Icon, Some(141)),
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
        self.advance(next, sound, state, content)
    }
    /// Takes the step after the current one: the sound that closes the old step, the opening of
    /// the new step's dialog, and its tutorial pages. A new warrior starts the M.O.S. step with
    /// the first M.O.S. and its portrait, and the race step with the first race.
    ///
    /// Original: Caste, Race
    fn advance(
        &mut self,
        next: Stage,
        sound: Option<i16>,
        state: &mut Session,
        content: &Content,
    ) -> Result<Vec<Event>, RuleError> {
        self.stage = next;
        self.page = 0;
        self.turned = false;
        if !self.read_only {
            match next {
                Stage::Mos => {
                    self.request.mos = 1;
                    self.request.portrait = 257;
                }
                Stage::Race => self.request.race = 1,
                _ => {}
            }
        }
        let mut events: Vec<Event> = sound
            .map(|resource_id| Event::Sound { resource_id })
            .into_iter()
            .collect();
        if matches!(
            next,
            Stage::Mos | Stage::Gender | Stage::Level | Stage::Race
        ) {
            events.extend(apply(Command::CreationDialog, state, content)?);
        }
        events.extend(Self::tutorial_events(next));
        Ok(events)
    }
    /// Done in the name dialog. An empty name sounds the error and asks whether to cancel the new
    /// warrior. A name too wide for the dialog's text or longer than the limit is refused with the
    /// warning that it is too long. A name another kept warrior has is flashed with the error
    /// sound, then asked about replacing. Any other name goes on to the level.
    ///
    /// Original: Name
    fn accept_name(
        &mut self,
        roster: &Roster,
        state: &mut Session,
        content: &Content,
    ) -> Result<Vec<Event>, RuleError> {
        if self.request.name.is_empty() {
            self.ask = Some(Ask::Cancel);
            return Ok(vec![Event::Sound {
                resource_id: ERROR_SOUND,
            }]);
        }
        if self.name_width > NAME_WIDTH_LIMIT
            || self.request.name.chars().count() > NAME_LENGTH_LIMIT
        {
            return Err(RuleError::Refused {
                feedback: vec![GameFeedback::Warning {
                    index: NAME_TOO_LONG_WARNING,
                }],
            });
        }
        if roster.holds_name(&self.request.name) {
            self.ask = Some(Ask::Replace);
            return Ok(vec![Event::Feedback(GameFeedback::Flash {
                address: game_strings::NAME_KEPT_ALREADY,
                x: 100,
                y: 200,
                wait: 0,
                sound: ERROR_SOUND,
            })]);
        }
        self.advance(Stage::Level, None, state, content)
    }
    /// Takes the answer to the question waiting. Cancelling abandons the warrior; replacing a
    /// kept warrior of the same name goes on to the level; refusing to replace it opens the name
    /// dialog again.
    ///
    /// Original: Name
    pub fn answer(
        &mut self,
        yes: bool,
        state: &mut Session,
        content: &Content,
    ) -> Result<Answered, RuleError> {
        let ask = self
            .ask
            .take()
            .ok_or("Creation confirmation is not open.")?;
        Ok(match (ask, yes) {
            (Ask::Cancel, true) => Answered::Cancelled,
            (Ask::Cancel, false) => Answered::Continue(vec![]),
            (Ask::Replace, true) => {
                Answered::Continue(self.advance(Stage::Level, None, state, content)?)
            }
            (Ask::Replace, false) => {
                self.request.name.clear();
                self.name_width = 0.;
                Answered::Continue(apply(Command::CreationDialog, state, content)?)
            }
        })
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
    /// Where the current step's dialog stands: the name entry and the starting level are moved by
    /// their routines, and the other dialogs stay centred on the screen.
    ///
    /// Original: Name, Startlevel
    fn origin(&self, assets: &Assets, placement: Placement) -> Vec2 {
        match self.stage {
            Stage::Name => placement.name_entry(),
            Stage::Level => placement.starting_level(),
            _ => assets.dialog_origin(self.dialog()),
        }
    }
    /// Draws the dialog for the current step with its choices and controls.
    pub fn draw(
        &self,
        content: &Content,
        assets: &Assets,
        placement: Placement,
        controls: &mut Vec<Control>,
    ) {
        if matches!(self.stage, Stage::Scans | Stage::Keep) {
            return;
        }
        let id = self.dialog();
        let origin = self.origin(assets, placement);
        if id != 170 {
            assets.draw_dialog_at(id, origin);
        } else {
            assets.panel(214, crate::layout::DIALOG_FRAME);
            for (i, item) in assets.appearance_dialog.iter().enumerate() {
                if item.kind == 64 && item.data.len() == 2 && i != 57 {
                    let picture = i16::from_be_bytes(item.data[..2].try_into().unwrap());
                    assets.panel(picture, picker::item_rect(assets, i + 1, PICKER_ORIGIN));
                }
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
                    let box_rect = |gender: i16| assets.dialog_rect(id, gender as usize + 3);
                    let cleared = self.cleared_gender.max(1);
                    if cleared != self.request.gender {
                        assets.panel(crate::layout::EMPTY_BOX, box_rect(cleared));
                    }
                    assets.panel(crate::layout::TICKED_BOX, box_rect(self.request.gender));
                }
            }
            Stage::Level => {
                enabled.extend([3, 4, 5, 6, 7, 8, 9, 29]);
                if let Some(index) = LEVELS.iter().position(|v| *v == self.request.level) {
                    let rect = assets.dialog_rect_at(id, index + 3, origin);
                    crate::presentation::bevel::draw_edge(assets, rect, false);
                }
            }
            Stage::Name => {
                enabled.push(4);
                assets.wrapped_color(
                    &self.request.name,
                    assets.dialog_rect_at(id, 2, origin),
                    18,
                    crate::assets::line_height(18),
                    YELLOW,
                );
            }
            Stage::Portrait | Stage::Icon => self.draw_picker(assets, &mut enabled),
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
                                crate::assets::line_height(*size),
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
                picker::item_rect(assets, item, PICKER_ORIGIN)
            } else {
                assets.dialog_rect_at(id, item, origin)
            };
            controls.push(Control {
                rect: r,
                enabled: true,
                action: Action::Application(ApplicationAction::CreationItem(item)),
            });
        }
        if let Some(click) = self.popup {
            let choices = self.choices(assets);
            let height = choices.len() as f32 * POPUP_ROW;
            let at = vec2(
                click.x.min(crate::layout::SIZE.x - POPUP_WIDTH - 1.),
                click
                    .y
                    .min(crate::layout::SIZE.y - height - 1.)
                    .max(crate::layout::MENU_BAR),
            );
            draw_rectangle(
                at.x,
                at.y,
                POPUP_WIDTH,
                height,
                Color::from_rgba(238, 238, 238, 255),
            );
            draw_rectangle_lines(
                at.x - 1.,
                at.y - 1.,
                POPUP_WIDTH + 2.,
                height + 2.,
                1.,
                BLACK,
            );
            draw_rectangle(at.x + 1., at.y + height + 1., POPUP_WIDTH + 1., 1., BLACK);
            draw_rectangle(at.x + POPUP_WIDTH + 1., at.y + 1., 1., height + 1., BLACK);
            for (i, (value, name)) in choices.iter().enumerate() {
                let r = Rect::new(at.x, at.y + i as f32 * POPUP_ROW, POPUP_WIDTH, POPUP_ROW);
                let selectable = self.selectable(content, *value);
                if let Some(image) = self
                    .listed_portrait(content, *value)
                    .and_then(|portrait| assets.portraits.get(&portrait))
                {
                    crate::assets::texture_in(image, Rect::new(r.x + 2., r.y + 2., 16., 16.));
                }
                let ink = if selectable {
                    BLACK
                } else {
                    Color::from_rgba(150, 150, 150, 255)
                };
                assets.menu_text(name, r.x + 22., r.y + 14., 12, ink);
                if selectable {
                    controls.push(Control {
                        rect: r,
                        enabled: true,
                        action: Action::Application(ApplicationAction::CreationChoice(*value)),
                    });
                }
            }
        }
    }

    /// The portrait or icon step: the page of choices with its labels, the title and caption, the
    /// warrior's race and M.O.S., and the preview of the figure chosen so far with the new
    /// warrior's name. A portrait is marked once chosen. The cells that have a figure are
    /// enabled.
    ///
    /// Original: portrait, iconpicture
    fn draw_picker(&self, assets: &Assets, enabled: &mut Vec<usize>) {
        let kind = if self.stage == Stage::Portrait {
            Picker::Portrait
        } else {
            Picker::Icon
        };
        let rect = |item| picker::item_rect(assets, item, PICKER_ORIGIN);
        for item in 3..=32 {
            let id = if item < 9 {
                item as i16 + 254 + (self.request.race - 1) * 6
            } else {
                item as i16 + 248 + self.page
            };
            let texture = if kind == Picker::Portrait {
                assets.portraits.get(&id)
            } else {
                assets.icons.get(&(id + 9000))
            };
            let chosen = kind == Picker::Portrait && id == self.request.portrait;
            picker::draw_cell(assets, kind, texture, rect(item), chosen);
            if texture.is_some() {
                enabled.push(item);
            }
        }
        enabled.extend([2, 57]);
        picker::print(assets, 71, PICKER_ORIGIN, &picker::title(assets, kind));
        picker::print(
            assets,
            72,
            PICKER_ORIGIN,
            &assets.appearance_dialog[71].text(),
        );
        let subject = Subject {
            race: self.request.race,
            mos: self.request.mos,
            gender: self.request.gender,
            portrait: self.request.portrait,
            icon: self.request.icon,
        };
        picker::draw_texts(assets, kind, Some(&subject), self.page, PICKER_ORIGIN);
        picker::draw_frames(assets, kind, PICKER_ORIGIN);
        let portrait = assets.portraits.get(&subject.portrait);
        if kind == Picker::Portrait {
            assets.panel(154, rect(33));
            picker::draw_leftover_edge(assets, PICKER_ORIGIN, if self.turned { 33 } else { 9 });
            if let Some(image) = portrait {
                let at = crate::portrait::plot_box(Some(image), rect(45).point());
                crate::assets::texture_in(image, at);
            }
        } else {
            if let Some(image) = portrait {
                let at = crate::portrait::plot_box(Some(image), rect(51).point());
                crate::assets::texture_in(image, at);
            }
            if let Some(image) = assets.icons.get(&subject.icon) {
                crate::assets::texture_in(image, rect(45));
            }
            crate::presentation::bevel::draw_edge(assets, rect(57), false);
        }
        picker::print(assets, 65, PICKER_ORIGIN, &self.request.name);
    }

    /// Race or M.O.S.: the warrior's portrait, the chosen name and the chosen row's fields in
    /// their dialog items. Rows that hide zero show a bonus with its sign.
    ///
    /// Original: Race, Caste
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
        if let Some((_, name)) = self.choices(assets).iter().find(|(v, _)| *v == chosen) {
            assets.dialog_text(name, assets.dialog_rect(id, 58));
        }
        let Some(map) = content.map(0) else {
            return;
        };
        let race = self.stage == Stage::Race;
        let (table, index, runs, singles) = if race {
            (
                ProfileTable::races(&map.races),
                self.request.race,
                &RACE_FIELD_RUNS[..],
                &RACE_FIELDS[..],
            )
        } else {
            (
                ProfileTable::mos_profiles(&map.profiles),
                self.request.mos,
                &PROFILE_FIELD_RUNS[..],
                &PROFILE_FIELDS[..],
            )
        };
        let Some(record) = (index > 0)
            .then(|| table.row(index as usize - 1).ok())
            .flatten()
        else {
            return;
        };
        let number = |offset: usize, hides_zero: bool| {
            word(record, offset).ok().map(|value| {
                if hides_zero {
                    signed_or_blank(value)
                } else {
                    value.to_string()
                }
            })
        };
        let mut texts: Vec<(usize, String)> = Vec::new();
        for &(first, count, offset, hides_zero) in runs {
            for i in 0..count {
                texts.extend(number(offset + i * 2, hides_zero).map(|text| (first + i, text)));
            }
        }
        for &(item, offset, hides_zero) in singles {
            texts.extend(number(offset, hides_zero).map(|text| (item, text)));
        }
        let conditions = assets
            .character_strings
            .get(&133)
            .and_then(|b| ui::string_list(b).ok())
            .unwrap_or_default();
        if race {
            let regenerates = if record.get(333).is_some_and(|b| *b != 0) {
                game_strings::REGENERATES_YES
            } else {
                game_strings::REGENERATES_NO
            };
            texts.extend(
                assets
                    .game_text
                    .text(regenerates, true)
                    .ok()
                    .map(|text| (49, text)),
            );
        } else {
            for (offset, first) in [(86, 94), (92, 95), (98, 96)] {
                if word(record, offset).is_ok_and(|flag| flag != 0) {
                    texts.extend(number(offset + 2, false).map(|text| (first, text)));
                    texts.extend(number(offset + 4, false).map(|text| (first + 3, text)));
                }
            }
        }
        let first_condition = if race { 112 } else { 138 };
        let shown = (0..40)
            .filter_map(|i| {
                let value = word(record, first_condition + i * 2).ok()?;
                (value != 0).then(|| (conditions.get(i).cloned().unwrap_or_default(), value))
            })
            .take(4);
        for (k, (name, value)) in shown.enumerate() {
            texts.push((52 + k, name));
            if !race {
                texts.push((100 + k, value.wrapping_abs().to_string()));
            }
        }
        for (item, text) in texts {
            if !text.is_empty() {
                assets.dialog_text(&text, assets.dialog_rect(id, item));
            }
        }
    }
}
/// Where the picker dialog (DITL 170) has its top left in the window.
const PICKER_ORIGIN: Vec2 = picker::ORIGIN;
/// The width of the race or M.O.S. list.
const POPUP_WIDTH: f32 = 240.;
/// The height of a row of the race or M.O.S. list.
const POPUP_ROW: f32 = 20.;

/// A run of 16-bit fields in the race or M.O.S. dialog: first dialog item, count, first record
/// offset, and whether the row hides zero and shows signs.
type FieldRun = (usize, usize, usize, bool);
/// A single 16-bit field in the race or M.O.S. dialog: dialog item, record offset, and whether
/// it hides zero and shows signs.
type Field = (usize, usize, bool);

/// Race dialog runs of fields.
const RACE_FIELD_RUNS: [FieldRun; 5] = [
    (2, 14, 74, false),
    (16, 7, 60, true),
    (36, 8, 0, true),
    (59, 14, 16, true),
    (28, 8, 44, true),
];
/// Race dialog single fields.
const RACE_FIELDS: [Field; 6] = [
    (44, 196, false),
    (45, 198, true),
    (46, 200, true),
    (47, 202, true),
    (50, 204, true),
    (51, 206, false),
];
/// M.O.S. dialog runs of fields.
const PROFILE_FIELD_RUNS: [FieldRun; 5] = [
    (2, 14, 110, false),
    (16, 7, 72, true),
    (28, 8, 56, true),
    (59, 14, 0, true),
    (75, 14, 28, false),
];
/// M.O.S. dialog single fields.
const PROFILE_FIELDS: [Field; 20] = [
    (44, 258, true),
    (45, 260, true),
    (46, 266, true),
    (49, 452, false),
    (50, 268, false),
    (51, 224, false),
    (37, 264, false),
    (38, 262, false),
    (40, 222, false),
    (90, 224, false),
    (41, 234, false),
    (91, 236, false),
    (42, 238, false),
    (92, 240, false),
    (43, 242, false),
    (93, 244, false),
    (104, 250, false),
    (106, 252, false),
    (105, 246, false),
    (107, 248, false),
];
