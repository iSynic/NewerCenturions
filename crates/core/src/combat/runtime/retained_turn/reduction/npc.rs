//! NPC reduction prefix; outdoor death can terminate before the remaining work.
use crate::{
    Content, Event, RuleError, Session,
    combat::{
        natural_conditions,
        runtime::{TurnFlow, outdoor_death},
    },
};

use crate::combat::runtime::enemy_round::EnemyRoundStage;

pub(super) fn conditions(
    state: &mut Session,
    content: &Content,
    (index, stage): (usize, EnemyRoundStage),
    events: &mut Vec<Event>,
) -> Result<TurnFlow, RuleError> {
    let record = &mut state.native_work.npc_backing.slot_mut(index)?.record;
    if stage == EnemyRoundStage::Start {
        record.record_mut().decrement_temporary_conditions();
        record.record_mut().clear_damage_marker();
        if let Some(heal) = natural_conditions::enemy_poison_heal(record)? {
            record.record_mut().set_stamina(heal.health);
            record.record_mut().mark_damaged();
            events.push(Event::Sound { resource_id: 624 });
        }
    }
    if stage == EnemyRoundStage::Start
        && let Some(tick) = natural_conditions::enemy_drain_tick(&record.bytes)?
    {
        record.record_mut().set_stamina(tick.health);
        events.push(Event::Sound { resource_id: 107 });
        if tick.killed && outdoor_death::defeat(state, content, index as i16 + 10, events)? {
            return Ok(TurnFlow::Terminated);
        }
    }
    let record = &mut state.native_work.npc_backing.slot_mut(index)?.record;
    if stage <= EnemyRoundStage::AfterDrain
        && let Some(tick) = natural_conditions::enemy_poison_tick(record)?
    {
        record.record_mut().set_stamina(tick.health);
        record.record_mut().mark_damaged();
        events.push(Event::Sound { resource_id: 104 });
        if tick.killed && outdoor_death::defeat(state, content, index as i16 + 10, events)? {
            return Ok(TurnFlow::Terminated);
        }
    }
    natural_conditions::reduce_enemy_psi(
        &mut state.native_work.npc_backing.slot_mut(index)?.record.bytes,
    )?;
    Ok(TurnFlow::Returned)
}
