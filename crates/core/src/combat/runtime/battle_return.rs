//! Battle return in the original combat execution order.
use crate::{Event, Phase, Presentation, RuleError, Session};
use centurions_content::Content;

pub(super) fn finish_victory(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    if super::selected_rewards::awaiting_battle(state) {
        return super::selected_rewards::after_battle(state, content);
    }
    let (mode, mut events) = super::ordinary_rewards::begin(state, content)?;
    if mode == 10 {
        events.extend(super::ordinary_rewards::storage(state, content)?);
        return Ok(events);
    }
    if mode == 2 {
        return crate::story_cleanup::begin_ordinary(state, content);
    }
    state.clear_tactical_fields();
    if let Some(survivors) = begin_survivors(state)? {
        events.extend(survivors);
    } else {
        events.extend(confirm_survivors(state, content)?);
    }
    Ok(events)
}

pub(crate) fn begin_survivors(state: &mut Session) -> Result<Option<Vec<Event>>, RuleError> {
    if state.suppress_allies != 0 {
        return Ok(None);
    }
    let records = &mut state
        .combat
        .as_mut()
        .ok_or("survivor roster missing")?
        .enemies
        .records;
    state.survivors = crate::combat::allies::prepare_survivors(records)?;
    if state.survivors.is_none() {
        return Ok(None);
    }
    state
        .exploration
        .as_mut()
        .ok_or("survivor map missing")?
        .presentation = Some(Presentation::Survivors);
    Ok(Some(vec![
        Event::Sound {
            resource_id: 29999_i16.wrapping_add(state.random.bound(4)),
        },
        Event::StoryPresented,
    ]))
}

pub fn confirm_survivors(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    if super::selected_rewards::awaiting_battle(state) {
        return super::selected_rewards::after_battle(state, content);
    }
    let (mode, mut events) = super::ordinary_rewards::begin(state, content)?;
    if mode == 10 {
        events.extend(super::ordinary_rewards::storage(state, content)?);
        return Ok(events);
    }
    if mode == 2 {
        return crate::story_cleanup::begin_ordinary(state, content);
    }
    if state.selected_battle_treasure.is_some() {
        events.extend(refit_after_combat(state, content)?);
    }
    events.extend(super::reward_context::survivors(state)?);
    state.exploration.as_mut().unwrap().presentation = None;
    if let Some(start) = state.battle_npc.as_ref() {
        let script = state
            .exploration
            .as_mut()
            .unwrap()
            .script
            .as_mut()
            .ok_or_else(|| RuleError::Invalid("NPC battle continuation missing".into()))?;
        script.cursor = start.restore(&mut script.record, false)?.unwrap() as u8;
    }
    let count = state.combat.as_ref().unwrap().enemies.records.len() as i16;
    let rewards = crate::combat::rewards::prepare_mode(
        &state.warriors,
        &state.combat.as_ref().unwrap().enemies.records,
        content,
        crate::combat::rewards::RewardContext {
            difficulty: state.difficulty,
            npc_denominator: 0,
            currency: state.party_currency,
            mode: crate::combat::rewards::RewardMode::select(
                mode,
                &state.native_work.treasure_record,
            )?,
        },
        &mut state.random,
    )?;
    super::ordinary_rewards::install(state, content, mode, rewards)?;
    let combat = state.combat.as_ref().unwrap();
    // booty105748 enters its reward window even when the crate is empty.
    state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
    events.push(Event::Sound {
        resource_id: crate::feedback::BOOTY_SOUND,
    });
    let mut health = crate::story_health::retained_context(
        state.selected_warrior,
        &state.native_work.health_context,
    );
    health.unavailable = combat.unavailable;
    let packet = &combat.attack_packet;
    health.effect = packet[18] as i8;
    state.native_work.health_context = health;
    super::reward_context::detach(state)?;
    super::reward_cleanup::finish(state, count, mode)?;
    state.reload_battle_land()?;
    if !state
        .combat_story
        .as_ref()
        .is_some_and(|story| story.waiting_child)
    {
        state.combat_story = None;
    } else {
        state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
    }
    state.handled_deaths.clear();
    state.phase = Phase::Exploration;
    if state.selected_battle_treasure.is_some() {
        super::selected_rewards::defer(state)?;
        return Ok(events);
    }
    if super::ordinary_rewards::defer(state) {
        return Ok(events);
    }
    events.extend(finish_battle_wrapper(state, content)?);
    Ok(events)
}

pub(crate) fn refit_after_combat(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    if state.command_preferences.auto_refit {
        for member in 0..state.warriors.len() {
            let result = crate::ammunition::refit_with_record(
                &state.warriors[member],
                &state.ammunition,
                content,
                crate::ammunition::RefitFlags {
                    in_combat: false,
                    force: false,
                    silent: true,
                    warnings: false,
                },
                &state.native_work.ammunition_work,
                &mut state.random,
                Some(&state.native_work.inventory_record),
            )?;
            events.extend(crate::ammunition::commit_result(state, member, result)?);
        }
    }
    Ok(events)
}

pub(super) fn finish_battle_wrapper(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    if super::selected_rewards::awaiting_battle(state) {
        return super::selected_rewards::after_battle(state, content);
    }
    if state.ordinary_battle_rewards.is_some() {
        return super::ordinary_rewards::outer(state, content);
    }
    if state.battle_treasure != 0 {
        return Err(RuleError::from("battle treasure without its reward caller"));
    }
    finish_wrapper(state, content, true, true)
}

pub(super) fn finish_prepared_battle_wrapper(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    finish_wrapper(state, content, false, true)
}

pub(super) fn finish_selected_battle_wrapper(
    state: &mut Session,
    content: &Content,
    retreated: bool,
) -> Result<Vec<Event>, RuleError> {
    finish_wrapper(state, content, false, !retreated)
}

fn finish_wrapper(
    state: &mut Session,
    content: &Content,
    refit: bool,
    victory: bool,
) -> Result<Vec<Event>, RuleError> {
    let mut events = if refit {
        refit_after_combat(state, content)?
    } else {
        vec![]
    };
    if state.battle_npc.take().is_some() {
        crate::story_npc_battle::finish(
            crate::story_npc_battle::NpcBattleReturn::Reward,
            &mut state.warriors,
            [0; 2],
        )?;
    }
    let failure_script = state.battle_group_script.take().unwrap_or(0);
    state.warriors =
        crate::story_party::group_return(&state.warriors, false, failure_script)?.warriors;
    events.push(Event::BattleEnded { victory });
    if state.battle_after_message != 0 {
        let map = state.land(content).unwrap();
        events.push(Event::Message {
            id: state.battle_after_message,
            text: map.dialogue(state.battle_after_message)?,
        });
    }
    state.battle_after_message = 0;
    state.battle_treasure = 0;
    if state
        .exploration
        .as_ref()
        .is_some_and(|position| position.presentation.is_none() && position.script.is_some())
        && !events
            .iter()
            .any(|event| matches!(event, Event::Message { .. }))
    {
        super::resume_turn_child(state)?;
        let dispatch_owns = state
            .turn_dispatch
            .as_ref()
            .is_some_and(|frame| frame.owns_script(state))
            || state
                .round_dispatch
                .as_ref()
                .is_some_and(|frame| frame.owns_script(state));
        if state.combat.is_none()
            && !dispatch_owns
            && state
                .combat_story
                .as_ref()
                .is_some_and(|story| story.waiting_child)
        {
            events.extend(super::continue_death(state, content)?);
        } else if state.combat_story.is_none() || dispatch_owns {
            events.extend(crate::exploration::run_script(state, content)?);
        }
    }
    Ok(events)
}
