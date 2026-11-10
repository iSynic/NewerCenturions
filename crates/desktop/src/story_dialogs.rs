//! The scenario's questions, encounter picker and GetWord text entry.
use crate::{
    application::ApplicationAction,
    assets::Assets,
    placement::Placement,
    presentation::{Action, Control, Inspection, UiState},
};
use centurions_content::Content;
use centurions_core::{Command, Presentation, Session, story::EncounterHeaders};
use macroquad::prelude::*;

/// Draws a yes-or-no question (DLOG 137) with its two answers, as the only controls.
pub fn question(assets: &Assets, text: &str, yes: Action, no: Action, controls: &mut Vec<Control>) {
    controls.clear();
    question_window(assets, text, yes, no, controls);
}

/// Draws the question dialog at the top left of the game window with `text` in its static text,
/// in the game font, and adds its yes (left) and no (right) buttons to `controls`.
///
/// Original: question
pub fn question_window(
    assets: &Assets,
    text: &str,
    yes: Action,
    no: Action,
    controls: &mut Vec<Control>,
) {
    let size = assets.dialog_sizes[&QUESTION_DIALOG];
    double_box_frame(Rect::new(
        QUESTION_ORIGIN.x,
        QUESTION_ORIGIN.y,
        size.x,
        size.y,
    ));
    assets.draw_dialog_with_parameters(QUESTION_DIALOG, QUESTION_ORIGIN, &[text]);
    for (item, action) in [(QUESTION_YES, yes), (QUESTION_NO, no)] {
        controls.push(Control {
            rect: assets.dialog_rect_at(QUESTION_DIALOG, item, QUESTION_ORIGIN),
            enabled: true,
            action,
        });
    }
}

/// Draws the double frame of a dialog box round `window`: a band of the content colour, a black
/// line, three shaded lines set one pixel apart from each other, and the yellow frame colour
/// outside them. Only what falls in the game window under the menu bar shows.
///
/// Original: dBoxProc
fn double_box_frame(window: Rect) {
    let (right, bottom) = (window.right() - 1., window.bottom() - 1.);
    // Each line runs from `near` pixels before the content to `far` pixels after its last pixel.
    let outline = |near: f32, far: f32, color: Color| {
        let (left, top) = (window.x - near, window.y - near);
        let (width, height) = (right + far - left + 1., bottom + far - top + 1.);
        for strip in [
            Rect::new(left, top, width, 1.),
            Rect::new(left, bottom + far, width, 1.),
            Rect::new(left, top, 1., height),
            Rect::new(right + far, top, 1., height),
        ] {
            if let Some(shown) = strip.intersect(GAME_WINDOW) {
                draw_rectangle(shown.x, shown.y, shown.w, shown.h, color);
            }
        }
    };
    for near in 1..=3 {
        outline(near as f32, near as f32, CONTENT_COLOR);
    }
    outline(4., 4., BLACK);
    outline(7., 5., Color::from_rgba(205, 202, 255, 255));
    outline(6., 6., Color::from_rgba(189, 185, 189, 255));
    outline(5., 7., Color::from_rgba(123, 121, 156, 255));
    outline(8., 8., FRAME_COLOR);
}

/// The game window under the menu bar, where dialog frames show.
const GAME_WINDOW: Rect = Rect::new(0., 20., 800., 580.);
/// The content colour of the question, warning and flash dialogs.
const CONTENT_COLOR: Color = Color::new(0.2, 0.2, 0., 1.);
/// The frame colour of the question, warning and flash dialogs.
const FRAME_COLOR: Color = Color::new(1., 1., 0., 1.);

/// The yes/no question dialog.
const QUESTION_DIALOG: i16 = 137;
/// The question window's place: the game window's top left, under the menu bar.
const QUESTION_ORIGIN: Vec2 = Vec2::new(0., 20.);
/// The question's yes button, the item its y key chooses.
const QUESTION_YES: usize = 2;
/// The question's no button, the item its n key chooses.
const QUESTION_NO: usize = 1;

/// The strip's item for the right half, which answers no.
const STRIP_NO: usize = 1;
/// The strip's item for the left half, which answers yes.
const STRIP_YES: usize = 2;
/// DLOG 136, the strip with a caption on each half.
const CAPTIONED_STRIP: i16 = 136;
/// DLOG 200, the strip of plain Yes and No pictures.
const PLAIN_STRIP: i16 = 200;
/// Where the character-case table sits in the game's data.
const LOWERCASE_TABLE: u32 = 0x3ffc50;

