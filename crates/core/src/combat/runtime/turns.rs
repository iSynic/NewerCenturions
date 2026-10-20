//! Turns in the original combat execution order.
use super::{advance_round, alive, finish_action};
use crate::combat::{Combat, initialize_turn};
use crate::{Event, RandomState, RuleError, Session};
use centurions_content::{Content, Warrior};

pub fn initialize_active_turn(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
) -> Result<(Vec<Event>, bool), RuleError> {
    let body = c
        .active_body()
        .ok_or_else(|| RuleError::Invalid("active combatant missing".into()))?;
    let tables = mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat tables missing".into()))?;
    if body < 6
        && (w[body as usize].record().active_flag() == 0
            || w[body as usize].record().turn_skip_duration()? != 0)
    {
        return Ok((vec![], true));
    }
    if body >= 10 {
        c.selected_enemy = (body - 10) as usize;
    }
    if body >= 10
        && c.enemies.records[(body - 10) as usize]
            .record()
            .final_turn_mode()?
            != 0
    {
        let turn = crate::combat::final_turn_stories::initialize(
            c,
            w,
            mission,
            &tables.monsters,
            &tables.encounters,
            random,
        )?;
        let mut events = vec![];
        for effect in turn.effects {
            use crate::combat::final_turn_stories::TurnEffect;
            match effect {
                TurnEffect::Sound(resource_id) => events.push(Event::Sound { resource_id }),
                TurnEffect::Text(id) => events.push(Event::Message {
                    id,
                    text: mission.dialogue(id)?,
                }),
                TurnEffect::Spawn { body, monster } => events.push(Event::CombatantAdded {
                    body,
                    definition: monster,
                }),
                TurnEffect::SelfRemoval { body } => events.push(Event::CombatantRemoved { body }),
            }
        }
        return Ok((events, turn.skipped_turn));
    }
    c.turn_origin = (body < 6).then(|| c.squad.positions[body as usize]);
    if body < 6 {
        let (skipped, delta) = crate::combat::initialize_warrior_record(
            &mut w[body as usize],
            &tables.race_moves,
            random,
        )?;
        c.unavailable = c.unavailable.wrapping_add(delta);
        if !skipped
            && c.automatic[body as usize]
            && w[body as usize].record().automatic_mode()? == 0
        {
            w[body as usize].record_mut().set_automatic_mode(1);
        }
        return Ok((vec![], skipped));
    }
    initialize_turn(body, w, &mut c.enemies.records, &tables.race_moves, random)?;
    let skipped = body >= 10
        && c.enemies.records[(body - 10) as usize]
            .record()
            .turn_skip_duration()
            != 0;
    Ok((vec![], skipped))
}

pub fn end_turn(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    select_turn(state, content, true)
}

pub(super) fn select_turn(
    state: &mut Session,
    content: &Content,
    advance: bool,
) -> Result<Vec<Event>, RuleError> {
    select_turn_bounded(state, content, advance, 2)
}

pub(crate) fn resume_selection(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    let combat = state.combat.as_mut().ok_or("combat selection missing")?;
    if !combat.selection_pending {
        return Err(RuleError::Invalid("combat selection is not pending".into()));
    }
    combat.selection_pending = false;
    select_turn_bounded(state, content, false, 2)
}

pub(super) fn select_turn_bounded(
    state: &mut Session,
    content: &Content,
    mut advance: bool,
    remaining_wraps: u8,
) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    if advance {
        let combat = state
            .combat
            .as_mut()
            .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
        combat.capture_actor = None;
        combat.turn_origin = None;
        state.party_mask.fill(0);
        state.bandage_ready = Some(1);
    }
    let c = state
        .combat
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
    if advance && let Some(packet) = c.attack_packet.as_mut() {
        centurions_content::records::AttackPacketMut(packet).reset_turn_attack_kind();
    }
    if !state.warriors.iter().any(|w| w.record().active_flag() != 0) {
        return Ok(crate::mission::defeat(state, content));
    }
    if advance
        && let Some(body) = c.active_body()
        && body < 10
    {
        let warrior = &mut state.warriors[body as usize];
        if warrior.record().automatic_mode()? > 0 {
            warrior.record_mut().set_automatic_mode(0);
        }
    }
    for _ in 0..220 {
        if advance {
            c.cursor += 1;
            c.spell_fields = crate::combat::spells::tick_fields(&c.spell_fields, c.cursor as i16)?;
            c.field_used = crate::combat::spells::reset_used(&c.field_used)?;
            if c.death_queue.count > 0
                || c.enemies.records.iter().all(|record| {
                    record.stamina().unwrap_or(0) <= 0
                        || record.record().removed()
                        || record.record().faction().unwrap_or(1) == 0
                })
            {
                let actor = c.active_body().unwrap_or(0);
                events.extend(finish_action(state, content, actor, true)?);
                return Ok(events);
            }
        }
        advance = true;
        if c.cursor == 110 {
            if remaining_wraps == 0 {
                c.selection_pending = true;
                return Ok(events);
            }
            c.round = c.round.wrapping_add(1);
            events.extend(advance_round(state, content, remaining_wraps - 1)?);
            return Ok(events);
        }
        let Some(body) = c.active_body() else {
            continue;
        };
        if !alive(c, &state.warriors, body) {
            continue;
        }
        if body < 10
            && state.warriors[body as usize]
                .record()
                .turn_skip_duration()?
                != 0
            || body >= 10
                && c.enemies.records[(body - 10) as usize]
                    .record()
                    .turn_skip_duration()
                    != 0
        {
            continue;
        }
        let mission = content
            .map(
                state
                    .exploration
                    .as_ref()
                    .map_or(0, |position| position.map),
            )
            .ok_or_else(|| RuleError::Invalid("combat tables missing".into()))?;
        let (turn, skipped) =
            initialize_active_turn(c, &mut state.warriors, mission, &mut state.random)?;
        events.extend(turn);
        if skipped {
            continue;
        }
        if body < 10 && state.warriors[body as usize].record().automatic_mode()? == 0 {
            events.push(Event::Sound { resource_id: 138 });
        }
        if body < 6 {
            state.selected_warrior = body as usize;
        }
        events.push(Event::TurnStarted {
            body,
            round: c.round,
        });
        return Ok(events);
    }
    Err(RuleError::Invalid("no active combatant".into()))
}
