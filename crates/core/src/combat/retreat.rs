use super::*;
use crate::feedback::NativeFeedback;
use centurions_content::mission::OpeningMission;

/// combatchoice11bdce..11bfb8 removes an escaped warrior without death cleanup.
pub(crate) fn leave_edge(
    state: &mut Session,
    content: &Content,
    member: usize,
) -> Result<Vec<Event>, RuleError> {
    leave(state, content, member, true)
}

pub(crate) fn leave(
    state: &mut Session,
    content: &Content,
    member: usize,
    edge_message: bool,
) -> Result<Vec<Event>, RuleError> {
    let c = state.combat.as_mut().ok_or("retreat combat missing")?;
    let point = c.squad.positions[member];
    let cell = OpeningMission::index(
        i32::from(point.0 + c.map.shift.0 + 5),
        i32::from(point.1 + c.map.shift.1 + 5),
    )?;
    c.map.tiles[cell] = c.squad.under_tiles[member];
    c.squad.positions[member] = (-1, -1);
    c.order[c.cursor] = -1;
    let w = &mut state.warriors[member].bytes;
    put_word(w, 0x20, -1);
    w[0x9df] = 0;
    let penalty = i32::from_be_bytes(w[0xa72..0xa76].try_into().unwrap()).wrapping_add(200);
    w[0xa72..0xa76].copy_from_slice(&penalty.to_be_bytes());
    let mut events = vec![];
    if edge_message {
        events.push(Event::Feedback(NativeFeedback::Flash {
            address: 0x403c3a,
            x: 100,
            y: 100,
            wait: 0,
            sound: 6000,
        }));
    }
    events.push(Event::CombatantRemoved {
        body: member as i16,
    });
    if state
        .warriors
        .iter()
        .any(|w| w.bytes[0x9df] != 0 && word(&w.bytes, 0x10).is_ok_and(|f| f == 0))
    {
        events.extend(runtime::end_turn(state, content)?);
    } else {
        events.extend(finish(state, content)?);
    }
    Ok(events)
}

/// Retreated booty takes its early return before loot, cash or promotion.
pub(crate) fn finish(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    if state.command_preferences.auto_refit {
        for member in 0..state.warriors.len() {
            let result = crate::ammunition::refit(
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
            )?;
            events.extend(crate::ammunition::commit_result(state, member, result)?);
        }
    }
    for warrior in &mut state.warriors {
        let w = &mut warrior.bytes;
        put_word(w, 0x10, 0);
        w[0x9de] = 0;
        put_word(w, 0x60, 0);
        for offset in (0x5e..=0x64).step_by(2) {
            if word(w, offset)? > 0 {
                put_word(w, offset, 0);
            }
        }
        if word(w, 0x90)? > 0 {
            put_word(w, 0x90, 0);
        }
        w[0x9df] = 1;
    }
    state.retain_npcs();
    let combat = state.combat.take().ok_or("retreat combat missing")?;
    state.attack_packet = combat.attack_packet;
    state.attack_identity = combat.attack_identity;
    state.weapon_work = combat.weapon_work;
    if !state
        .combat_story
        .as_ref()
        .is_some_and(|story| story.waiting_child)
    {
        state.combat_story = None;
    }
    state.handled_deaths.clear();
    state.phase = Phase::Exploration;
    state.battle_treasure = 0;
    state.battle_after_message = 0;
    for item in &mut state.booty_items {
        item.0.fill(0);
    }
    let position = state.exploration.as_mut().ok_or("retreat map missing")?;
    position.presentation = None;
    let mut replacement = state.battle_group_script.take();
    if let Some(start) = state.battle_npc.take() {
        let script = position
            .script
            .as_mut()
            .ok_or("NPC retreat continuation missing")?;
        script.cursor = start.restore(&mut script.record, false)?.unwrap() as u8;
        let outcome = start.return_kind(false, true);
        crate::story_npc_battle::finish(
            outcome,
            &mut state.warriors,
            [position.last_move[0] as u8, position.last_move[1] as u8],
        )?;
        match outcome {
            crate::story_npc_battle::NpcBattleReturn::EscapeScript(id) => replacement = Some(id),
            crate::story_npc_battle::NpcBattleReturn::EscapeMove => {
                events.extend([
                    Event::Feedback(NativeFeedback::Warning { index: 118 }),
                    Event::Sound { resource_id: 26260 },
                    Event::Feedback(NativeFeedback::Warning { index: 124 }),
                ]);
                position.x -= i32::from(position.last_move[0]);
                position.y -= i32::from(position.last_move[1]);
                OpeningMission::index(position.x, position.y)?;
                position.script = None;
                events.push(Event::Moved {
                    x: position.x,
                    y: position.y,
                });
            }
            _ => {}
        }
    }
    if let Some(id) = replacement {
        let map = content.map(position.map).ok_or("retreat content missing")?;
        let record = map.script(id)?.to_vec();
        if let Some(script) = position.script.as_mut() {
            script.record[8..].copy_from_slice(&record[8..]);
            script.cursor = 0;
            script.menu = None;
        } else {
            position.script = Some(ScriptState {
                record,
                cursor: 0,
                map_slot: 0,
                item_return: false,
                scheduled: true,
                menu: None,
                persist: false,
                origin: None,
            });
        }
    }
    events.push(Event::BattleEnded { victory: false });
    if state
        .combat_story
        .as_ref()
        .is_some_and(|story| story.waiting_child)
    {
        position.presentation = Some(Presentation::Loot);
        events.push(Event::StoryPresented);
    } else if position.script.is_some() {
        events.extend(crate::exploration::run_script(state, content)?);
    }
    Ok(events)
}