/// The captions on the two halves of the question on show, when its extra names them. A caption the
/// scenario's option texts do not reach reads as the first entry of STR# 3.
///
/// Original: question2
fn question_captions(
    content: &Content,
    state: &Session,
    assets: &Assets,
) -> Option<[centurions_content::question_options::QuestionCaption; 2]> {
    let labels = centurions_core::question_label_words(state, content)?;
    let preloaded = assets.warnings.first().map_or("", String::as_str);
    Some(state.land(content)?.question_captions(labels, preloaded))
}

/// Draws the question strip: a caption on each half, left to answer yes and right to answer no, or
/// the plain Yes and No pictures when the question has no captions. The question itself is printed
/// in the message box.
///
/// Original: question2
pub fn question_strip(
    content: &Content,
    state: &Session,
    assets: &Assets,
    placement: Placement,
    controls: &mut Vec<Control>,
) {
    let answers = (
        Action::Command(Command::Choose { answer: true }),
        Action::Command(Command::Choose { answer: false }),
    );
    let Some([yes, no]) = question_captions(content, state, assets) else {
        plain_question_strip(assets, placement, answers.0, answers.1, controls);
        return;
    };
    let (yes, no) = (yes.text(), no.text());
    let origin = placement.question_strip();
    assets.draw_dialog_with_parameters(CAPTIONED_STRIP, origin, &[&yes, &no]);
    strip_controls(assets, CAPTIONED_STRIP, origin, answers, controls);
}

/// Draws the question strip of plain Yes and No pictures, as a question asked with no captions
/// has it, and makes its left half give `yes` and its right half give `no`.
///
/// Original: question2
pub fn plain_question_strip(
    assets: &Assets,
    placement: Placement,
    yes: Action,
    no: Action,
    controls: &mut Vec<Control>,
) {
    let origin = placement.question_strip();
    assets.draw_dialog_at(PLAIN_STRIP, origin);
    strip_controls(assets, PLAIN_STRIP, origin, (yes, no), controls);
}

/// Adds the two halves of the strip drawn from `dialog` at `origin` as controls: the left half
/// answers yes and the right half no.
fn strip_controls(
    assets: &Assets,
    dialog: i16,
    origin: Vec2,
    (yes, no): (Action, Action),
    controls: &mut Vec<Control>,
) {
    for (item, action) in [(STRIP_YES, yes), (STRIP_NO, no)] {
        controls.push(Control {
            rect: assets.dialog_rect_at(dialog, item, origin),
            enabled: true,
            action,
        });
    }
}

/// The answer the typed characters give a question on the strip: the no key is tried before the
/// yes key, and any other character is ignored.
///
/// Original: question2
pub fn typed_answer(yes: u8, no: u8) -> Option<bool> {
    while let Some(character) = get_char_pressed() {
        let Ok(encoded) = centurions_content::game_text::GameText::encode(&character.to_string())
        else {
            continue;
        };
        if encoded == [no] {
            return Some(false);
        }
        if encoded == [yes] {
            return Some(true);
        }
    }
    None
}

/// The keys that answer the question on show, as (yes, no): the first letters of its two captions
/// folded to lower case, or y and n when it has none. Any other key leaves the question open.
///
/// Original: question2
pub fn question_keys(content: &Content, state: &Session, assets: &Assets) -> Option<(u8, u8)> {
    match state.exploration.as_ref()?.presentation.as_ref()? {
        Presentation::Avoidance { .. } => return Some((b'y', b'n')),
        Presentation::Question { .. } => {}
        _ => return None,
    }
    let Some([yes, no]) = question_captions(content, state, assets) else {
        return Some((b'y', b'n'));
    };
    let lowercase = assets.game_text.bytes(LOWERCASE_TABLE, 256).ok()?;
    Some((
        lowercase[usize::from(yes.key_byte())],
        lowercase[usize::from(no.key_byte())],
    ))
}

/// Whether the encounter menu's choice in that place is still on offer: its branch selector in
/// the live header is not zero.
fn menu_slot_enabled(headers: &EncounterHeaders, slot: usize) -> bool {
    headers
        .header(false)
        .get(96 + slot)
        .is_some_and(|selector| *selector != 0)
}

