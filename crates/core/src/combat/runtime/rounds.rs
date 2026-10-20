//! Rounds in the original combat execution order.
use super::{
    RoundContext, finish_action, finish_battle_wrapper, reduce_round_with_fields,
    select_turn_bounded,
};
use crate::{Event, Presentation, RuleError, ScriptState, Session};
use centurions_content::{Content, word};

pub(super) fn advance_round(
    state: &mut Session,
    content: &Content,
    remaining_wraps: u8,
) -> Result<Vec<Event>, RuleError> {
    let mission = state
        .map(
            content,
            state.exploration.as_ref().ok_or("round map missing")?.map,
        )
        .ok_or("round content missing")?;
    let tables = mission.combat.as_ref().ok_or("round tables missing")?;
    let c = state.combat.as_mut().ok_or("round combat missing")?;
    let mut scripts = c.round_scripts.clone();
    let outcome = crate::combat::round_stories::advance(
        c,
        &mut scripts,
        mission,
        &tables.monsters,
        &tables.encounters,
        crate::combat::round_stories::RoundCaller {
            active_enemy: c.script_enemy,
            last_defeated: state.health_context.as_ref().map(|health| health.last_body),
        },
        &mut state.random,
    )?;
    let paused = scripts.pending.is_some();
    c.round_scripts = scripts;
    let mut events = vec![];
    for effect in outcome.effects {
        use crate::combat::round_stories::RoundEffect;
        match effect {
            RoundEffect::Sound(resource_id) => events.push(Event::Sound { resource_id }),
            RoundEffect::Text(id) => {
                state.exploration.as_mut().unwrap().presentation = Some(Presentation::Dialogue {
                    id,
                    text: crate::exploration::dialogue(mission, &state.warriors, id)?,
                });
                events.push(Event::StoryPresented);
            }
            RoundEffect::Spawn { body, monster } => events.push(Event::CombatantAdded {
                body,
                definition: monster,
            }),
            RoundEffect::Removed(body) => events.push(Event::CombatantRemoved { body }),
            RoundEffect::Dispatch { script, cursor } => {
                let death_depth = crate::combat::round_stories::RoundDispatch::death_depth(state);
                let position = state.exploration.as_mut().ok_or("round map missing")?;
                state.round_dispatch = Some(crate::combat::round_stories::RoundDispatch {
                    suspended_script: position.script.take(),
                    parent: state.round_dispatch.take().map(Box::new),
                    death_depth: Some(death_depth),
                    ..Default::default()
                });
                position.script = Some(ScriptState {
                    record: mission.script(script)?.to_vec(),
                    cursor,
                    map_slot: 0,
                    item_return: false,
                    scheduled: true,
                    menu: None,
                    persist: false,
                    origin: None,
                });
                events.extend(crate::exploration::run_script(state, content)?);
                return Ok(events);
            }
        }
    }
    if paused {
        return Ok(events);
    }
    events.extend(reduce_and_resume_round(state, content, remaining_wraps)?);
    Ok(events)
}

pub(crate) fn continue_round(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    let program = state
        .combat
        .as_mut()
        .and_then(|c| c.round_scripts.pending.as_mut())
        .ok_or("round continuation missing")?;
    let id = program.text.take().ok_or("round dialogue missing")?;
    if !matches!(state.exploration.as_ref().and_then(|p| p.presentation.as_ref()), Some(Presentation::Dialogue { id: current, .. }) if *current == id)
    {
        return Err(RuleError::Invalid(
            "round continuation has no matching dialogue".into(),
        ));
    }
    state.exploration.as_mut().unwrap().presentation = None;
    advance_round(state, content, 2)
}

