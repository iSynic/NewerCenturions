//! Battle entry for combat.
use crate::combat::{
    Combat, EncounterBattle, allies, build_tactical_map, direction_offset, entry_party,
    place_enemies_with_allies, place_squad, round_stories, runtime,
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
    state.native_work.extra_record = encounter.to_vec();
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
    effects.extend(begin_battle_with_group(
        state,
        content,
        plan.battle,
        0,
        false,
        None,
        Some(plan.treasure),
    )?);
    Ok(effects)
}

pub fn begin_battle(
    state: &mut Session,
    content: &Content,
    id: i16,
    advantage: i16,
) -> Result<Vec<Event>, RuleError> {
    begin_battle_with_group(state, content, id, advantage, false, None, None)
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
    begin_group_return(state, content, parameter, treasure, false)
}

pub(crate) fn begin_selected_child(
    state: &mut Session,
    content: &Content,
    parameter: i16,
) -> Result<Vec<Event>, RuleError> {
    begin_group_return(state, content, parameter, true, true)
}

fn begin_group_return(
    state: &mut Session,
    content: &Content,
    parameter: i16,
    treasure: bool,
    child_return: bool,
) -> Result<Vec<Event>, RuleError> {
    if treasure && state.combat.is_some() && !child_return {
        return Err(RuleError::Unsupported {
            action: "story.selected_battle",
            dependency: "child battle without a suspended selected caller",
        });
    }
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
    state.native_work.extra_record = extra.to_vec();
    let plan = crate::story_party::group_encounter(extra, &mut state.random)?;
    let additional = if treasure {
        Some(runtime::SelectedBattleTreasure::new(
            state,
            plan.failure_script,
        )?)
    } else {
        None
    };
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
        additional,
        None,
    )?);
    state.battle_group_script = Some(if treasure { 0 } else { plan.failure_script });
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
    state.native_work.extra_record = extra.to_vec();
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
    additional: Option<runtime::SelectedBattleTreasure>,
    ordinary: Option<i16>,
) -> Result<Vec<Event>, RuleError> {
    if state.selected_battle_treasure.is_some() {
        return Err(RuleError::Unsupported {
            action: "story.selected_battle",
            dependency: "child battle without a suspended selected caller",
        });
    }
    state.retain_npcs();
    // combatsetup11d452 resets the selected WORD before the first Getup.
    state.selected_warrior = 0;
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
    let mut land = exploration.map_state();
    if let Some(combat) = &state.combat {
        land.floor.clone_from(&combat.map.tiles);
    }
    let saved_land = super::tactical_work::BattleLand {
        level: state.scenario_level,
        map: exploration.map,
        records: land,
    };
    let mut active_mission = mission.clone();
    active_mission.floor.clone_from(&saved_land.records.floor);
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
    let unavailable = if let Some(combat) = state.combat.as_ref() {
        combat.unavailable
    } else {
        crate::story_health::retained_context(
            state.selected_warrior,
            &state.native_work.health_context,
        )
        .unavailable
    };
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
    state.native_work.death_processing.queue = 0;
    state.native_work.battle_rewards_suppressed = 0;
    // combatsetup reads the battle record over the bestiary loop's tail at43979c.
    state.native_work.bestiary.battle_record_cold = false;
    state.native_work.early_booty = 0;
    state.handled_deaths.clear();
    state.combat_story = None;
    state.battle_treasure = ordinary.unwrap_or(0);
    state.ordinary_battle_rewards = ordinary.map(|_| runtime::OrdinaryBattleRewards::default());
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
    let ready = state
        .combat
        .as_ref()
        .map_or(state.native_work.ready, |combat| combat.ready);
    // combatsetup counts each newly created nonzero-faction NPC; getbattleorder
    // then clears the defeated WORD. This is an entry writer, not a live recount.
    let total = enemies.records.iter().try_fold(0_i16, |total, enemy| {
        Ok::<_, RuleError>(total.wrapping_add(i16::from(enemy.record().faction()? != 0)))
    })?;
    let mut combat = Combat::from_records(map, squad, enemies);
    combat.victory_work = super::victory_work::VictoryWork { total, defeated: 0 };
    combat.neighbor_work = state.native_work.neighbor_work.clone();
    combat.ready = ready;
    combat.difficulty = state.difficulty;
    combat.battle = id;
    combat.retained_npcs = state.native_work.npc_backing.clone();
    combat.order = order;
    combat.cursor = cursor;
    combat.script_enemy = script_enemy;
    combat.selected_enemy = selected_enemy;
    combat.round_scripts =
        round_stories::RoundScripts::read(&mission.combat.as_ref().unwrap().battles, id)?;
    combat.death_queue = death_queue;
    combat.attack_packet = attack_packet;
    combat.attack_identity = attack_identity;
    combat.weapon_work = state.native_work.weapon_work;
    combat.unavailable = unavailable;
    // combatsetup leaves the target mask; live combat carries its tail.
    combat
        .target_tail
        .clone_from(&state.native_work.spell_target_tail);
    combat.target_work = state.native_work.target_work;
    let work = &state.native_work.tactical_work;
    combat.spell_fields.clone_from(&work.spell_fields);
    combat.field_used.clone_from(&work.field_used);
    let mut work = super::tactical_work::TacticalWork::capture(&combat, work);
    work.saved_land = Some(saved_land);
    state.native_work.tactical_work = work;
    state.combat = Some(combat);
    state.combat.as_mut().unwrap().attack_packet[19] = 0;
    state.phase = Phase::Combat;
    state.selected_battle_treasure = additional;
    state.bandage_ready = 1;
    // getup 125a92 clears WORD44719a.
    state.native_work.fire_item = 0;
    // getup 125a50 calls cleartarget.
    state.clear_targets();
    effects.push(Event::CombatStarted {
        battle: id,
        enemies: count,
    });
    let (turn, initialized) =
        runtime::initialize_session_turn(state, content, runtime::TurnCaller::Initial)?;
    effects.extend(turn);
    if matches!(initialized, runtime::TurnInitialization::Ready(true)) {
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