/// What choosing a place in the encounter menu does. An offered choice picks the nth offered
/// entry, counting only the choices still on offer; a choice that is not offered sounds the error
/// and leaves the menu showing.
///
/// Original: encounter
pub fn menu_slot_action(headers: &EncounterHeaders, slot: usize) -> Action {
    if menu_slot_enabled(headers, slot) {
        let index = (0..slot)
            .filter(|earlier| menu_slot_enabled(headers, *earlier))
            .count();
        Action::Command(Command::SelectMenuEntry { index })
    } else {
        Action::Application(ApplicationAction::RefuseMenuEntry)
    }
}

/// Draws a scenario's menu of choices (DLOG 147) beside the message text. Choices that are no
/// longer offered are greyed but can still be clicked, and then sound the error. A menu the party
/// cannot back out of is cut short above its cancel button.
///
/// Original: encounter
pub fn menu(
    state: &Session,
    assets: &Assets,
    placement: Placement,
    text: &str,
    labels: &[String],
    controls: &mut Vec<Control>,
) {
    controls.clear();
    assets.wrapped_color(
        text,
        crate::layout::MESSAGE_TEXT,
        12,
        crate::assets::line_height(12),
        YELLOW,
    );
    let headers = &state.globals.encounter_headers;
    let can_back_out = headers
        .header(false)
        .get(100)
        .is_some_and(|byte| *byte != 0);
    let origin = placement.encounter_menu();
    let item = |number: usize| assets.dialog_rect_at(147, number, origin);
    assets.draw_sized_dialog(
        147,
        origin,
        vec2(259., if can_back_out { 320. } else { 267. }),
    );
    for (slot, label) in labels.iter().take(4).enumerate() {
        assets.wrapped_color(
            &super::presentation::authored_text(label, &state.warriors),
            item(5 + slot),
            12,
            crate::assets::line_height(12),
            if menu_slot_enabled(headers, slot) {
                YELLOW
            } else {
                GRAY
            },
        );
        controls.push(Control {
            rect: item(1 + slot),
            enabled: true,
            action: menu_slot_action(headers, slot),
        });
    }
    if can_back_out {
        controls.push(Control {
            rect: item(9),
            enabled: true,
            action: Action::Command(Command::CancelMenu),
        });
    }
}

/// Whether the encounter takes typed text: only when it has a password.
pub fn text_entry_available(state: &Session, content: &Content) -> bool {
    // encounter2 offers GetWord only when the encounter has a password selector.
    centurions_core::typed_record(state, content).is_ok_and(|record| {
        centurions_content::records::TypedEncounterRecord(&record).password_selector() != 0
    })
}

/// Draws the typed encounter dialog (DLOG 153) under the field, with its scan, physical action,
/// item, typed word and cancel controls, each available only as the encounter allows; an
/// encounter the party cannot back out of cuts the strip short before its cancel button. While a
/// word is being typed, draws the word entry (DLOG 157) with the text typed so far.
///
/// Original: encounter2, getword
pub fn typed(
    content: &Content,
    state: &Session,
    assets: &Assets,
    ui: &UiState,
    text: &str,
    controls: &mut Vec<Control>,
) {
    controls.clear();
    assets.wrapped_color(
        text,
        crate::layout::MESSAGE_TEXT,
        12,
        crate::assets::line_height(12),
        YELLOW,
    );
    let placement = Placement::new(ui.large_field());
    let origin = placement.typed_encounter();
    let can_back_out = state
        .globals
        .encounter_headers
        .header(true)
        .get(151)
        .is_some_and(|byte| *byte != 0);
    assets.draw_sized_dialog(
        153,
        origin,
        vec2(if can_back_out { 250. } else { 200. }, 50.),
    );
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
        if item == 7 && !can_back_out {
            continue;
        }
        let rect = assets.dialog_rect_at(153, item, origin);
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
        let origin = placement.word_entry();
        assets.draw_dialog_at(157, origin);
        assets.dialog_text(&ui.typed_text, assets.dialog_rect_at(157, 2, origin));
        controls.push(Control {
            rect: assets.dialog_rect_at(157, 4, origin),
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
