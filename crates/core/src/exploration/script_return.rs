//! Script return for maintained exploration.
use crate::{Event, RuleError, ScriptState, Session, combat};
use centurions_content::mission::OpeningMission;
use centurions_content::records::{EventHeaderTableMut, StoryRecord, StoryRecordMut};
use centurions_content::{Content, Warrior};

/// Derived caller ownership; no extra persistent state or new admission rule.
#[derive(Clone, Copy)]
enum ScriptOwner {
    Round,
    Death,
    Exploration,
}

fn script_owner(state: &Session) -> ScriptOwner {
    if state
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
    state.exploration.as_mut().unwrap().script = None;
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
    if let Some(events) = finish_dispatch(state, content)? {
        return Ok(events);
    }
    let position = state.exploration.as_mut().unwrap();
    state.scripted_cleanup = None;
    if script.scheduled {
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
    if exhausted && script.menu.is_none() {
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
    content
        .map_at(state.mission.as_deref(), plan.target.level, plan.target.map)
        .ok_or_else(|| RuleError::Invalid("door destination map unavailable".into()))?;
    if script.persist {
        let record =
            crate::story_destination::persistent_record(&script.record, origin.x, origin.y)?;
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
        EventHeaderTableMut(headers).replace(script.map_slot, &record);
    }
    crate::mission::travel(state, content, plan.target)?;
    let position = state.exploration.as_ref().unwrap();
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

/// loaddoor2 preserves the first eight bytes; caller frames restore actions and byte40 only.
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
