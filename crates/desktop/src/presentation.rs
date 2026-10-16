use crate::{assets::Assets, layout};
use centurions_content::{Content, Warrior, word};
use centurions_core::{Command, Event, Phase, Presentation, Session};
use macroquad::prelude::*;
use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Inspection {
    Items,
    Crate,
    FireTargets,
    Weapons,
    Conditions,
    Modifiers,
    Specials,
    Psi,
    PsiTargets,
    Bandage,
}
#[derive(Default)]
pub struct UiState {
    pub inventory: crate::inventory::Inventory,
    pub collection_slot: Option<usize>,
    pub collection_drag: bool,
    pub collection_drop: Option<usize>,
    pub collection_drop_scroll: usize,
    pub combat_view: crate::combat_controls::View,
    pub preferences: crate::preferences::Preferences,
    pub feedback: VecDeque<centurions_core::feedback::NativeFeedback>,
    pub persistent_feedback: Option<centurions_core::feedback::NativeFeedback>,
    pub feedback_started: Option<f64>,
    pub feedback_until: f64,
    pub help_request: Option<(usize, usize)>,
    pub technical_error: Option<String>,
    pub diagnostic_path: Option<std::path::PathBuf>,
    pub movement: crate::input::MovementInput,
    pub field_repeat: Option<(centurions_core::field_controls::Action, f64)>,
    pub field_repeating: bool,
    pub selected: usize,
    pub inspection: Option<Inspection>,
    pub return_information: Option<usize>,
    pub page: usize,
    pub typed_text: String,
    pub typed_entry: bool,
    pub effects: VecDeque<OutdoorAnimation>,
    pub delay_until: f64,
    pub psi_power: i16,
    pub psi_choice: Option<(i16, i16)>,
    pub psi_pending: Option<(i16, i16)>,
    pub psi_rotation: u8,
    pub psi_preview: Option<(i16, i16)>,
    pub firearm_map_target: bool,
    pub estimate_target: Option<i16>,
    pub estimate: std::cell::RefCell<Option<crate::targeting::EstimateCache>>,
    pub medical_cancel: bool,
}
pub struct OutdoorAnimation {
    pub effect: i16,
    pub x: i16,
    pub y: i16,
    pub size: i16,
    pub started: f64,
    pub frame_seconds: f64,
}
#[derive(Clone)]
pub enum Action {
    Inventory(crate::inventory::InventoryAction),
    Collection(crate::collection::CollectionAction),
    CombatView(crate::combat_controls::ViewAction),
    PanCombat(i16, i16),
    Application(crate::application::ApplicationAction),
    Command(Command),
    Save,
    FastSave,
    Load,
    Select(usize),
    Open(Inspection),
    InformationInspection(usize, Inspection),
    Close,
    Page(i32),
    ChoosePsi(i16),
    PsiPower(i16),
    RotatePsi,
    TargetQuantity(bool),
    MedicalCancel,
    TextEntry(bool),
    PreviewPsi(i16),
    KeepMedicalTarget,
    Drop(usize, usize),
}
pub(crate) fn authored_text(text: &str, warriors: &[Warrior]) -> String {
    let mut text = text.to_owned();
    for index in 0..warriors.len() {
        let selected = if warriors[index].stamina().unwrap_or(0) < 1 {
            (0..warriors.len())
                .rev()
                .find(|i| warriors[*i].stamina().unwrap_or(0) > 0)
                .unwrap_or(index)
        } else {
            index
        };
        text = text.replace(&format!("^{}", index + 1), &warriors[selected].name);
    }
    text
}

