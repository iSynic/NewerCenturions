#[cfg(all(test, feature = "original-fixtures"))]
use crate::*;
use crate::{Event, Phase, RuleError, ScriptState, Session, TimeContinuation, story_allies};
mod recovery;
use centurions_content::Content;
use centurions_content::records::{
    EnemyRecord, EnemyRecordMut, ScheduleEntry, ScheduleEntryMut, ScheduleRegion, ScheduledScripts,
    WarriorRecord, WarriorRecordMut,
};
use recovery::recover;

mod zero;
pub use zero::{ZeroOwners, ZeroTimeReturn};
pub(crate) use zero::{resume_owned, retain_zero_work, validate_zero, zero_tick};

pub(crate) fn advance(
    state: &mut Session,
    content: &Content,
    ticks: u8,
    minutes_per_tick: i16,
) -> Result<Vec<Event>, RuleError> {
    if ticks == 0 {
        return zero_tick(state, content);
    }
    let mut events = Vec::new();
    for tick in 0..ticks {
        let position = state.exploration.as_mut().unwrap();
        position.minute = position.minute.wrapping_add(minutes_per_tick);
        if position.minute < 60 {
            continue;
        }
        // CODE 1 timeclick: reduce precedes the hour, fatigue and energy updates.
        crate::combat::party_buffs::reduce(&mut state.party_buffs)?;
        let context = crate::story_health::retained_context(
            state.selected_warrior,
            &state.native_work.health_context,
        );
        let reduced = crate::story_reduce::reduce_outdoors(
            &state.warriors,
            &context,
            &mut state.random,
            state.energy_recovery_disabled,
        )?;
        let lost = reduced
            .events
            .iter()
            .any(|event| matches!(event, crate::story_health::HealthEvent::PartyLost));
        let control_returns = state.warriors.iter().zip(&reduced.warriors).try_fold(
            0_i16,
            |delta, (before, after)| {
                Ok::<_, RuleError>(
                    delta
                        - i16::from(
                            before.record().faction()? != 0 && after.record().faction()? == 0,
                        ),
                )
            },
        )?;
        state.native_work.victory_work.add_total(control_returns);
        state.warriors = reduced.warriors;
        state.native_work.health_context = reduced.context;
        events.extend(reduced.events.into_iter().filter_map(|event| match event {
            crate::story_health::HealthEvent::Sound(resource_id) => {
                Some(Event::Sound { resource_id })
            }
            crate::story_health::HealthEvent::Flash { address, sound } => {
                Some(Event::Feedback(crate::feedback::NativeFeedback::Flash {
                    address,
                    x: 100,
                    y: 30,
                    wait: 100,
                    sound,
                }))
            }
            _ => None,
        }));
        if lost {
            events.extend(crate::mission::defeat(state, content)?);
            return Ok(events);
        }
        position.minute -= 60;
        position.hour = position.hour.wrapping_add(1);
        position.fatigue = position.fatigue.saturating_add(1).clamp(4, 135);
        for warrior in &mut state.warriors {
            let bytes = &mut warrior.bytes;
            let energy = WarriorRecord::from_bytes(bytes).psi_points()?;
            let maximum = WarriorRecord::from_bytes(bytes).maximum_psi()?;
            if state.energy_recovery_disabled == 0
                && energy < maximum
                && WarriorRecord::from_bytes(bytes).stamina()? > 0
                && WarriorRecord::from_bytes(bytes).automatic_mode()? == 0
            {
                let level = WarriorRecord::from_bytes(bytes).level()?;
                let amount = (level.wrapping_add((level as u16 >> 15) as i16) >> 1) as i8;
                WarriorRecordMut::from_bytes(bytes)
                    .set_psi_points(energy.wrapping_add(i16::from(amount.max(1))).min(maximum));
            }
        }
        if state.energy_recovery_disabled == 0 {
            for bytes in state.allies[..state.ally_count * story_allies::ALLY_BYTES]
                .as_chunks_mut::<{ story_allies::ALLY_BYTES }>()
                .0
            {
                let energy = EnemyRecord::from_bytes(bytes).psi_points()?;
                let maximum = EnemyRecord::from_bytes(bytes).maximum_psi()?;
                if energy < maximum {
                    let level = EnemyRecord::from_bytes(bytes).level()?;
                    let amount = (level.wrapping_add((level as u16 >> 15) as i16) >> 1) as i8;
                    EnemyRecordMut::from_bytes(bytes)
                        .set_psi_points(energy.wrapping_add(i16::from(amount.max(1))).min(maximum));
                }
            }
        }
        if position.hour == 24 {
            position.hour = 0;
            position.day = position.day.wrapping_add(1);
            let mut pending = TimeContinuation {
                remaining_ticks: ticks - tick - 1,
                minutes_per_tick,
                schedule_cursor: 0,
            };
            if scheduled_events(state, content, &mut pending, &mut events)? {
                if state.exploration.is_some() {
                    state.time_continuation = Some(pending);
                }
                return Ok(events);
            }
        } else if position.hour == 12 {
            recover(state, content, &mut events)?;
            if state.phase != Phase::Exploration {
                return Ok(events);
            }
        }
        let position = state.exploration.as_ref().unwrap();
        events.push(Event::TimeAdvanced {
            day: position.day,
            hour: position.hour,
            minute: position.minute,
        });
    }
    Ok(events)
}

