use crate::exploration::Operation;
use crate::{Content, Event, Phase, RuleError, Session, combat};
use centurions_content::records::{Condition, ExtraTable, RetreatClassesExtra};
use centurions_content::word;

/// Where a summoned or converted monster comes from: its position and faction, and whether it came
/// off the death queue.
#[derive(Default)]
struct Origin {
    center: (i16, i16),
    faction: i16,
    queued: bool,
}

/// Continue If Monster keeps a battle script running only while a living enemy
/// has the monster id, compared as a signed word with the parameter.
pub(crate) fn monster_present(combat: &combat::Combat, id: i16) -> Result<bool, RuleError> {
    for monster in &combat.roster.records {
        if i16::from(monster.record().monster_id()) == id && monster.record().stamina()? > 0 {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Faction and summon operations act on the live monster, or on the death queue entry
/// while the queue is being emptied (`doing_que`).
fn origin(state: &Session, with_position: bool) -> Result<Origin, RuleError> {
    let queued = state.globals.death_processing.doing_queue != 0;
    let combat = state.combat.as_ref().ok_or("combat story roster missing")?;
    if !queued {
        let index = combat.macro_monster;
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
    // Round scripts and calendar scripts pass no caller, like their other script
    // calls.
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
    // A zero caller reads the entry before the death queue, an original quirk
    // (see docs/source-notes/rules.md).
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
        .and_then(|index| combat.todo_queue.entries.get(index))
        .ok_or(RuleError::Unsupported {
            action: "story.combat",
            dependency: "queued origin outside retained death entries",
        })?;
    Ok(Origin {
        center: (
            (entry.x as i16)
                .wrapping_sub(combat.field.map_shift.0)
                .wrapping_sub(5),
            (entry.y as i16)
                .wrapping_sub(combat.field.map_shift.1)
                .wrapping_sub(5),
        ),
        faction: entry.faction,
        queued: true,
    })
}

/// The battle script operations, keeping a live origin apart from a queued one.
///
/// Original: newland
pub(crate) fn apply(
    state: &mut Session,
    content: &Content,
    operation: Operation,
    parameter: i16,
) -> Result<(Vec<Event>, bool), RuleError> {
    if state.phase != Phase::Combat && operation != Operation::EndCombat {
        return Ok((vec![], false));
    }
    let map = state.land(content).ok_or("combat story map unavailable")?;
    let turn_owns = state
        .turn_dispatch
        .as_ref()
        .is_some_and(|frame| frame.owns_script(state));
    if operation == Operation::EndCombat {
        let combat = state.combat.as_mut().ok_or("combat story roster missing")?;
        if state.warriors.len() as i16 - 1 < i16::from(combat.kill_party) {
            for warrior in &mut state.warriors {
                warrior.record_mut().set_stamina(1);
                warrior.record_mut().set_computer_control(0);
            }
            combat.kill_party = 0;
            return Ok((vec![], true));
        }
        let record = combat.story_record_mut(combat.macro_monster)?;
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
    state.globals.extra_record = extra.to_vec();
    let combat = state.combat.as_ref().ok_or("combat story roster missing")?;
    let needs_origin = match operation {
        Operation::SetFactionCondition => RetreatClassesExtra(extra)
            .classes()
            .filter(|class| *class != 0)
            .any(|class| {
                combat
                    .roster
                    .records
                    .iter()
                    .any(|record| i16::from(record.record().monster_id()) == class)
            }),
        Operation::SummonActors => combat.roster.records.len() < 100 && word(extra, 4)? != 0,
        _ => false,
    };
    let source = if needs_origin {
        origin(state, operation == Operation::SummonActors)?
    } else {
        Origin::default()
    };
    let combat = state.combat.as_mut().unwrap();
    match operation {
        Operation::NeutralizeActors => combat::round_stories::neutralize(combat, extra)?,
        Operation::SetFactionCondition => {
            for class in RetreatClassesExtra(extra)
                .classes()
                .filter(|class| *class != 0)
            {
                for record in &mut combat.roster.records {
                    if i16::from(record.record().monster_id()) == class
                        && record.record().faction()? == source.faction
                    {
                        record.record_mut().set_condition(Condition::Retreating, -1);
                        record.record_mut().set_surrender_percent(70);
                    }
                }
            }
        }
        Operation::SummonActors => {
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
        Operation::RemoveActorClass => {
            let events = combat::round_stories::destroy_class(
                combat,
                extra,
                Some(state.globals.health_context.last_body),
                &state.globals.battlefield.pos,
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
