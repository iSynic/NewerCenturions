//! The scenario's questions, encounter picker and GetWord text entry.
use crate::{
    assets::Assets,
    presentation::{Action, Control, Inspection, UiState},
};
use centurions_content::Content;
use centurions_core::{Command, Session};
use macroquad::prelude::*;

/// Draws a yes-or-no question (DLOG 137) with its two answers.
pub fn question(assets: &Assets, text: &str, yes: Action, no: Action, controls: &mut Vec<Control>) {
    controls.clear();
    assets.draw_dialog(137);
    assets.dialog_text(text, assets.dialog_rect(137, 5));
    for (item, action) in [(7, yes), (6, no)] {
        controls.push(Control {
            rect: assets.dialog_rect(137, item),
            enabled: true,
            action,
        });
    }
}

/// Draws a scenario's menu of choices (DLOG 147) under the message text.
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
    assets.wrapped_color(text, crate::layout::MESSAGE_TEXT, 12, 16., YELLOW);
    assets.draw_dialog(147);
    let table = if state.menus.is_empty() {
        state
            .land(content)
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

/// Whether the encounter takes typed text: only when it has a password.
pub fn text_entry_available(state: &Session, content: &Content) -> bool {
    // encounter2 offers GetWord only when the encounter has a password selector.
    centurions_core::typed_record(state, content).is_ok_and(|record| {
        centurions_content::records::TypedEncounterRecord(&record).password_selector() != 0
    })
}

/// Draws the typed encounter dialog (DLOG 153) under the message text, with its scan, physical
/// action, item, typed word and cancel controls, each available only as the encounter allows.
/// While a word is being typed, draws the word entry (DLOG 157) with the text typed so far.
pub fn typed(
    content: &Content,
    state: &Session,
    assets: &Assets,
    ui: &UiState,
    text: &str,
    controls: &mut Vec<Control>,
) {
    controls.clear();
    assets.wrapped_color(text, crate::layout::MESSAGE_TEXT, 12, 16., YELLOW);
    assets.draw_dialog(153);
    for (item, action, enabled) in [
        (
            2,
            Action::Open(Inspection::Psi),
            centurions_core::typed_record(state, content)
                .is_ok_and(|record| centurions_content::word(&record, 106).unwrap_or(0) != 0),
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
                .is_ok_and(|record| centurions_content::word(&record, 136).unwrap_or(0) != 0),
        ),
        (
            6,
            Action::TextEntry(true),
            text_entry_available(state, content),
        ),
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
    if ui.typed_entry && text_entry_available(state, content) {
        controls.clear();
        assets.draw_dialog(157);
        assets.dialog_text(&ui.typed_text, assets.dialog_rect(157, 2));
        controls.push(Control {
            rect: assets.dialog_rect(157, 4),
            enabled: true,
            action: Action::Command(Command::SubmitText {
                text: ui.typed_text.clone(),
            }),
        });
    }
}

/// Draws the squad's portraits as a strip of choices; a member with no action
/// is not offered.
pub fn portraits(
    state: &Session,
    assets: &Assets,
    controls: &mut Vec<Control>,
    action: impl Fn(usize) -> Option<Action>,
) {
    // getchoice confines selection to the portrait strip.
    for (member, warrior) in state.warriors.iter().enumerate() {
        if let Some(action) = action(member) {
            let rect = Rect::new(490., 20. + member as f32 * 50., 50., 50.);
            if let Some(image) = warrior
                .record()
                .portrait()
                .ok()
                .and_then(|id| assets.portraits.get(&id))
            {
                crate::assets::texture_in(image, rect);
            }
            controls.push(Control {
                rect,
                enabled: true,
                action,
            });
        }
    }
}
