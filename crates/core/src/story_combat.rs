use crate::{Content, Event, Phase, RuleError, Session, combat};
use centurions_content::{span, word};

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
    let map = state
        .map(content, state.exploration.as_ref().map_or(0, |p| p.map))
        .ok_or("combat story map unavailable")?;
    let combat = state.combat.as_mut().ok_or("combat story roster missing")?;
    if opcode == 119 {
        if state.warriors.len() as i16 - 1 < i16::from(combat.unavailable) {
            for warrior in &mut state.warriors {
                warrior.bytes[0x50..0x52].copy_from_slice(&1_i16.to_be_bytes());
                warrior.bytes[0x90..0x92].fill(0);
            }
            combat.unavailable = 0;
            return Ok((vec![], true));
        }
        let record = combat.story_record_mut(combat.script_enemy)?;
        record.bytes[0x110..0x112].copy_from_slice(&1_i16.to_be_bytes());
        record.bytes[14..16].fill(0);
        record.bytes[0x82] = 1;
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
            (combat.story_position(index)?, word(&record.bytes, 14)?)
        }
    } else {
        ((0, 0), 0)
    };
    let extra = span(
        &map.combat
            .as_ref()
            .ok_or("combat story tables missing")?
            .encounters,
        parameter.unsigned_abs() as usize * 10,
        10,
    )?;
    match opcode {
        122 => combat::round_stories::neutralize(combat, extra)?,
        123 => {
            for class in extra
                .as_chunks::<2>()
                .0
                .iter()
                .map(|bytes| i16::from_be_bytes(*bytes))
                .filter(|class| *class != 0)
            {
                for record in &mut combat.enemies.records {
                    if i16::from(record.bytes[0x103] as i8) == class
                        && word(&record.bytes, 14)? == faction
                    {
                        record.bytes[0xb2] = 255;
                        record.bytes[0x46..0x48].copy_from_slice(&70_i16.to_be_bytes());
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
                state.health_context.as_ref().map(|health| health.last_body),
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
