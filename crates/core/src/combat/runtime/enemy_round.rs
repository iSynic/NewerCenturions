//! Enemy round in the original combat execution order.
use super::death::enemy_death_feedback;
use super::{FieldActor, collide_fields_impl};
use crate::combat::{Combat, lifecycle, put_word};
use crate::{Event, RandomState, RuleError};
use centurions_content::Warrior;

#[derive(Debug, PartialEq, Eq)]
pub struct EnemyRoundOutcome {
    pub events: Vec<Event>,
    pub last_defeated_body: Option<i16>,
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
    selected_warrior: usize,
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
    let events = reduce_enemy_round_impl(
        &mut next,
        &mut next_warriors,
        &mut next_random,
        mission,
        &mut next_count,
        &mut next_mask,
        selected_warrior,
    )?;
    *c = next;
    w.clone_from_slice(&next_warriors);
    *random = next_random;
    *count = next_count;
    *party_mask = next_mask;
    Ok(events)
}

pub(super) fn reduce_enemy_round_impl(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    mission: Option<&centurions_content::mission::OpeningMission>,
    count: &mut i16,
    party_mask: &mut [i16; 6],
    selected_warrior: usize,
) -> Result<EnemyRoundOutcome, RuleError> {
    let mut events = Vec::new();
    let mut last_defeated_body = None;
    for index in 0..c.enemies.records.len() {
        let m = &mut c.enemies.records[index];
        for o in 0xb2..0xda {
            if (m.bytes[o] as i8) > 0 {
                m.bytes[o] -= 1;
            }
        }
        c.enemies.records[index].bytes[0x105] = 0;
        if let Some(heal) =
            crate::combat::natural_conditions::enemy_poison_heal(&c.enemies.records[index])?
        {
            put_word(&mut c.enemies.records[index].bytes, 0x110, heal.health);
            c.enemies.records[index].bytes[0x105] = 1;
            events.push(Event::Sound { resource_id: 624 });
        }
        if let Some(tick) =
            crate::combat::natural_conditions::enemy_drain_tick(&c.enemies.records[index].bytes)?
        {
            put_word(&mut c.enemies.records[index].bytes, 0x110, tick.health);
            events.push(Event::Sound { resource_id: 107 });
            if tick.killed {
                let feedback = enemy_death_feedback(&c.enemies.records[index], index as i16 + 10)?;
                lifecycle::kill_body(c, w, index as i16 + 10)?;
                last_defeated_body = Some(index as i16 + 10);
                events.extend(feedback);
            }
        }
        if let Some(tick) =
            crate::combat::natural_conditions::enemy_poison_tick(&c.enemies.records[index])?
        {
            put_word(&mut c.enemies.records[index].bytes, 0x110, tick.health);
            c.enemies.records[index].bytes[0x105] = 1;
            events.push(Event::Sound { resource_id: 104 });
            if tick.killed {
                let feedback = enemy_death_feedback(&c.enemies.records[index], index as i16 + 10)?;
                lifecycle::kill_body(c, w, index as i16 + 10)?;
                last_defeated_body = Some(index as i16 + 10);
                events.extend(feedback);
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
                        selected_warrior: Some(selected_warrior),
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
        c.enemies.records[index].bytes[0x105] = 0;
    }
    // The native loop also visits the known zero records after the represented bodies.
    if c.enemies.records.len() < 100 {
        c.field_used = crate::combat::spells::reset_used(&c.field_used)?;
    }
    Ok(EnemyRoundOutcome {
        events,
        last_defeated_body,
    })
}
