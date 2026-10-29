//! Uninterrupted firearm/spell/collision services publish work at each reached call.
use super::super::{FireOutcome, TurnFlow};
use super::work;
use crate::{Content, Event, RuleError, Session};

pub(super) struct FirearmOutcome {
    pub events: Vec<Event>,
    pub outcome: FireOutcome,
    pub flow: TurnFlow,
}

pub(super) struct SpellOutcome {
    pub events: Vec<Event>,
    pub cast: bool,
    pub flow: TurnFlow,
}

pub(super) fn firearm(state: &mut Session, content: &Content) -> Result<FirearmOutcome, RuleError> {
    if state.combat.is_none() {
        return super::firearm::apply(state, content);
    }
    let selected = work::selected(state)?;
    let body = selected as i16 + 10;
    let mut work_buffer = work::enemy_work(state);
    let previous = work::party_hp(state)?;
    let mission = work::mission(state, content)?;
    let combat = state.combat.as_mut().unwrap();
    let record = combat.story_record(selected)?.record();
    let friendly = record.faction()? == 0 && record.primary_weapon()? == 1;
    let mut events = vec![];
    let outcome = if friendly {
        let item = work_buffer.item_context;
        let fired = crate::combat::ally_turns::fire_scientist(
            combat,
            &mut state.warriors,
            mission,
            &mut state.random,
            &content.items,
            &mut events,
            (item, body),
            &mut work_buffer.buffers,
            true,
            &mut work_buffer.attack_count,
        )?;
        if fired {
            FireOutcome::Physical
        } else {
            FireOutcome::Failed
        }
    } else {
        super::super::enemy_fire(
            combat,
            &mut state.warriors,
            mission,
            &mut state.random,
            (&content.items, &mut work_buffer),
            body,
            &mut events,
        )?
    };
    super::super::enemy_actions::record_attack_context(&mut work_buffer)?;
    work::publish(state, work_buffer);
    work::health(state, &events, previous)?;
    Ok(FirearmOutcome {
        events,
        outcome,
        flow: TurnFlow::Returned,
    })
}
pub(super) fn spell(state: &mut Session, content: &Content) -> Result<SpellOutcome, RuleError> {
    if state.combat.is_none() {
        return super::global_spells::apply(state, content);
    }
    let body = work::selected(state)? as i16 + 10;
    let mut work_buffer = work::enemy_work(state);
    let previous = work::party_hp(state)?;
    let mission = work::mission(state, content)?;
    let combat = state.combat.as_mut().unwrap();
    let mut events = vec![];
    let cast = super::super::enemy_spell(
        combat,
        &mut state.warriors,
        mission,
        &mut state.random,
        (&content.items, &mut work_buffer),
        body,
        &mut events,
    )?;
    super::super::enemy_actions::record_attack_context(&mut work_buffer)?;
    work::publish(state, work_buffer);
    work::health(state, &events, previous)?;
    Ok(SpellOutcome {
        events,
        cast,
        flow: TurnFlow::Returned,
    })
}
pub(super) fn collide(
    state: &mut Session,
    content: &Content,
) -> Result<(Vec<Event>, bool), RuleError> {
    let body = work::body(state)?;
    if state.combat.is_none() {
        let outcome = super::super::retained_turn::collide_fields(state, content, body)?;
        return Ok((outcome.events, outcome.flow == TurnFlow::Terminated));
    }
    let previous = work::party_hp(state)?;
    let mission = work::mission(state, content)?;
    let item = crate::combat::spells::retained_item_alias(&state.native_work.ammunition_work)?;
    let combat = state.combat.as_mut().unwrap();
    let events = super::super::collide_fields_with_work(
        combat,
        &mut state.warriors,
        mission,
        &mut state.native_work.attack_count,
        &mut state.random,
        (body, item, state.selected_warrior, &mut state.bandage_ready),
        &mut state.native_work.party_mask,
        Some((&content.items, &mut state.native_work.ammunition_work)),
    )?;
    state.native_work.attack_packet = combat.attack_packet.clone();
    state.native_work.attack_identity = combat.attack_identity;
    work::health(state, &events, previous)?;
    if state
        .combat
        .as_ref()
        .is_some_and(|combat| i16::from(combat.unavailable) >= state.warriors.len() as i16)
    {
        let mut events = events;
        events.extend(crate::mission::defeat(state, content)?);
        return Ok((events, true));
    }
    Ok((events, false))
}
