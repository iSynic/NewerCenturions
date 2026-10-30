use crate::story::allies::ALLY_BYTES;
use crate::{Event, RuleError, Session};
use centurions_content::combat::MONSTER_BYTES;
use centurions_content::{Content, span, ui, word};

/// Shows a note the party holds: its record and text. Refused when the note is not held, when a
/// note without a picture names a map the scenario lacks, or when its picture is not installed.
pub fn note(state: &Session, content: &Content, index: usize) -> Result<Vec<Event>, RuleError> {
    if state.notes.get(index).copied().unwrap_or(0) == 0 {
        return Err(crate::RuleError::Refused { feedback: vec![] });
    }
    let map = state.land(content).ok_or("note scenario unavailable")?;
    let record = span(&map.map_notes, index * 340, 340)?.to_vec();
    if word(&record, 66)? == 0 && state.map(content, i32::from(word(&record, 64)?)).is_none() {
        return Err("note map unavailable".into());
    }
    let picture = word(&record, 66)?;
    if picture != 0
        && !content
            .resources
            .iter()
            .any(|resource| resource.r#type == "PICT" && resource.id == picture)
    {
        return Err("note picture unavailable in installed resources".into());
    }
    let text = ui::pascal_text(&record[84..])?;
    Ok(vec![Event::NoteViewed {
        index,
        record,
        text,
    }])
}

/// The bestiary entries: the records before the terminator, minus hidden ones. Without a shipped
/// archive table they are sorted by name byte by byte.
///
/// Original: updatemonstermenu
pub fn archive_entries(
    state: &Session,
    content: &Content,
) -> Result<Vec<(usize, String)>, RuleError> {
    let map = state.land(content).ok_or("archive scenario unavailable")?;
    let records = &map
        .combat
        .as_ref()
        .ok_or("archive creature table unavailable")?
        .monsters;
    let mut entries = Vec::new();
    let order: Vec<_> = map
        .archive_order()
        .unwrap_or_else(|| (0..records.len() / MONSTER_BYTES).collect());
    for index in order {
        let record = span(records, index * MONSTER_BYTES, MONSTER_BYTES)?;
        if word(record, 0)? == 255 {
            break;
        }
        if word(record, 0)? != 0 && record[167] == 0 {
            entries.push((index, centurions_content::combat::monster_name(record)?));
        }
    }
    if map.archive_table.is_none() {
        entries.sort_by(|(a, _), (b, _)| {
            let a = a * MONSTER_BYTES + 218;
            let b = b * MONSTER_BYTES + 218;
            records[a + 1..a + 1 + usize::from(records[a])]
                .cmp(&records[b + 1..b + 1 + usize::from(records[b])])
        });
    }
    Ok(entries)
}

/// Shows the record of a bestiary creature with its description, or of a held-over ally, which has
/// no description. Refused when the entry is not available.
pub fn view(
    state: &Session,
    content: &Content,
    index: usize,
    ally: bool,
) -> Result<Vec<Event>, RuleError> {
    let map = state.land(content).ok_or("creature scenario unavailable")?;
    let (record, description) = if ally {
        if index >= state.held_over {
            return Err("ally entry unavailable".into());
        }
        (
            span(&state.holdover, index * ALLY_BYTES, ALLY_BYTES)?.to_vec(),
            String::new(),
        )
    } else {
        if !archive_entries(state, content)?
            .iter()
            .any(|(id, _)| *id == index)
        {
            return Err("archive entry unavailable".into());
        }
        let record = span(
            &map.combat.as_ref().unwrap().monsters,
            index * MONSTER_BYTES,
            MONSTER_BYTES,
        )?
        .to_vec();
        let text = span(&map.creature_descriptions, index * 256, 256)?;
        (record, ui::pascal_text(text)?)
    };
    Ok(vec![Event::CreatureViewed {
        index,
        ally,
        record,
        description,
    }])
}

/// The portrait of a selected body in battle shows the creature as it is now,
/// including changed conditions.
pub fn combat_body(state: &Session, content: &Content, body: i16) -> Result<Vec<Event>, RuleError> {
    crate::combat::controls::actor(state)?;
    if (0..state.warriors.len() as i16).contains(&body) {
        return Ok(vec![]);
    }
    let record = state
        .combat
        .as_ref()
        .unwrap()
        .roster
        .records
        .get(usize::try_from(body - 10).map_err(|_| "combat body invalid")?)
        .ok_or("combat body missing")?
        .bytes
        .clone();
    let map = state
        .land(content)
        .ok_or("combat description map missing")?;
    let definition = usize::try_from(word(&record, 0)?).map_err(|_| "combat definition invalid")?;
    let description = ui::pascal_text(span(&map.creature_descriptions, definition * 256, 256)?)?;
    Ok(vec![Event::CreatureViewed {
        index: body as usize,
        ally: false,
        record,
        description,
    }])
}
