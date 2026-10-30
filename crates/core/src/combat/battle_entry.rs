//! Starting a battle: the map, the squad, the monsters and the first round.
use crate::combat::{
    Combat, EncounterBattle, allies, build_tactical_map, direction_offset, entry_party,
    place_enemies_with_allies, place_squad, round_stories, runtime,
};
use crate::{Event, Phase, RandomState, RuleError, Session};
use centurions_content::combat::{BattleRecord, MonsterRecord};
use centurions_content::{Content, word};
use serde::{Deserialize, Serialize};

/// Reads an encounter record's battle, sound, message and treasure. The battle is the first number,
/// or a random pick from the first to the last when the last is not zero.
///
/// Original: newland, combatsetup
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

/// Starts the battle named by an encounter record: plays the encounter's sound, shows its message,
/// then enters the battle with the encounter's treasure.
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
    state.globals.extra_record = encounter.to_vec();
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

/// Starts a battle by its battle record, with the given initiative advantage.
pub fn begin_battle(
    state: &mut Session,
    content: &Content,
    id: i16,
    advantage: i16,
) -> Result<Vec<Event>, RuleError> {
    begin_battle_with_group(state, content, id, advantage, false, None, None)
}

/// Starts a group battle for a story's group encounter.
pub fn begin_group(
    state: &mut Session,
    content: &Content,
    parameter: i16,
) -> Result<Vec<Event>, RuleError> {
    begin_group_inner(state, content, parameter, false)
}

/// Starts a group battle whose treasure comes from the story that selected it.
pub fn begin_selected_battle(
    state: &mut Session,
    content: &Content,
    parameter: i16,
) -> Result<Vec<Event>, RuleError> {
    begin_group_inner(state, content, parameter, true)
}

/// Enters a group battle for a story caller; `treasure` is true when the story that selected the
/// battle supplies the treasure.
pub(super) fn begin_group_inner(
    state: &mut Session,
    content: &Content,
    parameter: i16,
    treasure: bool,
) -> Result<Vec<Event>, RuleError> {
    begin_group_return(state, content, parameter, treasure, false)
}

/// Starts the group battle that a selected battle asks for while another battle is running.
pub(crate) fn begin_selected_child(
    state: &mut Session,
    content: &Content,
    parameter: i16,
) -> Result<Vec<Event>, RuleError> {
    begin_group_return(state, content, parameter, true, true)
}

/// Enters the group battle an encounter record names. A selected battle (`treasure` true) asked for
/// while a battle is running is refused unless it comes through `begin_selected_child`.
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
    state.globals.extra_record = extra.to_vec();
    let plan = crate::story::party::group_encounter(extra, &mut state.random)?;
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

