//! castspell10b89e / DLOG144: level, power, learned scan and Perform Scan.
use crate::{
    assets::Assets,
    presentation::{Action, Control, UiState},
};
use centurions_content::{Content, word};
use centurions_core::{Command, Session};
use macroquad::prelude::*;

fn action(
    state: &Session,
    content: &Content,
    member: usize,
    spell: i16,
    power: i16,
    packet: &[u8],
) -> Action {
    if centurions_core::typed_record(state, content).is_ok() {
        Action::Command(Command::CastEncounterPsi {
            warrior: member,
            spell,
            power,
        })
    } else if state.combat.is_none() {
        Action::Command(Command::CastPartyPsi {
            warrior: member,
            spell,
            power,
        })
    } else if packet[1] == 0 {
        Action::Command(Command::BeginPsiSelection { spell, power })
    } else if packet[1] == 7 || packet[1] >= 9 || spell == 2302 {
        Action::Command(Command::CastPsi {
            rotation: 0,
            spell,
            power,
            target: None,
        })
    } else {
        Action::ChoosePsi(spell)
    }
}

fn allowed(
    state: &Session,
    content: &Content,
    member: usize,
    spell: i16,
    power: i16,
    packet: &[u8],
) -> bool {
    if centurions_core::typed_record(state, content).is_ok() {
        return centurions_core::typed_encounter::validate_spell(
            state, content, member, spell, power,
        )
        .is_ok();
    }
    let warrior = &state.warriors[member];
    if !warrior.knows_spell(spell).unwrap_or(false)
        || warrior.psi_points().unwrap_or(0)
            < centurions_core::combat::player_psi::cost(packet, power).unwrap_or(i16::MAX)
        || !centurions_core::combat::player_psi::mode_allowed(packet, state.combat.is_some())
            .unwrap_or(false)
    {
        return false;
    }
    if let Some(combat) = &state.combat {
        return combat.active_body() == Some(member as i16)
            && centurions_core::combat::player_psi::can_cast(
                &state.warriors,
                member as i16,
                state.psi_disabled,
            )
            .unwrap_or(false);
    }
    state
        .exploration
        .as_ref()
        .and_then(|position| content.map(position.map))
        .is_some_and(|mission| {
            centurions_core::combat::player_psi::validate_party(
                &state.warriors,
                mission,
                state.psi_disabled,
                centurions_core::combat::player_psi::PartyCastRequest {
                    warrior: member,
                    spell,
                    power,
                },
            )
            .is_ok()
        })
}

pub fn draw(
    content: &Content,
    state: &Session,
    assets: &Assets,
    ui: &UiState,
    member: usize,
    controls: &mut Vec<Control>,
) {
    controls.clear();
    let origin = vec2(480., 20.);
    let rectangle = |item| assets.dialog_rect_at(144, item, origin);
    assets.draw_dialog_at(144, origin);
    let warrior = &state.warriors[member];
    let Some(mission) = state
        .exploration
        .as_ref()
        .and_then(|position| content.map(position.map))
    else {
        return;
    };
    let Some(combat_content) = &mission.combat else {
        return;
    };
    let level = ui.page.min(6);
    let power = ui.psi_power.clamp(1, 7);
    let school = warrior.record().psi_school().unwrap_or(0);
    let field = |item, text: &str| assets.wrapped_color(text, rectangle(item), 12, 14., YELLOW);
    let mut chosen = None;
    for row in 0..7 {
        let spell = school * 1000 + (level as i16 + 1) * 100 + row as i16 + 1;
        let known = warrior.knows_spell(spell).unwrap_or(false);
        let rect = rectangle(58 + row);
        if known {
            field(
                1 + row,
                assets.spell_names.get(&spell).map_or("", String::as_str),
            );
        } else {
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0., 0., 0., 0.5));
        }
        if ui.psi_choice.is_some_and(|choice| choice.0 == spell) && known {
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1., YELLOW);
            chosen = Some(spell);
        }
        controls.push(Control {
            rect,
            enabled: known,
            action: Action::PreviewPsi(spell),
        });
        for (item, selected, action) in [
            (
                23 + row,
                row == level,
                Action::Page(row as i32 - level as i32),
            ),
            (
                30 + row,
                row as i16 + 1 == power,
                Action::PsiPower(row as i16 + 1),
            ),
        ] {
            let rect = rectangle(item);
            if selected {
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1., YELLOW);
            }
            controls.push(Control {
                rect,
                enabled: true,
                action,
            });
        }
    }
    if let Some(image) = warrior
        .record()
        .portrait()
        .ok()
        .and_then(|id| assets.portraits.get(&id))
    {
        let rect = rectangle(49);
        draw_texture_ex(
            image,
            rect.x,
            rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(rect.size()),
                ..Default::default()
            },
        );
    }
    field(44, &warrior.psi_points().unwrap_or(0).to_string());
    let mut perform = None;
    if let Some(spell) = chosen
        && let Ok(packet) =
            centurions_core::combat::spells::load_packet(&combat_content.attacks, spell)
    {
        if ui.preferences.enabled(11)
            && let Ok(description) =
                mission.scan_description(school, level, (spell % 100 - 1) as usize)
        {
            let description_origin = vec2(0., 436.);
            assets.draw_dialog_at(169, description_origin);
            assets.wrapped_color(
                &description,
                assets.dialog_rect_at(169, 2, description_origin),
                12,
                14.,
                YELLOW,
            );
        }
        let cost = centurions_core::combat::player_psi::cost(&packet, power).unwrap_or(0);
        field(41, &packet[30].to_string());
        field(54, &power.to_string());
        field(42, &cost.to_string());
        field(
            18,
            &centurions_core::combat::player_psi::range(&packet, power)
                .unwrap_or(0)
                .to_string(),
        );
        field(
            19,
            &if packet[1] == 0 {
                power.to_string()
            } else {
                "1".to_owned()
            },
        );
        draw_spell_fields(&packet, power, assets, rectangle);
        if allowed(state, content, member, spell, power, &packet) {
            perform = Some(action(state, content, member, spell, power, &packet));
        }
    }
    let rect = rectangle(37);
    if perform.is_none() {
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0., 0., 0., 0.5));
    }
    controls.push(Control {
        rect,
        enabled: perform.is_some(),
        action: perform.unwrap_or(Action::Close),
    });
    controls.push(Control {
        rect: rectangle(38),
        enabled: true,
        action: Action::Close,
    });
    if state.combat.is_none() {
        for (item, direction) in [(39, -1_i32), (40, 1)] {
            let next = (1..=state.warriors.len())
                .map(|step| {
                    (member as i32 + direction * step as i32)
                        .rem_euclid(state.warriors.len() as i32) as usize
                })
                .find(|candidate| {
                    centurions_core::combat::player_psi::can_cast(
                        &state.warriors,
                        *candidate as i16,
                        state.psi_disabled,
                    )
                    .unwrap_or(false)
                });
            controls.push(Control {
                rect: rectangle(item),
                enabled: next.is_some(),
                action: Action::Select(next.unwrap_or(member)),
            });
        }
    }
}

