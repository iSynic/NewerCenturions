use crate::{Content, Event, Phase, Presentation, RuleError, Session, combat};
use centurions_content::records::ActorReplacementExtra;
use serde::{Deserialize, Serialize};

/// Where a replacement stands: `Before` holds the replacement record with the given number, which
/// may be showing its introduction; `After` is showing its conclusion.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Replacement {
    Before(i16),
    After,
}

/// What the script does after a replacement step: wait for a dialogue, continue, or stop.
pub(crate) enum Progress {
    Waiting,
    Continue,
    Stop,
}

/// The actor-replacement record the script's extra names.
fn extra<'a>(
    state: &Session,
    content: &'a Content,
    id: i16,
) -> Result<ActorReplacementExtra<'a>, RuleError> {
    let map = crate::exploration::active_map(state, content)?;
    Ok(ActorReplacementExtra::select(
        &map.combat
            .as_ref()
            .ok_or("actor tables missing")?
            .encounters,
        id,
    )?)
}

/// Shows a dialogue page for the replacement.
fn page(state: &mut Session, content: &Content, id: i16) -> Result<Vec<Event>, RuleError> {
    let map = crate::exploration::active_map(state, content)?;
    let text = crate::exploration::dialogue(map, &state.warriors, id)?;
    state.exploration.as_mut().unwrap().presentation = Some(Presentation::Dialogue { id, text });
    Ok(vec![Event::StoryPresented])
}

/// Checks that a pending replacement belongs to a battle with a script showing its dialogue.
pub(crate) fn validate(state: &Session, content: &Content) -> Result<(), RuleError> {
    if let Some(pending) = &state.actor_replacement {
        let arena = state
            .combat
            .as_ref()
            .ok_or("actor continuation outside combat")?;
        if state.exploration.as_ref().is_none_or(|p| {
            p.script.is_none() || !matches!(p.presentation, Some(Presentation::Dialogue { .. }))
        }) || arena.monster_up >= arena.roster.records.len()
        {
            return Err("invalid actor continuation".into());
        }
        if let Replacement::Before(id) = pending {
            extra(state, content, *id)?;
        }
    }
    Ok(())
}

/// Acts on the monster last selected (`monster_up`), not the monster whose script is
/// running (`macro_monster`).
///
/// Original: newland
pub(crate) fn begin(
    state: &mut Session,
    content: &Content,
    id: i16,
) -> Result<(Vec<Event>, Progress), RuleError> {
    if state.phase != Phase::Combat || state.actor_replacement.is_some() {
        return Err("actor replacement requires an unsuspended combat script".into());
    }
    state.actor_replacement = Some(Replacement::Before(id));
    let record = extra(state, content, id)?;
    state.globals.extra_record = record.0.to_vec();
    let introduction = record.introduction()?;
    if introduction != 0 {
        return Ok((page(state, content, introduction)?, Progress::Waiting));
    }
    resume(state, content)
}

/// Carries out a replacement once its introduction is dismissed: the selected monster is removed
/// from the battle, or replaced by the monster the record names. The record's sound plays, and its
/// conclusion dialogue is shown if it has one.
pub(crate) fn resume(
    state: &mut Session,
    content: &Content,
) -> Result<(Vec<Event>, Progress), RuleError> {
    let pending = state
        .actor_replacement
        .take()
        .ok_or("actor continuation missing")?;
    let Replacement::Before(id) = pending else {
        return Ok((vec![], Progress::Continue));
    };
    let extra = extra(state, content, id)?;
    let replacement = extra.replacement()?;
    let arena = state.combat.as_mut().ok_or("actor combat missing")?;
    let selected = arena.monster_up;
    let mut events = vec![];
    if replacement == -1 {
        let record = arena
            .roster
            .records
            .get_mut(selected)
            .ok_or("selected NPC missing")?;
        record.record_mut().set_stamina(0);
        record.record_mut().set_todo_on_death(0);
        // killbody takes the battle order entry, whichever monster is selected.
        let body = arena
            .active_body()
            .ok_or("replacement order entry missing")?;
        combat::lifecycle::kill_body(arena, &mut state.warriors, body)?;
        events.push(Event::CombatantRemoved { body });
        return Ok((events, Progress::Stop));
    }
    let map = crate::exploration::active_map(state, content)?;
    let arena = state.combat.as_mut().unwrap();
    if combat::final_turn_stories::replace_body(
        arena,
        &map.combat.as_ref().unwrap().monsters,
        replacement,
        selected,
        &mut state.random,
    )? {
        return Ok((events, Progress::Stop));
    }
    let sound = extra.sound()?;
    if sound != 0 {
        events.push(Event::Sound { resource_id: sound });
    }
    let conclusion = extra.conclusion()?;
    if conclusion != 0 {
        state.actor_replacement = Some(Replacement::After);
        events.extend(page(state, content, conclusion)?);
        return Ok((events, Progress::Waiting));
    }
    Ok((events, Progress::Continue))
}
