//! Shots, spells and field collisions that run to completion, saving their work as
//! they go.
use super::super::{FireOutcome, TurnFlow};
use super::turn_state;
use crate::{Content, Event, RuleError, Session};

/// How a monster's shot ended: the events, whether it failed, struck physically or cast a spell,
/// and whether the battle's turn goes on.
pub(super) struct FirearmOutcome {
    pub events: Vec<Event>,
    pub outcome: FireOutcome,
    pub flow: TurnFlow,
}

/// How a monster's spell ended: the events, whether it was cast, and whether the turn goes on.
pub(super) struct SpellOutcome {
    pub events: Vec<Event>,
    pub cast: bool,
    pub flow: TurnFlow,
}

/// Lets the acting monster fire, in the battle or after it has ended. The friendly scientist fires
/// its rifle; any other monster fires as in its turn. The party's stamina is compared afterwards to
/// report health changes.
pub(super) fn firearm(state: &mut Session, content: &Content) -> Result<FirearmOutcome, RuleError> {
    if state.combat.is_none() {
        return super::firearm::apply(state, content);
    }
    let selected = turn_state::selected(state)?;
    let body = selected as i16 + 10;
    let mut globals = turn_state::enemy_globals(state);
    let previous = turn_state::party_hp(state)?;
    let mission = turn_state::mission(state, content)?;
    let combat = state.combat.as_mut().unwrap();
    let record = combat.story_record(selected)?.record();
    let friendly = record.faction()? == 0 && record.primary_weapon()? == 1;
    let mut events = vec![];
    let outcome = if friendly {
        let item = globals.item_context;
        let fired = crate::combat::ally_turns::fire_scientist(
            combat,
            &mut state.warriors,
            mission,
            &mut state.random,
            &content.items,
            &mut events,
            (item, body),
            &mut globals.buffers,
            true,
            &mut globals.attack_count,
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
            (&content.items, &mut globals),
            body,
            &mut events,
        )?
    };
    super::super::enemy_actions::record_attack_context(&mut globals)?;
    turn_state::publish(state, globals);
    turn_state::health(state, &events, previous)?;
    Ok(FirearmOutcome {
        events,
        outcome,
        flow: TurnFlow::Returned,
    })
}
/// Lets the acting monster cast, in the battle or after it has ended, and reports the health
/// changes.
pub(super) fn spell(state: &mut Session, content: &Content) -> Result<SpellOutcome, RuleError> {
    if state.combat.is_none() {
        return super::global_spells::apply(state, content);
    }
    let body = turn_state::selected(state)? as i16 + 10;
    let mut globals = turn_state::enemy_globals(state);
    let previous = turn_state::party_hp(state)?;
    let mission = turn_state::mission(state, content)?;
    let combat = state.combat.as_mut().unwrap();
    let mut events = vec![];
    let cast = super::super::enemy_spell(
        combat,
        &mut state.warriors,
        mission,
        &mut state.random,
        (&content.items, &mut globals),
        body,
        &mut events,
    )?;
    super::super::enemy_actions::record_attack_context(&mut globals)?;
    turn_state::publish(state, globals);
    turn_state::health(state, &events, previous)?;
    Ok(SpellOutcome {
        events,
        cast,
        flow: TurnFlow::Returned,
    })
}
/// Resolves the spell fields in the cell where the acting monster landed, in the battle or after it
/// has ended. True when the battle ended.
pub(super) fn collide(
    state: &mut Session,
    content: &Content,
) -> Result<(Vec<Event>, bool), RuleError> {
    let body = turn_state::body(state)?;
    if state.combat.is_none() {
        let outcome = super::super::out_of_combat_getup::collide_fields(state, content, body)?;
        return Ok((outcome.events, outcome.flow == TurnFlow::Terminated));
    }
    let previous = turn_state::party_hp(state)?;
    let mission = turn_state::mission(state, content)?;
    let item = crate::combat::spells::loaded_item_restriction(&state.globals.loaded_items)?;
    let combat = state.combat.as_mut().unwrap();
    let events = super::super::collide_fields_with_refresh(
        combat,
        &mut state.warriors,
        mission,
        &mut state.globals.attack_count,
        &mut state.random,
        (body, item, state.char_select_new, &mut state.bandage_ready),
        &mut state.globals.party_track,
        Some((&content.items, &mut state.globals.loaded_items)),
    )?;
    state.globals.spell_info = combat.spell_info.clone();
    state.globals.cast_identity = combat.cast_identity;
    turn_state::health(state, &events, previous)?;
    if state
        .combat
        .as_ref()
        .is_some_and(|combat| i16::from(combat.kill_party) >= state.warriors.len() as i16)
    {
        let mut events = events;
        events.extend(crate::mission::defeat(state, content)?);
        return Ok((events, true));
    }
    Ok((events, false))
}
