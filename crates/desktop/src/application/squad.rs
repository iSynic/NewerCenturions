//! The Modify Squad screen: the roster, the squad's rows, the mission's levels
//! and the difficulty choices.
use super::{Application, ApplicationAction, HandleOutcome, Screen};
use crate::{
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_content::Content;
use centurions_core::{Command, Event, Phase, Session, apply};
use macroquad::prelude::*;

/// Squad dialog (DITL 145) control that moves the roster list five rows up: item 17.
const ROSTER_UP: usize = 17;
/// Squad dialog (DITL 145) control that moves the roster list five rows down: item 18.
const ROSTER_DOWN: usize = 18;

impl Application {
    /// Carries out the Modify Squad choices: add, remove and erase warriors, the
    /// restrictions dialog, marching order, roster scrolling, difficulty and ending
    /// the mission.
    pub(super) fn handle_squad(
        &mut self,
        action: ApplicationAction,
        state: &mut Session,
        content: &Content,
    ) -> Result<HandleOutcome, centurions_core::RuleError> {
        let mut next = None;
        let mut events = Vec::new();
        match action {
            ApplicationAction::EndAnswer(answer) => {
                if !self.selection.end_question {
                    return Err("Mission ending question is not open.".into());
                }
                if state.phase == Phase::Combat && !answer {
                    self.selection.end_question = false;
                } else if answer {
                    self.finish_mission(state, content)?;
                } else {
                    self.selection.end_question = false;
                    self.end_after_save = true;
                    next = Some(Action::Save);
                }
            }
            ApplicationAction::EndSaved => self.finish_mission(state, content)?,
            ApplicationAction::SquadCancel => {
                self.screen = if self.selection.modify_squad {
                    Screen::Game
                } else {
                    Screen::Title
                };
                self.selection.modify_squad = false;
            }
            ApplicationAction::OrderCancel => self.selection.order = None,
            ApplicationAction::OrderMember(member) => {
                let order = self
                    .selection
                    .order
                    .as_mut()
                    .ok_or("Squad ordering is not open.")?;
                if member >= state.warriors.len() {
                    return Err("Ordering member outside squad.".into());
                }
                if let Some(i) = order.iter().position(|m| *m == member) {
                    order.remove(i);
                    events.push(Event::Sound { resource_id: 144 });
                } else {
                    order.push(member);
                    events.push(Event::Sound { resource_id: 141 });
                }
                if order.len() == state.warriors.len() {
                    let mut destinations = vec![0; order.len()];
                    for (i, member) in order.iter().enumerate() {
                        destinations[*member] = order.len() - i - 1;
                    }
                    events.extend(apply(
                        Command::ReorderSquad { destinations },
                        state,
                        content,
                    )?);
                    self.selection.order = None;
                }
            }
            ApplicationAction::Difficulty(level) => {
                if self.screen != Screen::Squad
                    || self.selection.modify_squad
                    || !(-2..=2).contains(&level)
                {
                    return Err("Difficulty selection is unavailable.".into());
                }
                self.selection.difficulty = level;
                events.push(Event::Sound { resource_id: 144 });
            }
            ApplicationAction::Warrior(i) => {
                self.selection.selected = Some(i);
                self.selection.member = None;
            }
            ApplicationAction::Member(i) => self.selection.member = Some(i),
            ApplicationAction::Add => {
                if let Some(i) = self.selection.selected
                    && self.squad.len() < 6
                    && !self.squad.contains(&i)
                {
                    if let Some(restrictions) = self
                        .squad_level(content, state)
                        .and_then(|level| level.restrictions.as_deref())
                    {
                        let warrior = self
                            .warriors(content)
                            .get(i)
                            .ok_or("Squad roster index missing.")?;
                        centurions_core::roster::admit_to_squad(
                            restrictions,
                            warrior,
                            self.squad.len(),
                        )?;
                    }
                    self.squad.push(i);
                    self.selection.member = Some(self.squad.len() - 1);
                    // The roster selection clears once minus blanks its row.
                    self.selection.selected = None;
                    events.push(Event::Sound { resource_id: 141 });
                }
            }
            ApplicationAction::Remove => {
                // Item 15 drops the marked member, member 0 until another is picked.
                let i = self.selection.member.take().unwrap_or(0);
                // The mark clears first; modifying a one-member squad then warns
                // with message 0x43 and keeps the member.
                if self.selection.modify_squad && self.squad.len() == 1 {
                    return Err(centurions_core::RuleError::Refused {
                        feedback: vec![centurions_core::feedback::GameFeedback::Warning {
                            index: 0x43,
                        }],
                    });
                }
                if i < self.squad.len() {
                    self.squad.remove(i);
                }
            }
            // Item 16 asks STR# 135 question 3 for the selected roster warrior.
            ApplicationAction::EraseWarrior => {
                let kept = self.warriors(content);
                self.selection
                    .selected
                    .filter(|i| !self.squad.contains(i))
                    .and_then(|i| kept.get(i))
                    .filter(|warrior| !warrior.name.is_empty())
                    .ok_or("Erase Warrior needs a roster warrior.")?;
                if self.roster.is_none() {
                    return Err("warrior roster missing".into());
                }
                self.selection.erase_question = true;
                events.push(Event::Sound { resource_id: 20005 });
            }
            ApplicationAction::EraseAnswer(answer) => {
                if !self.selection.erase_question {
                    return Err("Erase Warrior question is not open.".into());
                }
                let selected = self.selection.selected.ok_or("No warrior is selected.")?;
                if answer {
                    self.erase_warrior(selected)?;
                }
                // The question ends the selection whichever way it is answered.
                self.selection.erase_question = false;
                self.selection.selected = None;
            }
            // Item 20 opens StandardGetFile for a warrior file to copy into :Warriors:.
            ApplicationAction::ImportWarrior => {
                return Err(centurions_core::RuleError::Unsupported {
                    action: "import warrior",
                    dependency: "warrior file import",
                });
            }
            // Item 17 subtracts five from the first listed slot and item 18 adds
            // five, then a negative result becomes zero.
            ApplicationAction::RosterScroll(down) => {
                self.selection.roster_top = if down {
                    self.selection.roster_top + 5
                } else {
                    self.selection.roster_top.saturating_sub(5)
                };
            }
            ApplicationAction::ViewRestrictions => {
                if self
                    .squad_level(content, state)
                    .is_none_or(|level| level.restrictions.is_none())
                {
                    return Err("The level has no party restrictions.".into());
                }
                self.selection.restrictions = true;
            }
            ApplicationAction::CloseRestrictions => {
                if !self.selection.restrictions {
                    return Err("Party restrictions are not open.".into());
                }
                self.selection.restrictions = false;
            }
            ApplicationAction::Done => {
                let warriors = self
                    .squad
                    .iter()
                    .map(|i| {
                        self.warriors(content)
                            .get(*i)
                            .map(|record| {
                                if self.selection.modify_squad {
                                    state
                                        .warriors
                                        .iter()
                                        .find(|w| w.original_sha256 == record.original_sha256)
                                        .unwrap_or(record)
                                        .clone()
                                } else {
                                    record.clone()
                                }
                            })
                            .ok_or("Squad roster index missing.")
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if self.selection.modify_squad {
                    if warriors.is_empty() {
                        return Err(centurions_core::RuleError::Refused {
                            feedback: vec![centurions_core::feedback::GameFeedback::Sound {
                                resource_id: 6000,
                            }],
                        });
                    }
                    events = apply(Command::ReplaceSquad { warriors }, state, content)?;
                    self.selection.modify_squad = false;
                    self.screen = Screen::Game;
                } else if self.squad.is_empty() {
                    self.screen = Screen::Title;
                } else {
                    let mut candidate = Session::new(content);
                    candidate.random = state.random.clone();
                    candidate.library_random_seed = state.library_random_seed;
                    candidate.library_random_draws = state.library_random_draws;
                    apply(
                        Command::SetDifficulty {
                            level: self.selection.difficulty,
                        },
                        &mut candidate,
                        content,
                    )
                    .map_err(|error| error.to_string())?;
                    events = apply(
                        Command::SelectWarriors { warriors },
                        &mut candidate,
                        content,
                    )
                    .and_then(|mut events| {
                        events.extend(apply(
                            Command::EnterMission {
                                name: self
                                    .selected_scenario
                                    .clone()
                                    .or_else(|| content.scenario.as_ref().map(|s| s.name.clone()))
                                    .ok_or("no starting scenario")?,
                            },
                            &mut candidate,
                            content,
                        )?);
                        Ok(events)
                    })
                    .map_err(|e| e.to_string())?;
                    *state = candidate;
                    self.screen = Screen::Game;
                }
            }
            _ => unreachable!("application action routed to wrong screen"),
        }
        Ok(HandleOutcome::Complete(next, events))
    }

    /// Opens the roster file at `path` for the Modify Squad screen.
    pub fn load_roster(
        &mut self,
        path: &std::path::Path,
        content: &Content,
    ) -> Result<(), centurions_core::RuleError> {
        self.scenario_names = content.scenario_names();
        if let Some(map) = content.map(0) {
            self.notes.labels = map.note_labels.clone();
            self.notes.unknown_labels = map.unknown_note_labels.clone();
        }
        self.archives =
            centurions_core::reference_views::archive_entries(&Session::new(content), content)?;
        self.roster = Some(centurions_core::roster::Roster::open(path, content)?);
        Ok(())
    }
    /// The warriors the squad may be chosen from: the roster's if one is open,
    /// otherwise the scenario's supplied warriors.
    pub(super) fn warriors<'a>(
        &'a self,
        content: &'a Content,
    ) -> &'a [centurions_content::Warrior] {
        self.roster
            .as_ref()
            .map_or(&content.warriors, |r| &r.warriors)
    }
    /// Erases roster warrior `index` from the reconstruction's own roster, which keeps the
    /// squad on the same warriors and the roster list on the same rows. Nothing changes unless the
    /// roster accepts the erasure.
    ///
    /// Original: minus
    fn erase_warrior(&mut self, index: usize) -> Result<(), centurions_core::RuleError> {
        let roster = self.roster.as_mut().ok_or("warrior roster missing")?;
        let identity = roster
            .warriors
            .get(index)
            .ok_or("Squad roster index missing.")?
            .original_sha256
            .clone();
        roster.remove(&identity)?;
        for member in &mut self.squad {
            if *member > index {
                *member -= 1;
            }
        }
        if index < self.selection.roster_top {
            self.selection.roster_top -= 1;
        }
        Ok(())
    }

    /// Ends the mission, stores the squad's records in the roster and returns to the
    /// title.
    pub(super) fn finish_mission(
        &mut self,
        state: &mut Session,
        content: &Content,
    ) -> Result<(), centurions_core::RuleError> {
        let mut next = state.clone();
        apply(Command::EndMission, &mut next, content)?;
        self.roster
            .as_mut()
            .ok_or("warrior roster missing")?
            .store(&centurions_core::roster::mission_records(state, content)?)?;
        *state = next;
        self.selection.end_question = false;
        self.end_after_save = false;
        self.screen = Screen::Title;
        Ok(())
    }

    /// The party records partyselect shows: the live squad while modifying it,
    /// otherwise the roster records.
    fn squad_record<'a>(
        &'a self,
        content: &'a Content,
        state: &'a Session,
        index: usize,
    ) -> Option<&'a centurions_content::Warrior> {
        let record = self.warriors(content).get(index)?;
        Some(if self.selection.modify_squad {
            state
                .warriors
                .iter()
                .find(|w| w.original_sha256 == record.original_sha256)
                .unwrap_or(record)
        } else {
            record
        })
    }

    /// The level partyselect reads its recommended levels and Data RI for.
    fn squad_level<'a>(
        &self,
        content: &'a Content,
        state: &Session,
    ) -> Option<&'a centurions_content::mission::ScenarioContent> {
        if self.selection.modify_squad {
            state
                .mission
                .clone()
                .map(|name| (name, state.scenario_level))
        } else {
            self.selected_scenario
                .clone()
                .or_else(|| content.scenario.as_ref().map(|s| s.name.clone()))
                .map(|name| (name, 0))
        }
        .and_then(|(name, level)| content.scenario.as_ref()?.selection(&name, level))
    }

    /// Draws the Modify Squad screen (DITL 145): the roster, the squad's rows, the
    /// mission's levels and the difficulty choices.
    ///
    /// Original: partyselect
    pub(super) fn draw_squad(
        &self,
        content: &Content,
        state: &Session,
        assets: &Assets,
        controls: &mut Vec<Control>,
    ) {
        clear_background(BLACK);
        let origin = vec2(80., 80.);
        assets.panel(203, Rect::new(origin.x, origin.y, 640., 460.));
        assets.text(
            &assets.squad_text[69],
            origin.x + 7.,
            origin.y + 19.,
            12,
            WHITE,
        );
        self.roster_list(content, assets, origin, controls);
        let levels = self.member_rows(content, state, assets, origin, controls);
        self.button_column(assets, origin, controls);
        self.level_block(content, state, assets, origin, levels, controls);
        self.difficulty_block(assets, origin, controls);
    }

    /// The roster of warriors not in the squad, thirteen rows from the top.
    ///
    /// Original: partyselect, plus
    fn roster_list(
        &self,
        content: &Content,
        assets: &Assets,
        origin: Vec2,
        controls: &mut Vec<Control>,
    ) {
        // Each added warrior is blanked from the index by minus, so its row
        // stays empty until plus returns it.
        for (i, warrior) in self
            .warriors(content)
            .iter()
            .enumerate()
            .skip(self.selection.roster_top)
            .take(13)
        {
            let row = i - self.selection.roster_top;
            let rect = Rect::new(origin.x + 12., origin.y + 30. + row as f32 * 20., 155., 14.);
            let listed = !self.squad.contains(&i);
            if listed {
                // yellowColor for the selection, cyanColor otherwise.
                let color = if self.selection.selected == Some(i) {
                    YELLOW
                } else {
                    crate::layout::CYAN
                };
                assets.text(&warrior.name, rect.x, rect.y + 12., 12, color);
                assets.text(
                    &warrior.record().level().unwrap_or(0).to_string(),
                    origin.x + 184.,
                    rect.y + 12.,
                    12,
                    color,
                );
            }
            controls.push(Control {
                rect,
                enabled: listed,
                action: Action::Application(ApplicationAction::Warrior(i)),
            });
        }
    }

    /// One row per squad member; returns the sum of their levels.
    ///
    /// Original: partyselect
    fn member_rows(
        &self,
        content: &Content,
        state: &Session,
        assets: &Assets,
        origin: Vec2,
        controls: &mut Vec<Control>,
    ) -> i16 {
        let item = |number: usize| assets.dialog_rect_at(145, number, origin);
        let member = self.selection.member.unwrap_or(0);
        let mut levels = 0_i16;
        for (i, warrior) in self.squad.iter().enumerate() {
            let Some(warrior) = self.squad_record(content, state, *warrior) else {
                continue;
            };
            let record = warrior.record();
            let top = origin.y + 50. * i as f32;
            levels = levels.wrapping_add(record.level().unwrap_or(0));
            // DialogNum item 54 + member.
            assets.dialog_text(&record.level().unwrap_or(0).to_string(), item(54 + i));
            // An add draws PICT 1 over the bar and portrait box, then updatechar
            // mode 5 puts PICT 189 behind the row.
            assets.panel(1, Rect::new(origin.x + 321., top, 60., 50.));
            assets.panel(189, Rect::new(origin.x + 380., top, 260., 50.));
            // Item 34 + selected member takes PICT 403.
            if i == member {
                assets.panel(403, item(34 + i));
            }
            // updatepictbox mode 1: the 32-pixel box at the row's left + 10, inset 9.
            // In the partyselect dialog plotportrait places condition cicns left
            // of 573.
            crate::portrait::draw(
                assets,
                warrior,
                vec2(origin.x + 339., top + 9.),
                crate::portrait::Marks {
                    class_label: self
                        .preferences
                        .enabled(crate::preferences::pref::APPEND_MOS),
                    conditions_left: Some(origin.x + 573.),
                    automatic: state
                        .combat
                        .as_ref()
                        .is_some_and(|combat| combat.do_auto.get(i).copied().unwrap_or(false)),
                    in_combat: state.combat.is_some(),
                },
            );
            assets.text(&warrior.name, origin.x + 420., top + 22., 12, YELLOW);
            assets.text(
                &record.actions_per_round().unwrap_or(0).to_string(),
                origin.x + 610.,
                top + 22.,
                12,
                YELLOW,
            );
            // A maximum Psi of zero draws " ---".
            let maximum_psi = record.psi_points_max().unwrap_or(0);
            if maximum_psi != 0 {
                pool(
                    assets,
                    record.psi_points().unwrap_or(0),
                    maximum_psi,
                    vec2(origin.x + 466., top + 44.),
                );
            } else {
                assets.text(
                    &assets
                        .game_text
                        .text(crate::game_strings::NO_PSI, true)
                        .unwrap_or_default(),
                    origin.x + 466.,
                    top + 44.,
                    12,
                    YELLOW,
                );
            }
            pool(
                assets,
                record.stamina().unwrap_or(0),
                record.stamina_max().unwrap_or(0),
                vec2(origin.x + 427., top + 44.),
            );
            for (value, x) in [
                (record.shield_chance(), 524.),
                (record.shield_points(), 563.),
                (record.armor_rating(), 609.),
            ] {
                assets.text(
                    &value.unwrap_or(0).to_string(),
                    origin.x + x,
                    top + 44.,
                    12,
                    YELLOW,
                );
            }
            controls.push(Control {
                rect: item(21 + i),
                enabled: true,
                action: Action::Application(ApplicationAction::Member(i)),
            });
        }
        levels
    }

    /// Add, Remove, Erase, Done and Import, the roster arrows and the Edit picture.
    ///
    /// Original: partyselect
    fn button_column(&self, assets: &Assets, origin: Vec2, controls: &mut Vec<Control>) {
        let item = |number: usize| assets.dialog_rect_at(145, number, origin);
        let add = self
            .selection
            .selected
            .is_some_and(|i| !self.squad.contains(&i))
            && self.squad.len() < 6;
        // Edit (item 16) acts on a selected roster warrior; Import (item 20)
        // has no precondition.
        for (id, y, enabled, action) in [
            (160, 22., add, ApplicationAction::Add),
            (7, 72., !self.squad.is_empty(), ApplicationAction::Remove),
            (
                158,
                122.,
                self.selection
                    .selected
                    .is_some_and(|i| !self.squad.contains(&i)),
                ApplicationAction::EraseWarrior,
            ),
            (168, 222., true, ApplicationAction::Done),
            (183, 272., true, ApplicationAction::ImportWarrior),
        ] {
            let rect = Rect::new(origin.x + 221., origin.y + y, 50., 50.);
            assets.panel(id, rect);
            if !enabled {
                draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0., 0., 0., 0.55));
            }
            controls.push(Control {
                rect,
                enabled,
                action: Action::Application(action),
            });
        }
        for (number, down) in [(ROSTER_UP, false), (ROSTER_DOWN, true)] {
            controls.push(Control {
                rect: item(number),
                enabled: true,
                action: Action::Application(ApplicationAction::RosterScroll(down)),
            });
        }
        // A full squad marks Add with cicn 2019.
        if self.squad.len() >= 6
            && let Some(icon) = assets.icons.get(&2019)
        {
            crate::assets::texture_in(icon, Rect::new(origin.x + 221., origin.y + 22., 50., 50.));
        }
        assets.panel(132, Rect::new(origin.x + 221., origin.y + 172., 50., 50.));
    }

    /// Draws the Scenario Restrictions dialog (DLOG -7931) over the squad screen: the
    /// restriction text, the party size and level limits, and a check box beside each race and
    /// M.O.S. name, checked when the level bans it. A click on an enabled item or Return closes it.
    ///
    /// Original: viewrestrictions
    pub(super) fn draw_restrictions(
        &self,
        content: &Content,
        state: &Session,
        assets: &Assets,
        controls: &mut Vec<Control>,
    ) {
        let Some(limits) = self
            .squad_level(content, state)
            .and_then(|level| level.restrictions.as_deref())
            .map(centurions_content::records::PartyRestrictions)
        else {
            return;
        };
        let id = crate::assets::RESTRICTIONS_DIALOG;
        assets.draw_dialog(id);
        let item = |number: usize| assets.dialog_rect(id, number);
        assets.dialog_text(
            &limits.description().unwrap_or_default().replace('\r', "\n"),
            item(4),
        );
        for (number, value) in [
            (65, limits.maximum_warriors()),
            (66, limits.maximum_level()),
        ] {
            assets.dialog_text(&value.unwrap_or(0).to_string(), item(number));
        }
        let strings = |table: i16| {
            assets
                .character_strings
                .get(&table)
                .and_then(|bytes| centurions_content::ui::string_list(bytes).ok())
                .unwrap_or_default()
        };
        let (races, classes) = (strings(129), strings(131));
        for t in 0..30 {
            for (box_item, label_item, flag, names) in [
                (5 + t, 67 + t, limits.race_flag(t), &races),
                (35 + t, 97 + t, limits.mos_flag(t), &classes),
            ] {
                assets.panel(
                    if flag.unwrap_or(0) != 0 { 179 } else { 180 },
                    item(box_item),
                );
                assets.dialog_text(names.get(t).map_or("", String::as_str), item(label_item));
            }
        }
        // A click on an enabled item answers the dialog; the disabled backdrop picture ignores it
        // and the off-dialog default button is answered by Return.
        for (index, dialog_item) in assets.dialogs[&id].iter().enumerate() {
            if index > 0 && index < 126 && dialog_item.kind & 0x80 == 0 {
                controls.push(Control {
                    rect: item(index + 1),
                    enabled: true,
                    action: Action::Application(ApplicationAction::CloseRestrictions),
                });
            }
        }
    }

    /// The recommended and total levels and the View Restrictions button.
    ///
    /// Original: partyselect
    fn level_block(
        &self,
        content: &Content,
        state: &Session,
        assets: &Assets,
        origin: Vec2,
        levels: i16,
        controls: &mut Vec<Control>,
    ) {
        let item = |number: usize| assets.dialog_rect_at(145, number, origin);
        let scenario = self.squad_level(content, state);
        // Items 60/61 with DialogNumLong 65 and DialogNum 66.
        for (number, text) in [
            (60, assets.squad_text[59].clone()),
            (61, assets.squad_text[60].clone()),
            (
                65,
                scenario
                    .and_then(|s| s.recommended_levels)
                    .map(|n| n.to_string())
                    .unwrap_or_default(),
            ),
            (66, levels.to_string()),
        ] {
            assets.dialog_text(&text, item(number));
        }
        // View Restrictions (item 33, PICT 216) appears when the level has a Data RI.
        if scenario.is_some_and(|s| s.restrictions.is_some()) {
            let rect = item(33);
            crate::presentation::push_panel_control(
                assets,
                controls,
                216,
                rect,
                true,
                Action::Application(ApplicationAction::ViewRestrictions),
            );
        }
    }

    /// The difficulty choices, their check box and the squad notes.
    fn difficulty_block(&self, assets: &Assets, origin: Vec2, controls: &mut Vec<Control>) {
        assets.text(
            assets.squad_text[66].trim_end(),
            origin.x + 233.,
            origin.y + 360.,
            12,
            YELLOW,
        );
        for (i, name) in assets.squad_text[67].lines().enumerate() {
            assets.text(
                name,
                origin.x + 268.,
                origin.y + 375. + i as f32 * 16.,
                12,
                WHITE,
            );
            controls.push(Control {
                rect: Rect::new(origin.x + 241., origin.y + 364. + i as f32 * 16., 130., 16.),
                enabled: !self.selection.modify_squad,
                action: Action::Application(ApplicationAction::Difficulty(i as i16 - 2)),
            });
        }
        assets.panel(
            179,
            Rect::new(
                origin.x + 241.,
                origin.y + 364. + f32::from(self.selection.difficulty + 2) * 16.,
                13.,
                13.,
            ),
        );
        assets.wrapped_color(
            &assets.squad_text[68],
            Rect::new(origin.x + 12., origin.y + 311., 202., 119.),
            12,
            16.,
            YELLOW,
        );
    }
}

/// A pool below its maximum is drawn in RGB 0x3800/0xb300/0xf900, and values
/// above 999 at size 10.
///
/// Original: updatechar
fn pool(assets: &Assets, value: i16, maximum: i16, at: Vec2) {
    let color = if value < maximum {
        crate::layout::POOL_BELOW_MAXIMUM
    } else {
        YELLOW
    };
    let size = if value > 999 { 10 } else { 12 };
    assets.text(&value.to_string(), at.x, at.y, size, color);
}
