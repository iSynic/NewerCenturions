use crate::{Content, Event, Phase, RuleError, Session, combat};
use centurions_content::records::{Condition, ExtraTable, RetreatClassesExtra};
use centurions_content::word;

#[derive(Default)]
struct Origin {
    center: (i16, i16),
    faction: i16,
    queued: bool,
}

/// newland127 continues a macro only while a living enemy carries the definition
/// ID (BYTE 0x103, compared as a signed word with the parameter).
pub(crate) fn monster_present(combat: &combat::Combat, id: i16) -> Result<bool, RuleError> {
    for enemy in &combat.enemies.records {
        if i16::from(enemy.record().definition_id()) == id && enemy.record().stamina()? > 0 {
            return Ok(true);
        }
    }
    Ok(false)
}

/// newland123/124 choose the live NPC or retained queue entry using BYTE438584.
fn origin(state: &Session, with_position: bool) -> Result<Origin, RuleError> {
    let queued = state.native_work.death_processing.queue != 0;
    let combat = state.combat.as_ref().ok_or("combat story roster missing")?;
    if !queued {
        let index = combat.script_enemy;
        return Ok(Origin {
            center: if with_position {
                combat.story_position(index)?
            } else {
                (0, 0)
            },
            faction: combat.story_record(index)?.record().faction()?,
            queued: false,
        });
    }
    // The round macro (getup 125d7e) and timeclick's calendar call (15e8d8) push a
    // zero argument, as do their callers' other newland entries.
    let calendar = state
        .zero_time
        .as_ref()
        .is_some_and(|frame| frame.owners.is_some() && frame.innermost(state));
    let argument = if calendar {
        Some(0)
    } else if let Some(turn) = state
        .turn_dispatch
        .as_ref()
        .filter(|frame| frame.owns_script(state))
    {
        Some(turn.body - 10)
    } else if state
        .round_dispatch
        .as_ref()
        .is_some_and(|frame| frame.owns_script(state))
    {
        Some(0)
    } else {
        state
            .combat_story
            .as_ref()
            .filter(|story| story.shared_dispatch && !story.waiting_child)
            .and_then(|story| {
                if story.direct {
                    Some(story.body as i16 + 10)
                } else {
                    story.queue_cursor.map(|index| -(index as i16 + 1))
                }
            })
    }
    .ok_or("queued origin without a newland caller")?;
    // abs(0) - 1 reads the fourteen bytes before the queue at 0x437a96.
    if argument == 0 {
        return Err(RuleError::Unsupported {
            action: "story.combat",
            dependency: "queued origin before the death queue from a zero newland argument",
        });
    }
    let entry = argument
        .checked_abs()
        .and_then(|value| value.checked_sub(1))
        .and_then(|index| usize::try_from(index).ok())
        .and_then(|index| combat.death_queue.entries.get(index))
        .ok_or(RuleError::Unsupported {
            action: "story.combat",
            dependency: "queued origin outside retained death entries",
        })?;
    Ok(Origin {
        center: (
            (entry.x as i16)
                .wrapping_sub(combat.map.shift.0)
                .wrapping_sub(5),
            (entry.y as i16)
                .wrapping_sub(combat.map.shift.1)
                .wrapping_sub(5),
        ),
        faction: entry.faction,
        queued: true,
    })
}

/// Native newland body operations, with live versus queued origin kept separate.
pub(crate) fn apply(
    state: &mut Session,
    content: &Content,
    opcode: i16,
    parameter: i16,
) -> Result<(Vec<Event>, bool), RuleError> {
    if state.phase != Phase::Combat && opcode != 119 {
        return Ok((vec![], false));
    }
    let map = state.land(content).ok_or("combat story map unavailable")?;
    let turn_owns = state
        .turn_dispatch
        .as_ref()
        .is_some_and(|frame| frame.owns_script(state));
    if opcode == 119 {
        let combat = state.combat.as_mut().ok_or("combat story roster missing")?;
        if state.warriors.len() as i16 - 1 < i16::from(combat.unavailable) {
            for warrior in &mut state.warriors {
                warrior.record_mut().set_stamina(1);
                warrior.record_mut().set_automatic_mode(0);
            }
            combat.unavailable = 0;
            return Ok((vec![], true));
        }
        let record = combat.story_record_mut(combat.script_enemy)?;
        record.record_mut().set_stamina(1);
        record.record_mut().set_faction(0);
        record.record_mut().set_target_scan_excluded(true);
        return Ok((vec![], false));
    }
    let extra = ExtraTable(
        &map.combat
            .as_ref()
            .ok_or("combat story tables missing")?
            .encounters,
    )
    .by_magnitude(parameter)?;
    state.native_work.extra_record = extra.to_vec();
    let combat = state.combat.as_ref().ok_or("combat story roster missing")?;
    let needs_origin = match opcode {
        123 => RetreatClassesExtra(extra)
            .classes()
            .filter(|class| *class != 0)
            .any(|class| {
                combat
                    .enemies
                    .records
                    .iter()
                    .any(|record| i16::from(record.record().definition_id()) == class)
            }),
        124 => combat.enemies.records.len() < 100 && word(extra, 4)? != 0,
        _ => false,
    };
    let source = if needs_origin {
        origin(state, opcode == 124)?
    } else {
        Origin::default()
    };
    let combat = state.combat.as_mut().unwrap();
    match opcode {
        122 => combat::round_stories::neutralize(combat, extra)?,
        123 => {
            for class in RetreatClassesExtra(extra)
                .classes()
                .filter(|class| *class != 0)
            {
                for record in &mut combat.enemies.records {
                    if i16::from(record.record().definition_id()) == class
                        && record.record().faction()? == source.faction
                    {
                        record.record_mut().set_condition(Condition::Retreating, -1);
                        record.record_mut().set_fleeing_chance(70);
                    }
                }
            }
        }
        124 => {
            let mut result = combat::round_stories::RoundOutcome::default();
            combat::round_stories::spawn_at(
                combat,
                map,
                &map.combat.as_ref().unwrap().monsters,
                extra,
                source.center,
                if !source.queued
                    && state.round_dispatch.is_some()
                    && state.combat_story.is_none()
                    && !turn_owns
                {
                    -1
                } else {
                    source.faction
                },
                &mut state.random,
                &mut result,
            )?;
            let events = result
                .effects
                .into_iter()
                .map(|effect| {
                    Ok(match effect {
                        combat::round_stories::RoundEffect::Sound(resource_id) => {
                            Event::Sound { resource_id }
                        }
                        combat::round_stories::RoundEffect::Text(id) => Event::Message {
                            id,
                            text: map.dialogue(id)?,
                        },
                        combat::round_stories::RoundEffect::Spawn { body, monster } => {
                            Event::CombatantAdded {
                                body,
                                definition: monster,
                            }
                        }
                        combat::round_stories::RoundEffect::Removed(body) => {
                            Event::CombatantRemoved { body }
                        }
                        combat::round_stories::RoundEffect::Dispatch { .. } => {
                            return Err(RuleError::Invalid("unexpected summon dispatch".into()));
                        }
                    })
                })
                .collect::<Result<Vec<_>, RuleError>>()?;
            return Ok((events, false));
        }
        125 => {
            let events = combat::round_stories::destroy_class(
                combat,
                extra,
                Some(state.native_work.health_context.last_body),
                &state.native_work.tactical_work.positions,
                &mut state.random,
            )?
            .into_iter()
            .map(|body| Event::CombatantRemoved { body })
            .collect();
            return Ok((events, false));
        }
        _ => return Err(RuleError::Invalid("invalid combat story operation".into())),
    }
    Ok((vec![], false))
}