pub(crate) fn finish_round_dispatch(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    let frame = state
        .round_dispatch
        .take()
        .ok_or("round dispatch missing")?;
    if frame.child_battle && state.combat.is_some() {
        return Err(RuleError::Invalid(
            "round child battle has not returned".into(),
        ));
    }
    state.round_dispatch = frame.parent.map(|parent| *parent);
    state
        .exploration
        .as_mut()
        .ok_or("round map missing")?
        .script = frame.suspended_script;
    if let Some(arena) = state.combat.as_mut() {
        let program = arena
            .round_scripts
            .pending
            .as_mut()
            .ok_or("round program missing")?;
        program.delegated = false;
        program.cursor = 8;
        return advance_round(state, content, 2);
    }
    state.scripted_cleanup = None;
    if let Some(outer) = frame.outer_battle {
        state.battle_after_message = outer.after_message;
        state.battle_treasure = outer.treasure;
        state.battle_group_script = outer.group;
        state.battle_npc = outer.npc;
        state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
        let mut events = vec![Event::Sound { resource_id: 20 }];
        events.extend(finish_battle_wrapper(state, content)?);
        return Ok(events);
    }
    state.battle_after_message = 0;
    state.battle_treasure = 0;
    state.battle_group_script = None;
    if let Some(start) = state.battle_npc.take() {
        let script = state
            .exploration
            .as_mut()
            .unwrap()
            .script
            .as_mut()
            .ok_or("cleanup caller script missing")?;
        script.cursor = start
            .restore(&mut script.record, false)?
            .ok_or("cleanup caller cursor missing")? as u8;
        crate::story_npc_battle::finish(
            crate::story_npc_battle::NpcBattleReturn::Reward,
            &mut state.warriors,
            [0; 2],
        )?;
    }
    let mut events = vec![Event::BattleEnded { victory: false }];
    if state
        .combat_story
        .as_ref()
        .is_some_and(|story| story.waiting_child)
    {
        state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
        events.push(Event::StoryPresented);
    } else if state
        .exploration
        .as_ref()
        .is_some_and(|p| p.script.is_some())
    {
        events.extend(crate::exploration::run_script(state, content)?);
    }
    Ok(events)
}

pub(super) fn reduce_and_resume_round(
    state: &mut Session,
    content: &Content,
    remaining_wraps: u8,
) -> Result<Vec<Event>, RuleError> {
    let mission = state
        .map(
            content,
            state.exploration.as_ref().ok_or("round map missing")?.map,
        )
        .ok_or("round content missing")?;
    let c = state.combat.as_mut().ok_or("round combat missing")?;
    if c.inactive_rounds == 9 {
        return crate::combat::retreat::finish(state, content);
    }
    c.inactive_rounds += 1;
    c.cursor = 0;
    let mut events = vec![];
    crate::combat::party_buffs::reduce(&mut state.party_buffs)?;
    let outcome = reduce_round_with_fields(
        c,
        &mut state.warriors,
        &mut state.random,
        Some(mission),
        &mut state.attack_count,
        &mut state.party_mask,
        RoundContext {
            audio: true,
            selected_warrior: state.selected_warrior,
            energy_recovery_disabled: state.energy_recovery_disabled,
        },
    )?;
    if let Some(body) = outcome.last_defeated_body
        && let Some(health) = state.health_context.as_mut()
    {
        health.last_body = body;
    }
    events.extend(outcome.events);
    if state
        .warriors
        .iter()
        .any(|warrior| warrior.bytes[0x9de] != 0)
    {
        events.push(Event::Sound { resource_id: 10122 });
    }
    events.extend([
        Event::Sound { resource_id: -139 },
        Event::Sound { resource_id: -139 },
    ]);
    if !state.command_preferences.suppress_round_message_sound {
        events.push(Event::Sound { resource_id: -139 });
    }

    if c.death_queue.count > 0
        || c.enemies
            .records
            .iter()
            .all(|m| m.stamina().unwrap_or(0) <= 0 || word(&m.bytes, 14).unwrap_or(1) == 0)
    {
        events.extend(finish_action(
            state,
            content,
            state.selected_warrior as i16,
            true,
        )?);
    } else {
        events.extend(select_turn_bounded(state, content, false, remaining_wraps)?);
    }
    Ok(events)
}
