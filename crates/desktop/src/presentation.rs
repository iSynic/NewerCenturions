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
    Psi,
    PsiTargets,
    Bandage,
}
#[derive(Default)]
pub struct UiState {
    pub movement: crate::input::MovementInput,
    pub selected: usize,
    pub inspection: Option<Inspection>,
    pub page: usize,
    pub messages: VecDeque<String>,
    pub typed_text: String,
    pub effects: VecDeque<OutdoorAnimation>,
    pub delay_until: f64,
    pub psi_power: i16,
    pub psi_choice: Option<(i16, i16)>,
    pub psi_pending: Option<(i16, i16)>,
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
    Application(crate::application::ApplicationAction),
    Command(Command),
    Save,
    Load,
    Select(usize),
    Open(Inspection),
    Close,
    Page(i32),
    ChoosePsi(i16),
    PsiPower(i16),
    PsiMapTarget,
    MedicalCancel,
    KeepMedicalTarget,
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

pub struct Control {
    pub rect: Rect,
    pub action: Action,
    pub enabled: bool,
}

impl UiState {
    pub fn message(&mut self, text: String) {
        if !text.is_empty() {
            self.messages.push_back(text);
            while self.messages.len() > 4 {
                self.messages.pop_front();
            }
        }
    }
    pub fn events(&mut self, events: &[Event], state: &Session) {
        for event in events {
            if let Event::Delay { ticks } = event {
                self.delay_until = self.delay_until.max(get_time()) + f64::from(*ticks) / 60.;
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
                    frame_seconds: f64::from((*delay).max(1)) / 60.,
                });
            }
            let text = match event {
                Event::AttackResolved {
                    attacker,
                    target,
                    damage,
                    defeated,
                } => {
                    let name = |body: i16| {
                        if (0..6).contains(&body) {
                            state.warriors[body as usize].name.clone()
                        } else {
                            format!("Enemy {}", body - 9)
                        }
                    };
                    format!(
                        "{} → {}: {}{}",
                        name(*attacker),
                        name(*target),
                        if *damage == 0 {
                            "miss".into()
                        } else {
                            format!("{damage} damage")
                        },
                        if *defeated { ", defeated" } else { "" }
                    )
                }
                Event::SpellResolved {
                    target,
                    damage,
                    defeated,
                    ..
                } => {
                    let name = if (0..6).contains(target) {
                        state.warriors[*target as usize].name.clone()
                    } else {
                        "A combatant".into()
                    };
                    if *damage == 0 {
                        format!("Psi affects {name}.")
                    } else {
                        format!(
                            "{name} takes {damage} psi damage{}.",
                            if *defeated { " and is defeated" } else { "" }
                        )
                    }
                }
                Event::CombatantAdded { .. } => "A combatant joins the battle.".into(),
                Event::CombatantRemoved { .. } => "A combatant leaves the battle.".into(),
                Event::AllyAdded { .. } => "An ally joins the squad.".into(),
                Event::LevelAdvanced { warrior, level } => format!(
                    "{} advances to level {level}.",
                    state.warriors[*warrior].name
                ),
                Event::CollectionRejected { warning } => {
                    format!("The action could not be completed (message {warning}).")
                }
                Event::BattleEnded { victory: true } => {
                    "Victory. The squad returns to exploration.".into()
                }
                Event::AmmunitionAdded {
                    ammunition,
                    quantity,
                    reserve,
                } => format!("Ammo {ammunition}: +{quantity} rounds. Reserve {reserve}."),
                Event::TimeAdvanced { day, hour, minute } if *hour == 0 || *hour == 12 => {
                    format!("Day {day}, {hour:02}:{minute:02}. Squad recovery completed.")
                }
                _ => String::new(),
            };
            self.message(text);
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
        .unwrap_or(((position.x - 7).clamp(0, 75), (position.y - 6).clamp(0, 77)));
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
    let (vx, vy) = ((position.x - 7).clamp(0, 75), (position.y - 6).clamp(0, 77));
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
    assets.panel(197, Rect::new(480., 429., 320., 19.));
    assets.panel(220, layout::MESSAGE);
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
        assets.text(&w.name, rect.x + 87., rect.y + 21., 12, YELLOW);
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
        (layout::ITEMS, layout::CONDITIONS)
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
            .is_some_and(|b| (0..6).contains(&b));
        let unobstructed = state
            .exploration
            .as_ref()
            .is_some_and(|p| p.presentation.is_none())
            && (state.phase == Phase::Exploration || player);
        let firearm = state.warriors[selected].bytes[0x9dd] != 0;
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
            crate::input::refill_action(state, selected),
        );
        button(
            assets,
            &mut controls,
            "End [Space]",
            layout::END_TURN,
            player,
            Action::Command(Command::EndTurn),
        );
        button(
            assets,
            &mut controls,
            "Psi [P]",
            Rect::new(486., 516., 126., 26.),
            unobstructed && (state.phase == Phase::Exploration || player),
            Action::Open(Inspection::Psi),
        );
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
                Rect::new(624., 516., 126., 26.),
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
        } else {
            assets.text("Unavailable", 708., 561., 12, GRAY);
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
                Presentation::Question { text, .. } => {
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
                Presentation::WarriorChoice { .. } => {
                    assets.wrapped(
                        "Choose a living warrior from the squad [1-6].",
                        Rect::new(12., 467., 444., 82.),
                        12,
                        16.,
                    );
                }
                Presentation::StageChange { .. } => {
                    assets.wrapped(
                        "Proceed to the next mission?",
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
                }
                Presentation::Loot => {
                    assets.text(
                        "You find equipment! Open Items to collect or store it.",
                        12.,
                        477.,
                        12,
                        YELLOW,
                    );
                }
                Presentation::Survivors => {
                    assets.text("Surviving allies", 12., 250., 16, WHITE);
                    assets.text(
                        "Continue to keep the surviving allies with the squad.",
                        12.,
                        280.,
                        12,
                        YELLOW,
                    );
                }
            }
            if !matches!(
                p,
                Presentation::Question { .. }
                    | Presentation::Menu { .. }
                    | Presentation::Typed { .. }
                    | Presentation::StageChange { .. }
                    | Presentation::WarriorChoice { .. }
            ) {
                let keep_spells = matches!(p, Presentation::Loot)
                    && centurions_core::needs_spell_confirmation(state);
                button(
                    assets,
                    &mut controls,
                    if keep_spells {
                        "Keep psi selections"
                    } else {
                        "Continue"
                    },
                    layout::CONTINUE,
                    !state.booty_items.iter().any(|i| i.id() != 0),
                    Action::Command(if keep_spells {
                        Command::KeepCurrentSpells
                    } else {
                        Command::Continue
                    }),
                );
            }
        } else if state.phase == Phase::Combat {
            assets.wrapped(
                &ui.messages.iter().cloned().collect::<Vec<_>>().join("\n"),
                Rect::new(490., 462., 304., 68.),
                12,
                16.,
            );
        } else {
            assets.wrapped(
                &ui.messages.iter().cloned().collect::<Vec<_>>().join("\n"),
                Rect::new(12., 467., 444., 99.),
                12,
                17.,
            );
            assets.text(
                "Arrows: move   Click enemy: attack   F / R / Space",
                12.,
                578.,
                12,
                YELLOW,
            );
        }
    }
    if let Some(inspection) = ui.inspection {
        controls.clear();
        draw_rectangle(0., 20., 800., 580., Color::from_rgba(0, 0, 0, 160));
        assets.panel(224, layout::INSPECT);
        let title = match inspection {
            Inspection::Items | Inspection::Crate => "Items",
            Inspection::FireTargets => "Firearm targets",
            Inspection::Conditions => "Conditions",
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
            6
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
    if let Some((spell, _)) = ui.psi_pending {
        draw_rectangle(0., 449., 470., 139., Color::from_rgba(12, 18, 28, 245));
        let prompt = match spell {
            2101 => {
                "Crystalize: click a map location. Its five-cell cross damages every body it reaches and leaves a lingering field. Esc cancels."
            }
            1305 => {
                "Electric Shield: click a map location. Eligible squad members in its area receive electric protection. Esc cancels."
            }
            _ => {
                "Energy Storm: click a map location to cast. The storm affects everyone in its area. Esc cancels."
            }
        };
        assets.wrapped(prompt, Rect::new(12., 465., 444., 90.), 14, 20.);
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
    controls
}

fn draw_bandage(state: &Session, assets: &Assets, controls: &mut Vec<Control>) {
    assets.wrapped(
        "Choose a warrior to stop bleeding. This ends the current turn.",
        Rect::new(48., 124., 650., 48.),
        14,
        20.,
    );
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
        if learned {
            "Heal Mk I — choose a portrait"
        } else {
            "StimNeedle — choose a target"
        },
        48.,
        70.,
        18,
        YELLOW,
    );
    if ui.medical_cancel {
        assets.wrapped(
            if learned {
                "Cancel this cast? Some psi is refunded; canceling costs 3 movement."
            } else {
                "Cancel this use? The needle has already been consumed."
            },
            Rect::new(48., 148., 650., 70.),
            16,
            23.,
        );
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
    assets.wrapped(if learned {
        "Choose an unconscious warrior. Revived warriors return to formation. Heal cannot help PDP-11. Psi has been charged; this action uses the caster's turn."
    } else {
        "Choose an organic warrior. Unconscious warriors return to formation if revived. Canceling still consumes the needle."
    },
        Rect::new(48., 110., 650., 56.), 14, 20.);
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
    let power = ui.psi_power.clamp(1, 4);
    let psi = warrior.psi_points().unwrap_or(0);
    let combat = state.combat.as_ref();
    let Some(mission) = state.exploration.as_ref().and_then(|p| content.map(p.map)) else {
        return;
    };
    assets.text(&format!("Psi {psi}   Power {power}"), 48., 116., 14, WHITE);
    if ui.inspection == Some(Inspection::Psi) {
        for (value, x) in [(1, 48.), (2, 166.), (3, 284.), (4, 402.)] {
            button(
                assets,
                controls,
                &format!("Power {value}"),
                Rect::new(x, 136., 110., 25.),
                value <= 2
                    || (combat.is_some()
                        && warrior.knows_spell(1101).unwrap_or(false)
                        && psi >= 6 * value),
                Action::PsiPower(value),
            );
        }
        let mut y = 182.;
        for (spell, label, cost, outdoors, portrait) in [
            (1101, "Heal Mk I: choose a living body", 6, false, false),
            (
                1101,
                "Heal Mk I: choose an unconscious warrior",
                6,
                false,
                true,
            ),
            (
                1103,
                "Inertial Shield Mk I: party protection",
                15,
                true,
                false,
            ),
            (
                1305,
                "Electric Shield Mk II: choose a location",
                8,
                false,
                false,
            ),
            (2101, "Crystalize Mk I: lingering cross", 8, false, false),
            (
                2204,
                "Energy Storm Mk II: choose a location",
                25,
                false,
                false,
            ),
            (
                2302,
                "Cosmic Blast Mk I: all hostile bodies",
                25,
                false,
                false,
            ),
        ] {
            if !warrior.knows_spell(spell).unwrap_or(false) || outdoors != combat.is_none() {
                continue;
            }
            let action = if outdoors {
                Action::Command(Command::CastPartyPsi {
                    warrior: selected,
                    spell,
                    power,
                })
            } else if portrait {
                Action::Command(Command::BeginLearnedHeal { power })
            } else if spell == 2302 {
                Action::Command(Command::CastPsi {
                    spell,
                    power,
                    target: None,
                })
            } else {
                Action::ChoosePsi(spell)
            };
            let enabled = (portrait || power <= 2)
                && psi >= cost * power
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
                };
            button(
                assets,
                controls,
                &format!("{label}   {} psi", cost * power),
                Rect::new(48., y, 674., 32.),
                enabled,
                action,
            );
            y += 38.;
        }
        if y == 182. {
            assets.text("No available learned psi spells.", 48., y, 14, WHITE);
        }
        let explanation = if combat.is_some() {
            "Point casts cost 2 AP and 3 movement. Living Heal reaches one tile. Unconscious Heal supports powers 1–4 and uses the caster's turn. PDP-11 is immune. Electric Shield reaches eight tiles and requires sight."
        } else {
            "Inertial Shield protects the party from melee attacks. Its duration decreases each hour outdoors and each combat round. Select a squad member, then cast a learned spell."
        };
        assets.wrapped(
            explanation,
            Rect::new(48., (y + 12.).max(360.), 674., 78.),
            14,
            20.,
        );
    } else if let (Some(combat), Some((spell, power))) = (combat, ui.psi_choice) {
        if spell != 1101 {
            button(
                assets,
                controls,
                "Choose a location on the map",
                Rect::new(48., 136., 330., 28.),
                true,
                Action::PsiMapTarget,
            );
        }
        let mut targets = Vec::new();
        if matches!(spell, 1101 | 1305) {
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
        if spell != 1101 {
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
        assets.text(
            if spell == 1101 {
                "Choose a body within one tile. Esc cancels."
            } else {
                "Area psi affects every body in its area. Esc cancels."
            },
            48.,
            542.,
            14,
            WHITE,
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
        y += 20.;
    }
    if y == 146. {
        assets.text("No active conditions.", 48., y, 14, WHITE);
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
    assets.text(
        if crate_view {
            "Stored equipment — recipient selected above"
        } else if loot {
            "Found equipment — recipient selected above"
        } else {
            "Carried equipment"
        },
        48.,
        142.,
        14,
        WHITE,
    );
    for (row, (slot, item)) in items.iter().skip(ui.page * 8).take(8).enumerate() {
        let y = 159. + row as f32 * 39.;
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
                ammo || item.0[10] == 23 || (item.id() == 700 && item.0[10] == 28),
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
        } else if matches!(
            item.id(),
            1 | 2 | 38 | 165 | 400 | 403 | 420 | 440 | 460 | 600 | 637
        ) {
            let equipped = item.0[9] != 0;
            let eligible = state.phase == Phase::Exploration
                || state.combat.as_ref().and_then(|c| c.active_body()) == Some(ui.selected as i16);
            if matches!(item.id(), 1 | 2 | 38 | 165) && equipped {
                let (ammunition, reserve) = match item.id() {
                    1 => (200, 0),
                    2 => (201, 1),
                    38 => (203, 3),
                    165 => (280, 80),
                    _ => unreachable!(),
                };
                let loaded = word(&item.0, 2).unwrap_or(-1);
                let selected = word(&item.0, 4).unwrap_or(-1);
                let native_instance = loaded >= 0 && (selected == 0 || selected == ammunition);
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
            if item.id() == 637 && equipped {
                button(
                    assets,
                    controls,
                    "Charge",
                    Rect::new(504., y + 3., 108., 26.),
                    eligible
                        && state.phase == Phase::Exploration
                        && word(&item.0, 2).unwrap_or(-1) == 0
                        && word(&item.0, 4).unwrap_or(-1) == 0
                        && state.ammunition[107] > 0,
                    Action::Command(Command::FitAmmunition {
                        warrior: ui.selected,
                        slot: *slot,
                        ammunition: 307,
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
            let medical = item.id() == 700 && state.phase == Phase::Combat;
            let medical_available = medical
                && matches!(
                    (item.0[10], word(&item.0, 2).unwrap_or(0)),
                    (24, 1 | 2) | (28, 1)
                )
                && state.combat.as_ref().and_then(|c| c.active_body()) == Some(ui.selected as i16)
                && word(&state.warriors[ui.selected].bytes, 0x40).unwrap_or(0) >= 1;
            let available = item.id() == 200
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
                Action::Command(if medical {
                    Command::BeginMedicalItem {
                        warrior: ui.selected,
                        slot: *slot,
                    }
                } else if scripted {
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
    assets.text(
        &format!("Starter ammunition reserve: {} rounds", state.ammunition[0]),
        48.,
        499.,
        14,
        YELLOW,
    );
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
    _content: &Content,
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
    let targets: Vec<_> = combat
        .enemies
        .records
        .iter()
        .enumerate()
        .filter(|(_, m)| m.stamina().unwrap_or(0) > 0 && word(&m.bytes, 0xe).unwrap_or(0) != 0)
        .collect();
    assets.text(
        &format!("{} — firearm shot costs 2 AP", warrior.name),
        48.,
        142.,
        14,
        YELLOW,
    );
    let origin = combat.squad.positions[body as usize];
    for (row, (index, monster)) in targets.iter().skip(ui.page * 8).take(8).enumerate() {
        let p = combat.enemies.positions[*index];
        let distance = ((f32::from(p.0 - origin.0).powi(2) + f32::from(p.1 - origin.1).powi(2))
            .sqrt()
            .trunc()) as i16;
        button(
            assets,
            controls,
            &format!(
                "Enemy {}: HP {}   ({}, {})   Distance {distance}",
                index + 1,
                monster.stamina().unwrap_or(0),
                p.0,
                p.1
            ),
            Rect::new(48., 181. + row as f32 * 35., 674., 30.),
            true,
            Action::Command(Command::ShootEnemy {
                body: *index as i16 + 10,
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
    assets.text(
        "Choose a target. Range, ammunition and line of sight apply.",
        48.,
        542.,
        14,
        WHITE,
    );
}
