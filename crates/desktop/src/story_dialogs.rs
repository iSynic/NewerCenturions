//! Original Question, encounter picker and GetWord resources.
use crate::{
    assets::Assets,
    presentation::{Action, Control, Inspection, UiState},
};
use centurions_content::Content;
use centurions_core::{Command, Session};
use macroquad::prelude::*;

pub fn question(assets: &Assets, text: &str, yes: Action, no: Action, controls: &mut Vec<Control>) {
    controls.clear();
    assets.draw_dialog(137);
    assets.wrapped_color(text, assets.dialog_rect(137, 5), 12, 14., YELLOW);
    for (item, action) in [(7, yes), (6, no)] {
        controls.push(Control {
            rect: assets.dialog_rect(137, item),
            enabled: true,
            action,
        });
    }
}

pub fn menu(
    content: &Content,
    state: &Session,
    assets: &Assets,
    id: i16,
    text: &str,
    labels: &[String],
    controls: &mut Vec<Control>,
) {
    controls.clear();
    assets.wrapped_color(text, Rect::new(12., 467., 444., 82.), 12, 16., YELLOW);
    assets.draw_dialog(147);
    let table = if state.menus.is_empty() {
        content
            .map(state.exploration.as_ref().map_or(0, |p| p.map))
            .map(|map| map.menus.as_slice())
            .unwrap_or_default()
    } else {
        &state.menus
    };
    for (index, label) in labels.iter().take(4).enumerate() {
        let enabled = table
            .get(id as usize * centurions_core::story::MENU_RECORD_BYTES + 96 + index)
            .is_some_and(|code| *code != 0);
        assets.wrapped_color(
            &super::presentation::authored_text(label, &state.warriors),
            assets.dialog_rect(147, 5 + index),
            12,
            14.,
            if enabled { YELLOW } else { GRAY },
        );
        controls.push(Control {
            rect: assets.dialog_rect(147, 1 + index),
            enabled,
            action: Action::Command(Command::SelectMenuEntry { index }),
        });
    }
    controls.push(Control {
        rect: assets.dialog_rect(147, 9),
        enabled: true,
        action: Action::Command(Command::CancelMenu),
    });
}

pub fn typed(
    content: &Content,
    state: &Session,
    assets: &Assets,
    ui: &UiState,
    text: &str,
    controls: &mut Vec<Control>,
) {
    controls.clear();
    assets.wrapped_color(text, Rect::new(12., 467., 444., 82.), 12, 16., YELLOW);
    assets.draw_dialog(153);
    for (item, action, enabled) in [
        (
            2,
            Action::Open(Inspection::Psi),
            centurions_core::typed_record(state, content)
                .is_ok_and(|record| centurions_content::word(record, 106).unwrap_or(0) != 0),
        ),
        (
            4,
            Action::Command(Command::BeginEncounterActions),
            centurions_core::typed_record(state, content).is_ok_and(|record| record[96] != 0),
        ),
        (
            5,
            Action::Open(Inspection::Items),
            centurions_core::typed_record(state, content)
                .is_ok_and(|record| centurions_content::word(record, 136).unwrap_or(0) != 0),
        ),
        (6, Action::TextEntry(true), true),
        (7, Action::Command(Command::CancelMenu), true),
    ] {
        let rect = assets.dialog_rect(153, item);
        if !enabled {
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0., 0., 0., 0.6));
        }
        controls.push(Control {
            rect,
            enabled,
            action,
        });
    }
    if ui.typed_entry {
        controls.clear();
        assets.draw_dialog(157);
        assets.wrapped_color(&ui.typed_text, assets.dialog_rect(157, 2), 12, 14., YELLOW);
        controls.push(Control {
            rect: assets.dialog_rect(157, 4),
            enabled: true,
            action: Action::Command(Command::SubmitText {
                text: ui.typed_text.clone(),
            }),
        });
    }
}

pub fn portraits(
    state: &Session,
    assets: &Assets,
    controls: &mut Vec<Control>,
    action: impl Fn(usize) -> Option<Action>,
) {
    // getchoice125112 confines selection to the existing portrait strip.
    for (member, warrior) in state.warriors.iter().enumerate() {
        if let Some(action) = action(member) {
            let rect = Rect::new(490., 20. + member as f32 * 50., 50., 50.);
            if let Some(image) = warrior
                .record()
                .portrait()
                .ok()
                .and_then(|id| assets.portraits.get(&id))
            {
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
            controls.push(Control {
                rect,
                enabled: true,
                action,
            });
        }
    }
}
