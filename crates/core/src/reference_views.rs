use crate::{Event, RuleError, Session};
use centurions_content::{Content, span, ui, word};

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

/// updatemonstermenu 0x135bb2: terminated records, hidden flag, then bytewise name ordering.
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
        .archive_order
        .clone()
        .unwrap_or_else(|| (0..records.len() / 288).collect());
    for index in order {
        let record = span(records, index * 288, 288)?;
        if word(record, 0)? == 255 {
            break;
        }
        if word(record, 0)? != 0 && record[167] == 0 {
            entries.push((index, centurions_content::combat::monster_name(record)?));
        }
    }
    if map.archive_order.is_none() {
        entries.sort_by(|(a, _), (b, _)| {
            let a = a * 288 + 218;
            let b = b * 288 + 218;
            records[a + 1..a + 1 + usize::from(records[a])]
                .cmp(&records[b + 1..b + 1 + usize::from(records[b])])
        });
    }
    Ok(entries)
}

pub fn view(
    state: &Session,
    content: &Content,
    index: usize,
    ally: bool,
) -> Result<Vec<Event>, RuleError> {
    let map = state.land(content).ok_or("creature scenario unavailable")?;
    let (record, description) = if ally {
        if index >= state.ally_count {
            return Err("ally entry unavailable".into());
        }
        (
            span(&state.allies, index * 288, 288)?.to_vec(),
            String::new(),
        )
    } else {
        if !archive_entries(state, content)?
            .iter()
            .any(|(id, _)| *id == index)
        {
            return Err("archive entry unavailable".into());
        }
        let record = span(&map.combat.as_ref().unwrap().monsters, index * 288, 288)?.to_vec();
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

/// combat's selected-body portrait uses the live creature, including changed conditions.
pub fn combat_body(state: &Session, content: &Content, body: i16) -> Result<Vec<Event>, RuleError> {
    crate::combat::controls::actor(state)?;
    if (0..state.warriors.len() as i16).contains(&body) {
        return Ok(vec![]);
    }
    let record = state
        .combat
        .as_ref()
        .unwrap()
        .enemies
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
