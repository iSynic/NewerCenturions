//! Battle return in the original combat execution order.
use crate::{Event, Phase, Presentation, RuleError, Session};
use centurions_content::Content;

pub(super) fn finish_victory(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    if let Some(events) = begin_survivors(state)? {
        return Ok(events);
    }
    confirm_survivors(state, content)
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
    let combat = state
        .combat
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
    let synchronization = if let Some(selection) = state.survivors.take() {
        crate::combat::allies::commit_survivors(
            &state.allies,
            state.ally_count,
            &mut combat.enemies.records,
            Some(&selection),
        )?
    } else if state.suppress_allies != 0 {
        crate::combat::allies::AllySynchronization {
            menu: state.allies.clone(),
            count: state.ally_count,
            modal: false,
            sounds: vec![],
        }
    } else {
        crate::combat::allies::synchronize(
            &state.allies,
            state.ally_count,
            &mut combat.enemies.records,
            &mut state.random,
        )?
    };
    state.allies = synchronization.menu;
    state.ally_count = synchronization.count;
    let mut events: Vec<_> = synchronization
        .sounds
        .into_iter()
        .map(|resource_id| Event::Sound { resource_id })
        .collect();
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
    let rewards = crate::combat::rewards::prepare(
        &state.warriors,
        &combat.enemies.records,
        content,
        state.difficulty,
        0,
        state.party_currency,
        &mut state.random,
    )?;
    combat.unavailable = rewards.unavailable;
    if let Some(id) = rewards.last_item_id {
        state.loaded_item_field32 = Some(state.ammunition_work.load_item(content, id)?);
    }
    state.warriors = rewards.warriors;
    if let Some(record) = rewards.inventory_record {
        state.inventory_record = Some(record);
    }
    combat.enemies.records = rewards.enemies;
    state.party_currency = rewards.currency;
    for item in &mut state.booty_items {
        item.0.fill(0);
    }
    for (index, item) in rewards.items.into_iter().enumerate() {
        if index >= state.booty_items.len() {
            return Err(RuleError::Invalid(
                "battle loot exceeds booty capacity".into(),
            ));
        }
        state.booty_items[index] = item;
    }
    // booty105748 enters its reward window even when the crate is empty.
    state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
    events.push(Event::Sound { resource_id: 20 });
    state.attack_packet = combat.attack_packet.clone();
    state.attack_identity = combat.attack_identity;
    state.weapon_work = combat.weapon_work;
    let mut health = crate::story_health::context(
        &state.warriors,
        state.selected_warrior,
        state.health_context.as_ref(),
    );
    health.unavailable = combat.unavailable;
    if let Some(packet) = combat.attack_packet.as_ref() {
        health.effect = packet[18] as i8;
    }
    state.health_context = Some(health);
    state.selected_enemy = combat.selected_enemy;
    state.script_enemy = combat.script_enemy;
    state.npc_backing.capture(combat);
    state.retained_death_queue = Some(combat.death_queue.clone());
    state.combat = None;
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
                &state.ammunition_work,
                &mut state.random,
                state.inventory_record.as_ref(),
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
    let mut events = refit_after_combat(state, content)?;
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
    events.push(Event::BattleEnded { victory: true });
    if state.battle_after_message != 0 {
        let map = content
            .map(state.exploration.as_ref().unwrap().map)
            .unwrap();
        events.push(Event::Message {
            id: state.battle_after_message,
            text: map.dialogue(state.battle_after_message)?,
        });
    }
    state.battle_after_message = 0;
    if state.battle_treasure != 0 {
        let map = content
            .map(state.exploration.as_ref().unwrap().map)
            .unwrap();
        let treasure = centurions_content::span(
            &map.treasure,
            state.battle_treasure.unsigned_abs() as usize * 88,
            88,
        )?;
        for item in crate::story_inventory::grant_story_treasure(
            treasure,
            content,
            &mut state.random,
            state.difficulty,
        )? {
            state.load_item_work(content, item.id())?;
            let slot = state
                .booty_items
                .iter()
                .position(|item| item.id() == 0)
                .ok_or_else(|| {
                    RuleError::Invalid("battle treasure exceeds booty capacity".into())
                })?;
            state.booty_items[slot] = item;
        }
        state.battle_treasure = 0;
        if state.booty_items.iter().any(|item| item.id() != 0) {
            state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
        }
    }
    if state.combat_story.is_none()
        && state
            .exploration
            .as_ref()
            .is_some_and(|position| position.presentation.is_none() && position.script.is_some())
        && !events
            .iter()
            .any(|event| matches!(event, Event::Message { .. }))
    {
        events.extend(crate::exploration::run_script(state, content)?);
    }
    Ok(events)
}