fn draw_clinic(
    state: &Session,
    assets: &Assets,
    member: usize,
    percent: i16,
    controls: &mut Vec<Control>,
) {
    assets.draw_dialog(159);
    for item in [6, 7, 11, 15, 16, 17, 18, 20, 21, 22] {
        controls.push(Control {
            rect: assets.dialog_rect(159, item),
            enabled: true,
            action: Action::Command(Command::ClinicControl { item: item as u16 }),
        });
    }
    for (item, action) in [
        (2, Action::Open(Inspection::Conditions)),
        (
            3,
            Action::Application(crate::application::ApplicationAction::OpenEditor(member)),
        ),
        (
            5,
            Action::Application(crate::application::ApplicationAction::OpenInformation(
                member,
            )),
        ),
    ] {
        controls.push(Control {
            rect: assets.dialog_rect(159, item),
            enabled: true,
            action,
        });
    }
    let field = |item, text: String| {
        assets.wrapped_color(&text, assets.dialog_rect(159, item), 12, 14., YELLOW)
    };
    for (index, price) in centurions_core::clinic::prices(percent).iter().enumerate() {
        if matches!(index, 0..=3 | 5..=7) {
            field(46 + index, price.to_string());
        }
    }
    let warrior = &state.warriors[member];
    field(55, state.party_currency.to_string());
    field(65, warrior.name.clone());
    field(66, word(&warrior.bytes, 0x50).unwrap_or(0).to_string());
    field(67, word(&warrior.bytes, 0x52).unwrap_or(0).to_string());
    for (row, index) in (0..40)
        .filter(|i| word(&warrior.bytes, 0x5e + i * 2).unwrap_or(0) != 0)
        .take(5)
        .enumerate()
    {
        field(
            59 + row,
            assets.conditions.get(index).cloned().unwrap_or_default(),
        );
    }
    let rect = assets.dialog_rect(159, 14);
    if let Some(portrait) = assets
        .portraits
        .get(&word(&warrior.bytes, 0x54).unwrap_or(0))
    {
        draw_texture_ex(
            portrait,
            rect.x,
            rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(rect.size()),
                ..Default::default()
            },
        );
    }
    let health = word(&warrior.bytes, 0x50).unwrap_or(0);
    if health < 1 {
        for id in [2019, if health < -9 { 2015 } else { 177 }] {
            if let Some(icon) = assets.icons.get(&id) {
                draw_texture_ex(
                    icon,
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
    }
}

pub struct Control {
    pub rect: Rect,
    pub action: Action,
    pub enabled: bool,
}

impl UiState {
    pub fn resolve_inventory(
        &mut self,
        action: Option<Action>,
        state: &Session,
        content: &Content,
        now: f64,
    ) -> Result<Option<Action>, centurions_core::RuleError> {
        match action {
            Some(Action::Collection(action)) => Ok(crate::collection::resolve(action, state, self)),
            Some(Action::Inventory(command)) => {
                let category = matches!(command, crate::inventory::InventoryAction::Category(_));
                let result = self.inventory.handle(command, state, content, now)?;
                if category {
                    self.feedback
                        .push_back(centurions_core::feedback::NativeFeedback::Sound {
                            resource_id: 141,
                        });
                }
                Ok(result)
            }
            other => Ok(other),
        }
    }
    pub fn error(&mut self, text: String) {
        self.movement.refuse();
        if self.technical_error.as_ref() != Some(&text) {
            eprintln!("{text}");
            if let Some(path) = &self.diagnostic_path {
                use std::io::Write;
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if std::fs::metadata(path).is_ok_and(|m| m.len() > 1024 * 1024) {
                    let _ = std::fs::remove_file(path);
                }
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                {
                    let _ = writeln!(file, "{text}");
                }
            }
            self.technical_error = Some(text);
        }
    }
    pub fn failure(&mut self, error: centurions_core::RuleError) {
        if let centurions_core::RuleError::Refused { feedback }
        | centurions_core::RuleError::RefusedAfterEffects { feedback } = error
        {
            self.feedback.extend(feedback);
            self.movement.refuse();
        } else {
            self.error(error.to_string());
        }
    }
    pub fn medical_result(&mut self, state: &Session, events: &[Event]) {
        if events
            .iter()
            .any(|event| matches!(event, Event::ItemUpgraded { .. }))
        {
            return;
        }
        if state.medical_continuation.is_some() || state.outdoor_psi.is_some() {
            self.selected = state.selected_warrior;
        }
        self.inspection = None;
        self.psi_pending = None;
        self.medical_cancel = false;
    }

    pub fn events(&mut self, events: &[Event]) {
        for event in events {
            if matches!(event, Event::ItemUpgraded { .. }) {
                self.inventory.selected = None;
            }
            if let Event::Feedback(feedback) = event {
                self.movement.refuse();
                self.feedback.push_back(feedback.clone());
            }
            if let Event::Delay { ticks } = event {
                self.delay_until = self.delay_until.max(get_time())
                    + f64::from(*ticks)
                        / if self.preferences.enabled(10) {
                            120.
                        } else {
                            60.
                        };
            }
            if let Event::OutdoorEffect {
                effect,
                x,
                y,
                size,
                delay,
            } = event
            {
                let started = self
                    .effects
                    .back()
                    .map_or(get_time(), |last| last.started + last.frame_seconds * 8.);
                self.effects.push_back(OutdoorAnimation {
                    effect: *effect,
                    x: *x,
                    y: *y,
                    size: *size,
                    started,
                    frame_seconds: f64::from((*delay).max(1))
                        / if self.preferences.enabled(9) {
                            120.
                        } else {
                            60.
                        },
                });
            }
            if let Event::CollectionRejected { warning } = event {
                self.feedback
                    .push_back(centurions_core::feedback::NativeFeedback::Warning {
                        index: *warning,
                    });
            }
        }
    }
}
fn button(
    assets: &Assets,
    controls: &mut Vec<Control>,
    label: &str,
    rect: Rect,
    enabled: bool,
    action: Action,
) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::from_rgba(43, 46, 0, 245),
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.,
        Color::from_rgba(128, 127, 54, 255),
    );
    let width = measure_text(label, Some(&assets.font), 12, 1.).width;
    assets.text(
        label,
        rect.x + (rect.w - width) / 2.,
        rect.y + rect.h / 2. + 4.,
        12,
        if enabled { WHITE } else { GRAY },
    );
    controls.push(Control {
        rect,
        action,
        enabled,
    });
}
fn field(w: &Warrior, offset: usize) -> String {
    word(&w.bytes, offset)
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "?".into())
}

