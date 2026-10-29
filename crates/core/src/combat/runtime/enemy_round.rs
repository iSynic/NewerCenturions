//! Enemy round in the original combat execution order.
use super::death::enemy_death_feedback;
use super::{FieldActor, collide_fields_impl};
use crate::combat::{Combat, death_scripts::DeathProcessing, lifecycle};
use crate::{Event, RandomState, RuleError};
use centurions_content::Warrior;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq)]
pub struct EnemyRoundOutcome {
    pub events: Vec<Event>,
    pub last_defeated_body: Option<i16>,
    /// getup's health loop called killbody directly; its newland runs before this point.
    pub suspended: Option<EnemyRoundCursor>,
}

/// The step after a direct killbody inside one NPC iteration of getup's health loop.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EnemyRoundStage {
    #[default]
    Start,
    AfterDrain,
    AfterPoison,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnemyRoundCursor {
    pub index: usize,
    pub stage: EnemyRoundStage,
}

pub(super) fn record_spell_deaths(events: &[Event], last_body: &mut Option<i16>) {
    for event in events {
        if let Event::SpellResolved {
            target,
            defeated: true,
            ..
        } = event
        {
            *last_body = Some(*target);
        }
    }
}

pub fn reduce_enemy_round_with_fields(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    mission: Option<&centurions_content::mission::OpeningMission>,
    count: &mut i16,
    party_mask: &mut [i16; 6],
    (selected_warrior, movement_gate): (usize, &mut i16),
) -> Result<EnemyRoundOutcome, RuleError> {
    if c.enemies.records.len() > 100
        || c.enemies.positions.len() != c.enemies.records.len()
        || c.enemies
            .records
            .iter()
            .any(|record| record.bytes.len() != 288)
        || !(1..=6).contains(&w.len())
        || w.iter().any(|warrior| warrior.bytes.len() != 3138)
        || selected_warrior >= w.len()
    {
        return Err(RuleError::Invalid("invalid enemy round input".into()));
    }
    c.validate(w)?;
    let mut next = c.clone();
    let mut next_warriors = w.to_vec();
    let mut next_random = random.clone();
    let mut next_count = *count;
    let mut next_mask = *party_mask;
    let mut next_gate = *movement_gate;
    let events = reduce_enemy_round_impl(
        &mut next,
        &mut next_warriors,
        &mut next_random,
        mission,
        &mut next_count,
        &mut next_mask,
        (selected_warrior, None, &mut next_gate, None),
        EnemyRoundCursor::default(),
        None,
    )?;
    *movement_gate = next_gate;
    *c = next;
    w.clone_from_slice(&next_warriors);
    *random = next_random;
    *count = next_count;
    *party_mask = next_mask;
    Ok(events)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn reduce_enemy_round_impl(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    mission: Option<&centurions_content::mission::OpeningMission>,
    count: &mut i16,
    party_mask: &mut [i16; 6],
    mut actor: (
        usize,
        Option<i16>,
        &mut i16,
        crate::combat::spell_summon::ItemRefresh<'_>,
    ),
    start: EnemyRoundCursor,
    processing: Option<DeathProcessing>,
) -> Result<EnemyRoundOutcome, RuleError> {
    let mut events = Vec::new();
    let mut last_defeated_body = None;
    // Live getup supplies its death predicate; record-only callers keep queueing.
    let kill = |c: &mut Combat, w: &mut [Warrior], index: usize, events: &mut Vec<Event>| {
        let feedback = enemy_death_feedback(&c.enemies.records[index], index as i16 + 10)?;
        lifecycle::kill_body_with_processing(c, w, index as i16 + 10, processing)?;
        events.extend(feedback);
        Ok::<_, RuleError>(processing.is_some() && c.direct_death.is_some())
    };
    for index in start.index..c.enemies.records.len() {
        let stage = if index == start.index {
            start.stage
        } else {
            EnemyRoundStage::Start
        };
        if stage == EnemyRoundStage::Start {
            let enemy = &mut c.enemies.records[index];
            enemy.record_mut().decrement_temporary_conditions();
            enemy.record_mut().clear_damage_marker();
            if let Some(heal) =
                crate::combat::natural_conditions::enemy_poison_heal(&c.enemies.records[index])?
            {
                c.enemies.records[index]
                    .record_mut()
                    .set_stamina(heal.health);
                c.enemies.records[index].record_mut().mark_damaged();
                events.push(Event::Sound { resource_id: 624 });
            }
            if let Some(tick) = crate::combat::natural_conditions::enemy_drain_tick(
                &c.enemies.records[index].bytes,
            )? {
                c.enemies.records[index]
                    .record_mut()
                    .set_stamina(tick.health);
                events.push(Event::Sound { resource_id: 107 });
                if tick.killed {
                    last_defeated_body = Some(index as i16 + 10);
                    if kill(c, w, index, &mut events)? {
                        return Ok(suspended(
                            events,
                            last_defeated_body,
                            index,
                            EnemyRoundStage::AfterDrain,
                        ));
                    }
                }
            }
        }
        if stage <= EnemyRoundStage::AfterDrain
            && let Some(tick) =
                crate::combat::natural_conditions::enemy_poison_tick(&c.enemies.records[index])?
        {
            c.enemies.records[index]
                .record_mut()
                .set_stamina(tick.health);
            c.enemies.records[index].record_mut().mark_damaged();
            events.push(Event::Sound { resource_id: 104 });
            if tick.killed {
                last_defeated_body = Some(index as i16 + 10);
                if kill(c, w, index, &mut events)? {
                    return Ok(suspended(
                        events,
                        last_defeated_body,
                        index,
                        EnemyRoundStage::AfterPoison,
                    ));
                }
            }
        }
        crate::combat::natural_conditions::reduce_enemy_psi(&mut c.enemies.records[index].bytes)?;
        c.field_used = crate::combat::spells::reset_used(&c.field_used)?;
        if c.enemies.records[index].record().stamina()? > 0 {
            if let Some(mission) = mission {
                let collision_events = collide_fields_impl(
                    c,
                    w,
                    mission,
                    count,
                    random,
                    FieldActor {
                        body: index as i16 + 10,
                        selected_warrior: actor.0,
                        item_work_alias: actor.1,
                        movement_gate: &mut *actor.2,
                        items: actor
                            .3
                            .as_mut()
                            .map(|(items, buffers)| (*items, &mut **buffers)),
                    },
                    party_mask,
                )?;
                record_spell_deaths(&collision_events, &mut last_defeated_body);
                events.extend(collision_events);
            } else if c
                .spell_fields
                .as_chunks::<20>()
                .0
                .iter()
                .any(|field| field[15] != 0)
            {
                return Err(RuleError::Unsupported {
                    action: "combat.round",
                    dependency: "spell field content missing",
                });
            }
        }
        c.enemies.records[index].record_mut().clear_damage_marker();
    }
    // The native loop also visits the known zero records after the represented bodies.
    if c.enemies.records.len() < 100 {
        c.field_used = crate::combat::spells::reset_used(&c.field_used)?;
    }
    Ok(EnemyRoundOutcome {
        events,
        last_defeated_body,
        suspended: None,
    })
}

fn suspended(
    events: Vec<Event>,
    last_defeated_body: Option<i16>,
    index: usize,
    stage: EnemyRoundStage,
) -> EnemyRoundOutcome {
    EnemyRoundOutcome {
        events,
        last_defeated_body,
        suspended: Some(EnemyRoundCursor { index, stage }),
    }
}
