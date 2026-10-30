//! The monsters' health and conditions at the start of a round.
use super::death::enemy_death_feedback;
use super::{FieldActor, collide_fields_impl};
use crate::combat::natural_conditions;
use crate::combat::spells;
use crate::combat::spells::summon::ItemRefresh;
use crate::combat::{Combat, death_scripts::DeathProcessing, lifecycle};
use crate::{Event, RandomState, RuleError};
use centurions_content::WARRIOR_BYTES;
use centurions_content::Warrior;
use centurions_content::combat::MONSTER_BYTES;
use serde::{Deserialize, Serialize};

/// What the monsters' round start produced: the events, the last monster to fall and, when a death
/// story must run first, where to resume.
#[derive(Debug, PartialEq, Eq)]
pub struct EnemyRoundOutcome {
    pub events: Vec<Event>,
    pub last_defeated_body: Option<i16>,
    /// A death in getup's health loop, whose death story runs before the loop resumes.
    pub suspended: Option<EnemyRoundCursor>,
}

/// Where a monster's step in getup's health loop resumes after its death story.
///
/// Original: getup
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EnemyRoundStage {
    #[default]
    Start,
    AfterDrain,
    AfterPoison,
}

/// Where the monsters' health loop resumes after a death story: which monster and which step.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnemyRoundCursor {
    pub index: usize,
    pub stage: EnemyRoundStage,
}

/// Remembers the last body that a spell in `events` defeated.
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

/// Runs the monsters' start of round for a record-only battle, checking its inputs first and
/// leaving the battle untouched when anything fails.
///
/// Original: getup
pub fn reduce_enemy_round_with_fields(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    mission: Option<&centurions_content::mission::OpeningMission>,
    count: &mut i16,
    party_track: &mut [i16; 6],
    (char_select_new, movement_gate): (usize, &mut i16),
) -> Result<EnemyRoundOutcome, RuleError> {
    if combat.roster.records.len() > 100
        || combat.roster.mon_pos.len() != combat.roster.records.len()
        || combat
            .roster
            .records
            .iter()
            .any(|record| record.bytes.len() != MONSTER_BYTES)
        || !(1..=6).contains(&warriors.len())
        || warriors
            .iter()
            .any(|warrior| warrior.bytes.len() != WARRIOR_BYTES)
        || char_select_new >= warriors.len()
    {
        return Err(RuleError::Invalid("invalid enemy round input".into()));
    }
    combat.validate(warriors)?;
    let mut next = combat.clone();
    let mut next_warriors = warriors.to_vec();
    let mut next_random = random.clone();
    let mut next_count = *count;
    let mut next_mask = *party_track;
    let mut next_gate = *movement_gate;
    let events = reduce_enemy_round_impl(
        &mut next,
        &mut next_warriors,
        &mut next_random,
        mission,
        &mut next_count,
        &mut next_mask,
        (char_select_new, None, &mut next_gate, None),
        EnemyRoundCursor::default(),
        None,
    )?;
    *movement_gate = next_gate;
    *combat = next;
    warriors.clone_from_slice(&next_warriors);
    *random = next_random;
    *count = next_count;
    *party_track = next_mask;
    Ok(events)
}

/// Runs each monster's start of round: temporary conditions tick down, poison heals or harms, a psi
/// drain wears on, a monster whose stamina runs out is killed, its psi points recover, and a
/// monster still standing meets the spell fields in its cell. A death that starts a story suspends
/// the loop to be resumed after it.
///
/// Original: getup
#[allow(clippy::too_many_arguments)]
pub(super) fn reduce_enemy_round_impl(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    mission: Option<&centurions_content::mission::OpeningMission>,
    count: &mut i16,
    party_track: &mut [i16; 6],
    mut actor: (usize, Option<i16>, &mut i16, ItemRefresh<'_>),
    start: EnemyRoundCursor,
    processing: Option<DeathProcessing>,
) -> Result<EnemyRoundOutcome, RuleError> {
    let mut events = Vec::new();
    let mut last_defeated_body = None;
    // Live getup supplies its death predicate; record-only callers keep queueing.
    let kill =
        |combat: &mut Combat, warriors: &mut [Warrior], index: usize, events: &mut Vec<Event>| {
            let feedback = enemy_death_feedback(&combat.roster.records[index], index as i16 + 10)?;
            lifecycle::kill_body_with_processing(combat, warriors, index as i16 + 10, processing)?;
            events.extend(feedback);
            Ok::<_, RuleError>(processing.is_some() && combat.direct_death.is_some())
        };
    for index in start.index..combat.roster.records.len() {
        let stage = if index == start.index {
            start.stage
        } else {
            EnemyRoundStage::Start
        };
        if stage == EnemyRoundStage::Start {
            let monster = &mut combat.roster.records[index];
            monster.record_mut().decrement_temporary_conditions();
            monster.record_mut().clear_been_attacked();
            if let Some(heal) =
                natural_conditions::enemy_poison_heal(&combat.roster.records[index])?
            {
                combat.roster.records[index]
                    .record_mut()
                    .set_stamina(heal.health);
                combat.roster.records[index]
                    .record_mut()
                    .mark_been_attacked();
                events.push(Event::Sound { resource_id: 624 });
            }
            if let Some(tick) =
                natural_conditions::enemy_drain_tick(&combat.roster.records[index].bytes)?
            {
                combat.roster.records[index]
                    .record_mut()
                    .set_stamina(tick.health);
                events.push(Event::Sound { resource_id: 107 });
                if tick.killed {
                    last_defeated_body = Some(index as i16 + 10);
                    if kill(combat, warriors, index, &mut events)? {
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
                natural_conditions::enemy_poison_tick(&combat.roster.records[index])?
        {
            combat.roster.records[index]
                .record_mut()
                .set_stamina(tick.health);
            combat.roster.records[index]
                .record_mut()
                .mark_been_attacked();
            events.push(Event::Sound { resource_id: 104 });
            if tick.killed {
                last_defeated_body = Some(index as i16 + 10);
                if kill(combat, warriors, index, &mut events)? {
                    return Ok(suspended(
                        events,
                        last_defeated_body,
                        index,
                        EnemyRoundStage::AfterPoison,
                    ));
                }
            }
        }
        natural_conditions::reduce_enemy_psi(&mut combat.roster.records[index].bytes)?;
        combat.collide_check = spells::reset_used(&combat.collide_check)?;
        if combat.roster.records[index].record().stamina()? > 0 {
            if let Some(mission) = mission {
                let collision_events = collide_fields_impl(
                    combat,
                    warriors,
                    mission,
                    count,
                    random,
                    FieldActor {
                        body: index as i16 + 10,
                        char_select_new: actor.0,
                        item_restriction: actor.1,
                        movement_gate: &mut *actor.2,
                        items: actor
                            .3
                            .as_mut()
                            .map(|(items, buffers)| (*items, &mut **buffers)),
                    },
                    party_track,
                )?;
                record_spell_deaths(&collision_events, &mut last_defeated_body);
                events.extend(collision_events);
            } else if combat
                .spell_queue
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
        combat.roster.records[index]
            .record_mut()
            .clear_been_attacked();
    }
    // The loop visits all 100 monster records, including the empty ones.
    if combat.roster.records.len() < 100 {
        combat.collide_check = spells::reset_used(&combat.collide_check)?;
    }
    Ok(EnemyRoundOutcome {
        events,
        last_defeated_body,
        suspended: None,
    })
}

/// A round start suspended at `index` and `stage`, waiting for a death story.
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