fn draw_survivors(state: &Session, assets: &Assets, controls: &mut Vec<Control>) {
    assets.draw_dialog(173);
    controls.push(Control {
        rect: assets.dialog_rect(173, 2),
        enabled: true,
        action: Action::Command(Command::Continue),
    });
    let (Some(selection), Some(combat)) = (&state.survivors, &state.combat) else {
        return;
    };
    for row in 0..32 {
        let rect = assets.dialog_rect(173, row + 3);
        controls.push(Control {
            rect,
            enabled: true,
            action: Action::Command(Command::SelectSurvivor { row }),
        });
        let Some(index) = selection.eligible.get(row) else {
            continue;
        };
        let bytes = &combat.enemies.records[*index].bytes;
        if word(bytes, 0x5a).unwrap_or(0) == -1
            && let Some(icon) = assets.icons.get(&2003)
        {
            draw_texture(icon, rect.x + 3., rect.y + 3., WHITE);
        }
        if let Some(icon) = assets.icons.get(&word(bytes, 80).unwrap_or(0)) {
            let (horizontal, vertical) = match word(bytes, 0x10).unwrap_or(0) {
                0 => (19., 19.),
                1 => (19., 3.),
                2 => (3., 19.),
                _ => (3., 3.),
            };
            draw_texture_ex(
                icon,
                rect.x + horizontal,
                rect.y + vertical,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(rect.w - horizontal * 2., rect.h - vertical * 2.)),
                    ..Default::default()
                },
            );
        }
        let (light, dark) = if selection.selected[row] {
            (DARKGRAY, LIGHTGRAY)
        } else {
            (LIGHTGRAY, DARKGRAY)
        };
        draw_line(rect.x, rect.y, rect.right() - 1., rect.y, 1., light);
        draw_line(rect.x, rect.y, rect.x, rect.bottom() - 1., 1., light);
        draw_line(
            rect.right() - 1.,
            rect.y + 1.,
            rect.right() - 1.,
            rect.bottom() - 1.,
            1.,
            dark,
        );
        draw_line(
            rect.x + 1.,
            rect.bottom() - 1.,
            rect.right() - 1.,
            rect.bottom() - 1.,
            1.,
            dark,
        );
        for (offset, x, y) in [
            (0x110, 5., 14.),
            (0x112, 40., 14.),
            (0x10e, 5., 29.),
            (0x52, 40., 29.),
        ] {
            assets.text(
                &word(bytes, offset).unwrap_or(0).to_string(),
                rect.x + x,
                rect.bottom() + y,
                12,
                YELLOW,
            );
        }
    }
    let remaining = selection.limit
        - selection
            .selected
            .iter()
            .filter(|selected| **selected)
            .count();
    assets.panel(
        if remaining <= 7 {
            147 + remaining as i16
        } else {
            147
        },
        assets.dialog_rect(173, 36),
    );
}

fn terrain(assets: &Assets, terrain: u8, tile: i16, x: f32, y: f32) {
    let Some(texture) = assets.terrain.get(&terrain) else {
        return;
    };
    if (1..=300).contains(&tile) {
        let image = tile - 1;
        draw_texture_ex(
            texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(
                    f32::from(image % 20) * 32.,
                    f32::from(image / 20) * 32.,
                    32.,
                    32.,
                )),
                ..Default::default()
            },
        );
    }
}
fn map(content: &Content, state: &Session, assets: &Assets, ui: &UiState) {
    let Some(position) = &state.exploration else {
        return;
    };
    let Some(mission) = content.map(position.map) else {
        return;
    };
    let combat = state.combat.as_ref();
    let (vx, vy) = combat
        .and_then(|_| crate::combat_controls::origin(state, ui))
        .map(|(x, y)| (i32::from(x), i32::from(y)))
        .unwrap_or(layout::outdoor_origin(position.x, position.y));
    for y in 0..13 {
        for x in 0..15 {
            let sx = layout::MAP.x + x as f32 * 32.;
            let sy = layout::MAP.y + y as f32 * 32.;
            if let Some(c) = combat {
                let index = (vx + x + i32::from(c.map.shift.0) + 5) as usize * 90
                    + (vy + y + i32::from(c.map.shift.1) + 5) as usize;
                let tile = c.map.tiles[index];
                let id = tile.wrapping_abs();
                let under = if (0..6).contains(&id) {
                    c.squad.under_tiles[id as usize]
                } else if id >= 10 && id < 10 + c.enemies.records.len() as i16 {
                    word(&c.enemies.records[(id - 10) as usize].bytes, 0x11e)
                        .unwrap_or(1000 + mission.default_tile)
                } else {
                    tile
                };
                terrain(
                    assets,
                    mission.terrain,
                    if under.wrapping_abs() >= 1000 {
                        under.wrapping_abs() - 1000
                    } else {
                        mission.default_tile
                    },
                    sx,
                    sy,
                );
                let icon = if tile < 0 && id >= 1000 {
                    Some(id)
                } else if (0..6).contains(&id) {
                    word(&state.warriors[id as usize].bytes, 0x56).ok()
                } else if id >= 10 && id < 10 + c.enemies.records.len() as i16 {
                    word(&c.enemies.records[(id - 10) as usize].bytes, 0x50).ok()
                } else {
                    None
                };
                if let Some(texture) = icon.and_then(|i| assets.icons.get(&i)) {
                    draw_texture(texture, sx, sy, WHITE);
                }
                if ui.combat_view.reveal && (0..state.warriors.len() as i16).contains(&id) {
                    draw_rectangle_lines(sx + 1., sy + 1., 30., 30., 1., GREEN);
                }
                if c.active_body() == Some(id) {
                    draw_rectangle_lines(sx, sy, 32., 32., 1., YELLOW);
                }
            } else {
                let index = (vx + x) as usize * 90 + (vy + y) as usize;
                terrain(assets, mission.terrain, position.floor[index], sx, sy);
                if position.objects[index] > 0 {
                    terrain(assets, mission.terrain, position.objects[index], sx, sy);
                }
                if position.objects[index] < 0
                    && let Some(texture) = assets.icons.get(&position.objects[index].wrapping_abs())
                {
                    draw_texture(texture, sx, sy, WHITE);
                }
            }
        }
    }
    if combat.is_none()
        && let Some(texture) =
            assets.icons.get(
                &if state.field_controls.as_ref().is_some_and(|c| c.camping) {
                    178
                } else {
                    192 + i16::from(position.facing)
                },
            )
    {
        draw_texture(
            texture,
            (position.x - vx) as f32 * 32.,
            layout::MAP.y + (position.y - vy) as f32 * 32.,
            WHITE,
        );
    }
}

