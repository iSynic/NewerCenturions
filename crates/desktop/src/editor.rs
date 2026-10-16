use crate::{
    application::ApplicationAction,
    assets::Assets,
    presentation::{Action, Control},
};
use centurions_content::{Content, Warrior, word};
use centurions_core::{Command, Event, Session, apply};
use macroquad::prelude::*;

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
    pub fn handle(
        &mut self,
        item: usize,
        content: &Content,
    ) -> Result<Vec<Event>, centurions_core::RuleError> {
        if item == 81 && !self.free {
            return Err("Level editing requires the active warrior editor.".into());
        }
        if self.level_information {
            self.level_information = false;
            return Ok(vec![]);
        }
        if self.scans.is_some() {
            return self.scan_item(item, content).map_err(Into::into);
        }
        match item {
            39 => {
                if word(&self.draft.warriors[self.member].bytes, 0xac6)
                    .map_err(|e| e.to_string())?
                    == 0
                {
                    // upgradewarrior1628b6 returns to the editor after this native flash.
                    return Ok(vec![Event::Feedback(
                        centurions_core::feedback::NativeFeedback::Flash {
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
            81 => {
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
            48..=80 => self.selector = (item - 48) as u16,
            84 | 85 => {
                let count = self.draft.warriors.len();
                self.member = (self.member + if item == 84 { count - 1 } else { 1 }) % count;
                self.initial = self.draft.warriors[self.member].clone();
                self.previous = self.initial.clone();
                self.selector = 0;
            }
            82 | 83 => {
                self.draft.warriors[self.member] = if item == 82 {
                    self.previous.clone()
                } else {
                    self.initial.clone()
                };
                centurions_core::character::calculate_statistics(
                    &mut self.draft.warriors[self.member],
                )
                .map_err(|e| e.to_string())?;
                return Ok(vec![Event::Sound {
                    resource_id: if item == 82 { 138 } else { 133 },
                }]);
            }
            45 => {
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
    pub fn cancel_current(&mut self) -> Result<(), String> {
        self.draft.warriors[self.member] = self.initial.clone();
        centurions_core::character::calculate_statistics(&mut self.draft.warriors[self.member])
            .map_err(|e| e.to_string())
    }
    fn rect(assets: &Assets, item: usize) -> Rect {
        let [top, left, bottom, right] = assets.upgrade_dialog[item - 1].rect;
        Rect::new(
            80. + f32::from(left),
            80. + f32::from(top),
            f32::from(right - left),
            f32::from(bottom - top),
        )
    }
    pub fn draw(&self, content: &Content, assets: &Assets, controls: &mut Vec<Control>) {
        clear_background(BLACK);
        assets.panel(165, Rect::new(80., 80., 640., 460.));
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
            if word(&warrior.bytes, 0xf4 + index * 2).unwrap_or(0) < 1 {
                assets.panel(181, Self::rect(assets, 90 + index));
            }
        }
        assets.panel(
            if word(&warrior.bytes, 0x36).unwrap_or(0) == 0 {
                181
            } else {
                151
            },
            Self::rect(assets, 99),
        );
        if let Ok(id) = word(&warrior.bytes, 0x54)
            && let Some(portrait) = assets.portraits.get(&id)
        {
            let rect = Self::rect(assets, 3);
            draw_texture_ex(
                portrait,
                rect.x,
                rect.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(rect.w, rect.h)),
                    ..Default::default()
                },
            );
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
            let (item, offset, derived) = match selector {
                0..=6 => (5 + selector, 0x9e2 + selector * 2, None),
                7 => (12, 0x9d8, None),
                8..=15 => (
                    13 + selector - 8,
                    0x112 + (selector - 8) * 2,
                    Some(0xb48 + (selector - 8) * 2),
                ),
                16..=19 => (
                    21 + selector - 16,
                    4 + (selector - 16) * 2,
                    Some(0xb58 + (selector - 16) * 2),
                ),
                20 => (25, 0xb42, Some(0xb46)),
                21 => (26, 0xb40, Some(0xb44)),
                22 => (27, 0x22, Some(0xb60)),
                23..=31 => (
                    28 + selector - 23,
                    0xf4 + (selector - 23) * 2,
                    Some(0xb62 + (selector - 23) * 2),
                ),
                _ => (37, 12, Some(0xb7a)),
            };
            let value = word(&warrior.bytes, offset).unwrap_or(0);
            let value = value.wrapping_add(
                derived
                    .and_then(|p| word(&warrior.bytes, p).ok())
                    .unwrap_or(0),
            );
            if !(selector == 21 && word(&warrior.bytes, 0x36).unwrap_or(0) == 0
                || (23..=31).contains(&selector) && word(&warrior.bytes, offset).unwrap_or(0) < 1)
            {
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
        if let Ok(mos) = word(&warrior.bytes, 0x34)
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
        number(101, word(&warrior.bytes, 0x3a).unwrap_or(0));
        let rect = Self::rect(assets, 43);
        let points = i32::from_be_bytes(warrior.bytes[0x9d2..0x9d6].try_into().unwrap());
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
            if let Ok(id) = word(&warrior.bytes, 0x54)
                && let Some(portrait) = assets.portraits.get(&id)
            {
                draw_texture_ex(
                    portrait,
                    origin.x + 22.,
                    origin.y + 22.,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(32., 32.)),
                        ..Default::default()
                    },
                );
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

pub struct ScanEditor {
    selection: centurions_core::story_promotion::ScanSelection,
    controls: Vec<u16>,
    pub(crate) question: bool,
    description: Option<(usize, usize)>,
    warning: bool,
}
impl ScanEditor {
    pub fn draw_promotion(
        progress: &centurions_core::story_promotion::Progress,
        content: &Content,
        assets: &Assets,
        controls: &mut Vec<Control>,
    ) {
        let view = Self {
            selection: progress.selection.clone(),
            controls: vec![],
            question: progress.question,
            description: progress.description,
            warning: progress.warning,
        };
        view.draw(content, assets, controls);
        for control in controls {
            control.action = match control.action {
                Action::Application(ApplicationAction::EditorItem(item)) => {
                    Action::Command(Command::PromotionControl { item: item as u16 })
                }
                Action::Application(ApplicationAction::ScanAnswer(yes)) => {
                    Action::Command(Command::PromotionAnswer { yes })
                }
                ref other => other.clone(),
            };
        }
    }
    fn rect(assets: &Assets, item: usize) -> Rect {
        let [top, left, bottom, right] = assets.scan_dialog[item - 1].rect;
        Rect::new(
            222. + f32::from(left),
            80. + f32::from(top),
            f32::from(right - left),
            f32::from(bottom - top),
        )
    }
    fn draw(&self, content: &Content, assets: &Assets, controls: &mut Vec<Control>) {
        let bounds = Rect::new(222., 80., 356., 460.);
        draw_rectangle(bounds.x, bounds.y, bounds.w, bounds.h, BLACK);
        for item in [22, 23, 31, 35, 38, 41, 49, 50] {
            let dialog = &assets.scan_dialog[item - 1];
            let id = i16::from_be_bytes(dialog.data[..2].try_into().unwrap());
            let full = Self::rect(assets, item);
            if let Some(clipped) = full.intersect(bounds) {
                let texture = &assets.panels[&id];
                draw_texture_ex(
                    texture,
                    clipped.x,
                    clipped.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(clipped.w, clipped.h)),
                        source: Some(Rect::new(
                            (clipped.x - full.x) * texture.width() / full.w,
                            (clipped.y - full.y) * texture.height() / full.h,
                            clipped.w * texture.width() / full.w,
                            clipped.h * texture.height() / full.h,
                        )),
                        ..Default::default()
                    },
                );
            }
        }
        assets.panel(130, Self::rect(assets, 9));
        let warrior = &self.selection.warrior;
        let school = word(&warrior.bytes, 0x36).unwrap_or(0);
        let portrait = word(&warrior.bytes, 0x54).unwrap_or(0);
        if let Some(texture) = assets.portraits.get(&portrait) {
            let rect = Self::rect(assets, 34);
            draw_texture_ex(
                texture,
                rect.x,
                rect.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(rect.w, rect.h)),
                    ..Default::default()
                },
            );
        }
        let rect = Self::rect(assets, 33);
        assets.text(&warrior.name, rect.x, rect.bottom() - 2., 12, YELLOW);
        let rect = Self::rect(assets, 40);
        assets.text(
            &self.selection.remaining.to_string(),
            rect.x,
            rect.bottom() - 2.,
            12,
            YELLOW,
        );
        for (tier, cost) in centurions_core::story_promotion::SCAN_COSTS
            .into_iter()
            .enumerate()
        {
            let rect = Self::rect(assets, 24 + tier);
            assets.text(&cost.to_string(), rect.x, rect.bottom() - 2., 12, YELLOW);
            let rect = Self::rect(assets, 2 + tier);
            if tier == self.selection.tier {
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1., YELLOW);
            }
            controls.push(Control {
                rect,
                enabled: true,
                action: Action::Application(ApplicationAction::EditorItem(2 + tier)),
            });
        }
        for column in 0..7 {
            let rect = Self::rect(assets, 10 + column);
            let spell = school * 1000 + (self.selection.tier as i16 + 1) * 100 + column as i16 + 1;
            let label = Self::rect(assets, 42 + column);
            assets.text(
                assets.spell_names.get(&spell).map_or("", String::as_str),
                label.x,
                label.bottom() - 2.,
                12,
                YELLOW,
            );
            if warrior.bytes[0x9f0 + self.selection.tier * 7 + column] != 0 {
                assets.panel(179, Rect::new(rect.x - 17., rect.y + 5., 9., 9.));
            }
            controls.push(Control {
                rect,
                enabled: true,
                action: Action::Application(ApplicationAction::EditorItem(10 + column)),
            });
        }
        if let Some((tier, column)) = self.description
            && let Some(map) = content.map(0)
            && let Ok(description) = map.scan_description(school, tier, column)
        {
            let mut rect = Self::rect(assets, 39);
            rect.y += 12.;
            rect.h -= 12.;
            assets.wrapped_color(&description, rect, 12, 15., YELLOW);
        }
        let rect = Self::rect(assets, 32);
        assets.panel(168, rect);
        controls.push(Control {
            rect,
            enabled: true,
            action: Action::Application(ApplicationAction::EditorItem(32)),
        });
        if self.question || self.warning {
            controls.clear();
            let origin = vec2(221., 203.);
            draw_texture_ex(
                &assets.panels[&214],
                origin.x,
                origin.y,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(140., 167., 358., 195.)),
                    ..Default::default()
                },
            );
            assets.panel(182, Rect::new(origin.x + 22., origin.y + 23., 320., 84.));
            let text = if self.question {
                &assets.questions[16]
            } else {
                &assets.warnings[73]
            };
            assets.wrapped(
                text,
                Rect::new(origin.x + 36., origin.y + 43., 284., 65.),
                14,
                17.,
            );
            for (id, x, answer) in [(2, 66., true), (4, 208., false)] {
                if self.warning && !answer {
                    continue;
                }
                let rect = Rect::new(origin.x + x, origin.y + 127., 70., 35.);
                assets.panel(if self.warning { 168 } else { id }, rect);
                controls.push(Control {
                    rect,
                    enabled: true,
                    action: Action::Application(if self.warning {
                        ApplicationAction::EditorItem(0)
                    } else {
                        ApplicationAction::ScanAnswer(answer)
                    }),
                });
            }
        }
    }
}

pub struct ItemEditor {
    pub draft: Session,
    member: usize,
    slot: usize,
}
impl ItemEditor {
    pub fn new(state: &Session, member: usize, slot: usize) -> Result<Self, String> {
        let count = state
            .warriors
            .get(member)
            .ok_or("Editor warrior is outside the squad.")?
            .inventory()
            .map_err(|e| e.to_string())?
            .len();
        if slot > count || slot >= 100 {
            return Err("Editor item is outside the inventory.".into());
        }
        Ok(Self {
            draft: state.clone(),
            member,
            slot,
        })
    }
    pub fn handle(&mut self, control: usize, content: &Content) -> Result<Vec<Event>, String> {
        match control {
            67 | 68 => {
                let count = self.draft.warriors.len();
                self.member = (self.member + if control == 67 { count - 1 } else { 1 }) % count;
                self.slot = 0;
            }
            69 => self.slot = self.slot.saturating_sub(1),
            70 => {
                let count = self.draft.warriors[self.member]
                    .inventory()
                    .map_err(|e| e.to_string())?
                    .len();
                self.slot = (self.slot + 1).min(count).min(99);
            }
            40..=63 => {
                return apply(
                    Command::EditCarriedItem {
                        warrior: self.member,
                        slot: self.slot,
                        control: control as u16,
                    },
                    &mut self.draft,
                    content,
                )
                .map_err(|e| e.to_string());
            }
            _ => return Err("This item editor action has not been reconstructed.".into()),
        }
        Ok(vec![])
    }
    fn rect(assets: &Assets, item: usize) -> Rect {
        let [top, left, bottom, right] = assets.item_dialog[item - 1].rect;
        Rect::new(
            80. + f32::from(left),
            80. + f32::from(top),
            f32::from(right - left),
            f32::from(bottom - top),
        )
    }
    pub fn draw(&self, content: &Content, assets: &Assets, controls: &mut Vec<Control>) {
        clear_background(BLACK);
        assets.panel(215, Rect::new(80., 80., 640., 460.));
        assets.panel(168, Self::rect(assets, 2));
        for item in 16..=27 {
            let rect = Self::rect(assets, item);
            assets.text(
                &assets.item_dialog[item - 1].text(),
                rect.x,
                rect.bottom() - 2.,
                12,
                YELLOW,
            );
        }
        let warrior = &self.draft.warriors[self.member];
        let item = centurions_content::InventoryRecord(
            warrior.bytes[0x132 + self.slot * 22..0x132 + (self.slot + 1) * 22]
                .try_into()
                .unwrap(),
        );
        for (row, offset) in [13, 15, 16, 11, 17, 18, 19, 14, 21, 20]
            .into_iter()
            .enumerate()
        {
            let rect = Self::rect(assets, 28 + row);
            assets.text(
                &(item.0[offset] as i8).to_string(),
                rect.x,
                rect.bottom() - 2.,
                12,
                YELLOW,
            );
        }
        for (number, offset) in [(38, 2), (39, 6)] {
            let rect = Self::rect(assets, number);
            assets.text(
                &word(&item.0, offset).unwrap_or(0).to_string(),
                rect.x,
                rect.bottom() - 2.,
                12,
                YELLOW,
            );
        }
        for (number, text) in [(66, assets.name(item.id())), (72, warrior.name.clone())] {
            let rect = Self::rect(assets, number);
            assets.text(&text, rect.x, rect.bottom() - 2., 12, YELLOW);
        }
        if let Ok(id) = word(&warrior.bytes, 0x54)
            && let Some(portrait) = assets.portraits.get(&id)
        {
            let rect = Self::rect(assets, 71);
            draw_texture_ex(
                portrait,
                rect.x,
                rect.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(rect.w, rect.h)),
                    ..Default::default()
                },
            );
        }
        if let Some(definition) = content.items.get(usize::from(item.id().unsigned_abs())) {
            if let Ok(id) = word(definition, 10)
                && let Some(icon) = assets.icons.get(&id)
            {
                let rect = Self::rect(assets, 64);
                draw_texture(icon, rect.x, rect.y, WHITE);
            }
            if let Ok(descriptor) = word(definition, 102)
                && let Some(map) = content.map(0)
                && let Ok(text) = map.item_description(descriptor)
            {
                let mut rect = Self::rect(assets, 65);
                rect.y += 12.;
                rect.h -= 12.;
                assets.wrapped_color(&text, rect, 12, 15., YELLOW);
            }
        }
        for item in (40..=63).chain([2, 67, 68, 69, 70]) {
            controls.push(Control {
                rect: Self::rect(assets, item),
                enabled: true,
                action: Action::Application(ApplicationAction::ItemEditorItem(item)),
            });
        }
    }
}
