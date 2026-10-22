use crate::{Content, Event, Phase, RuleError, Session, combat};
use centurions_content::records::{Condition, ExtraTable, RetreatClassesExtra};

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

/// Native newland body operations share the queued origin even after an encounter choice.
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
    let combat = state.combat.as_mut().ok_or("combat story roster missing")?;
    if opcode == 119 {
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
    let (center, faction) = if matches!(opcode, 123 | 124) {
        if let Some(story) = &state.combat_story {
            (
                (
                    story.entry.x as i16 - combat.map.shift.0 - 5,
                    story.entry.y as i16 - combat.map.shift.1 - 5,
                ),
                story.entry.faction,
            )
        } else {
            let index = combat.script_enemy;
            let record = combat.story_record(index)?;
            (combat.story_position(index)?, record.record().faction()?)
        }
    } else {
        ((0, 0), 0)
    };
    let extra = ExtraTable(
        &map.combat
            .as_ref()
            .ok_or("combat story tables missing")?
            .encounters,
    )
    .by_magnitude(parameter)?;
    match opcode {
        122 => combat::round_stories::neutralize(combat, extra)?,
        123 => {
            for class in RetreatClassesExtra(extra)
                .classes()
                .filter(|class| *class != 0)
            {
                for record in &mut combat.enemies.records {
                    if i16::from(record.record().definition_id()) == class
                        && record.record().faction()? == faction
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
                center,
                if state.round_dispatch.is_some() && state.combat_story.is_none() {
                    -1
                } else {
                    faction
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
                state
                    .native_work
                    .health_context
                    .as_ref()
                    .map(|health| health.last_body),
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