fn draw_outdoor_effect(state: &Session, assets: &Assets, ui: &UiState) {
    let Some(effect) = ui.effects.front() else {
        return;
    };
    let elapsed = get_time() - effect.started;
    if !(0. ..effect.frame_seconds * 8.).contains(&elapsed) {
        return;
    }
    let frame = (elapsed / effect.frame_seconds) as i16;
    let Some(texture) = assets.icons.get(&(12000 + effect.effect * 8 + frame)) else {
        return;
    };
    let Some(position) = state.exploration.as_ref() else {
        return;
    };
    let (vx, vy) = layout::outdoor_origin(position.x, position.y);
    let size = f32::from(effect.size);
    draw_texture_ex(
        texture,
        layout::MAP.x + (i32::from(effect.x) - vx) as f32 * 32. - size / 2.,
        layout::MAP.y + (i32::from(effect.y) - vy) as f32 * 32. - size / 2.,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(size, size)),
            ..Default::default()
        },
    );
}

pub fn draw(content: &Content, state: &Session, assets: &Assets, ui: &UiState) -> Vec<Control> {
    clear_background(Color::from_rgba(41, 42, 0, 255));
    let mut controls = Vec::new();
    map(content, state, assets, ui);
    draw_outdoor_effect(state, assets, ui);
    if let (Some((spell, power)), Some(point), Some((vx, vy))) = (
        ui.psi_pending,
        ui.psi_preview,
        crate::combat_controls::origin(state, ui),
    ) && let Some(mission) = content.map(state.exploration.as_ref().map_or(0, |p| p.map))
        && let Some(tables) = mission.combat.as_ref()
        && let Ok(packet) = centurions_core::combat::spells::load_packet(&tables.attacks, spell)
    {
        let shape = match packet[1] {
            3 => {
                i16::from(packet[31])
                    + if packet[7] != 0 {
                        i16::from(ui.psi_rotation)
                    } else {
                        0
                    }
            }
            4 => power,
            _ => 1,
        };
        assets.target_pattern(
            shape,
            vec2(
                layout::MAP.x + f32::from(point.0 - 3 - vx) * 32. - 2.,
                layout::MAP.y + f32::from(point.1 - 3 - vy) * 32. - 2.,
            ),
            layout::MAP,
        );
    }
    if state.phase == Phase::Exploration {
        assets.panel(199, Rect::new(480., 428., 320., 21.));
    } else {
        assets.panel(197, Rect::new(480., 429., 320., 19.));
    }
    let outdoor_idle = state.phase == Phase::Exploration
        && state
            .exploration
            .as_ref()
            .is_some_and(|p| p.presentation.is_none());
    assets.panel(if outdoor_idle { 222 } else { 220 }, layout::MESSAGE);
    if outdoor_idle && let Some(warrior) = state.warriors.get(ui.selected) {
        let points = i32::from_be_bytes(warrior.bytes[0x9d2..0x9d6].try_into().unwrap());
        assets.text(&points.to_string(), 215., 528., 14, YELLOW);
    }
    if state.phase == Phase::Combat {
        assets.panel(153, Rect::new(468., 449., 332., 89.));
        assets.panel(156, Rect::new(568., 538., 232., 50.));
    } else {
        assets.panel(176, Rect::new(468., 449., 332., 139.));
    }
    assets.panel(224, layout::INFORMATION);
    let selected = state
        .combat
        .as_ref()
        .and_then(|c| c.active_body())
        .filter(|body| (0..6).contains(body))
        .map(|body| body as usize)
        .unwrap_or(ui.selected);
    for (index, w) in state.warriors.iter().enumerate() {
        let rect = layout::row(index);
        assets.panel(189, Rect::new(rect.x + 50., rect.y, 260., 50.));
        if let Ok(id) = word(&w.bytes, 0x54)
            && let Some(texture) = assets.portraits.get(&id)
        {
            draw_texture_ex(
                texture,
                rect.x + 2.,
                rect.y + 2.,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(46., 46.)),
                    ..Default::default()
                },
            );
        }
        let name = if ui.preferences.enabled(17) {
            let mos = word(&w.bytes, 0x34).unwrap_or(0);
            let names = assets
                .character_strings
                .get(&131)
                .and_then(|b| centurions_content::ui::string_list(b).ok())
                .unwrap_or_default();
            names
                .get(mos.saturating_sub(1) as usize)
                .filter(|n| !n.is_empty())
                .map_or_else(|| w.name.clone(), |n| format!("{} {n}", w.name))
        } else {
            w.name.clone()
        };
        assets.text(&name, rect.x + 87., rect.y + 21., 12, YELLOW);
        assets.text(
            &field(
                w,
                if state.phase == Phase::Combat {
                    0x40
                } else {
                    0x12
                },
            ),
            rect.x + 281.,
            rect.y + 21.,
            12,
            WHITE,
        );
        assets.text(&field(w, 0x50), rect.x + 99., rect.y + 44., 12, SKYBLUE);
        assets.text(
            &if word(&w.bytes, 0x5a).ok() == Some(0) {
                "---".into()
            } else {
                field(w, 0x58)
            },
            rect.x + 148.,
            rect.y + 44.,
            12,
            WHITE,
        );
        for (offset, x) in [(0x24, 208.), (0x26, 247.), (0x2c, 294.)] {
            assets.text(&field(w, offset), rect.x + x, rect.y + 44., 12, YELLOW);
        }
        if index == selected {
            draw_rectangle_lines(rect.x, rect.y, 308., 50., 1., YELLOW);
        }
        controls.push(Control {
            rect,
            enabled: true,
            action: if state
                .exploration
                .as_ref()
                .is_some_and(|p| matches!(p.presentation, Some(Presentation::WarriorChoice { .. })))
            {
                Action::Command(Command::ChooseWarrior { index })
            } else {
                Action::Select(index)
            },
        });
    }
    if let Some(p) = &state.exploration {
        if state.phase == Phase::Exploration {
            // xy138a40 and updatefat160a8e; PICT199 supplies the original field labels.
            assets.text(&p.x.to_string(), 512., 444., 12, SKYBLUE);
            assets.text(&p.y.to_string(), 566., 444., 12, SKYBLUE);
            let fatigue = p.fatigue.clamp(4, 135);
            // Original solid pixel patterns131/129/130 change at70 and105.
            let color = if fatigue > 105 {
                Color::from_rgba(255, 0, 0, 255)
            } else if fatigue > 70 {
                Color::from_rgba(255, 255, 0, 255)
            } else {
                Color::from_rgba(51, 51, 0, 255)
            };
            let width = f32::from(fatigue);
            draw_rectangle(660., 432., width, 13., color);
            // downbutton153720: solid patterns136 and133 frame the indicator.
            draw_rectangle(660., 432., width, 1., BLACK);
            draw_rectangle(660., 432., 1., 13., BLACK);
            let edge = Color::from_rgba(102, 102, 51, 255);
            draw_rectangle(659. + width, 433., 1., 12., edge);
            draw_rectangle(661., 444., width - 1., 1., edge);
        }
        assets.text(
            &format!(
                "{}:{:02} {}",
                (p.hour + 11) % 12 + 1,
                p.minute,
                if p.hour < 12 { "AM" } else { "PM" }
            ),
            709.,
            if state.phase == Phase::Combat {
                530.
            } else {
                516.
            },
            12,
            YELLOW,
        );
        if state.phase == Phase::Exploration {
            assets.text(&p.day.to_string(), 744., 530., 12, YELLOW);
        }
    }
    if let Some(w) = state.warriors.get(selected) {
        let r = layout::INFORMATION;
        let inventory = w.inventory().unwrap_or_default();
        let weapon = word(&w.bytes, 0xe4).unwrap_or(0);
        let loaded = inventory
            .iter()
            .find(|i| i.id() == weapon && i.0[9] == 1)
            .and_then(|i| word(&i.0, 2).ok());
        assets.text(
            &format!("{}  Stamina {}/{}", w.name, field(w, 0x50), field(w, 0x52)),
            r.x + 10.,
            r.y + 19.,
            12,
            WHITE,
        );
        assets.text(
            &format!(
                "Psi {}/{}   Shield {}  Armor {}",
                field(w, 0x58),
                field(w, 0x5a),
                field(w, 0x24),
                field(w, 0x2c)
            ),
            r.x + 10.,
            r.y + 37.,
            12,
            WHITE,
        );
        assets.text(&assets.name(weapon), r.x + 10., r.y + 55., 12, YELLOW);
        assets.text(
            &format!(
                "Loaded {}  Reserve {}",
                loaded.map(|v| v.to_string()).unwrap_or_else(|| "—".into()),
                equipped_ammunition_reserve(state, w)
            ),
            r.x + 10.,
            r.y + 73.,
            12,
            WHITE,
        );
        if let Some(c) = &state.combat {
            assets.text(
                &format!(
                    "AP {}  MP {}  Round {}",
                    field(w, 0x40),
                    field(w, 0x3c),
                    c.round + 1
                ),
                r.x + 10.,
                r.y + 91.,
                12,
                WHITE,
            );
        }
    }
    let mission_ready = state.phase != Phase::SquadSelection;
    let (items_rect, conditions_rect) = if state.phase == Phase::Combat {
        (layout::COMBAT_ITEMS, layout::COMBAT_CONDITIONS)
    } else {
        (assets.main_controls[&168], assets.main_controls[&169])
    };
    if state.phase == Phase::Combat {
        draw_combat_information(
            content,
            state,
            assets,
            ui.combat_view.selected(state).unwrap_or(selected as i16),
        );
    }
    for (rect, inspection) in [
        (items_rect, Inspection::Items),
        (conditions_rect, Inspection::Conditions),
    ] {
        let action = Action::Open(inspection);
        controls.push(Control {
            rect,
            enabled: mission_ready && crate::input::available(state, selected, &action),
            action,
        });
    }
    if state.phase != Phase::SquadSelection {
        if state.phase == Phase::Combat {
            let capture =
                crate::input::available(state, selected, &Action::Command(Command::BeginCapture));
            assets.panel(if capture { 161 } else { 154 }, assets.main_controls[&301]);
            if !crate::input::available(state, selected, &Action::Open(Inspection::Psi)) {
                assets.panel(154, assets.main_controls[&300]);
            }
            let mut buttons = crate::combat_controls::buttons(state, content, selected);
            for button in &mut buttons {
                match button.action {
                    Action::Open(Inspection::Weapons) => button.rect = assets.main_controls[&166],
                    Action::Application(
                        crate::application::ApplicationAction::CombatInformation(_),
                    ) => button.rect = assets.main_controls[&167],
                    _ => {}
                }
            }
            if let Some(body) = ui.combat_view.selected(state) {
                for button in &mut buttons {
                    if let Action::Application(
                        crate::application::ApplicationAction::CombatInformation(selected),
                    ) = &mut button.action
                    {
                        *selected = body;
                    }
                }
            }
            controls.extend(buttons);
            crate::combat_controls::draw_automatic(state, assets, &mut controls);
        }
        if state.phase == Phase::Exploration {
            let context = assets.main_controls[&301];
            if state
                .field_controls
                .as_ref()
                .and_then(|c| c.clinic)
                .is_some()
            {
                assets.panel(200, context);
            } else if state.shop_access.is_some() {
                assets.panel(225, context);
            } else {
                assets.panel(208, context);
            }
            for (id, action) in crate::input::field_buttons(state) {
                let rect = assets.main_controls[&id];
                let enabled = crate::input::available(state, selected, &action);
                if !enabled {
                    assets.panel(154, rect);
                }
                controls.push(Control {
                    rect,
                    enabled,
                    action,
                });
            }
            if state.warriors.len() < 2 {
                assets.panel(154, assets.main_controls[&305]);
            }
        }
        if let Some(p) = state
            .exploration
            .as_ref()
            .and_then(|p| p.presentation.as_ref())
        {
            match p {
                Presentation::Messages { pages, .. } => {
                    if state.phase == Phase::Combat {
                        assets.panel(221, layout::MESSAGE);
                    }
                    controls.clear();
                    if let Some((_, text)) = pages.first() {
                        assets.wrapped(
                            &authored_text(text, &state.warriors),
                            Rect::new(12., 467., 444., 82.),
                            12,
                            16.,
                        );
                    }
                }
                Presentation::Dialogue { text, .. } => {
                    // textbox15deac replaces the combat panel with PICT221.
                    if state.phase == Phase::Combat {
                        assets.panel(221, layout::MESSAGE);
                    }
                    controls.clear();
                    assets.wrapped(
                        &authored_text(text, &state.warriors),
                        Rect::new(12., 467., 444., 82.),
                        12,
                        16.,
                    );
                }
                Presentation::Question { text, .. } | Presentation::Avoidance { text, .. } => {
                    crate::story_dialogs::question(
                        assets,
                        &authored_text(text, &state.warriors),
                        Action::Command(Command::Choose { answer: true }),
                        Action::Command(Command::Choose { answer: false }),
                        &mut controls,
                    );
                }
                Presentation::Menu { id, text, labels } => {
                    crate::story_dialogs::menu(
                        content,
                        state,
                        assets,
                        *id,
                        &authored_text(text, &state.warriors),
                        labels,
                        &mut controls,
                    );
                }
                Presentation::WarriorChoice { .. } => {
                    controls.clear();
                    crate::story_dialogs::portraits(state, assets, &mut controls, |index| {
                        (state.warriors[index].stamina().unwrap_or(0) > 0)
                            .then_some(Action::Command(Command::ChooseWarrior { index }))
                    });
                }
                Presentation::StageChange { .. } => {
                    crate::story_dialogs::question(
                        assets,
                        assets.questions.get(7).map_or("", String::as_str),
                        Action::Command(Command::Choose { answer: true }),
                        Action::Command(Command::Choose { answer: false }),
                        &mut controls,
                    );
                }
                Presentation::Prologue { .. } => {}
                Presentation::Typed { text, .. } => {
                    crate::story_dialogs::typed(
                        content,
                        state,
                        assets,
                        ui,
                        &authored_text(text, &state.warriors),
                        &mut controls,
                    );
                }
                Presentation::Loot | Presentation::Shop | Presentation::Clinic { .. } => {}
                Presentation::Survivors | Presentation::ScriptPause => {}
            }
            if !matches!(
                p,
                Presentation::Question { .. }
                    | Presentation::Dialogue { .. }
                    | Presentation::Messages { .. }
                    | Presentation::Avoidance { .. }
                    | Presentation::Menu { .. }
                    | Presentation::Typed { .. }
                    | Presentation::StageChange { .. }
                    | Presentation::WarriorChoice { .. }
                    | Presentation::Clinic { .. }
                    | Presentation::Survivors
                    | Presentation::ScriptPause
            ) {
                let keep_spells = matches!(p, Presentation::Loot)
                    && centurions_core::needs_spell_confirmation(state);
                button(
                    assets,
                    &mut controls,
                    "Continue",
                    layout::CONTINUE,
                    !state.booty_items.iter().any(|i| i.id() != 0),
                    Action::Command(if keep_spells {
                        Command::BeginPromotion
                    } else {
                        Command::Continue
                    }),
                );
            }
        }
    }
    if state
        .exploration
        .as_ref()
        .is_some_and(|p| p.presentation == Some(Presentation::Shop))
    {
        ui.inventory.draw(content, state, assets, &mut controls);
        return controls;
    }
    if crate::collection::active(state, ui) && state.promotion.is_none() {
        crate::collection::draw(content, state, assets, ui, &mut controls);
        return controls;
    }
    if let Some(Presentation::Clinic { member, percent }) = state
        .exploration
        .as_ref()
        .and_then(|p| p.presentation.as_ref())
    {
        controls.clear();
        draw_clinic(state, assets, *member, *percent, &mut controls);
        if ui.inspection.is_none() {
            return controls;
        }
    }
    if state
        .exploration
        .as_ref()
        .is_some_and(|position| position.presentation == Some(Presentation::Survivors))
    {
        controls.clear();
        draw_survivors(state, assets, &mut controls);
        return controls;
    }
    if let Some(selected) = state.encounter_actions {
        controls.clear();
        assets.draw_dialog(148);
        if let Ok(record) = centurions_core::typed_record(state, content) {
            for (index, chosen) in selected.into_iter().enumerate() {
                assets.panel(156, assets.dialog_rect(148, index + 9));
                let checkbox = assets.dialog_rect(148, index + 1);
                assets.panel(179, checkbox);
                if chosen {
                    draw_rectangle(
                        checkbox.x + 2.,
                        checkbox.y + 2.,
                        checkbox.w - 4.,
                        checkbox.h - 4.,
                        YELLOW,
                    );
                }
                let label = &record[156 + index * 40..196 + index * 40];
                let text =
                    centurions_content::ui::c_string(&label[1..1 + usize::from(label[0]).min(39)]);
                assets.wrapped_color(&text, assets.dialog_rect(148, index + 19), 12, 14., YELLOW);
            }
        }
        for item in 9..=18 {
            controls.push(Control {
                rect: assets.dialog_rect(148, item),
                enabled: true,
                action: Action::Command(Command::EncounterActionControl { item: item as u16 }),
            });
        }
        return controls;
    }
    if let Some(inspection) = ui.inspection {
        if crate::status_popup::active(ui) {
            controls.clear();
            crate::status_popup::draw(state, assets, ui, &mut controls);
            return controls;
        }
        if inspection == Inspection::Specials {
            return controls;
        }
        if inspection == Inspection::Items {
            ui.inventory.draw(content, state, assets, &mut controls);
            return controls;
        }
        if inspection == Inspection::Weapons {
            controls.clear();
            crate::combat_controls::draw_weapons(state, assets, ui, &mut controls);
            return controls;
        }
        if inspection == Inspection::Psi {
            let inspected = state
                .combat
                .as_ref()
                .and_then(|c| c.active_body())
                .filter(|body| (0..state.warriors.len() as i16).contains(body))
                .map_or(ui.selected, |body| body as usize);
            crate::scan_picker::draw(content, state, assets, ui, inspected, &mut controls);
            return controls;
        }
    }
    if let Some(progress) = &state.promotion {
        controls.clear();
        crate::editor::ScanEditor::draw_promotion(progress, content, assets, &mut controls);
    }
    if crate::targeting::active(state, ui) {
        crate::targeting::draw(content, state, assets, ui, &mut controls);
    }
    for control in &mut controls {
        control.enabled &= crate::input::available(state, ui.selected, &control.action);
    }
    controls
}