/// Starts the battle an NPC's script asks for: the NPC's sound and text come first, the battle is
/// drawn from the encounter record, and the NPC is remembered for the end of the battle.
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
    state.globals.extra_record = extra.to_vec();
    let mut start = crate::story::npc_battle::NpcBattleStart::capture(
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

/// Sets up and enters a battle: builds the tactical map around the party, places the squad, allies
/// and monsters, deals the battle order, resets the death and reward state and runs the first turn.
/// The advantage is forced to -1 for a negative battle number.
///
/// Original: combatsetup
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
    state.store_monsters();
    state.char_select_new = 0;
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
        land.floor.clone_from(&combat.field.tiles);
    }
    let saved_land = super::battlefield::BattleLand {
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
    state.globals.monsters.rebase_origin(map.map_shift);
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
        &state.holdover,
        if state.suspend_allies == 0 {
            state.held_over
        } else {
            0
        },
        &mut map,
        mission,
        squad.pos[0],
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
    for (monster, position) in enemies.records.iter_mut().zip(&enemies.mon_pos) {
        if position.0 < squad.pos[0].0 {
            monster.record_mut().set_horizontal_direction(1);
        } else if position.0 > squad.pos[0].0 {
            monster.record_mut().set_horizontal_direction(-1);
        }
    }
    let enemy_speeds = enemies
        .records
        .iter()
        .map(|monster| monster.agility())
        .collect::<Result<Vec<_>, _>>()?;
    let advantage = if id < 0 { -1 } else { advantage };
    let kill_party = if let Some(combat) = state.combat.as_ref() {
        combat.kill_party
    } else {
        crate::story::health::reselected_context(
            state.char_select_new,
            &state.globals.health_context,
        )
        .kill_party
    };
    let party_order = entry_party::initiative(
        &state.warriors,
        &enemy_speeds,
        advantage,
        kill_party,
        &mut state.random,
    )?;
    state.warriors = party_order.warriors;
    let mut order = party_order.order;
    let setup = crate::story::party::apply_group_mask(
        &state.warriors,
        &map,
        &squad,
        &order,
        &state.globals.party_track,
        group,
    )?;
    state.warriors = setup.warriors;
    map = setup.map;
    squad = setup.squad;
    order = setup.order;
    let cursor = initial_cursor(&order)?;
    let count = enemies.records.len();
    let mut todo_queue = state
        .combat
        .as_ref()
        .map(|combat| combat.todo_queue.clone())
        .unwrap_or_else(|| state.globals.outer_todo_queue.clone().unwrap_or_default());
    todo_queue.reset_for_battle();
    state.globals.death_processing.doing_queue = 0;
    state.globals.battle_rewards_suppressed = 0;
    // The battle record overwrites the bestiary's cached record.
    state.globals.bestiary.battle_record_cold = false;
    state.globals.coward = 0;
    state.handled_deaths.clear();
    state.combat_story = None;
    state.battle_treasure = ordinary.unwrap_or(0);
    state.ordinary_battle_rewards = ordinary.map(|_| runtime::OrdinaryBattleRewards::default());
    state.battle_after_message = battle.after_message;
    state.battle_group_script = None;
    state.battle_npc = None;
    let (spell_info, cast_identity) = state.combat.as_ref().map_or_else(
        || {
            (
                state.globals.spell_info.clone(),
                state.globals.cast_identity,
            )
        },
        |combat| (combat.spell_info.clone(), combat.cast_identity),
    );
    let (monster_up, macro_monster) = state.combat.as_ref().map_or(
        (state.globals.monster_up, state.globals.macro_monster),
        |combat| (combat.monster_up, combat.macro_monster),
    );
    let in_spell = state
        .combat
        .as_ref()
        .map_or(state.globals.in_spell, |combat| combat.in_spell);
    // Counts the monsters that join the battle; the battle order starts with
    // none defeated.
    let total = enemies.records.iter().try_fold(0_i16, |total, monster| {
        Ok::<_, RuleError>(total.wrapping_add(i16::from(monster.record().faction()? != 0)))
    })?;
    let mut combat = Combat::from_records(map, squad, enemies);
    combat.kill_counts = super::runtime::kill_counts::KillCounts {
        num_enemy: total,
        kill_mon: 0,
    };
    combat.neighbors = state.globals.neighbors.clone();
    combat.in_spell = in_spell;
    combat.difficulty = state.difficulty;
    combat.battle = id;
    combat.monsters = state.globals.monsters.clone();
    combat.battle_order = order;
    combat.battle_order_index = cursor;
    combat.macro_monster = macro_monster;
    combat.monster_up = monster_up;
    combat.round_macros =
        round_stories::RoundMacros::read(&mission.combat.as_ref().unwrap().battles, id)?;
    combat.todo_queue = todo_queue;
    combat.spell_info = spell_info;
    combat.cast_identity = cast_identity;
    combat.weapon_traits = state.globals.weapon_traits;
    combat.kill_party = kill_party;
    // The battle keeps the target mask's tail from before it began.
    combat.track_tail.clone_from(&state.globals.track_tail);
    combat.target_picks = state.globals.target_picks;
    let battlefield = &state.globals.battlefield;
    combat.spell_queue.clone_from(&battlefield.spell_queue);
    combat.collide_check.clone_from(&battlefield.collide_check);
    let mut battlefield = super::battlefield::Battlefield::capture(&combat, battlefield);
    battlefield.saved_land = Some(saved_land);
    state.globals.battlefield = battlefield;
    state.combat = Some(combat);
    state.combat.as_mut().unwrap().spell_info[19] = 0;
    state.phase = Phase::Combat;
    state.selected_battle_treasure = additional;
    state.bandage_ready = 1;
    state.globals.item_used = 0;
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
/// The first slot of the battle order that holds a combatant. Slot zero is never a turn.
pub(super) fn initial_cursor(order: &[i16]) -> Result<usize, RuleError> {
    order
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, body)| **body >= 0)
        .map(|(index, _)| index)
        .ok_or_else(|| RuleError::Invalid("no initial combatant".into()))
}

/// The tactical map: one tile value per cell, the value read for a cell beyond the edge, and the
/// shift that maps a position to a cell.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CombatField {
    pub tiles: Vec<i16>,
    pub edge_word: i16,
    #[serde(rename = "shift")]
    pub map_shift: (i16, i16),
}

/// Where the six warriors stand on the tactical map, and the tile under each.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SquadPlacement {
    #[serde(rename = "positions")]
    pub pos: Vec<(i16, i16)>,
    #[serde(rename = "under_tiles")]
    pub char_under: Vec<i16>,
}

/// The monsters a battle starts with and where each stands.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnemySetup {
    pub records: Vec<MonsterRecord>,
    #[serde(rename = "positions")]
    pub mon_pos: Vec<(i16, i16)>,
}