fn scheduled_events(
    state: &mut Session,
    content: &Content,
    pending: &mut TimeContinuation,
    events: &mut Vec<Event>,
) -> Result<bool, RuleError> {
    loop {
        let selected = scan_schedule(state, &mut pending.schedule_cursor, events)?;
        recover(state, content, events)?;
        if state.phase != Phase::Exploration {
            return Ok(true);
        }
        let Some(script_id) = selected else {
            return Ok(false);
        };
        let map = state
            .map(
                content,
                state
                    .exploration
                    .as_ref()
                    .ok_or("schedule position missing")?
                    .map,
            )
            .ok_or("schedule content unavailable")?;
        let record = ScheduledScripts(&map.scheduled_scripts)
            .record(script_id.unsigned_abs())?
            .to_vec();
        state.exploration.as_mut().unwrap().script = Some(ScriptState::scheduled(record, 0));
        // Keep the caller visible while a child executes a nested time opcode.
        state.time_continuation = Some(pending.clone());
        events.extend(super::exploration::run_script(state, content)?);
        state.time_continuation = None;
        if state.phase != Phase::Exploration
            || state
                .exploration
                .as_ref()
                .is_none_or(|p| p.presentation.is_some() || p.script.is_some())
        {
            return Ok(true);
        }
    }
}

pub(crate) fn resume(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    if state.zero_time.is_some() {
        return zero::resume(state, content);
    }
    let Some(mut pending) = state.time_continuation.take() else {
        return Ok(vec![]);
    };
    let mut events = vec![];
    if scheduled_events(state, content, &mut pending, &mut events)? {
        if state.exploration.is_some() {
            state.time_continuation = Some(pending);
        }
        return Ok(events);
    }
    let p = state
        .exploration
        .as_ref()
        .ok_or("schedule position missing")?;
    events.push(Event::TimeAdvanced {
        day: p.day,
        hour: p.hour,
        minute: p.minute,
    });
    events.extend(advance(
        state,
        content,
        pending.remaining_ticks,
        pending.minutes_per_tick,
    )?);
    Ok(events)
}

fn scan_schedule(
    state: &mut Session,
    cursor: &mut usize,
    events: &mut Vec<Event>,
) -> Result<Option<i16>, RuleError> {
    let position = state.exploration.as_ref().unwrap();
    let day = position.day;
    let count = ScheduleEntry::count(&position.schedule);
    while *cursor < count {
        let slot = *cursor;
        *cursor += 1;
        let record = ScheduleEntry::at(&state.exploration.as_ref().unwrap().schedule, slot)
            .0
            .to_vec();
        let entry = ScheduleEntry(&record);
        let due = entry.due_day()?;
        if due != day {
            if due == 0 {
                break;
            }
            continue;
        }
        let next_day = due.wrapping_add(entry.interval()?);
        let mut eligible = state.random.bound(100) <= entry.chance()?;
        if eligible && entry.required_item()? > 0 {
            let required = entry.required_item()?;
            eligible = state.warriors.iter().any(|w| {
                w.inventory()
                    .is_ok_and(|items| items.iter().any(|item| item.id() == required))
            });
        }
        if eligible && entry.flag()? >= 0 {
            let flag = entry.flag()? as usize;
            eligible = *state
                .flags
                .get(flag)
                .ok_or("schedule flag outside mission state")?
                != 0;
        }
        if eligible && entry.location_mode()? > 0 {
            let position = state.exploration.as_ref().unwrap();
            if entry.location_mode()? != 1 || i32::from(entry.map()?) != position.map {
                continue;
            }
            let region = entry.region()?;
            if region >= 0 {
                let rectangle = ScheduleRegion::read(&position.region_state, region as usize)?;
                eligible = position.y >= i32::from(rectangle.top()?)
                    && position.x >= i32::from(rectangle.left()?)
                    && position.y < i32::from(rectangle.bottom()?)
                    && position.x < i32::from(rectangle.right()?);
            }
            let x = entry.x()?;
            if x >= 0 && (position.x != i32::from(x) || position.y != i32::from(entry.y()?)) {
                continue;
            }
        }
        ScheduleEntryMut::at(&mut state.exploration.as_mut().unwrap().schedule, slot)
            .set_due_day(next_day);
        events.push(Event::ScheduleRescheduled {
            slot,
            next_day,
            dispatched: eligible,
        });
        if eligible {
            return Ok(Some(entry.script()?));
        }
        *cursor = count;
        break;
    }
    Ok(None)
}
#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../tests/unit/time_reference.rs"]
mod tests;
