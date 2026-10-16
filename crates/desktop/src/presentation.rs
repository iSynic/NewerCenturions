use crate::{assets::Assets, layout};
use centurions_content::{Content, Warrior, word};
use centurions_core::{Command, Event, MedicalTarget, Phase, Presentation, Session};
use macroquad::prelude::*;
use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Inspection {
    Items,
    Crate,
    FireTargets,
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
    pub preferences: crate::preferences::Preferences,
    pub feedback: VecDeque<centurions_core::feedback::NativeFeedback>,
    pub feedback_started: Option<f64>,
    pub feedback_until: f64,
    pub help_request: Option<(usize, usize)>,
    pub technical_error: Option<String>,
    pub diagnostic_path: Option<std::path::PathBuf>,
    pub movement: crate::input::MovementInput,
    pub selected: usize,
    pub inspection: Option<Inspection>,
    pub return_information: Option<usize>,
    pub page: usize,
    pub typed_text: String,
    pub effects: VecDeque<OutdoorAnimation>,
    pub delay_until: f64,
    pub psi_power: i16,
    pub psi_choice: Option<(i16, i16)>,
    pub psi_pending: Option<(i16, i16)>,
    pub psi_rotation: u8,
    pub psi_preview: Option<(i16, i16)>,
    pub firearm_map_target: bool,
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
    PsiMapTarget,
    RotatePsi,
    FireMapTarget,
    MedicalCancel,
    KeepMedicalTarget,
    Drop(usize, usize),
}
fn authored_text(text: &str, warriors: &[Warrior]) -> String {
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

pub fn tactical_view(session: &Session) -> Option<(i16, i16)> {
    let c = session.combat.as_ref()?;
    let body = c.active_body().unwrap_or(0);
    let p = if body < 10 {
        c.squad.positions[body as usize]
    } else {
        c.enemies.positions[(body - 10) as usize]
    };
    let shift = (c.map.shift.0 + 5, c.map.shift.1 + 5);
    Some((
        (p.0 - 7).clamp(-shift.0, 75 - shift.0),
        (p.1 - 6).clamp(-shift.1, 77 - shift.1),
    ))
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
fn map(content: &Content, state: &Session, assets: &Assets) {
    let Some(position) = &state.exploration else {
        return;
    };
    let Some(mission) = content.map(position.map) else {
        return;
    };
    let combat = state.combat.as_ref();
    let (vx, vy) = combat
        .and_then(|_| tactical_view(state))
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
        && let Some(texture) = assets.icons.get(&(192 + i16::from(position.facing)))
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
    map(content, state, assets);
    draw_outdoor_effect(state, assets, ui);
    if let (Some((spell, power)), Some(point), Some((vx, vy))) =
        (ui.psi_pending, ui.psi_preview, tactical_view(state))
        && let Some(mission) = content.map(state.exploration.as_ref().map_or(0, |p| p.map))
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
    assets.panel(197, Rect::new(480., 429., 320., 19.));
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
    assets.panel(224, Rect::new(480., 449., 320., 140.));
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
        assets.text(&format!("X: {}  Y: {}", p.x, p.y), 489., 443., 12, SKYBLUE);
        assets.text(&format!("Fatigue {}", p.fatigue), 603., 443., 12, GREEN);
        assets.text(
            &format!("{:02}:{:02}  Day {}", p.hour, p.minute, p.day),
            709.,
            516.,
            12,
            YELLOW,
        );
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
        } else {
            let count = (0x5e..0xae)
                .step_by(2)
                .filter(|o| word(&w.bytes, *o).unwrap_or(0) != 0)
                .count();
            assets.text(
                &format!("{count} conditions  |  {} inventory slots", inventory.len()),
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
        draw_combat_information(content, state, assets, selected);
    }
    button(
        assets,
        &mut controls,
        "Items",
        items_rect,
        mission_ready,
        Action::Open(Inspection::Items),
    );
    button(
        assets,
        &mut controls,
        "Conditions",
        conditions_rect,
        mission_ready,
        Action::Open(Inspection::Conditions),
    );
    if state.phase != Phase::SquadSelection {
        let player = state
            .combat
            .as_ref()
            .and_then(|c| c.active_body())
            .is_some_and(|b| (0..6).contains(&b))
            && centurions_core::combat::runtime::controlled_warrior(state).is_none();
        let unobstructed = state
            .exploration
            .as_ref()
            .is_some_and(|p| p.presentation.is_none())
            && (state.phase == Phase::Exploration || player);
        let firearm = state.warriors[selected].bytes[0x9dd] != 0;
        if state.phase == Phase::Combat {
            button(
                assets,
                &mut controls,
                if firearm { "Fists [F]" } else { "Firearm [F]" },
                layout::FIREARM,
                player,
                Action::Command(Command::SetFirearmMode {
                    warrior: selected,
                    enabled: !firearm,
                }),
            );
            button(
                assets,
                &mut controls,
                "Load [R]",
                layout::RELOAD,
                unobstructed,
                crate::input::refill_action(state, content, selected),
            );
            button(
                assets,
                &mut controls,
                "End [Space]",
                layout::END_TURN,
                player,
                Action::Command(Command::EndTurn),
            );
        }
        if state.phase == Phase::Exploration {
            let rect = assets.main_controls[&300];
            let enabled =
                unobstructed && word(&state.warriors[selected].bytes, 0x36).unwrap_or(0) != 0;
            assets.panel(if enabled { 162 } else { 154 }, rect);
            controls.push(Control {
                rect,
                enabled,
                action: Action::Open(Inspection::Psi),
            });
        } else {
            button(
                assets,
                &mut controls,
                "Psi [P]",
                Rect::new(486., 516., 126., 26.),
                unobstructed && (state.phase == Phase::Exploration || player),
                Action::Open(Inspection::Psi),
            );
        }
        if state.phase == Phase::Combat {
            button(
                assets,
                &mut controls,
                "Stop bleeding [B]",
                Rect::new(486., 486., 208., 24.),
                unobstructed && state.bandage_ready.is_some_and(|ready| ready != 0),
                Action::Open(Inspection::Bandage),
            );
            button(
                assets,
                &mut controls,
                "Shoot targets",
                Rect::new(624., 516., 126., 26.),
                player && firearm,
                Action::Open(Inspection::FireTargets),
            );
        } else {
            button(
                assets,
                &mut controls,
                "Encounter [E]",
                assets.main_controls[&306],
                unobstructed,
                Action::Command(Command::Encounter),
            );
        }
        if state.phase == Phase::Combat {
            button(
                assets,
                &mut controls,
                "Guard [G]",
                layout::GUARD,
                unobstructed
                    && player
                    && state
                        .combat_story
                        .as_ref()
                        .is_none_or(|story| story.waiting_child)
                    && state.warriors[selected].bytes[0x9df] != 0
                    && word(&state.warriors[selected].bytes, 0x50).unwrap_or(0) > 0
                    && word(&state.warriors[selected].bytes, 0x40).unwrap_or(0) >= 2,
                Action::Command(Command::Guard),
            );
        }
        if let Some(p) = state
            .exploration
            .as_ref()
            .and_then(|p| p.presentation.as_ref())
        {
            match p {
                Presentation::Messages { pages, .. } => {
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
                    assets.wrapped(
                        &authored_text(text, &state.warriors),
                        Rect::new(12., 467., 444., 82.),
                        12,
                        16.,
                    );
                }
                Presentation::Question { text, .. } | Presentation::Avoidance { text, .. } => {
                    assets.wrapped(
                        &authored_text(text, &state.warriors),
                        Rect::new(12., 467., 444., 82.),
                        12,
                        16.,
                    );
                    for (label, x, answer) in [("Yes [Y]", 490., true), ("No [N]", 625., false)] {
                        button(
                            assets,
                            &mut controls,
                            label,
                            Rect::new(x, 565., 125., 28.),
                            true,
                            Action::Command(Command::Choose { answer }),
                        );
                    }
                }
                Presentation::Menu { id, text, labels } => {
                    assets.panel(224, Rect::new(12., 34., 456., 394.));
                    assets.wrapped(
                        &authored_text(text, &state.warriors),
                        Rect::new(25., 56., 430., 150.),
                        12,
                        16.,
                    );
                    for (i, label) in labels.iter().enumerate() {
                        let table = if state.menus.is_empty() {
                            &content.map(0).unwrap().menus
                        } else {
                            &state.menus
                        };
                        let enabled = table
                            .get(*id as usize * centurions_core::story::MENU_RECORD_BYTES + 96 + i)
                            .is_some_and(|code| *code != 0);
                        button(
                            assets,
                            &mut controls,
                            &format!("{}  [{}]", authored_text(label, &state.warriors), i + 1),
                            Rect::new(25., 215. + i as f32 * 43., 430., 40.),
                            enabled,
                            Action::Command(Command::SelectMenuEntry { index: i }),
                        );
                    }
                    button(
                        assets,
                        &mut controls,
                        "Cancel [Esc]",
                        layout::CONTINUE,
                        true,
                        Action::Command(Command::CancelMenu),
                    );
                }
                Presentation::WarriorChoice { .. } => {}
                Presentation::StageChange { .. } => {
                    for (label, x, answer) in [("Yes [Y]", 490., true), ("No [N]", 625., false)] {
                        button(
                            assets,
                            &mut controls,
                            label,
                            Rect::new(x, 565., 125., 28.),
                            true,
                            Action::Command(Command::Choose { answer }),
                        );
                    }
                }
                Presentation::Prologue { .. } => {}
                Presentation::Typed { text, .. } => {
                    assets.panel(224, Rect::new(12., 34., 456., 394.));
                    assets.wrapped(
                        &authored_text(text, &state.warriors),
                        Rect::new(25., 56., 430., 230.),
                        13,
                        17.,
                    );
                    draw_rectangle_lines(25., 315., 430., 28., 1., WHITE);
                    assets.text(&ui.typed_text, 32., 334., 13, WHITE);
                    button(
                        assets,
                        &mut controls,
                        "Submit [Enter]",
                        Rect::new(25., 360., 200., 26.),
                        true,
                        Action::Command(Command::SubmitText {
                            text: ui.typed_text.clone(),
                        }),
                    );
                    button(
                        assets,
                        &mut controls,
                        "Cancel [Esc]",
                        Rect::new(245., 360., 200., 26.),
                        true,
                        Action::Command(Command::CancelMenu),
                    );
                    for (index, action) in [
                        (1, Action::Open(Inspection::Psi)),
                        (3, Action::Command(Command::BeginEncounterActions)),
                        (4, Action::Open(Inspection::Items)),
                    ] {
                        let item_rect = Rect::new(25. + index as f32 * 50., 390., 50., 50.);
                        let enabled =
                            centurions_core::typed_record(state, content).is_ok_and(|record| {
                                if index == 1 {
                                    centurions_content::word(record, 106).is_ok_and(|id| id != 0)
                                } else if index == 3 {
                                    record[96] != 0
                                } else {
                                    centurions_content::word(record, 136).is_ok_and(|id| id != 0)
                                }
                            });
                        if let Some(item) =
                            assets.dialogs.get(&153).and_then(|items| items.get(index))
                            && let Ok(id) = centurions_content::word(&item.data, 0)
                        {
                            assets.panel(id, item_rect);
                            if !enabled {
                                draw_rectangle(
                                    item_rect.x,
                                    item_rect.y,
                                    item_rect.w,
                                    item_rect.h,
                                    Color::new(0., 0., 0., 0.6),
                                );
                            }
                            controls.push(Control {
                                rect: item_rect,
                                enabled,
                                action,
                            });
                        }
                    }
                }
                Presentation::Loot | Presentation::Shop | Presentation::Clinic { .. } => {}
                Presentation::Survivors => {}
            }
            if !matches!(
                p,
                Presentation::Question { .. }
                    | Presentation::Avoidance { .. }
                    | Presentation::Menu { .. }
                    | Presentation::Typed { .. }
                    | Presentation::StageChange { .. }
                    | Presentation::WarriorChoice { .. }
                    | Presentation::Clinic { .. }
                    | Presentation::Survivors
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
        if inspection == Inspection::Items
            && state
                .exploration
                .as_ref()
                .is_some_and(|position| position.presentation.is_none())
        {
            ui.inventory.draw(content, state, assets, &mut controls);
            return controls;
        }
        controls.clear();
        draw_rectangle(0., 20., 800., 580., Color::from_rgba(0, 0, 0, 160));
        assets.panel(224, layout::INSPECT);
        let title = match inspection {
            Inspection::Items | Inspection::Crate => "Items",
            Inspection::FireTargets => "Target",
            Inspection::Conditions => "Conditions",
            Inspection::Modifiers | Inspection::Specials => "",
            Inspection::Psi | Inspection::PsiTargets => "Psi",
            Inspection::Bandage => "Stop bleeding",
        };
        let inspected = if matches!(
            inspection,
            Inspection::Psi | Inspection::PsiTargets | Inspection::FireTargets
        ) {
            state
                .combat
                .as_ref()
                .and_then(|c| c.active_body())
                .filter(|b| (0..6).contains(b))
                .map(|b| b as usize)
                .unwrap_or(ui.selected)
        } else {
            ui.selected
        };
        let title = if matches!(inspection, Inspection::Modifiers | Inspection::Specials) {
            assets.information_dialog[if inspection == Inspection::Modifiers {
                4
            } else {
                3
            }]
            .text()
        } else {
            title.to_owned()
        };
        assets.text(
            &format!("{title} — {}", state.warriors[inspected].name),
            48.,
            70.,
            18,
            YELLOW,
        );
        button(
            assets,
            &mut controls,
            "Close [Esc]",
            layout::CLOSE,
            true,
            if inspection == Inspection::Bandage {
                Action::Command(Command::StopBleeding { warrior: None })
            } else {
                Action::Close
            },
        );
        for i in 0..if matches!(
            inspection,
            Inspection::Psi | Inspection::PsiTargets | Inspection::FireTargets
        ) && state.phase == Phase::Combat
        {
            0
        } else {
            state.warriors.len()
        } {
            button(
                assets,
                &mut controls,
                &state.warriors[i].name,
                Rect::new(48. + i as f32 * 116., 88., 110., 25.),
                true,
                Action::Select(i),
            );
        }
        match inspection {
            Inspection::Conditions => draw_conditions(state, assets, ui),
            Inspection::Modifiers => draw_modifiers(state, assets, ui),
            Inspection::Specials => draw_specials(state, assets, ui),
            Inspection::Bandage => draw_bandage(state, assets, &mut controls),
            Inspection::Items | Inspection::Crate => {
                draw_items(content, state, assets, ui, &mut controls)
            }
            Inspection::FireTargets => draw_fire_targets(content, state, assets, ui, &mut controls),
            Inspection::Psi | Inspection::PsiTargets => {
                draw_psi(content, state, assets, ui, &mut controls, inspected)
            }
        }
    }
    if let Some(progress) = &state.promotion {
        controls.clear();
        crate::editor::ScanEditor::draw_promotion(progress, content, assets, &mut controls);
    }
    if state.outdoor_psi.is_some() {
        controls.clear();
        draw_outdoor_psi(state, assets, &mut controls);
    }
    if state.psi_selection.is_some() {
        controls.clear();
        draw_rectangle(0., 20., 800., 580., Color::from_rgba(0, 0, 0, 180));
        assets.panel(224, layout::INSPECT);
        draw_psi_selection(content, state, assets, ui, &mut controls);
    }
    if state.medical_continuation.is_some() {
        controls.clear();
        draw_rectangle(0., 20., 800., 580., Color::from_rgba(0, 0, 0, 180));
        assets.panel(224, layout::INSPECT);
        draw_medical_target(content, state, assets, ui, &mut controls);
        button(
            assets,
            &mut controls,
            "Save",
            Rect::new(48., 510., 90., 26.),
            true,
            Action::Save,
        );
        button(
            assets,
            &mut controls,
            "Reload",
            Rect::new(148., 510., 90., 26.),
            true,
            Action::Load,
        );
    }
    for control in &mut controls {
        control.enabled &= crate::input::available(state, ui.selected, &control.action);
    }
    controls
}

fn draw_bandage(state: &Session, assets: &Assets, controls: &mut Vec<Control>) {
    for (index, warrior) in state.warriors.iter().enumerate() {
        let y = 185. + index as f32 * 44.;
        assets.text(
            &format!(
                "{}  {}  {}",
                warrior.name,
                warrior.stamina().unwrap_or(0),
                if warrior.bytes[0x9de] != 0 {
                    "Bleeding"
                } else {
                    ""
                }
            ),
            48.,
            y + 22.,
            14,
            WHITE,
        );
        button(
            assets,
            controls,
            "Bandage",
            Rect::new(596., y, 132., 28.),
            true,
            Action::Command(Command::StopBleeding {
                warrior: Some(index),
            }),
        );
    }
    button(
        assets,
        controls,
        "Cancel and end turn",
        Rect::new(480., 492., 248., 28.),
        true,
        Action::Command(Command::StopBleeding { warrior: None }),
    );
}

fn draw_outdoor_psi(state: &Session, assets: &Assets, controls: &mut Vec<Control>) {
    draw_rectangle(0., 20., 800., 580., Color::from_rgba(0, 0, 0, 180));
    assets.panel(224, layout::INSPECT);
    let item_prompt = state
        .outdoor_psi
        .as_ref()
        .filter(|pending| pending.item.is_some())
        .and_then(|_| assets.native_text.text(0x4033f6, false).ok());
    assets.wrapped_color(
        item_prompt
            .as_deref()
            .unwrap_or_else(|| assets.warnings.get(46).map_or("", String::as_str)),
        Rect::new(48., 80., 674., 72.),
        14,
        18.,
        YELLOW,
    );
    for (member, warrior) in state.warriors.iter().enumerate() {
        let y = 165. + member as f32 * 48.;
        if let Ok(id) = word(&warrior.bytes, 0x54)
            && let Some(texture) = assets.portraits.get(&id)
        {
            draw_texture_ex(
                texture,
                48.,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(44., 44.)),
                    ..Default::default()
                },
            );
        }
        button(
            assets,
            controls,
            &warrior.name,
            Rect::new(100., y, 622., 44.),
            true,
            Action::Command(Command::SelectOutdoorPsiTarget { warrior: member }),
        );
        let rank = state.party_mask[member];
        if rank > 0 {
            assets.text(&rank.to_string(), 685., y + 28., 18, YELLOW);
        }
    }
    button(
        assets,
        controls,
        &assets.native_text.text(0x4001ec, true).unwrap_or_default(),
        Rect::new(48., 485., 110., 26.),
        true,
        Action::Command(Command::CancelOutdoorPsi),
    );
}

fn draw_psi_selection(
    content: &Content,
    state: &Session,
    assets: &Assets,
    ui: &UiState,
    controls: &mut Vec<Control>,
) {
    let pending = state.psi_selection.as_ref().unwrap();
    assets.text(
        assets
            .spell_names
            .get(&pending.spell)
            .map_or("", String::as_str),
        48.,
        70.,
        18,
        YELLOW,
    );
    if ui.medical_cancel {
        let prompt = assets.native_text.text(0x403c72, false).unwrap_or_default();
        assets.wrapped_color(&prompt, Rect::new(48., 110., 674., 130.), 14, 18., YELLOW);
        for (x, address, action) in [
            (48., 0x400fb2, Action::Command(Command::CancelPsiSelection)),
            (252., 0x400fae, Action::KeepMedicalTarget),
        ] {
            button(
                assets,
                controls,
                &assets.native_text.text(address, true).unwrap_or_default(),
                Rect::new(x, 250., 188., 32.),
                true,
                action,
            );
        }
        return;
    }
    let arena = state.combat.as_ref().unwrap();
    let targets: Vec<_> = state
        .warriors
        .iter()
        .enumerate()
        .map(|(index, w)| (index as i16, w.name.clone(), arena.squad.positions[index]))
        .chain(arena.enemies.records.iter().enumerate().map(|(index, m)| {
            (
                index as i16 + 10,
                centurions_content::ui::c_string(&m.bytes[218..]),
                arena.enemies.positions[index],
            )
        }))
        .collect();
    let selected = pending
        .targets
        .iter()
        .filter_map(|body| targets.iter().find(|(id, _, _)| id == body))
        .map(|(_, name, _)| name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    assets.text(&selected, 48., 145., 14, YELLOW);
    for (row, (_, name, position)) in targets.iter().skip(ui.page * 8).take(8).enumerate() {
        button(
            assets,
            controls,
            name,
            Rect::new(48., 181. + row as f32 * 35., 674., 30.),
            centurions_core::combat::psi_selection::admit_target(state, content, *position).is_ok(),
            Action::Command(Command::SelectPsiTarget {
                x: position.0,
                y: position.1,
            }),
        );
    }
    button(
        assets,
        controls,
        &assets.upgrade_dialog[0].text(),
        Rect::new(48., 485., 110., 26.),
        !pending.targets.is_empty(),
        Action::Command(Command::FirePsiSelection),
    );
    button(
        assets,
        controls,
        &assets.native_text.text(0x4001ec, true).unwrap_or_default(),
        Rect::new(166., 485., 110., 26.),
        true,
        Action::MedicalCancel,
    );
    for (x, change, enabled, glyph) in [
        (530., -1, ui.page > 0, "<"),
        (648., 1, (ui.page + 1) * 8 < targets.len(), ">"),
    ] {
        button(
            assets,
            controls,
            glyph,
            Rect::new(x, 485., 74., 26.),
            enabled,
            Action::Page(change),
        );
    }
}

fn draw_medical_target(
    content: &Content,
    state: &Session,
    assets: &Assets,
    ui: &UiState,
    controls: &mut Vec<Control>,
) {
    let pending = state.medical_continuation.as_ref().unwrap();
    let learned = pending.origin == centurions_core::MedicalOrigin::LearnedHeal;
    assets.text(
        if learned { "Heal Mk I" } else { "StimNeedle" },
        48.,
        70.,
        18,
        YELLOW,
    );
    if ui.medical_cancel {
        button(
            assets,
            controls,
            "Yes — cancel",
            Rect::new(48., 250., 188., 32.),
            true,
            Action::Command(Command::SelectMedicalTarget {
                selection: MedicalTarget::ConfirmedCancel,
            }),
        );
        button(
            assets,
            controls,
            "Keep choosing",
            Rect::new(252., 250., 188., 32.),
            true,
            Action::KeepMedicalTarget,
        );
        return;
    }
    let arena = state.combat.as_ref().unwrap();
    let mission = content
        .map(state.exploration.as_ref().unwrap().map)
        .unwrap();
    for (index, warrior) in state.warriors.iter().enumerate() {
        let hp = warrior.stamina().unwrap_or(-10);
        let y = 180. + index as f32 * 44.;
        assets.text(
            &format!(
                "{}   {}/{}",
                warrior.name,
                hp,
                word(&warrior.bytes, 0x52).unwrap_or(0)
            ),
            48.,
            y + 22.,
            14,
            WHITE,
        );
        let (selection, allowed) = if hp <= 0 {
            (MedicalTarget::Portrait { warrior: index }, hp > -10)
        } else {
            let (x, y) = arena.squad.positions[index];
            let from = arena.squad.positions[pending.actor];
            let allowed = !learned
                && word(&state.warriors[pending.actor].bytes, 0x40).unwrap_or(0) >= 2
                && centurions_core::combat::player_psi::in_range(from, (x, y), 4)
                && centurions_core::combat::lifecycle::can_see(arena, mission, from, (x, y), 0)
                    .unwrap_or(false)
                && word(&warrior.bytes, 0x9a).unwrap_or(1) == 0;
            (MedicalTarget::LivingPoint { x, y }, allowed)
        };
        button(
            assets,
            controls,
            "Use",
            Rect::new(596., y, 132., 28.),
            allowed,
            Action::Command(Command::SelectMedicalTarget { selection }),
        );
    }
    button(
        assets,
        controls,
        "Cancel [Esc]",
        Rect::new(540., 510., 188., 26.),
        true,
        Action::MedicalCancel,
    );
}

fn draw_psi(
    content: &Content,
    state: &Session,
    assets: &Assets,
    ui: &UiState,
    controls: &mut Vec<Control>,
    selected: usize,
) {
    let warrior = &state.warriors[selected];
    let power = ui.psi_power.clamp(1, 7);
    let psi = warrior.psi_points().unwrap_or(0);
    let combat = state.combat.as_ref();
    let Some(mission) = state.exploration.as_ref().and_then(|p| content.map(p.map)) else {
        return;
    };
    assets.text(&format!("Psi {psi}   Power {power}"), 48., 116., 14, WHITE);
    if ui.preferences.enabled(11)
        && let Some((spell, _)) = ui.psi_choice
    {
        let school = spell / 1000;
        let tier = (spell % 1000 / 100).saturating_sub(1) as usize;
        let column = (spell % 100).saturating_sub(1) as usize;
        if let Ok(text) = mission.scan_description(school, tier, column) {
            assets.wrapped_color(&text, Rect::new(48., 470., 674., 78.), 12, 14., YELLOW);
        }
    }
    if ui.inspection == Some(Inspection::Psi) {
        for value in 1..=7 {
            let x = 48. + (value - 1) as f32 * 96.;
            button(
                assets,
                controls,
                &format!("Power {value}"),
                Rect::new(x, 136., 90., 25.),
                value <= 7,
                Action::PsiPower(value),
            );
        }
        let mut y = 182.;
        let encounter = centurions_core::typed_record(state, content).is_ok();
        let mut choices: Vec<_> = [(1101, false, false), (1101, false, true)]
            .into_iter()
            .chain(
                centurions_core::combat::player_psi::PARTY_SPELLS
                    .iter()
                    .copied()
                    .map(|id| (id, true, false)),
            )
            .chain(
                centurions_core::combat::player_psi::OUTDOOR_TARGET_SPELLS
                    .iter()
                    .copied()
                    .map(|id| (id, true, false)),
            )
            .chain(
                centurions_core::combat::player_psi::OUTDOOR_PARTY_SPELLS
                    .iter()
                    .copied()
                    .map(|id| (id, true, false)),
            )
            .chain(
                centurions_core::combat::player_psi::COMBAT_SPELLS
                    .iter()
                    .copied()
                    .filter(|id| *id != 1101)
                    .map(|id| (id, false, false)),
            )
            .filter(|(spell, outdoors, _)| {
                warrior.knows_spell(*spell).unwrap_or(false) && *outdoors == combat.is_none()
            })
            .collect();
        if encounter {
            let school = centurions_content::word(&warrior.bytes, 0x36).unwrap_or(0);
            choices = (1..=7)
                .flat_map(|row| (1..=7).map(move |column| school * 1000 + row * 100 + column))
                .filter(|spell| warrior.knows_spell(*spell).unwrap_or(false))
                .map(|spell| (spell, true, false))
                .collect();
        }
        for (spell, outdoors, portrait) in choices.iter().copied().skip(ui.page * 7).take(7) {
            let packet = mission
                .combat
                .as_ref()
                .and_then(|t| centurions_core::combat::spells::load_packet(&t.attacks, spell).ok());
            let cost = packet
                .as_ref()
                .and_then(|p| centurions_core::combat::player_psi::cost(p, power).ok())
                .unwrap_or(i16::MAX);
            let global = packet
                .as_ref()
                .is_some_and(|packet| packet[1] == 7 || packet[1] >= 9);
            let multi = packet.as_ref().is_some_and(|packet| packet[1] == 0);
            let action = if encounter {
                Action::Command(Command::CastEncounterPsi {
                    warrior: selected,
                    spell,
                    power,
                })
            } else if outdoors {
                Action::Command(Command::CastPartyPsi {
                    warrior: selected,
                    spell,
                    power,
                })
            } else if multi {
                Action::Command(Command::BeginPsiSelection { spell, power })
            } else if portrait {
                Action::Command(Command::BeginLearnedHeal { power })
            } else if global || spell == 2302 {
                Action::Command(Command::CastPsi {
                    rotation: ui.psi_rotation,
                    spell,
                    power,
                    target: None,
                })
            } else {
                Action::ChoosePsi(spell)
            };
            let enabled = if encounter {
                centurions_core::typed_encounter::validate_spell(
                    state, content, selected, spell, power,
                )
                .is_ok()
            } else {
                psi >= cost
                    && packet.as_ref().is_some_and(|p| {
                        centurions_core::combat::player_psi::mode_allowed(p, combat.is_some())
                            .unwrap_or(false)
                    })
                    && if let Some(combat) = combat {
                        centurions_core::combat::player_psi::can_cast(
                            &state.warriors,
                            selected as i16,
                            state.psi_disabled,
                        )
                        .unwrap_or(false)
                            && combat.active_body() == Some(selected as i16)
                    } else {
                        centurions_core::combat::player_psi::validate_party(
                            &state.warriors,
                            mission,
                            state.psi_disabled,
                            centurions_core::combat::player_psi::PartyCastRequest {
                                warrior: selected,
                                spell,
                                power,
                            },
                        )
                        .is_ok()
                    }
            };
            button(
                assets,
                controls,
                assets.spell_names.get(&spell).map_or("", String::as_str),
                Rect::new(48., y, 674., 32.),
                enabled,
                action,
            );
            y += 38.;
        }
        button(
            assets,
            controls,
            "Previous",
            Rect::new(48., 485., 110., 26.),
            ui.page > 0,
            Action::Page(-1),
        );
        button(
            assets,
            controls,
            "Next",
            Rect::new(166., 485., 110., 26.),
            (ui.page + 1) * 7 < choices.len(),
            Action::Page(1),
        );
    } else if let (Some(combat), Some((spell, power))) = (combat, ui.psi_choice) {
        if spell != 1101 {
            button(
                assets,
                controls,
                "Map",
                Rect::new(48., 136., 330., 28.),
                true,
                Action::PsiMapTarget,
            );
        }
        let mut targets = Vec::new();
        {
            for (body, warrior) in state.warriors.iter().enumerate() {
                if warrior.stamina().unwrap_or(-10) > -10 {
                    targets.push((
                        warrior.name.clone(),
                        warrior.stamina().unwrap_or(0),
                        combat.squad.positions[body],
                    ));
                }
            }
        }
        {
            for (index, monster) in combat.enemies.records.iter().enumerate() {
                if monster.stamina().unwrap_or(0) > 0 {
                    targets.push((
                        format!("Enemy {}", index + 1),
                        monster.stamina().unwrap_or(0),
                        combat.enemies.positions[index],
                    ));
                }
            }
        }
        for (row, (name, hp, position)) in targets.iter().skip(ui.page * 8).take(8).enumerate() {
            let enabled = centurions_core::combat::player_psi::validate(
                combat,
                &state.warriors,
                mission,
                state.psi_disabled,
                centurions_core::combat::player_psi::CastRequest {
                    rotation: ui.psi_rotation,
                    spell,
                    power,
                    target: Some(*position),
                },
            )
            .is_ok();
            button(
                assets,
                controls,
                &format!("{name}: HP {hp}   ({}, {})", position.0, position.1),
                Rect::new(48., 181. + row as f32 * 35., 674., 30.),
                enabled,
                Action::Command(Command::CastPsi {
                    rotation: ui.psi_rotation,
                    spell,
                    power,
                    target: Some(*position),
                }),
            );
        }
        button(
            assets,
            controls,
            "Previous",
            Rect::new(48., 485., 110., 26.),
            ui.page > 0,
            Action::Page(-1),
        );
        button(
            assets,
            controls,
            "Next",
            Rect::new(166., 485., 110., 26.),
            (ui.page + 1) * 8 < targets.len(),
            Action::Page(1),
        );
    }
}

fn draw_combat_information(content: &Content, state: &Session, assets: &Assets, selected: usize) {
    let w = &state.warriors[selected];
    assets.panel(166, Rect::new(0., 449., 308., 139.));
    assets.panel(228, Rect::new(310., 449., 160., 140.));
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
fn draw_conditions(state: &Session, assets: &Assets, ui: &UiState) {
    let w = &state.warriors[ui.selected];
    let mut y = 146.;
    for (i, offset) in (0x5e..0xae).step_by(2).enumerate() {
        let value = word(&w.bytes, offset).unwrap_or(0);
        if value == 0 {
            continue;
        }
        let label = assets
            .conditions
            .get(i)
            .filter(|s| !s.is_empty())
            .cloned()
            .unwrap_or_else(|| format!("Condition {}", i + 1));
        assets.text(&format!("{label}: {value}"), 48., y, 14, WHITE);
        y += 20.;
    }
    if w.bytes[0x9de] != 0 {
        assets.text("Bleeding", 48., y, 14, WHITE);
    }
    assets.text(
        &format!(
            "Stamina {}/{}  Psi {}/{}",
            field(w, 0x50),
            field(w, 0x52),
            field(w, 0x58),
            field(w, 0x5a)
        ),
        48.,
        524.,
        14,
        WHITE,
    );
}
fn draw_modifiers(state: &Session, assets: &Assets, ui: &UiState) {
    let labels = assets
        .character_strings
        .get(&132)
        .and_then(|bytes| centurions_content::ui::string_list(bytes).ok())
        .unwrap_or_default();
    let warrior = &state.warriors[ui.selected];
    let mut y = 146.;
    for (index, label) in labels.iter().take(12).enumerate() {
        let value = word(&warrior.bytes, 0xae + index * 2).unwrap_or(0);
        if value != 0 && !label.is_empty() {
            assets.text(&format!("{value:+} {label}"), 48., y, 14, WHITE);
            y += 20.;
        }
    }
}

fn draw_specials(state: &Session, assets: &Assets, ui: &UiState) {
    let warrior = &state.warriors[ui.selected];
    let mut y = 146.;
    if let Some(labels) = assets.reference_strings.get(&1201) {
        for (index, label) in labels.iter().take(14).enumerate() {
            let value = word(&warrior.bytes, 0xf4 + index * 2).unwrap_or(0);
            if value > 0 && !label.is_empty() {
                let value =
                    value.wrapping_add(word(&warrior.bytes, 0xb62 + index * 2).unwrap_or(0));
                assets.text(&format!("{label} {value}"), 48., y, 14, WHITE);
                y += 20.;
            }
        }
    }
}

fn draw_items(
    content: &Content,
    state: &Session,
    assets: &Assets,
    ui: &UiState,
    controls: &mut Vec<Control>,
) {
    let loot = state
        .exploration
        .as_ref()
        .is_some_and(|p| matches!(p.presentation, Some(Presentation::Loot)));
    let crate_view = ui.inspection == Some(Inspection::Crate);
    let items = if crate_view {
        state
            .crate_items
            .iter()
            .enumerate()
            .filter(|(_, i)| i.id() != 0)
            .map(|(n, i)| (n, i.clone()))
            .collect::<Vec<_>>()
    } else if loot {
        state
            .booty_items
            .iter()
            .enumerate()
            .filter(|(_, i)| i.id() != 0)
            .map(|(n, i)| (n, i.clone()))
            .collect::<Vec<_>>()
    } else {
        state.warriors[ui.selected]
            .inventory()
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .collect()
    };
    if !loot {
        button(
            assets,
            controls,
            "Carried",
            Rect::new(48., 116., 120., 24.),
            true,
            Action::Open(Inspection::Items),
        );
        button(
            assets,
            controls,
            "Crate",
            Rect::new(180., 116., 120., 24.),
            state.phase == Phase::Exploration,
            Action::Open(Inspection::Crate),
        );
    }
    for (row, (slot, item)) in items.iter().skip(ui.page * 8).take(8).enumerate() {
        let y = 159. + row as f32 * 39.;
        if !loot && !crate_view && state.phase == Phase::Exploration {
            let rect = Rect::new(412., y + 3., 28., 26.);
            assets.panel(7, rect);
            controls.push(Control {
                rect,
                enabled: true,
                action: Action::Drop(ui.selected, *slot),
            });
        }
        if let Some(def) = content.items.get(item.id().unsigned_abs() as usize)
            && let Ok(icon) = word(def, 10)
            && let Some(texture) = assets.icons.get(&icon)
        {
            draw_texture_ex(
                texture,
                48.,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(32., 32.)),
                    ..Default::default()
                },
            );
        }
        assets.text(&assets.name(item.id()), 88., y + 17., 14, WHITE);
        assets.text(
            &format!(
                "Qty {}{}",
                word(&item.0, 2).unwrap_or(0),
                if item.0[9] == 1 { "  equipped" } else { "" }
            ),
            344.,
            y + 17.,
            12,
            YELLOW,
        );
        if crate_view {
            button(
                assets,
                controls,
                "Take",
                Rect::new(620., y + 3., 108., 26.),
                (matches!(
                    item.id(),
                    38 | 165 | 400 | 403 | 420 | 440 | 460 | 600 | 637 | 700
                ) || item.0
                    == [
                        0, 2, 0, 0, 0, 0, 0, 12, 0, 0, 15, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
                    ])
                    && state.phase == Phase::Exploration,
                Action::Command(Command::TakeCrate {
                    slot: *slot,
                    warrior: ui.selected,
                }),
            );
        } else if loot {
            let ammo = (200..400).contains(&item.id()) && item.0[10] == 26;
            button(
                assets,
                controls,
                if ammo { "Collect ammo" } else { "Take" },
                Rect::new(518., y + 3., 102., 26.),
                true,
                Action::Command(Command::TakeLoot {
                    slot: *slot,
                    warrior: ui.selected,
                }),
            );
            button(
                assets,
                controls,
                "Store",
                Rect::new(636., y + 3., 92., 26.),
                true,
                Action::Command(Command::StoreLoot { slot: *slot }),
            );
        } else if matches!(item.0[10], 1..=20) {
            let equipped = item.0[9] != 0;
            let eligible = state.phase == Phase::Exploration
                || state.combat.as_ref().and_then(|c| c.active_body()) == Some(ui.selected as i16);
            if matches!(item.0[10], 2 | 15) && equipped {
                let ammunition = centurions_core::ammunition::compatible_ammunition(content, item)
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|id| state.ammunition[(*id - 200) as usize] > 0)
                    .min_by_key(|id| {
                        if *id == word(&item.0, 4).unwrap_or(0) {
                            0
                        } else {
                            *id
                        }
                    })
                    .unwrap_or(200);
                let reserve = (ammunition - 200) as usize;
                let loaded = word(&item.0, 2).unwrap_or(-1);
                let selected = word(&item.0, 4).unwrap_or(-1);
                let native_instance =
                    loaded >= 0 && (selected == 0 || (200..400).contains(&selected));
                button(
                    assets,
                    controls,
                    "Fit ammo",
                    Rect::new(504., y + 3., 108., 26.),
                    eligible && native_instance && state.ammunition[reserve] > 0,
                    Action::Command(Command::FitAmmunition {
                        warrior: ui.selected,
                        slot: *slot,
                        ammunition,
                    }),
                );
            }
            if item.0[10] == 20 && equipped {
                let ammunition = centurions_core::ammunition::compatible_ammunition(content, item)
                    .unwrap_or_default()
                    .into_iter()
                    .find(|id| state.ammunition[(*id - 200) as usize] > 0)
                    .unwrap_or(307);
                button(
                    assets,
                    controls,
                    "Charge",
                    Rect::new(504., y + 3., 108., 26.),
                    eligible
                        && state.phase == Phase::Exploration
                        && word(&item.0, 2).unwrap_or(-1) < word(&item.0, 6).unwrap_or(0)
                        && state.ammunition[(ammunition - 200) as usize] > 0,
                    Action::Command(Command::FitAmmunition {
                        warrior: ui.selected,
                        slot: *slot,
                        ammunition,
                    }),
                );
            }
            button(
                assets,
                controls,
                if equipped { "Remove" } else { "Equip" },
                Rect::new(620., y + 3., 108., 26.),
                eligible,
                Action::Command(if equipped {
                    Command::UnequipItem {
                        warrior: ui.selected,
                        slot: *slot,
                    }
                } else {
                    Command::EquipItem {
                        warrior: ui.selected,
                        slot: *slot,
                    }
                }),
            );
        } else {
            let scripted = item.0[10] == 23;
            let medical = (700..=703).contains(&item.id());
            let medical_available = medical
                && matches!(item.0[10], 24 | 28)
                && word(&item.0, 2).unwrap_or(0) > 0
                && (state.phase == Phase::Exploration
                    || (state.combat.as_ref().and_then(|c| c.active_body())
                        == Some(ui.selected as i16)
                        && word(&state.warriors[ui.selected].bytes, 0x40).unwrap_or(0) >= 1));
            let available = (200..400).contains(&item.id())
                && item.0[10] == 26
                && item.0[9] != 1
                && word(&item.0, 2).unwrap_or(0) > 0
                && state.phase == Phase::Exploration
                && state
                    .exploration
                    .as_ref()
                    .is_some_and(|p| p.presentation.is_none());
            button(
                assets,
                controls,
                if scripted || medical {
                    "Use"
                } else {
                    "Use pack"
                },
                Rect::new(620., y + 3., 108., 26.),
                medical_available
                    || available
                    || (scripted
                        && state.phase == Phase::Exploration
                        && state
                            .exploration
                            .as_ref()
                            .is_some_and(|p| p.presentation.is_none() && p.script.is_none())),
                Action::Command(if medical || scripted {
                    Command::UseItem {
                        warrior: ui.selected,
                        slot: *slot,
                    }
                } else {
                    Command::UseAmmunitionPack {
                        warrior: ui.selected,
                        slot: *slot,
                    }
                }),
            );
        }
    }
    let reserve = state
        .ammunition
        .iter()
        .enumerate()
        .filter(|(i, v)| *i != 0 && **v > 0)
        .map(|(i, v)| format!("{}: {v}", assets.name(200 + i as i16)))
        .collect::<Vec<_>>()
        .join("   ");
    assets.wrapped(&reserve, Rect::new(48., 520., 500., 28.), 12, 15.);
    button(
        assets,
        controls,
        "Previous",
        Rect::new(555., 526., 82., 24.),
        ui.page > 0,
        Action::Page(-1),
    );
    button(
        assets,
        controls,
        "Next",
        Rect::new(650., 526., 78., 24.),
        (ui.page + 1) * 8 < items.len(),
        Action::Page(1),
    );
}

fn draw_fire_targets(
    content: &Content,
    state: &Session,
    assets: &Assets,
    ui: &UiState,
    controls: &mut Vec<Control>,
) {
    let Some(combat) = state.combat.as_ref() else {
        return;
    };
    let Some(body) = combat.active_body().filter(|body| (0..6).contains(body)) else {
        return;
    };
    let warrior = &state.warriors[body as usize];
    let targets: Vec<_> = state
        .warriors
        .iter()
        .enumerate()
        .filter(|(_, w)| w.bytes[0x9df] != 0 && w.stamina().unwrap_or(0) > 0)
        .map(|(index, w)| (index as i16, w.name.clone()))
        .chain(
            combat
                .enemies
                .records
                .iter()
                .enumerate()
                .filter(|(_, m)| m.stamina().unwrap_or(0) > 0 && m.bytes[0xb0] != 255)
                .map(|(index, m)| {
                    (
                        index as i16 + 10,
                        centurions_content::ui::c_string(&m.bytes[218..]),
                    )
                }),
        )
        .collect();
    assets.text(&warrior.name, 48., 142., 14, YELLOW);
    if centurions_core::combat::shooting::location_targeting(state, content) {
        button(
            assets,
            controls,
            "Target",
            Rect::new(530., 485., 110., 26.),
            true,
            Action::FireMapTarget,
        );
    }
    if let Some(selection) = &combat.firearm_selection {
        let names = selection
            .targets
            .iter()
            .map(|body| {
                targets
                    .iter()
                    .find(|(id, _)| id == body)
                    .map_or("", |(_, name)| name.as_str())
            })
            .collect::<Vec<_>>()
            .join(", ");
        assets.text(&names, 48., 164., 12, YELLOW);
        button(
            assets,
            controls,
            "Target",
            Rect::new(530., 485., 110., 26.),
            true,
            Action::Command(Command::FireSelectedTargets),
        );
    }
    for (row, (target, name)) in targets.iter().skip(ui.page * 8).take(8).enumerate() {
        button(
            assets,
            controls,
            name,
            Rect::new(48., 181. + row as f32 * 35., 674., 30.),
            true,
            Action::Command(Command::ShootEnemy { body: *target }),
        );
    }
    button(
        assets,
        controls,
        "Previous",
        Rect::new(48., 485., 110., 26.),
        ui.page > 0,
        Action::Page(-1),
    );
    button(
        assets,
        controls,
        "Next",
        Rect::new(166., 485., 110., 26.),
        (ui.page + 1) * 8 < targets.len(),
        Action::Page(1),
    );
}
