//! Panels presentation.
use super::{Action, Control, Inspection, UiState, field};
use crate::assets::Assets;
use centurions_content::{Content, Warrior};
use centurions_core::{Command, Session};
use macroquad::prelude::*;

/// Draws the clinic dialog (DLOG 159) for `member`, with its controls.
pub(super) fn draw_clinic(
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
    let field = |item, text: String| assets.dialog_text(&text, assets.dialog_rect(159, item));
    for (index, price) in centurions_core::clinic::prices(percent).iter().enumerate() {
        if matches!(index, 0..=3 | 5..=7) {
            field(46 + index, price.to_string());
        }
    }
    let warrior = &state.warriors[member];
    field(55, state.party_currency.to_string());
    field(65, warrior.name.clone());
    field(66, warrior.record().stamina().unwrap_or(0).to_string());
    field(67, warrior.record().stamina_max().unwrap_or(0).to_string());
    for (row, index) in (0..40)
        .filter(|i| warrior.record().condition_duration(*i).unwrap_or(0) != 0)
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
        .get(&warrior.record().portrait().unwrap_or(0))
    {
        crate::assets::texture_in(portrait, rect);
    }
    let health = warrior.record().stamina().unwrap_or(0);
    if health < 1 {
        for id in [2019, if health < -9 { 2015 } else { 177 }] {
            if let Some(icon) = assets.icons.get(&id) {
                crate::assets::texture_in(icon, rect);
            }
        }
    }
}

/// Draws the survivors dialog (DLOG 173) and its Continue control.
pub(super) fn draw_survivors(state: &Session, assets: &Assets, controls: &mut Vec<Control>) {
    assets.draw_dialog(173);
    controls.push(Control {
        rect: assets.dialog_rect(173, 2),
        enabled: true,
        action: Action::Command(Command::Continue),
    });
    let Some(selection) = &state.survivors else {
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
        let Some(record) = state.survivor_record(*index) else {
            continue;
        };
        let record = record.record();
        if record.survivor_marker().unwrap_or(0) == -1
            && let Some(icon) = assets.icons.get(&2003)
        {
            draw_texture(icon, rect.x + 3., rect.y + 3., WHITE);
        }
        if let Some(icon) = assets.icons.get(&record.icon().unwrap_or(0)) {
            let (horizontal, vertical) = match record.size().unwrap_or(0) {
                0 => (19., 19.),
                1 => (19., 3.),
                2 => (3., 19.),
                _ => (3., 3.),
            };
            crate::assets::texture_in(
                icon,
                Rect::new(
                    rect.x + horizontal,
                    rect.y + vertical,
                    rect.w - horizontal * 2.,
                    rect.h - vertical * 2.,
                ),
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
        for (value, x, y) in [
            (record.stamina(), 5., 14.),
            (record.stamina_max(), 40., 14.),
            (record.psi_points(), 5., 29.),
            (record.psi_points_max(), 40., 29.),
        ] {
            assets.text(
                &value.unwrap_or(0).to_string(),
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

/// Draws the information panels under the map (PICT 166 and 228) for the body in
/// view: a monster's stamina, psi points and shield, or a warrior's.
pub(super) fn draw_combat_information(
    content: &Content,
    state: &Session,
    assets: &Assets,
    body: i16,
) {
    assets.panel(166, Rect::new(0., 449., 308., 139.));
    assets.panel(228, Rect::new(310., 449., 160., 140.));
    if !crate::acting::is_warrior_body(body) {
        let Some(monster) = state
            .combat
            .as_ref()
            .and_then(|c| c.roster.records.get((body - 10) as usize))
        else {
            return;
        };
        let record = monster.record();
        for (value, x, y) in [
            (record.stamina(), 115., 470.),
            (record.psi_points(), 115., 493.),
            (record.shield_points(), 223., 470.),
            (record.armor_rating(), 274., 470.),
        ] {
            assets.text(&value.unwrap_or(0).to_string(), x, y, 12, WHITE);
        }
        for (value, x) in [(record.movement(), 115.), (record.action_points(), 211.)] {
            assets.text(&value.max(0).to_string(), x, 516., 12, WHITE);
        }
        if let Ok(icon) = record.icon()
            && let Some(texture) = assets.icons.get(&icon)
        {
            crate::assets::texture_in(texture, Rect::new(8., 456., 64., 64.));
        }
        assets.text(&record.name().unwrap_or_default(), 126., 536., 12, WHITE);
        return;
    }
    let Some(w) = state.warriors.get(body as usize) else {
        return;
    };
    for (value, x, y) in [
        (w.record().stamina(), 115., 470.),
        (w.record().psi_points(), 115., 493.),
        (w.record().movement(), 115., 516.),
        (w.record().action_points(), 211., 516.),
        (w.record().shield_chance(), 181., 470.),
        (w.record().shield_points(), 223., 470.),
        (w.record().armor_rating(), 274., 470.),
    ] {
        assets.text(&field(value), x, y, 12, WHITE);
    }
    if let Ok(id) = w.record().icon()
        && let Some(texture) = assets.icons.get(&id)
    {
        draw_texture(texture, 22., 472., WHITE);
    }
    assets.text(&w.name, 126., 536., 12, WHITE);
    let id = w.record().equipped_weapon().unwrap_or(0);
    assets.text(&assets.name(id), 126., 558., 12, YELLOW);
    if let Some(def) = content.items.get(id.unsigned_abs() as usize)
        && let Ok(icon) = centurions_content::records::ItemDefinition(def).icon()
        && let Some(texture) = assets.icons.get(&icon)
    {
        draw_texture(texture, 8., 529., WHITE);
    }
    if let Some(item) = w
        .inventory()
        .unwrap_or_default()
        .iter()
        .find(|i| i.id() == id && i.equipped() == 1)
    {
        assets.text(
            &item.charge().unwrap_or(0).to_string(),
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
/// The reserve ammunition in the pack for the warrior's equipped weapon.
pub(super) fn equipped_ammunition_reserve(state: &Session, warrior: &Warrior) -> i32 {
    let weapon = warrior.record().equipped_weapon().unwrap_or(0);
    warrior
        .inventory()
        .unwrap_or_default()
        .iter()
        .find(|item| item.id() == weapon && item.equipped() == 1)
        .and_then(|item| item.ammunition_id().ok())
        .filter(|ammunition| (200..400).contains(ammunition))
        .map(|ammunition| (ammunition - 200) as usize)
        .and_then(|index| state.ammunition.get(index))
        .copied()
        .unwrap_or(0)
}
/// DLOG 160: one 32-pixel header and one row per named ability.
///
/// Original: showspecial
pub fn draw_specials(state: &Session, assets: &Assets, ui: &UiState, controls: &mut Vec<Control>) {
    let warrior = &state.warriors[ui.selected];
    let rows: Vec<_> = assets.reference_strings[&1201]
        .iter()
        .take(14)
        .enumerate()
        .filter_map(|(index, label)| {
            let base = warrior.record().spec(index).unwrap_or(0);
            (base > 0 && !label.is_empty()).then(|| {
                let value =
                    base.wrapping_add(warrior.record().special_skill_bonus(index).unwrap_or(0));
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
        assets.dialog_text(
            value,
            Rect::new(
                origin.x + left as f32,
                origin.y + top as f32,
                (right - left) as f32,
                (bottom - top) as f32,
            ),
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