fn draw_combat_information(content: &Content, state: &Session, assets: &Assets, body: i16) {
    assets.panel(166, Rect::new(0., 449., 308., 139.));
    assets.panel(228, Rect::new(310., 449., 160., 140.));
    if body >= 10 {
        let Some(enemy) = state
            .combat
            .as_ref()
            .and_then(|c| c.enemies.records.get((body - 10) as usize))
        else {
            return;
        };
        let record = &enemy.bytes;
        for (offset, x, y) in [
            (0x110, 115., 470.),
            (0x10e, 115., 493.),
            (0x58, 223., 470.),
            (0x8, 274., 470.),
        ] {
            assets.text(
                &word(record, offset).unwrap_or(0).to_string(),
                x,
                y,
                12,
                WHITE,
            );
        }
        for (offset, x) in [(0x106, 115.), (0x109, 211.)] {
            assets.text(
                &(record[offset] as i8).max(0).to_string(),
                x,
                516.,
                12,
                WHITE,
            );
        }
        if let Ok(icon) = word(record, 0x50)
            && let Some(texture) = assets.icons.get(&icon)
        {
            draw_texture_ex(
                texture,
                8.,
                456.,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(64., 64.)),
                    ..Default::default()
                },
            );
        }
        assets.text(
            &centurions_content::combat::monster_name(record).unwrap_or_default(),
            126.,
            536.,
            12,
            WHITE,
        );
        return;
    }
    let Some(w) = state.warriors.get(body as usize) else {
        return;
    };
    for (offset, x, y) in [
        (0x50, 115., 470.),
        (0x58, 115., 493.),
        (0x3c, 115., 516.),
        (0x40, 211., 516.),
        (0x24, 181., 470.),
        (0x26, 223., 470.),
        (0x2c, 274., 470.),
    ] {
        assets.text(&field(w, offset), x, y, 12, WHITE);
    }
    if let Ok(id) = word(&w.bytes, 0x56)
        && let Some(texture) = assets.icons.get(&id)
    {
        draw_texture(texture, 22., 472., WHITE);
    }
    assets.text(&w.name, 126., 536., 12, WHITE);
    let id = word(&w.bytes, 0xe4).unwrap_or(0);
    assets.text(&assets.name(id), 126., 558., 12, YELLOW);
    if let Some(def) = content.items.get(id.unsigned_abs() as usize)
        && let Ok(icon) = word(def, 10)
        && let Some(texture) = assets.icons.get(&icon)
    {
        draw_texture(texture, 8., 529., WHITE);
    }
    if let Some(item) = w
        .inventory()
        .unwrap_or_default()
        .iter()
        .find(|i| i.id() == id && i.0[9] == 1)
    {
        assets.text(
            &word(&item.0, 2).unwrap_or(0).to_string(),
            72.,
            580.,
            12,
            YELLOW,
        );
    }
    assets.text(
        &format!("Reserve {}", equipped_ammunition_reserve(state, w)),
        116.,
        580.,
        12,
        WHITE,
    );
}
fn equipped_ammunition_reserve(state: &Session, warrior: &Warrior) -> i32 {
    let weapon = word(&warrior.bytes, 0xe4).unwrap_or(0);
    warrior
        .inventory()
        .unwrap_or_default()
        .iter()
        .find(|item| item.id() == weapon && item.0[9] == 1)
        .and_then(|item| word(&item.0, 4).ok())
        .filter(|ammunition| (200..400).contains(ammunition))
        .map(|ammunition| (ammunition - 200) as usize)
        .and_then(|index| state.ammunition.get(index))
        .copied()
        .unwrap_or(0)
}
pub fn draw_specials(state: &Session, assets: &Assets, ui: &UiState, controls: &mut Vec<Control>) {
    // showspecial15743e: DLOG160, one32-pixel header and one row per named ability.
    let warrior = &state.warriors[ui.selected];
    let rows: Vec<_> = assets.reference_strings[&1201]
        .iter()
        .take(14)
        .enumerate()
        .filter_map(|(index, label)| {
            let base = word(&warrior.bytes, 0xf4 + index * 2).unwrap_or(0);
            (base > 0 && !label.is_empty()).then(|| {
                let value = base.wrapping_add(word(&warrior.bytes, 0xb62 + index * 2).unwrap_or(0));
                (label, value)
            })
        })
        .collect();
    let origin = vec2(100., 170.);
    let port = Rect::new(origin.x, origin.y, 300., (rows.len() + 1) as f32 * 32.);
    for row in 0..=rows.len() {
        assets.panel(
            177,
            Rect::new(origin.x, origin.y + row as f32 * 32., 300., 32.),
        );
    }
    let text = |item: usize, value: &str| {
        let [top, left, bottom, right] = assets.dialogs[&160][item - 1].rect;
        assets.wrapped_color(
            value,
            Rect::new(
                origin.x + left as f32,
                origin.y + top as f32,
                (right - left) as f32,
                (bottom - top) as f32,
            ),
            12,
            14.,
            YELLOW,
        );
    };
    for item in [16, 43] {
        text(item, &assets.dialogs[&160][item - 1].text());
    }
    for (row, (label, value)) in rows.iter().enumerate() {
        text(17 + row, label);
        text(30 + row, &value.to_string());
    }
    controls.push(Control {
        rect: port,
        enabled: true,
        action: Action::Close,
    });
}
