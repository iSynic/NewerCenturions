use crate::combat::runtime;
use crate::feedback::GameFeedback;
use crate::{Event, Phase, Presentation, RuleError, ScriptState, Session};
use centurions_content::mission::OpeningMission;
use centurions_content::{
    Content,
    records::{CharacterStatisticsMut, Statistic},
};

/// A warrior who leaves by the map edge is gone from the battle, without death
/// cleanup.
///
/// Original: combatchoice
pub(crate) fn leave_edge(
    state: &mut Session,
    content: &Content,
    member: usize,
) -> Result<Vec<Event>, RuleError> {
    leave(state, content, member, true)
}

/// Removes a warrior from the battle: the cell is cleared, the warrior leaves the fight and takes a
/// score penalty of 200. If another warrior on the party's side is still in the battle the turn
/// ends; with none left the party has run away and the battle ends without loot.
pub(crate) fn leave(
    state: &mut Session,
    content: &Content,
    member: usize,
    edge_message: bool,
) -> Result<Vec<Event>, RuleError> {
    let combat = state.combat.as_mut().ok_or("retreat combat missing")?;
    let point = combat.squad.pos[member];
    let cell = OpeningMission::index(
        i32::from(point.0 + combat.field.map_shift.0 + 5),
        i32::from(point.1 + combat.field.map_shift.1 + 5),
    )?;
    combat.field.tiles[cell] = combat.squad.char_under[member];
    combat.squad.pos[member] = (-1, -1);
    combat.battle_order[combat.battle_order_index] = -1;
    let warrior = &mut state.warriors[member];
    warrior.record_mut().set_position(-1);
    warrior.record_mut().set_in_battle(0);
    CharacterStatisticsMut(&mut warrior.bytes).add(Statistic::ScorePenalty, 200);
    let mut events = vec![];
    if edge_message {
        events.push(Event::Feedback(GameFeedback::Flash {
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
        .any(|w| w.record().in_battle() != 0 && w.record().faction().is_ok_and(|f| f == 0))
    {
        events.extend(runtime::end_turn(state, content)?);
    } else {
        state.globals.coward = 1;
        events.extend(finish(state, content)?);
    }
    Ok(events)
}

/// After a retreat booty returns before loot, cash or promotion.
///
/// Original: booty
pub(crate) fn finish(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    if state.selected_battle_treasure.is_some() {
        return runtime::retreat_selected_rewards(state, content);
    }
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
                &state.globals.loaded_items,
                &mut state.random,
                Some(&state.globals.inventory_record),
            )?;
            events.extend(crate::ammunition::commit_result(state, member, result)?);
        }
    }
    for warrior in &mut state.warriors {
        crate::story::inventory::normalize_warrior_after_battle(warrior)?;
        warrior.record_mut().set_in_battle(1);
    }
    state.clear_tactical_fields();
    state.store_monsters();
    state.store_spell_fatigue();
    let combat = state.combat.take().ok_or("retreat combat missing")?;
    state.globals.in_spell = combat.in_spell;
    state.globals.spell_info = combat.spell_info;
    state.globals.cast_identity = combat.cast_identity;
    state.globals.weapon_traits = combat.weapon_traits;
    state.reload_battle_land()?;
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
    state.ordinary_battle_rewards = None;
    state.selected_battle_treasure = None;
    state.battle_after_message = 0;
    for item in &mut state.booty_list {
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
        crate::story::npc_battle::finish(
            outcome,
            &mut state.warriors,
            [position.last_move[0] as u8, position.last_move[1] as u8],
        )?;
        match outcome {
            crate::story::npc_battle::NpcBattleReturn::EscapeScript(id) => replacement = Some(id),
            crate::story::npc_battle::NpcBattleReturn::EscapeMove => {
                events.extend([
                    Event::Feedback(GameFeedback::Warning { index: 118 }),
                    Event::Sound { resource_id: 26260 },
                    Event::Feedback(GameFeedback::Warning { index: 124 }),
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
            position.script = Some(ScriptState::scheduled(record, 0));
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
