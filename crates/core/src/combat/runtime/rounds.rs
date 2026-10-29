//! Rounds in the original combat execution order.
use super::{
    RoundContext, finish_action, finish_battle_wrapper, reduce_round_with_fields,
    select_turn_bounded,
};
use crate::{Event, Presentation, RuleError, ScriptState, Session};
use centurions_content::Content;
use serde::{Deserialize, Serialize};

/// getup's NPC health loop suspended at a direct killbody; newland returns here.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoundReturn {
    cursor: super::enemy_round::EnemyRoundCursor,
    remaining_wraps: u8,
}

impl RoundReturn {
    pub(super) fn validate(&self) -> Result<(), RuleError> {
        if self.cursor.index >= 100
            || self.remaining_wraps > 2
            || self.cursor.stage == super::enemy_round::EnemyRoundStage::Start
        {
            return Err("invalid saved round health return".into());
        }
        Ok(())
    }
}

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
    let outcome = crate::combat::round_stories::advance_for_session(
        c,
        &mut scripts,
        mission,
        &tables.monsters,
        &tables.encounters,
        crate::combat::round_stories::RoundCaller {
            active_enemy: c.script_enemy,
            last_defeated: Some(state.native_work.health_context.last_body),
            party_positions: state.native_work.tactical_work.positions,
        },
        &mut state.random,
    )?;
    if let Some(record) = outcome.extra_record {
        state.native_work.extra_record = record;
    }
    let paused = scripts.pending.is_some();
    state.native_work.round_work.processing = Some(u8::from(paused));
    state.native_work.round_work.slot =
        Some(scripts.pending.as_ref().map_or(5, |p| i16::from(p.slot)));
    c.round_scripts = scripts;
    state.retain_monster_work();
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
                let mut record = mission.script(script)?.to_vec();
                if state
                    .exploration
                    .as_ref()
                    .is_some_and(|position| position.script.is_none())
                    && let Some(mut shared) = state
                        .turn_dispatch
                        .as_ref()
                        .filter(|frame| frame.returning)
                        .and_then(|frame| frame.work.clone())
                {
                    centurions_content::records::StoryRecordMut(&mut shared).replace_body(&record);
                    record = shared;
                }
                let position = state.exploration.as_mut().ok_or("round map missing")?;
                state.round_dispatch = Some(crate::combat::round_stories::RoundDispatch {
                    suspended_script: position.script.take(),
                    parent: state.round_dispatch.take().map(Box::new),
                    death_depth: Some(death_depth),
                    ..Default::default()
                });
                position.script = Some(ScriptState::scheduled(record, cursor));
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
    state.native_work.round_work.processing = Some(0);
    if frame.retained {
        // A battle entered while this frame is topmost marks it child_battle, and a
        // turn child resumes only after combat clears, so no owner returns early.
        if state.combat.is_some() {
            return Err("retained round macro returned with combat mode set".into());
        }
        let mut events = super::turn_stories::finish_returning(state, content)?;
        events.extend(super::selected_rewards::after_movement(state, content)?);
        return Ok(events);
    }
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
        state.ordinary_battle_rewards = outer.rewards;
        state.selected_battle_treasure = outer.selected;
        state.battle_group_script = outer.group;
        state.battle_npc = outer.npc;
        if super::npc_return::owns_pending_return(state) {
            return Ok(vec![]);
        }
        state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
        let mut events = if state.ordinary_battle_rewards.is_some()
            || state.selected_battle_treasure.is_some()
        {
            vec![]
        } else {
            vec![Event::Sound {
                resource_id: crate::feedback::BOOTY_SOUND,
            }]
        };
        events.extend(finish_battle_wrapper(state, content)?);
        return Ok(events);
    }
    if super::npc_return::owns_pending_return(state) {
        return Ok(vec![]);
    }
    if state.ordinary_battle_rewards.is_some() {
        if let Some(start) = state.battle_npc.as_ref() {
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
        }
        return finish_battle_wrapper(state, content);
    }
    state.battle_after_message = 0;
    state.battle_treasure = 0;
    state.ordinary_battle_rewards = None;
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
        c.victory_work.finish();
        state.native_work.early_booty = 1;
        return crate::combat::retreat::finish(state, content);
    }
    c.inactive_rounds += 1;
    c.cursor = 0;
    let events = vec![];
    crate::combat::party_buffs::reduce(&mut state.party_buffs)?;
    let outcome = reduce_round_with_fields(
        c,
        &mut state.warriors,
        &mut state.random,
        Some(mission),
        &mut state.native_work.attack_count,
        &mut state.native_work.party_mask,
        RoundContext {
            audio: true,
            selected_warrior: state.selected_warrior,
            energy_recovery_disabled: state.energy_recovery_disabled,
            item_work_alias: crate::combat::spells::retained_item_alias(
                &state.native_work.ammunition_work,
            )?,
            processing: Some(state.native_work.death_processing),
            movement_gate: &mut state.bandage_ready,
            items: Some((&content.items, &mut state.native_work.ammunition_work)),
        },
    )?;
    after_health(state, content, outcome, remaining_wraps, events)
}

