//! Menus for maintained exploration.
use super::{active_map, finish_typed_selection};
use crate::{
    Event, MenuContinuation, Presentation, RuleError, ScriptState, Session, story, typed_encounter,
};
use centurions_content::mission::OpeningMission;
use centurions_content::records::{EncounterMenu, StoryRecordMut};
use centurions_content::{Content, ContentError, span};

pub(super) fn present_menu(
    map: &OpeningMission,
    table: &[u8],
    script: &mut ScriptState,
    id: i16,
    work: &mut story::EncounterWork,
    reload: bool,
) -> Result<Option<Presentation>, RuleError> {
    if reload {
        work.load(table, id, false)?;
    }
    work.clear_disabled_labels()?;
    let bytes = work.record(false)?;
    let menu = bytes.as_slice();
    let record = EncounterMenu(menu);
    if script.menu.is_none() {
        script.menu = Some(MenuContinuation {
            id,
            remaining: i16::from(record.initial_remaining()),
            original: script.record.clone(),
            typed: false,
            eliminated: 0,
        });
        script.caller_action = Some(i16::from(script.cursor).wrapping_sub(1));
    }
    let continuation = script.menu.as_mut().unwrap();
    if reload {
        continuation.id = id;
        continuation.typed = false;
        continuation.remaining = i16::from(record.initial_remaining());
        continuation.eliminated = 0;
    }
    let before = continuation.remaining;
    if record.first_selector() != -4 {
        continuation.remaining = before.wrapping_sub(1);
    }
    if record.first_selector() == -4 || before == 0 {
        StoryRecordMut(&mut script.record).write_actions(&story::dispatch_menu_branch(menu, 4)?);
        script.cursor = 0;
        return Ok(None);
    }
    let labels = (0..4)
        .map(|i| {
            Ok(encoding_rs::MACINTOSH
                .decode(record.label(i)?)
                .0
                .into_owned())
        })
        .collect::<Result<Vec<_>, ContentError>>()?;
    Ok(Some(Presentation::Menu {
        id,
        text: map.dialogue(record.dialogue()?.wrapping_neg())?,
        labels,
    }))
}

pub(super) fn present_typed(
    map: &OpeningMission,
    table: &[u8],
    script: &mut ScriptState,
    id: i16,
    work: &mut story::EncounterWork,
    reload: bool,
) -> Result<Presentation, RuleError> {
    if reload {
        work.load(table, id, true)?;
    }
    let bytes = work.record(true)?;
    let record = bytes.as_slice();
    if script.menu.is_none() {
        script.menu = Some(MenuContinuation {
            id,
            remaining: typed_encounter::attempt_count(record)?,
            original: script.record.clone(),
            typed: true,
            eliminated: 0,
        });
        script.caller_action = Some(i16::from(script.cursor).wrapping_sub(1));
    }
    if reload {
        let continuation = script.menu.as_mut().unwrap();
        continuation.id = id;
        continuation.typed = true;
        continuation.remaining = typed_encounter::attempt_count(record)?;
        continuation.eliminated = 0;
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
        typed_encounter::select_password(&record, &input, native.bytes(0x3ffc50, 256)?)?
    else {
        return Ok(vec![]);
    };
    finish_typed_selection(state, content, selected)
}

pub fn typed_record(state: &Session, _content: &Content) -> Result<Vec<u8>, RuleError> {
    if !matches!(
        state
            .exploration
            .as_ref()
            .and_then(|p| p.presentation.as_ref()),
        Some(Presentation::Typed { .. })
    ) {
        return Err(RuleError::Invalid("no typed encounter active".into()));
    }
    state.native_work.encounter_work.record(true)
}

/// saveencounter writes the current header, leaving the file labels untouched.
pub(super) fn save_encounter(
    state: &mut Session,
    content: &Content,
    id: i16,
    typed: bool,
) -> Result<(), RuleError> {
    let header = state.native_work.encounter_work.header(typed).to_vec();
    let map = active_map(state, content)?;
    let authored = if typed {
        &map.typed_encounters
    } else {
        &map.menus
    };
    let saved = if typed {
        &mut state.typed_encounters
    } else {
        &mut state.menus
    };
    let stride = if typed {
        typed_encounter::RECORD_BYTES
    } else {
        story::MENU_RECORD_BYTES
    };
    let offset =
        usize::try_from(id).map_err(|_| RuleError::Invalid("negative encounter".into()))? * stride;
    let previous = if saved.is_empty() { authored } else { &*saved };
    if span(previous, offset, header.len())? == header {
        return Ok(());
    }
    if saved.is_empty() {
        *saved = authored.clone();
    }
    saved[offset..offset + header.len()].copy_from_slice(&header);
    Ok(())
}
