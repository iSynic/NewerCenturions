//! Menus for maintained exploration.
use super::{active_map, finish_typed_selection};
use crate::{
    Event, MenuContinuation, Presentation, RuleError, ScriptState, Session, story, typed_encounter,
};
use centurions_content::mission::OpeningMission;
use centurions_content::{Content, ContentError, span, word};

pub(super) fn present_menu(
    map: &OpeningMission,
    table: &[u8],
    script: &mut ScriptState,
    id: i16,
) -> Result<Option<Presentation>, RuleError> {
    if id < 0 {
        return Err(RuleError::Invalid("negative encounter menu".into()));
    }
    let menu = span(
        table,
        id as usize * story::MENU_RECORD_BYTES,
        story::MENU_RECORD_BYTES,
    )?;
    if script.menu.is_none() {
        script.menu = Some(MenuContinuation {
            id,
            remaining: i16::from(menu[101] as i8),
            original: script.record.clone(),
            typed: false,
        });
    }
    let continuation = script.menu.as_mut().unwrap();
    let before = continuation.remaining;
    if menu[96] as i8 != -4 {
        continuation.remaining = before.wrapping_sub(1);
    }
    if menu[96] as i8 == -4 || before == 0 {
        script.record[8..40].copy_from_slice(&story::dispatch_menu_branch(menu, 4)?);
        script.cursor = 0;
        return Ok(None);
    }
    let labels = (0..4)
        .map(|i| {
            let label = span(menu, 106 + i * 80, 80)?;
            Ok(encoding_rs::MACINTOSH
                .decode(span(label, 1, usize::from(label[0]))?)
                .0
                .into_owned())
        })
        .collect::<Result<Vec<_>, ContentError>>()?;
    Ok(Some(Presentation::Menu {
        id,
        text: map.dialogue(word(menu, 104)?.wrapping_neg())?,
        labels,
    }))
}

pub(super) fn present_typed(
    map: &OpeningMission,
    table: &[u8],
    script: &mut ScriptState,
    id: i16,
) -> Result<Presentation, RuleError> {
    if id < 0 {
        return Err(RuleError::Invalid("negative typed encounter".into()));
    }
    let record = span(
        table,
        id as usize * typed_encounter::RECORD_BYTES,
        typed_encounter::RECORD_BYTES,
    )?;
    if script.menu.is_none() {
        script.menu = Some(MenuContinuation {
            id,
            remaining: typed_encounter::attempt_count(record)?,
            original: script.record.clone(),
            typed: true,
        });
    }
    Ok(Presentation::Typed {
        id,
        text: map.dialogue(typed_encounter::dialogue(record)?.wrapping_neg())?,
    })
}

pub fn submit_text(
    state: &mut Session,
    content: &Content,
    text: &str,
) -> Result<Vec<Event>, RuleError> {
    let input = centurions_content::native_text::NativeText::encode(text)?;
    if input.len() > 40 {
        return Err(RuleError::Invalid(
            "typed input exceeds the original 40-byte field".into(),
        ));
    }
    let record = typed_record(state, content)?;
    if input.is_empty() {
        return Ok(vec![]);
    }
    let native =
        centurions_content::native_text::NativeText::load(content, std::path::Path::new("."))?;
    let typed_encounter::PasswordSelection::Selected(selected) =
        typed_encounter::select_password(record, &input, native.bytes(0x3ffc50, 256)?)?
    else {
        return Ok(vec![]);
    };
    finish_typed_selection(state, content, selected)
}

pub fn typed_record<'a>(state: &'a Session, content: &'a Content) -> Result<&'a [u8], RuleError> {
    let map = active_map(state, content)?;
    let position = state
        .exploration
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("no active mission".into()))?;
    let Some(Presentation::Typed { id, .. }) = position.presentation.as_ref() else {
        return Err(RuleError::Invalid("no typed encounter active".into()));
    };
    Ok(span(
        if state.typed_encounters.is_empty() {
            &map.typed_encounters
        } else {
            &state.typed_encounters
        },
        *id as usize * typed_encounter::RECORD_BYTES,
        typed_encounter::RECORD_BYTES,
    )?)
}