/// The death story of a direct getup killbody has returned to the NPC health loop.
pub(super) fn resume_round(
    state: &mut Session,
    content: &Content,
    frame: RoundReturn,
) -> Result<Vec<Event>, RuleError> {
    frame.validate()?;
    let mission = state
        .map(
            content,
            state.exploration.as_ref().ok_or("round map missing")?.map,
        )
        .ok_or("round content missing")?;
    let alias = crate::combat::spells::retained_item_alias(&state.native_work.ammunition_work)?;
    let processing = state.native_work.death_processing;
    // A death story that ended combat leaves getup's loop iterating the static
    // records; the detached reduction continues it and the retained scan follows.
    if state.combat.is_none() {
        let outcome = super::retained_turn::reduction::resume_npcs(
            state,
            content,
            (frame.cursor.index, frame.cursor.stage),
            vec![],
        )?;
        if outcome.flow == super::TurnFlow::Terminated {
            return Ok(outcome.events);
        }
        return super::retained_turn::continue_after_reduction(state, content, outcome.events);
    }
    let c = state.combat.as_mut().ok_or("round combat missing")?;
    if frame.cursor.index >= c.enemies.records.len() {
        return Err("saved round health return outside the roster".into());
    }
    let outcome = super::reduce_enemy_round_impl(
        c,
        &mut state.warriors,
        &mut state.random,
        Some(mission),
        &mut state.native_work.attack_count,
        &mut state.native_work.party_mask,
        (
            state.selected_warrior,
            alias,
            &mut state.bandage_ready,
            Some((&content.items, &mut state.native_work.ammunition_work)),
        ),
        frame.cursor,
        Some(processing),
    )?;
    after_health(state, content, outcome, frame.remaining_wraps, vec![])
}

fn after_health(
    state: &mut Session,
    content: &Content,
    outcome: super::EnemyRoundOutcome,
    remaining_wraps: u8,
    mut events: Vec<Event>,
) -> Result<Vec<Event>, RuleError> {
    if let Some(body) = outcome.last_defeated_body {
        state.native_work.health_context.last_body = body;
    }
    events.extend(outcome.events);
    if let Some(cursor) = outcome.suspended {
        let actor = state.selected_warrior as i16;
        events.extend(super::death::suspend_reaction(
            state,
            content,
            actor,
            super::CombatReturn::Round(Box::new(RoundReturn {
                cursor,
                remaining_wraps,
            })),
        )?);
        return Ok(events);
    }
    let c = state.combat.as_mut().ok_or("round combat missing")?;
    if state
        .warriors
        .iter()
        .any(|warrior| warrior.record().bleeding_flag() != 0)
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

    if c.death_queue.count > 0 || !c.victory_work.has_remaining() {
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
