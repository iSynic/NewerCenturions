use super::*;
use centurions_content::word;

fn put(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

pub(crate) fn advance(
    state: &mut Session,
    content: &Content,
    ticks: u8,
    minutes_per_tick: i16,
) -> Result<Vec<Event>, RuleError> {
    let mut events = Vec::new();
    for _ in 0..ticks {
        let position = state.exploration.as_mut().unwrap();
        position.minute = position.minute.wrapping_add(minutes_per_tick);
        if position.minute < 60 {
            continue;
        }
        // CODE 1 timeclick: reduce precedes the hour, fatigue and energy updates.
        crate::combat::party_buffs::reduce(&mut state.party_buffs)?;
        let context = crate::story_health::context(
            &state.warriors,
            state.selected_warrior,
            state.health_context.as_ref(),
        );
        let reduced =
            crate::story_reduce::reduce_outdoors(&state.warriors, &context, &mut state.random)?;
        if reduced
            .events
            .iter()
            .any(|event| matches!(event, crate::story_health::HealthEvent::PartyLost))
        {
            return Err(RuleError::Unsupported {
                action: "time.conditions",
                dependency: "party defeat transition",
            });
        }
        state.warriors = reduced.warriors;
        state.health_context = Some(reduced.context);
        events.extend(reduced.events.into_iter().filter_map(|event| {
            if let crate::story_health::HealthEvent::Sound(resource_id) = event {
                Some(Event::Sound { resource_id })
            } else {
                None
            }
        }));
        position.minute -= 60;
        position.hour += 1;
        position.fatigue = position.fatigue.saturating_add(1).clamp(4, 135);
        for warrior in &mut state.warriors {
            let bytes = &mut warrior.bytes;
            let energy = word(bytes, 0x58)?;
            let maximum = word(bytes, 0x5a)?;
            if energy < maximum && word(bytes, 0x50)? > 0 && word(bytes, 0x90)? == 0 {
                let level = word(bytes, 0x3a)?;
                let amount = (level.wrapping_add((level as u16 >> 15) as i16) >> 1) as i8;
                put(
                    bytes,
                    0x58,
                    energy.wrapping_add(i16::from(amount.max(1))).min(maximum),
                );
            }
        }
        if position.hour == 24 {
            position.hour = 0;
            position.day = position.day.wrapping_add(1);
            let mut cursor = 0;
            loop {
                let selected = scan_schedule(state, &mut cursor, &mut events)?;
                recover(state, content)?;
                let Some(script_id) = selected else { break };
                let map = content
                    .map(
                        state
                            .exploration
                            .as_ref()
                            .map_or(0, |position| position.map),
                    )
                    .ok_or_else(|| RuleError::Invalid("schedule content unavailable".into()))?;
                let record = centurions_content::span(
                    &map.scheduled_scripts,
                    usize::from(script_id.unsigned_abs()) * 42,
                    42,
                )?
                .to_vec();
                state.exploration.as_mut().unwrap().script = Some(ScriptState {
                    record,
                    cursor: 0,
                    map_slot: 0,
                    scheduled: true,
                    menu: None,
                    persist: false,
                    origin: None,
                });
                events.extend(super::exploration::run_script(state, content)?);
                if state.exploration.as_ref().unwrap().presentation.is_some() {
                    return Err(RuleError::Unsupported {
                        action: "time.schedule",
                        dependency: "interactive scheduled-event continuation",
                    });
                }
            }
        } else if position.hour == 12 {
            recover(state, content)?;
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

fn recover(state: &mut Session, content: &Content) -> Result<(), RuleError> {
    for index in 0..state.warriors.len() {
        let bytes = &state.warriors[index].bytes;
        let stamina = word(bytes, 0x50)?;
        let maximum = word(bytes, 0x52)?;
        if stamina <= -10 || stamina >= maximum {
            continue;
        }
        let level = word(bytes, 0x3a)?;
        let mut amount = if word(bytes, 0x32)? == 3 {
            level
        } else {
            level / 4
        } as i8;
        if !consume_food(state, content)? {
            amount /= 2;
        }
        let healing =
            i16::from(amount.max(1)).wrapping_add(word(&state.warriors[index].bytes, 0x66)?);
        let context = crate::story_health::context(
            &state.warriors,
            state.selected_warrior,
            state.health_context.as_ref(),
        );
        let healed = crate::story_health::heal_outdoors(
            &state.warriors,
            index,
            healing,
            false,
            &context,
            &mut state.random,
        )?;
        if !healed.events.is_empty() {
            return Err(RuleError::Unsupported {
                action: "time.recover",
                dependency: "terminal recovery presentation",
            });
        }
        state.warriors = healed.warriors;
        state.health_context = Some(healed.context);
    }
    Ok(())
}

fn consume_food(state: &mut Session, content: &Content) -> Result<bool, RuleError> {
    for warrior in &mut state.warriors {
        for (slot, item) in warrior.inventory()?.iter().enumerate() {
            if item.id() != 800 {
                continue;
            }
            let quantity = word(&item.0, 2)?;
            if quantity <= 0 {
                return Ok(false);
            }
            let definition = content
                .items
                .get(800)
                .ok_or_else(|| RuleError::Invalid("food definition unavailable".into()))?;
            let weight = word(definition, 0x24)?;
            let carried = word(&warrior.bytes, 0x9d6)?.wrapping_sub(weight);
            put(&mut warrior.bytes, 0x9d6, carried);
            put(&mut warrior.bytes, 0x132 + slot * 22 + 2, quantity - 1);
            if quantity == 1 && word(definition, 0x26)? != 0 {
                super::items::remove_inventory_slot(warrior, slot)?;
            }
            state.load_item_work(content, 800)?;
            return Ok(true);
        }
    }
    Ok(false)
}

fn scan_schedule(
    state: &mut Session,
    cursor: &mut usize,
    events: &mut Vec<Event>,
) -> Result<Option<i16>, RuleError> {
    let position = state.exploration.as_ref().unwrap();
    let day = position.day;
    let count = position.schedule.len() / 40;
    while *cursor < count {
        let slot = *cursor;
        *cursor += 1;
        let record =
            state.exploration.as_ref().unwrap().schedule[slot * 40..slot * 40 + 40].to_vec();
        let due = word(&record, 0)?;
        if due != day {
            if due == 0 {
                break;
            }
            continue;
        }
        let next_day = due.wrapping_add(word(&record, 2)?);
        let mut eligible = state.random.bound(100) <= word(&record, 4)?;
        if eligible && word(&record, 16)? > 0 {
            let required = word(&record, 16)?;
            eligible = state.warriors.iter().any(|w| {
                w.inventory()
                    .is_ok_and(|items| items.iter().any(|item| item.id() == required))
            });
        }
        if eligible && word(&record, 18)? >= 0 {
            return Err(RuleError::Unsupported {
                action: "time.schedule",
                dependency: "mission flag prerequisite",
            });
        }
        if eligible && word(&record, 20)? > 0 {
            let position = state.exploration.as_ref().unwrap();
            if word(&record, 20)? != 1 || i32::from(word(&record, 8)?) != position.map {
                continue;
            }
            let region = word(&record, 10)?;
            if region >= 0 {
                return Err(RuleError::Unsupported {
                    action: "time.schedule",
                    dependency: "schedule region prerequisite",
                });
            }
            let x = word(&record, 12)?;
            if x >= 0 && (position.x != i32::from(x) || position.y != i32::from(word(&record, 14)?))
            {
                continue;
            }
        }
        put(
            &mut state.exploration.as_mut().unwrap().schedule,
            slot * 40,
            next_day,
        );
        events.push(Event::ScheduleRescheduled {
            slot,
            next_day,
            dispatched: eligible,
        });
        if eligible {
            return Ok(Some(word(&record, 6)?));
        }
        *cursor = count;
        break;
    }
    Ok(None)
}
#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../tests/unit/time_reference.rs"]
mod tests;
