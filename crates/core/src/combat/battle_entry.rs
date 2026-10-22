//! Battle entry for combat.
use crate::combat::{
    Combat, EncounterBattle, allies, build_tactical_map, direction_offset, empty_field_used,
    empty_spell_fields, entry_party, place_enemies_with_allies, place_squad, round_stories,
    runtime,
};
use crate::{Event, Phase, RandomState, RuleError, Session};
use centurions_content::combat::{BattleRecord, MonsterRecord};
use centurions_content::{Content, word};
use serde::{Deserialize, Serialize};

/// CODE 1 newland opcode 2, after the ten-byte extra record is loaded.
pub fn encounter_battle(
    extra: &[u8],
    random: &mut RandomState,
) -> Result<EncounterBattle, RuleError> {
    let first = word(extra, 0)?;
    let last = word(extra, 2)?;
    let sound = word(extra, 4)?;
    let message = word(extra, 6)?;
    let treasure = word(extra, 8)?;
    let battle = if last == 0 {
        first
    } else {
        random.range(first, last)
    };
    Ok(EncounterBattle {
        battle,
        sound,
        message,
        treasure,
    })
}

pub fn begin(
    state: &mut Session,
    content: &Content,
    encounter_id: i16,
) -> Result<Vec<Event>, RuleError> {
    let mission = state
        .land(content)
        .ok_or_else(|| RuleError::Invalid("mission missing".into()))?;
    let tables = mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat tables missing".into()))?;
    let encounter = centurions_content::span(
        &tables.encounters,
        usize::from(encounter_id.unsigned_abs()) * 10,
        10,
    )?;
    let plan = encounter_battle(encounter, &mut state.random)?;
    let mut effects = vec![];
    let sound = plan.sound;
    if sound != 0 {
        effects.push(Event::Sound { resource_id: sound });
    }
    let message = plan.message;
    if message != 0 {
        effects.push(Event::Message {
            id: message,
            text: mission.dialogue(message)?,
        });
    }
    effects.extend(begin_battle(state, content, plan.battle, 0)?);
    state.battle_treasure = plan.treasure;
    Ok(effects)
}

pub fn begin_battle(
    state: &mut Session,
    content: &Content,
    id: i16,
    advantage: i16,
) -> Result<Vec<Event>, RuleError> {
    begin_battle_with_group(state, content, id, advantage, false)
}

pub fn begin_group(
    state: &mut Session,
    content: &Content,
    parameter: i16,
) -> Result<Vec<Event>, RuleError> {
    begin_group_inner(state, content, parameter, false)
}

pub fn begin_selected_battle(
    state: &mut Session,
    content: &Content,
    parameter: i16,
) -> Result<Vec<Event>, RuleError> {
    begin_group_inner(state, content, parameter, true)
}

pub(super) fn begin_group_inner(
    state: &mut Session,
    content: &Content,
    parameter: i16,
    treasure: bool,
) -> Result<Vec<Event>, RuleError> {
    let map = content
        .map(
            state
                .exploration
                .as_ref()
                .ok_or_else(|| RuleError::Invalid("exploration missing".into()))?
                .map,
        )
        .ok_or_else(|| RuleError::Invalid("map missing".into()))?;
    let extra = centurions_content::span(
        &map.combat.as_ref().unwrap().encounters,
        parameter.unsigned_abs() as usize * 10,
        10,
    )?;
    let plan = crate::story_party::group_encounter(extra, &mut state.random)?;
    if treasure && plan.failure_script != 0 {
        return Err(RuleError::Unsupported {
            action: "story.selected_battle",
            dependency: "additional post-battle treasure continuation",
        });
    }
    let mut events = vec![];
    if plan.sound != 0 {
        events.push(Event::Sound {
            resource_id: plan.sound,
        });
    }
    if plan.message != 0 {
        events.push(Event::Message {
            id: plan.message,
            text: map.dialogue(plan.message)?,
        });
    }
    events.extend(begin_battle_with_group(
        state,
        content,
        plan.battle,
        0,
        true,
    )?);
    state.battle_group_script = Some(plan.failure_script);
    Ok(events)
}

