mod items;
mod scans;
pub use items::{ITEM_EDITOR_CLOSE, ItemEditor};
pub use scans::ScanEditor;
mod controls;
use crate::{
    application::ApplicationAction,
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_content::{Content, Warrior};
use centurions_core::{Command, Event, Session, apply};
use controls::UpgradeControl;
use macroquad::prelude::*;

/// The Upgrade Warrior dialog: a draft of the session in which a warrior's
/// attributes and level are changed, with the choices to undo them.
pub struct WarriorEditor {
    pub free: bool,
    pub draft: Session,
    member: usize,
    selector: u16,
    initial: Warrior,
    previous: Warrior,
    pub(crate) level_information: bool,
    pub(crate) scans: Option<ScanEditor>,
}
impl WarriorEditor {
    /// Opens the editor on squad member `member`, remembering the warrior as it is.
    pub fn new(state: &Session, member: usize) -> Result<Self, String> {
        let initial = state
            .warriors
            .get(member)
            .ok_or("Editor warrior is outside the squad.")?
            .clone();
        Ok(Self {
            free: true,
            draft: state.clone(),
            member,
            selector: 0,
            previous: initial.clone(),
            initial,
            level_information: false,
            scans: None,
        })
    }
    /// upgradewarrior's entry for the current member.
    ///
    /// Original: upgradewarrior
    pub fn enter(&mut self, content: &Content) -> Result<(), centurions_core::RuleError> {
        apply(
            Command::EnterWarriorUpgrade {
                warrior: self.member,
            },
            &mut self.draft,
            content,
        )?;
        self.initial = self.draft.warriors[self.member].clone();
        self.previous = self.initial.clone();
        Ok(())
    }
    /// Carries out a click on an Upgrade Warrior item: selecting an attribute,
    /// applying an upgrade, undoing, editing the level, opening the scan selection, or moving to
    /// another warrior.
    pub fn handle(
        &mut self,
        item: usize,
        content: &Content,
    ) -> Result<Vec<Event>, centurions_core::RuleError> {
        let control = UpgradeControl::from_item(item);
        if control == UpgradeControl::EditLevel && !self.free {
            return Err("Level editing requires the active warrior editor.".into());
        }
        if self.level_information {
            self.level_information = false;
            return Ok(vec![]);
        }
        if self.scans.is_some() {
            return self.scan_item(item, content).map_err(Into::into);
        }
        match control {
            UpgradeControl::ScanSelection => {
                if self.draft.warriors[self.member]
                    .record()
                    .max_spells_attacks()
                    .map_err(|e| e.to_string())?
                    == 0
                {
                    // upgradewarrior returns to the editor after this flash.
                    return Ok(vec![Event::Feedback(
                        centurions_core::feedback::GameFeedback::Flash {
                            address: 0x4005d4,
                            x: 50,
                            y: 50,
                            wait: 0,
                            sound: 6000,
                        },
                    )]);
                }
                let map = content.map(0).ok_or("Scan profiles unavailable.")?;
                self.scans = Some(ScanEditor {
                    selection: centurions_core::story_promotion::ScanSelection::new(
                        &self.draft.warriors[self.member],
                        &map.profiles,
                    )
                    .map_err(|e| e.to_string())?,
                    controls: vec![],
                    question: false,
                    description: None,
                    warning: false,
                });
                return Ok(vec![Event::Sound { resource_id: 3000 }]);
            }
            UpgradeControl::EditLevel => {
                let events = apply(
                    Command::EditWarriorLevel {
                        warrior: self.member,
                    },
                    &mut self.draft,
                    content,
                )?;
                self.level_information = events.iter().any(|e| {
                    matches!(
                        e,
                        Event::EditorLevelAdvanced {
                            information: true,
                            ..
                        }
                    )
                });
                self.initial = self.draft.warriors[self.member].clone();
                self.previous = self.initial.clone();
                return Ok(events);
            }
            UpgradeControl::SelectAttribute(selector) => self.selector = selector,
            UpgradeControl::PreviousWarrior | UpgradeControl::NextWarrior => {
                let count = self.draft.warriors.len();
                self.member = (self.member
                    + if control == UpgradeControl::PreviousWarrior {
                        count - 1
                    } else {
                        1
                    })
                    % count;
                self.selector = 0;
                self.enter(content)?;
            }
            UpgradeControl::UndoLast | UpgradeControl::UndoAll => {
                self.draft.warriors[self.member] = if control == UpgradeControl::UndoLast {
                    self.previous.clone()
                } else {
                    self.initial.clone()
                };
                centurions_core::character::calculate_statistics(
                    &mut self.draft.warriors[self.member],
                )
                .map_err(|e| e.to_string())?;
                return Ok(vec![Event::Sound {
                    resource_id: if control == UpgradeControl::UndoLast {
                        138
                    } else {
                        133
                    },
                }]);
            }
            UpgradeControl::ApplyUpgrade => {
                let previous = self.draft.warriors[self.member].clone();
                let events = apply(
                    if self.free {
                        Command::EditWarriorUpgrade {
                            warrior: self.member,
                            selector: self.selector,
                        }
                    } else {
                        Command::UpgradeWarrior {
                            warrior: self.member,
                            selector: self.selector,
                        }
                    },
                    &mut self.draft,
                    content,
                )?;
                self.previous = previous;
                return Ok(events);
            }
            _ => return Err("This editor action has not been reconstructed.".into()),
        }
        Ok(vec![])
    }
    /// Carries out a click on the scan selection: Done (asking first when scans
    /// remain to be chosen), choosing a tier, or toggling a scan. A warning is
    /// dismissed by the next click.
    fn scan_item(&mut self, item: usize, content: &Content) -> Result<Vec<Event>, String> {
        let scan = self.scans.as_mut().unwrap();
        if scan.warning {
            scan.warning = false;
            return Ok(vec![]);
        }
        if scan.question {
            return Ok(vec![]);
        }
        match item {
            1 | 32 => {
                if scan.selection.remaining != 0 {
                    scan.question = true;
                } else {
                    self.scan_answer(true, content)?;
                }
            }
            2..=8 => {
                if scan.selection.select_tier(item - 2).is_err() {
                    scan.warning = true;
                    return Ok(vec![]);
                }
                scan.controls.push(item as u16);
                return Ok(vec![Event::Sound { resource_id: 137 }]);
            }
            10..=16 => {
                scan.description = Some((scan.selection.tier, item - 10));
                let sound = scan
                    .selection
                    .toggle(item - 10)
                    .map_err(|e| e.to_string())?;
                scan.controls.push(item as u16);
                return Ok(sound
                    .into_iter()
                    .map(|resource_id| Event::Sound { resource_id })
                    .collect());
            }
            _ => return Err("Unknown scan-selection control.".into()),
        }
        Ok(vec![])
    }
    /// Answers the question raised by finishing with scans left to choose: yes
    /// applies the chosen scans to the warrior and closes the selection.
    pub fn scan_answer(&mut self, answer: bool, content: &Content) -> Result<(), String> {
        let scan = self.scans.as_mut().ok_or("Scan editor is not open.")?;
        scan.question = false;
        if !answer {
            return Ok(());
        }
        let mut controls = scan.controls.clone();
        controls.push(1);
        apply(
            Command::EditWarriorScans {
                warrior: self.member,
                controls,
            },
            &mut self.draft,
            content,
        )
        .map_err(|e| e.to_string())?;
        self.initial = self.draft.warriors[self.member].clone();
        self.previous = self.initial.clone();
        self.scans = None;
        Ok(())
    }
    /// Puts the warrior back as it was when the editor last entered it or accepted a level or
    /// scan change; upgrades since then are undone.
    pub fn cancel_current(&mut self) -> Result<(), String> {
        self.draft.warriors[self.member] = self.initial.clone();
        centurions_core::character::calculate_statistics(&mut self.draft.warriors[self.member])
            .map_err(|e| e.to_string())
    }
    /// Item `item` of the Upgrade Warrior dialog as a rectangle in the window.
    fn rect(assets: &Assets, item: usize) -> Rect {
        let [top, left, bottom, right] = assets.upgrade_dialog[item - 1].rect;
        Rect::new(
            80. + f32::from(left),
            80. + f32::from(top),
            f32::from(right - left),
            f32::from(bottom - top),
        )
    }
    /// Draws the Upgrade Warrior dialog: the warrior's attributes, statistics and
    /// the controls.
    pub fn draw(&self, content: &Content, assets: &Assets, controls: &mut Vec<Control>) {
        clear_background(BLACK);
        assets.panel(165, crate::layout::DIALOG_FRAME);
        for item in [39, 88] {
            let picture = i16::from_be_bytes(
                assets.upgrade_dialog[item - 1].data[..2]
                    .try_into()
                    .unwrap(),
            );
            assets.panel(picture, Self::rect(assets, item));
        }
        assets.panel(233, Self::rect(assets, 100));
        let warrior = &self.draft.warriors[self.member];
        for index in 0..9 {
            if warrior.record().spec(index).unwrap_or(0) < 1 {
                assets.panel(181, Self::rect(assets, 90 + index));
            }
        }
        assets.panel(
            if warrior.record().psi_school().unwrap_or(0) == 0 {
                181
            } else {
                151
            },
            Self::rect(assets, 99),
        );
        if let Ok(id) = warrior.record().portrait()
            && let Some(portrait) = assets.portraits.get(&id)
        {
            let rect = Self::rect(assets, 3);
            crate::assets::texture_in(portrait, rect);
        }
        let number = |item, value: i16| {
            let rect = Self::rect(assets, item);
            assets.text(
                &value.to_string(),
                rect.x + 2.,
                rect.bottom() - 2.,
                12,
                YELLOW,
            );
        };
        for selector in 0..33 {
            let item = match selector {
                0..=6 => 5 + selector,
                7 => 12,
                8..=15 => 13 + selector - 8,
                16..=19 => 21 + selector - 16,
                20..=22 => 25 + selector - 20,
                23..=31 => 28 + selector - 23,
                _ => 37,
            };
            let attribute = warrior.record().upgrade(selector as u16);
            let value = attribute.display_value();
            if attribute.display_available() {
                number(item, value);
            }
            let rect = Self::rect(assets, 48 + selector);
            if selector == usize::from(self.selector) {
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1., YELLOW);
            }
        }
        let rect = Self::rect(assets, 42);
        assets.text(
            &assets.upgrade_names[usize::from(self.selector)],
            rect.x,
            rect.bottom() - 2.,
            12,
            YELLOW,
        );
        let mut rect = Self::rect(assets, 47);
        rect.y += 12.;
        rect.h -= 12.;
        assets.wrapped_color(
            &assets.upgrade_formulas[usize::from(self.selector)],
            rect,
            12,
            15.,
            YELLOW,
        );
        if let Ok(mos) = warrior.record().profession()
            && let Some(profile) = content.map(0).and_then(|map| {
                let start = usize::try_from(mos.checked_sub(1)?)
                    .ok()?
                    .checked_mul(576)?;
                map.profiles.get(start..start + 576)
            })
            && let Ok((cost, percent)) =
                centurions_core::character::upgrade_cost(warrior, self.selector, profile)
        {
            for (item, value) in [(44, cost), (46, percent)] {
                let rect = Self::rect(assets, item);
                assets.text(&value.to_string(), rect.x, rect.bottom() - 2., 12, YELLOW);
            }
        }
        let rect = Self::rect(assets, 2);
        assets.text(&warrior.name, rect.x, rect.bottom() - 2., 12, YELLOW);
        number(101, warrior.record().level().unwrap_or(0));
        let rect = Self::rect(assets, 43);
        let points = warrior.record().upgrade_points();
        assets.text(&points.to_string(), rect.x, rect.bottom() - 2., 12, YELLOW);
        for item in (48..=80).chain([1, 4, 39, 45, 81, 82, 83, 84, 85]) {
            controls.push(Control {
                rect: Self::rect(assets, item),
                enabled: item != 81 || self.free,
                action: Action::Application(ApplicationAction::EditorItem(item)),
            });
        }
        if self.level_information {
            controls.clear();
            let origin = vec2(275., 245.);
            let rect = Rect::new(origin.x, origin.y, 249., 111.);
            assets.panel(173, rect);
            assets.text(&warrior.name, origin.x + 73., origin.y + 35., 12, YELLOW);
            assets.text("1", origin.x + 212., origin.y + 83., 12, YELLOW);
            if let Ok(id) = warrior.record().portrait()
                && let Some(portrait) = assets.portraits.get(&id)
            {
                crate::assets::icon_in(portrait, vec2(origin.x + 22., origin.y + 22.));
            }
            controls.push(Control {
                rect,
                enabled: true,
                action: Action::Application(ApplicationAction::EditorItem(0)),
            });
        }
        if let Some(scan) = &self.scans {
            controls.clear();
            scan.draw(content, assets, controls);
        }
    }
}