fn draw_spell_fields(
    packet: &[u8],
    power: i16,
    assets: &Assets,
    rectangle: impl Fn(usize) -> Rect,
) {
    // spellinfoupdate15947c reads signed BYTE modifiers and wrapping WORD totals.
    let field = |item, text: &str| assets.wrapped_color(text, rectangle(item), 12, 14., YELLOW);
    for (modifier, base, item) in [(11, 32, 13), (12, 34, 50)] {
        let value = i16::from(packet[modifier] as i8)
            .wrapping_mul(power)
            .wrapping_add(word(packet, base).unwrap_or(0));
        field(item, &value.to_string());
    }
    for (modifier, base, item) in [(15, 13, 14), (16, 14, 15)] {
        let value = i16::from(packet[modifier] as i8)
            .wrapping_mul(power)
            .wrapping_add(i16::from(packet[base] as i8))
            .wrapping_abs();
        field(item, &value.to_string());
    }
    let damage_label = match packet[20] {
        0 => 0x400c38,
        1 => 0x400c2e,
        2 => 0x400c28,
        3 => 0x400c22,
        4 => 0x400c18,
        5 => 0x400c10,
        6 => 0x400c08,
        7 => 0x400bfe,
        8 => 0x400bf6,
        10 => 0x400bee,
        _ => 0x400c42,
    };
    field(
        52,
        &assets
            .native_text
            .text(damage_label, true)
            .unwrap_or_default(),
    );
    let duration = packet[9] as i8;
    let icon = |id, item| {
        if let Some(image) = assets.icons.get(&id) {
            let rect = rectangle(item);
            draw_texture_ex(
                image,
                rect.x,
                rect.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(rect.size()),
                    ..Default::default()
                },
            );
        }
    };
    if duration == 1 || duration > 2 {
        icon(129, 65);
    } else if packet[10] != 0 {
        field(
            65,
            &i16::from(packet[10] as i8).wrapping_mul(power).to_string(),
        );
    }
    if duration > 1 {
        icon(130, 66);
    } else if packet[8] != 0 || packet[6] != 0 {
        field(
            66,
            &i16::from(packet[8] as i8)
                .wrapping_mul(power)
                .wrapping_add(i16::from(packet[6] as i8))
                .to_string(),
        );
    }
    let effect = word(packet, 40).unwrap_or(0);
    if effect != 0 && packet[1] != 7 {
        if let Some(text) = assets
            .reference_strings
            .get(&134)
            .and_then(|labels| labels.get(effect.saturating_sub(1) as usize))
        {
            field(67, text);
        }
    } else {
        let label = match packet[1] {
            0 => 0x400bd8,
            1 => 0x400bca,
            3 => 0x400bb8,
            4 => 0x400ba2,
            5 => 0x400b8e,
            6 => 0x400b7a,
            7 => 0x400b64,
            8 => 0x400b52,
            9 => 0x400b3c,
            10 => 0x400b28,
            11 => 0x400b18,
            _ => 0x400bed,
        };
        field(
            67,
            &assets.native_text.text(label, true).unwrap_or_default(),
        );
    }
    let shape = match packet[1] {
        3 => i16::from(packet[31]) + 9999,
        4 => power + 9999,
        5 => {
            if packet[31] == 0 {
                10215
            } else {
                i16::from(packet[31]) + 9999
            }
        }
        6 => 10213,
        7 => 10214,
        9 => 10211,
        10 => 10210,
        11 => 10212,
        _ => 10000,
    };
    icon(shape, 16);
}
