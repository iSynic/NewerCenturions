//! Script return for maintained exploration.
use crate::{Event, RuleError, ScriptState, Session, combat};
use centurions_content::mission::OpeningMission;
use centurions_content::records::{EventHeaderTableMut, StoryRecord, StoryRecordMut};
use centurions_content::{Content, Warrior};

/// Derived caller ownership; no extra persistent state or new admission rule.
#[derive(Clone, Copy)]
enum ScriptOwner {
    Turn,
    Round,
    Death,
    Exploration,
}

fn script_owner(state: &Session) -> ScriptOwner {
    // A calendar child entered under a battle owner returns to timeclick first.
    if state
        .zero_time
        .as_ref()
        .is_some_and(|frame| frame.owners.is_some() && frame.innermost(state))
    {
        ScriptOwner::Exploration
    } else if state
        .turn_dispatch
        .as_ref()
        .is_some_and(|frame| frame.owns_script(state))
    {
        ScriptOwner::Turn
    } else if state
        .round_dispatch
        .as_ref()
        .is_some_and(|frame| frame.owns_script(state))
    {
        ScriptOwner::Round
    } else if state
        .combat_story
        .as_ref()
        .is_some_and(|story| story.shared_dispatch && !story.waiting_child)
    {
        ScriptOwner::Death
    } else {
        ScriptOwner::Exploration
    }
}

pub(super) fn finish_dispatch(
    state: &mut Session,
    content: &Content,
) -> Result<Option<Vec<Event>>, RuleError> {
    match script_owner(state) {
        ScriptOwner::Turn => combat::runtime::finish_turn_dispatch(state, content).map(Some),
        ScriptOwner::Round => combat::runtime::finish_round_dispatch(state, content).map(Some),
        ScriptOwner::Death => combat::runtime::finish_death_story(state, content).map(Some),
        ScriptOwner::Exploration => Ok(None),
    }
}

/// newland's early `return 0`: the script stops without its end processing
/// (event retirement, encounter loop or destination); battle owners still finish.
pub(super) fn abandon_script(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    if let Some(script) = state.exploration.as_mut().unwrap().script.take() {
        retain_turn_header(state, &script.record);
        combat::runtime::retain_death_header(state, &script.record);
    }
    Ok(finish_dispatch(state, content)?.unwrap_or_default())
}

