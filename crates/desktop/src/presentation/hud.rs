//! Hud presentation.
use super::{Action, Control, UiState, equipped_ammunition_reserve, field};
use crate::{assets::Assets, layout};
use centurions_core::{Command, Phase, Presentation, Session};
use macroquad::prelude::*;
pub(super) fn roster(
    state: &Session,
    assets: &Assets,
    ui: &UiState,
    controls: &mut Vec<Control>,
) -> usize {
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
        if let Ok(id) = w.record().portrait()
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
            let mos = w.record().profession().unwrap_or(0);
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
            &field(if state.phase == Phase::Combat {
                w.record().action_points()
            } else {
                w.record().actions_per_round()
            }),
            rect.x + 281.,
            rect.y + 21.,
            12,
            WHITE,
        );
        assets.text(
            &field(w.record().stamina()),
            rect.x + 99.,
            rect.y + 44.,
            12,
            SKYBLUE,
        );
        assets.text(
            &if w.record().maximum_psi().ok() == Some(0) {
                "---".into()
            } else {
                field(w.record().psi_points())
            },
            rect.x + 148.,
            rect.y + 44.,
            12,
            WHITE,
        );
        for (value, x) in [
            (w.record().shield_chance(), 208.),
            (w.record().shield_points(), 247.),
            (w.record().armor(), 294.),
        ] {
            assets.text(&field(value), rect.x + x, rect.y + 44., 12, YELLOW);
        }
        if index == selected {
            draw_rectangle_lines(rect.x, rect.y, 308., 50., 1., YELLOW);
        }
        controls.push(Control {
            rect,
            enabled: true,
            action: if state.exploration.as_ref().is_some_and(|p| {
                matches!(
                    p.presentation,
                    Some(Presentation::WarriorChoice { .. } | Presentation::PartyPick { .. })
                )
            }) {
                Action::Command(Command::ChooseWarrior { index })
            } else {
                Action::Select(index)
            },
        });
    }
    selected
}

pub(super) fn clock(state: &Session, assets: &Assets) {
    if let Some(p) = &state.exploration {
        if state.phase == Phase::Exploration {
            // xy138a40 and updatefat160a8e; PICT199 supplies the original field labels.
            let hidden = (state.native_work.coordinates_hidden != 0)
                .then(|| assets.native_text.text(0x40226a, true).unwrap_or_default());
            let coordinate = |value: i32| hidden.clone().unwrap_or_else(|| value.to_string());
            assets.text(&coordinate(p.x), 512., 444., 12, SKYBLUE);
            assets.text(&coordinate(p.y), 566., 444., 12, SKYBLUE);
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
}

pub(super) fn information(state: &Session, assets: &Assets, selected: usize) {
    if let Some(w) = state.warriors.get(selected) {
        let r = layout::INFORMATION;
        let inventory = w.inventory().unwrap_or_default();
        let weapon = w.record().equipped_weapon().unwrap_or(0);
        let loaded = inventory
            .iter()
            .find(|i| i.id() == weapon && i.equipment_flag() == 1)
            .and_then(|i| i.quantity().ok());
        assets.text(
            &format!(
                "{}  Stamina {}/{}",
                w.name,
                field(w.record().stamina()),
                field(w.record().maximum_stamina())
            ),
            r.x + 10.,
            r.y + 19.,
            12,
            WHITE,
        );
        assets.text(
            &format!(
                "Psi {}/{}   Shield {}  Armor {}",
                field(w.record().psi_points()),
                field(w.record().maximum_psi()),
                field(w.record().shield_chance()),
                field(w.record().armor())
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
                    field(w.record().action_points()),
                    field(w.record().movement_points()),
                    c.round + 1
                ),
                r.x + 10.,
                r.y + 91.,
                12,
                WHITE,
            );
        }
    }
}