pub fn begin_npc(
    state: &mut Session,
    content: &Content,
    parameter: i16,
) -> Result<Vec<Event>, RuleError> {
    let position = state
        .exploration
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("exploration missing".into()))?;
    let script = position
        .script
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("NPC battle script missing".into()))?;
    let map = content
        .map(position.map)
        .ok_or_else(|| RuleError::Invalid("map missing".into()))?;
    let extra = centurions_content::span(
        &map.combat.as_ref().unwrap().encounters,
        parameter.unsigned_abs() as usize * 10,
        10,
    )?;
    let mut start = crate::story_npc_battle::NpcBattleStart::capture(
        extra,
        &script.record,
        i16::from(script.cursor),
    )?;
    let mut events = vec![];
    if start.sound != 0 {
        events.push(Event::Sound {
            resource_id: start.sound,
        });
    }
    if start.text != 0 {
        events.push(Event::Message {
            id: start.text,
            text: map.dialogue(start.text)?,
        });
    }
    let battle = start.select_battle(&mut state.random);
    events.extend(begin_battle(state, content, battle, 0)?);
    state.battle_npc = Some(start);
    Ok(events)
}

pub(super) fn begin_battle_with_group(
    state: &mut Session,
    content: &Content,
    id: i16,
    advantage: i16,
    group: bool,
) -> Result<Vec<Event>, RuleError> {
    state.retain_npcs();
    let mission = state
        .land(content)
        .ok_or_else(|| RuleError::Invalid("mission missing".into()))?;
    let tables = mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat tables missing".into()))?;
    let battle = BattleRecord::read(&tables.battles, id)?;
    let exploration = state
        .exploration
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("exploration missing".into()))?;
    let mut active_mission = mission.clone();
    active_mission.floor.clone_from(&exploration.floor);
    active_mission.events.clone_from(&exploration.events);
    let mut effects = vec![];
    if battle.before_message != 0 {
        effects.push(Event::Message {
            id: battle.before_message,
            text: mission.dialogue(battle.before_message)?,
        });
    }
    let mut map = build_tactical_map(
        &active_mission,
        (exploration.x, exploration.y),
        0,
        &mut state.random,
    )?;
    let direction = state.random.bound(360);
    state.native_work.npc_backing.rebase_origin(map.shift);
    let distance = state.random.bound(i16::from(battle.distance));
    let base = direction_offset(direction, distance);
    let mut squad = place_squad(
        &mut map,
        mission,
        content,
        &mut state.warriors,
        battle.squad_start,
        direction,
        &tables.race_moves,
    )?;
    let allies = allies::place_allies(
        &state.allies,
        if state.suppress_allies == 0 {
            state.ally_count
        } else {
            0
        },
        &mut map,
        mission,
        squad.positions[0],
        (5, 5, 85, 85),
        &mut state.random,
    )?;
    let mut enemies = place_enemies_with_allies(
        &mut map,
        mission,
        &battle,
        &tables.monsters,
        base,
        &mut state.random,
        allies,
        state.difficulty,
    )?;
    for (monster, position) in enemies.records.iter_mut().zip(&enemies.positions) {
        if position.0 < squad.positions[0].0 {
            monster.record_mut().set_horizontal_direction(1);
        } else if position.0 > squad.positions[0].0 {
            monster.record_mut().set_horizontal_direction(-1);
        }
    }
    let enemy_speeds = enemies
        .records
        .iter()
        .map(|monster| monster.speed())
        .collect::<Result<Vec<_>, _>>()?;
    let advantage = if id < 0 { -1 } else { advantage };
    let unavailable = state.combat.as_ref().map_or_else(
        || {
            crate::story_health::context(
                &state.warriors,
                state.selected_warrior,
                state.native_work.health_context.as_ref(),
            )
            .unavailable
        },
        |combat| combat.unavailable,
    );
    let party_order = entry_party::initiative(
        &state.warriors,
        &enemy_speeds,
        advantage,
        unavailable,
        &mut state.random,
    )?;
    state.warriors = party_order.warriors;
    let mut order = party_order.order;
    let setup = crate::story_party::apply_group_mask(
        &state.warriors,
        &map,
        &squad,
        &order,
        &state.native_work.party_mask,
        group,
    )?;
    state.warriors = setup.warriors;
    map = setup.map;
    squad = setup.squad;
    order = setup.order;
    let cursor = initial_cursor(&order)?;
    let count = enemies.records.len();
    let mut death_queue = state
        .combat
        .as_ref()
        .map(|combat| combat.death_queue.clone())
        .unwrap_or_else(|| {
            state
                .native_work
                .retained_death_queue
                .clone()
                .unwrap_or_default()
        });
    death_queue.reset_for_battle();
    state.handled_deaths.clear();
    state.combat_story = None;
    state.battle_treasure = 0;
    state.battle_after_message = battle.after_message;
    state.battle_group_script = None;
    state.battle_npc = None;
    let (attack_packet, attack_identity) = state.combat.as_ref().map_or_else(
        || {
            (
                state.native_work.attack_packet.clone(),
                state.native_work.attack_identity,
            )
        },
        |combat| (combat.attack_packet.clone(), combat.attack_identity),
    );
    let (selected_enemy, script_enemy) = state.combat.as_ref().map_or(
        (
            state.native_work.selected_enemy,
            state.native_work.script_enemy,
        ),
        |combat| (combat.selected_enemy, combat.script_enemy),
    );
    state.combat = Some(Combat {
        difficulty: state.difficulty,
        battle: id,
        map,
        squad,
        enemies,
        retained_npcs: state.native_work.npc_backing.clone(),
        direct_deaths: false,
        direct_death: None,
        order,
        cursor,
        round: 0,
        inactive_rounds: 0,
        selection_pending: false,
        script_enemy,
        selected_enemy,
        round_scripts: round_stories::RoundScripts::read(
            &mission.combat.as_ref().unwrap().battles,
            id,
        )?,
        death_queue,
        spell_fields: empty_spell_fields(),
        field_used: empty_field_used(),
        attack_packet,
        attack_identity,
        weapon_work: state.native_work.weapon_work,
        unavailable,
        pending_last_defeated: None,
        firearm_selection: None,
        capture_actor: None,
        automatic: [false; 6],
        turn_origin: None,
    });
    if let Some(packet) = state.combat.as_mut().unwrap().attack_packet.as_mut() {
        packet[19] = 0;
    }
    state.phase = Phase::Combat;
    state.bandage_ready = Some(1);
    state.native_work.party_mask.fill(0);
    effects.push(Event::CombatStarted {
        battle: id,
        enemies: count,
    });
    let (turn, skipped) = runtime::initialize_active_turn(
        state.combat.as_mut().unwrap(),
        &mut state.warriors,
        mission,
        &mut state.random,
    )?;
    effects.extend(turn);
    if skipped {
        effects.extend(runtime::end_turn(state, content)?);
    }
    Ok(effects)
}
pub(super) fn initial_cursor(order: &[i16]) -> Result<usize, RuleError> {
    order
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, body)| **body >= 0)
        .map(|(index, _)| index)
        .ok_or_else(|| RuleError::Invalid("no initial combatant".into()))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TacticalMap {
    pub tiles: Vec<i16>,
    pub edge_word: i16,
    pub shift: (i16, i16),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SquadPlacement {
    pub positions: Vec<(i16, i16)>,
    pub under_tiles: Vec<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnemySetup {
    pub records: Vec<MonsterRecord>,
    pub positions: Vec<(i16, i16)>,
}