pub(super) fn finish_script(
    state: &mut Session,
    content: &Content,
    exhausted: bool,
) -> Result<Vec<Event>, RuleError> {
    let position = state
        .exploration
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("no active map".into()))?;
    let Some(mut script) = position.script.take() else {
        crate::trace::declined(
            "exploration.finish_script",
            "exploration",
            "no active script",
        );
        return Ok(vec![]);
    };
    retain_turn_header(state, &script.record);
    combat::runtime::retain_death_header(state, &script.record);
    if let Some(events) = finish_dispatch(state, content)? {
        return Ok(events);
    }
    state.scripted_cleanup = None;
    crate::time::retain_zero_work(state, &script.record);
    let calendar = state.zero_time.is_some();
    let position = state.exploration.as_mut().unwrap();
    if script.scheduled && !calendar {
        crate::trace::declined(
            "exploration.finish_script",
            "exploration",
            "scheduled script",
        );
        return Ok(vec![]);
    }
    let origin = script.origin.clone().unwrap_or_else(|| {
        let key = StoryRecord(&script.record).location_key().unwrap_or(0);
        crate::ScriptOrigin {
            map: key / 10000,
            x: key % 100,
            y: key / 100 % 100,
            level: state.scenario_level,
        }
    });
    if exhausted && script.menu.is_none() && !calendar {
        StoryRecordMut(&mut script.record).set_trigger_chance(255);
        let headers = if origin.level != state.scenario_level {
            &mut state
                .visited_levels
                .get_mut(&origin.level)
                .and_then(|l| l.maps.get_mut(&origin.map))
                .ok_or("event origin level missing")?
                .event_headers
        } else if origin.map == position.map {
            &mut position.event_headers
        } else {
            &mut state
                .visited_maps
                .get_mut(&origin.map)
                .ok_or_else(|| RuleError::Invalid("event origin map missing".into()))?
                .event_headers
        };
        EventHeaderTableMut(headers).set_trigger_chance(script.map_slot, 255);
    }
    let current = crate::story_destination::Location {
        map: position.map,
        x: position.x,
        y: position.y,
        level: state.scenario_level,
    };
    let Some(plan) = crate::story_destination::destination(
        &script.record,
        current,
        state.phase == crate::Phase::Combat,
        script.chained,
    )?
    else {
        crate::trace::declined(
            "exploration.finish_script",
            "exploration",
            "no story destination",
        );
        return Ok(vec![]);
    };
    OpeningMission::index(plan.target.x, plan.target.y)?;
    let moved =
        (plan.target.map, plan.target.x, plan.target.y) != (current.map, current.x, current.y);
    content
        .map_at(state.mission.as_deref(), plan.target.level, plan.target.map)
        .ok_or_else(|| RuleError::Invalid("door destination map unavailable".into()))?;
    crate::mission::travel(state, content, plan.target)?;
    let row = state.native_work.header_row;
    let position = state.exploration.as_mut().unwrap();
    // 14686c..1468c0 copy DOOR, with the saved A4/A3 low bytes, into header row
    // WORD4427c2 of the land now current, after any land change.
    if script.persist {
        let (x, y) = script.persist_cell;
        let record =
            crate::story_destination::persistent_record(&script.record, x.into(), y.into())?;
        EventHeaderTableMut(&mut position.event_headers).replace(row as usize, &record);
    }
    if calendar {
        // newland1468ce..146916 clears the movement offsets before its tile test.
        position.last_move = [0; 2];
        // A new location on a header tile runs that walking event inside the calendar
        // newland (14692e..140320), which returns to the schedule scan afterwards.
        if moved
            && position.floor[OpeningMission::index(position.x, position.y)?].wrapping_abs() >= 1000
        {
            let mut events = vec![Event::Moved {
                x: position.x,
                y: position.y,
            }];
            // The matched header replaces DOOR before its chance test.
            let key = position.y * 100 + position.x + position.map * 10_000;
            let matched = position
                .event_headers
                .as_chunks::<42>()
                .0
                .iter()
                .find(|record| {
                    centurions_content::mission::long(&record[..], 0).is_ok_and(|v| v == key)
                });
            if let Some(record) = matched.map(|record| record.to_vec()) {
                crate::time::retain_zero_work(state, &record);
            }
            events.extend(super::movement::walking_header(state, content)?);
            return Ok(events);
        }
    }
    Ok(vec![Event::Moved {
        x: position.x,
        y: position.y,
    }])
}

pub(crate) fn dialogue(
    map: &OpeningMission,
    warriors: &[Warrior],
    id: i16,
) -> Result<String, RuleError> {
    let mut text = map.dialogue(id)?;
    for index in 0..warriors.len() {
        let selected = if warriors[index].stamina()? < 1 {
            (0..warriors.len())
                .rev()
                .find(|i| warriors[*i].stamina().unwrap_or(0) > 0)
                .unwrap_or(index)
        } else {
            index
        };
        text = text.replace(&format!("^{}", index + 1), &warriors[selected].name);
    }
    Ok(text)
}

/// loaddoor2 leaves this byte shared. Turn/round frames restore actions and byte40;
/// death frames restore actions only.
pub(crate) fn retain_script_stamp(state: &mut Session, stamp: u8) {
    fn write(script: Option<&mut ScriptState>, stamp: u8) {
        if let Some(script) = script {
            centurions_content::records::StoryRecordMut(&mut script.record)
                .write_dispatch_stamp(stamp);
        }
    }
    write(
        state.exploration.as_mut().and_then(|p| p.script.as_mut()),
        stamp,
    );
    let mut turn = state.turn_dispatch.as_mut();
    while let Some(frame) = turn {
        write(frame.suspended_script.as_mut(), stamp);
        turn = frame.parent.as_deref_mut();
    }
    let mut round = state.round_dispatch.as_mut();
    while let Some(frame) = round {
        write(frame.suspended_script.as_mut(), stamp);
        round = frame.parent.as_deref_mut();
    }
    let mut death = state.combat_story.as_mut();
    while let Some(story) = death {
        write(story.suspended_script.as_mut(), stamp);
        death = story.parent.as_deref_mut();
    }
}

/// Each surrounding battle caller owns its saved actions or the live DOOR body.
pub(crate) fn retain_turn_header(state: &mut Session, record: &[u8]) {
    let mut current = state.turn_dispatch.as_mut();
    while let Some(frame) = current {
        frame.work = Some(record.to_vec());
        if let Some(caller) = frame.suspended_script.as_mut() {
            if frame.shared_record {
                caller.record.copy_from_slice(record);
            } else {
                StoryRecordMut(&mut caller.record).copy_battle_return_header(record);
            }
        }
        current = frame.parent.as_deref_mut();
    }
}
