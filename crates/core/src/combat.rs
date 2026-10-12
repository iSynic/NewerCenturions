use super::*;
use centurions_content::combat::{BattleRecord, MonsterRecord};
pub mod allies;
pub mod ally_turns;
pub mod attack;
pub mod damage;
pub mod death_scripts;
pub mod enemy;
pub mod enemy_weapons;
pub mod entry_party;
pub mod final_turn_stories;
pub mod firearm;
pub mod footprint;
pub mod lifecycle;
pub mod medical;
pub mod mission_deaths;
pub mod natural_conditions;
pub mod party_buffs;
pub mod player_psi;
pub mod rewards;
pub mod round_stories;
pub mod runtime;
pub mod spell_conditions;
pub mod spells;
pub mod weapon_work;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Combat {
    pub battle: i16,
    pub map: TacticalMap,
    pub squad: SquadPlacement,
    pub enemies: EnemySetup,
    pub order: Vec<i16>,
    pub cursor: usize,
    pub round: i16,
    #[serde(default)]
    pub round_scripts: round_stories::RoundScripts,
    #[serde(default)]
    pub death_queue: death_scripts::DeathQueue,
    #[serde(default = "empty_spell_fields")]
    pub spell_fields: Vec<u8>,
    #[serde(default = "empty_field_used")]
    pub field_used: Vec<u8>,
    #[serde(default)]
    pub attack_packet: Option<Vec<u8>>,
    #[serde(default)]
    pub attack_identity: Option<[i16; 3]>,
    #[serde(default)]
    pub weapon_work: weapon_work::WeaponWork,
    #[serde(default)]
    pub unavailable: i8,
}
fn empty_spell_fields() -> Vec<u8> {
    vec![0; 1200]
}
fn empty_field_used() -> Vec<u8> {
    vec![0; 128]
}
impl Combat {
    pub fn validate(&self, warriors: &[Warrior]) -> Result<(), RuleError> {
        if warriors.len() != 6
            || self.map.tiles.len() != 8100
            || self.squad.positions.len() != 6
            || self.squad.under_tiles.len() != 6
            || self.enemies.records.len() > 100
            || self.enemies.positions.len() != self.enemies.records.len()
            || self.order.len() != 110
            || self.cursor >= 110
            || self.round < 0
            || self.death_queue.entries.len() != 100
            || self.death_queue.count > 100
            || self.death_queue.cursor > 100
            || self.spell_fields.len() != 1200
            || self.field_used.len() != 128
            || self
                .attack_packet
                .as_ref()
                .is_some_and(|packet| packet.len() != 58)
            || self.enemies.records.iter().any(|m| m.bytes.len() != 288)
            || self.order.iter().any(|id| {
                *id < -1 || (*id >= 6 && *id < 10) || *id >= self.enemies.records.len() as i16 + 10
            })
            || self
                .squad
                .positions
                .iter()
                .enumerate()
                .any(|(index, (x, y))| {
                    if (*x, *y) == (-1, -1)
                        && warriors[index].bytes[0x9df] == 0
                        && word(&warriors[index].bytes, 0x20).unwrap_or(0) == -1
                    {
                        return false;
                    }
                    let x = i32::from(*x) + i32::from(self.map.shift.0) + 5;
                    let y = i32::from(*y) + i32::from(self.map.shift.1) + 5;
                    !(0..90).contains(&x) || !(0..90).contains(&y)
                })
            || self.enemies.positions.iter().any(|(x, y)| {
                let x = i32::from(*x) + i32::from(self.map.shift.0) + 5;
                let y = i32::from(*y) + i32::from(self.map.shift.1) + 5;
                !(0..90).contains(&x) || !(0..90).contains(&y)
            })
        {
            return Err(RuleError::Invalid("invalid tactical state".into()));
        }
        Ok(())
    }
    pub fn active_body(&self) -> Option<i16> {
        self.order.get(self.cursor).copied().filter(|id| *id >= 0)
    }
}

pub fn move_warrior(
    state: &mut Session,
    content: &Content,
    dx: i8,
    dy: i8,
) -> Result<Vec<Event>, RuleError> {
    if !(-1..=1).contains(&dx) || !(-1..=1).contains(&dy) || (dx == 0 && dy == 0) {
        return Err(RuleError::Invalid("invalid tactical direction".into()));
    }
    let combat = state
        .combat
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
    let body = combat
        .active_body()
        .ok_or_else(|| RuleError::Invalid("no active body".into()))?;
    if !(0..6).contains(&body) {
        return Err(RuleError::Unsupported {
            action: "combat.move",
            dependency: "enemy turn dispatch",
        });
    }
    let body = body as usize;
    let mission = content
        .map(
            state
                .exploration
                .as_ref()
                .map_or(0, |position| position.map),
        )
        .ok_or_else(|| RuleError::Invalid("mission missing".into()))?;
    let position = combat.squad.positions[body];
    let destination = (
        position.0.wrapping_add(i16::from(dx)),
        position.1.wrapping_add(i16::from(dy)),
    );
    let origin = (combat.map.shift.0 + 5, combat.map.shift.1 + 5);
    let current_index = centurions_content::mission::OpeningMission::index(
        i32::from(position.0 + origin.0),
        i32::from(position.1 + origin.1),
    )?;
    let next_index = centurions_content::mission::OpeningMission::index(
        i32::from(destination.0 + origin.0),
        i32::from(destination.1 + origin.1),
    )?;
    let tile = combat.map.tiles[next_index].wrapping_abs();
    if tile <= 110 {
        return reload_at_occupied_cell(state, content, mission, body, tile, destination, dx);
    }
    if tile < 1000 {
        return Err(RuleError::Unsupported {
            action: "combat.move",
            dependency: "non-terrain tactical marker",
        });
    }
    if word(&state.warriors[body].bytes, 0x90)? != 0 {
        return Err(RuleError::Unsupported {
            action: "combat.move",
            dependency: "involuntary movement",
        });
    }
    let definitions = &content.items;
    let attacks = mission
        .combat
        .as_ref()
        .map_or(&[][..], |tables| &tables.attacks);
    let mut effects = runtime::check_neighbors_with_work(
        combat,
        &mut state.warriors,
        &mut state.random,
        body as i16,
        definitions,
        attacks,
        &state.party_buffs,
        &mut state.ammunition_work,
    )?;
    let bytes = &mut state.warriors[body].bytes;
    if dx < 0 {
        bytes[0x9e0] = 1;
    } else if dx > 0 {
        bytes[0x9e0] = 0;
    }
    if mission.tile_word(tile - 1000, 4)? != 0 {
        effects.push(Event::Sound { resource_id: -129 });
        effects.push(Event::Sound { resource_id: -130 });
        return Ok(effects);
    }
    let cost = crate::recovered_preparation::movement_cost(&mission.tile_properties, tile, dx, dy)?;
    let remaining = word(bytes, 0x3c)?.wrapping_sub(cost);
    if remaining < 0 {
        effects.push(Event::Sound { resource_id: -130 });
        return Ok(effects);
    }
    effects.extend(runtime::warrior_departure_with_work(
        combat,
        &mut state.warriors,
        &mut state.random,
        body as i16,
        destination,
        definitions,
        attacks,
        &state.party_buffs,
        &mut state.ammunition_work,
    )?);
    if state.warriors[body].bytes[0x9df] == 0 {
        effects.extend(runtime::end_turn(state, content)?);
        return Ok(effects);
    }
    let remaining = word(&state.warriors[body].bytes, 0x3c)?.wrapping_sub(cost);
    put_word(&mut state.warriors[body].bytes, 0x3c, remaining);
    combat.map.tiles[current_index] = combat.squad.under_tiles[body];
    combat.squad.under_tiles[body] = combat.map.tiles[next_index];
    combat.map.tiles[next_index] = body as i16;
    combat.squad.positions[body] = destination;
    effects.extend([
        Event::TacticalMoved {
            body: body as i16,
            x: destination.0,
            y: destination.1,
        },
        Event::Sound {
            resource_id: mission.tile_word(tile - 1000, 0)?,
        },
    ]);
    effects.extend(runtime::collide_fields_with_mask(
        combat,
        &mut state.warriors,
        mission,
        &mut state.attack_count,
        &mut state.random,
        body as i16,
        &mut state.party_mask,
    )?);
    if state.warriors[body].stamina()? < 1 {
        put_word(&mut state.warriors[body].bytes, 0x40, 0);
        put_word(&mut state.warriors[body].bytes, 0x3c, 0);
    } else {
        effects.extend(runtime::check_neighbors_with_work(
            combat,
            &mut state.warriors,
            &mut state.random,
            body as i16,
            definitions,
            attacks,
            &state.party_buffs,
            &mut state.ammunition_work,
        )?);
        if state.warriors[body].bytes[0x9df] == 0 {
            effects.extend(runtime::end_turn(state, content)?);
        }
    }
    Ok(effects)
}

fn reload_at_occupied_cell(
    state: &mut Session,
    content: &Content,
    mission: &centurions_content::mission::OpeningMission,
    actor: usize,
    target: i16,
    destination: (i16, i16),
    dx: i8,
) -> Result<Vec<Event>, RuleError> {
    let unsupported = || RuleError::Unsupported {
        action: "combat.move",
        dependency: "other occupied-cell attack, exchange or reload continuation",
    };
    let combat = state.combat.as_mut().ok_or_else(unsupported)?;
    let member = &state.warriors[actor];
    if actor != 4
        || word(&member.bytes, 0x34)? != 4
        || member.stamina()? <= 0
        || member.bytes[0x9df] != 1
        || word(&member.bytes, 0x10)? != 0
        || word(&member.bytes, 0x90)? != 0
        || word(&member.bytes, 0x98)? != 0
        || word(&member.bytes, 0x40)? < 2
        || !(10..110).contains(&target)
    {
        return Err(unsupported());
    }
    let enemy = combat
        .enemies
        .records
        .get((target - 10) as usize)
        .ok_or_else(unsupported)?;
    let absolute = (
        i32::from(destination.0) + i32::from(combat.map.shift.0) + 5,
        i32::from(destination.1) + i32::from(combat.map.shift.1) + 5,
    );
    if !(3..=86).contains(&absolute.0)
        || !(3..=86).contains(&absolute.1)
        || combat.enemies.positions[(target - 10) as usize] != destination
        || enemy.bytes[0xb0] != 0
        || word(&enemy.bytes, 0xe)? == word(&member.bytes, 0x10)?
        || runtime::neighbors(combat, &state.warriors, actor as i16)?
            .iter()
            .any(|body| *body < 10)
    {
        return Err(unsupported());
    }
    let attacks = mission
        .combat
        .as_ref()
        .map_or(&[][..], |tables| &tables.attacks);
    let effects = runtime::check_neighbors_with_work(
        combat,
        &mut state.warriors,
        &mut state.random,
        actor as i16,
        &content.items,
        attacks,
        &state.party_buffs,
        &mut state.ammunition_work,
    )?;
    if dx < 0 {
        state.warriors[actor].bytes[0x9e0] = 1;
    } else if dx > 0 {
        state.warriors[actor].bytes[0x9e0] = 0;
    }
    let weapon_id = word(&state.warriors[actor].bytes, 0xe4)?;
    let equipped = state.warriors[actor]
        .inventory()?
        .first()
        .cloned()
        .ok_or_else(unsupported)?;
    if !matches!(weapon_id, 1 | 2 | 38 | 165)
        || state.warriors[actor].bytes[0x9dd] != 1
        || equipped.id() != weapon_id
        || equipped.0[9] != 1
        || word(&equipped.0, 2)? != 0
    {
        return Err(unsupported());
    }
    let weapon = crate::ammunition::definition(content, weapon_id)?;
    combat.weapon_work.load_weapon(&weapon)?;
    state.ammunition_work.weapon = Some(weapon);
    state.attack_count = 0;
    let refit = crate::ammunition::refit(
        &state.warriors[actor],
        &state.ammunition,
        content,
        crate::ammunition::RefitFlags {
            in_combat: true,
            force: true,
            silent: true,
            warnings: false,
        },
        &state.ammunition_work,
        &mut state.random,
    )?;
    let completed = refit.native_return != 0;
    let mut effects = effects;
    effects.extend(crate::ammunition::commit_result(state, actor, refit)?);
    if !completed {
        put_word(&mut state.warriors[actor].bytes, 0x90, 0);
        effects.push(Event::Sound { resource_id: 6000 });
        effects.push(Event::CollectionRejected { warning: 25 });
    }

    Ok(effects)
}

pub fn direction_offset(direction: i16, distance: i16) -> (i16, i16) {
    let radians = (f64::from(direction) / 57.3) as f32 as f64;
    (
        ((radians.cos() * f64::from(distance) - 5.0).trunc() as i32) as i16,
        ((radians.sin() * f64::from(distance) - 5.0).trunc() as i32) as i16,
    )
}

pub fn initialize_turn(
    body: i16,
    warriors: &mut [Warrior],
    enemies: &mut [MonsterRecord],
    race_moves: &[i8],
    random: &mut RandomState,
) -> Result<(), RuleError> {
    if (0..6).contains(&body) {
        let warrior = &mut warriors[body as usize];
        if [0x60, 0x98]
            .iter()
            .any(|offset| word(&warrior.bytes, *offset).unwrap_or(1) != 0)
            || word(&warrior.bytes, 0x10)? != 0
            || word(&warrior.bytes, 0x90)? != 0
        {
            return Err(RuleError::Unsupported {
                action: "combat.turn",
                dependency: "inactive or confused warrior",
            });
        }
        items::compact(warrior)?;
        let bytes = &mut warrior.bytes;
        let carry = word(bytes, 0x40)?.clamp(0, 1);
        let mut points = carry
            .wrapping_add(word(bytes, 0x12)?)
            .wrapping_add(word(bytes, 0x1e)?);
        if word(bytes, 0x8c)? != 0 {
            points = points.wrapping_add(4);
        }
        put_word(bytes, 0x40, points);
        let race = word(bytes, 0x32)? - 1;
        let race_move = *race_moves
            .get(race as usize)
            .ok_or_else(|| RuleError::Invalid("race movement missing".into()))?;
        calculate_movement(warrior, race_move)?;
        let maximum = word(&warrior.bytes, 0x3e)?;
        put_word(&mut warrior.bytes, 0x3c, maximum);
        put_word(&mut warrior.bytes, 0xacc, 0);
        put_word(&mut warrior.bytes, 0xac8, 0);
    } else if body >= 10 {
        let monster = enemies
            .get_mut((body - 10) as usize)
            .ok_or_else(|| RuleError::Invalid("enemy outside battle".into()))?;
        let bytes = &mut monster.bytes;
        if bytes[0xb3] != 0 || bytes[0xcf] != 0 || word(bytes, 0x110)? <= 0 {
            return Err(RuleError::Unsupported {
                action: "combat.turn",
                dependency: "inactive or confused enemy",
            });
        }
        let chance = word(bytes, 0x5c)?;
        if random.bound(100) <= chance {
            return Err(RuleError::Unsupported {
                action: "combat.turn",
                dependency: "enemy turn script",
            });
        }
        put_word(bytes, 0x5c, chance.wrapping_abs());
        let mut points = (bytes[0x109] as i8).min(1).wrapping_add(bytes[0x8a] as i8);
        let mut movement = (word(bytes, 6)? as i8).wrapping_sub(bytes[0xb4] as i8);
        if bytes[0xb8] != 0 {
            movement /= 2;
        }
        if bytes[0xc9] != 0 {
            movement = movement.wrapping_mul(2);
            points = points.wrapping_add(4).min(11);
        }
        bytes[0x109] = points as u8;
        bytes[0x106] = movement as u8;
    } else {
        return Err(RuleError::Invalid("invalid active body".into()));
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
pub struct EncounterBattle {
    pub battle: i16,
    pub sound: i16,
    pub message: i16,
    pub treasure: i16,
}

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
    let mission = content
        .map(
            state
                .exploration
                .as_ref()
                .map_or(0, |position| position.map),
        )
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

fn begin_battle_with_group(
    state: &mut Session,
    content: &Content,
    id: i16,
    advantage: i16,
    group: bool,
) -> Result<Vec<Event>, RuleError> {
    let mission = content
        .map(
            state
                .exploration
                .as_ref()
                .map_or(0, |position| position.map),
        )
        .ok_or_else(|| RuleError::Invalid("mission missing".into()))?;
    let tables = mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat tables missing".into()))?;
    let battle = BattleRecord::read(&tables.battles, id)?;
    if state
        .warriors
        .iter()
        .any(|w| word(&w.bytes, 0x90).unwrap_or(1) != 0)
    {
        return Err(RuleError::Unsupported {
            action: "combat.enter",
            dependency: "squad confusion or selection status",
        });
    }
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
        state.ally_count,
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
    )?;
    for (monster, position) in enemies.records.iter_mut().zip(&enemies.positions) {
        if position.0 < squad.positions[0].0 {
            monster.bytes[0x107] = 1;
        } else if position.0 > squad.positions[0].0 {
            monster.bytes[0x107] = 0xff;
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
                state.health_context.as_ref(),
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
        &state.party_mask,
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
        .unwrap_or_default();
    death_queue.reset_for_battle();
    state.handled_deaths.clear();
    state.combat_story = None;
    state.battle_treasure = 0;
    state.battle_after_message = battle.after_message;
    state.battle_group_script = None;
    state.battle_npc = None;
    let (attack_packet, attack_identity) = state.combat.as_ref().map_or_else(
        || (state.attack_packet.clone(), state.attack_identity),
        |combat| (combat.attack_packet.clone(), combat.attack_identity),
    );
    state.combat = Some(Combat {
        battle: id,
        map,
        squad,
        enemies,
        order,
        cursor,
        round: 0,
        round_scripts: round_stories::RoundScripts::read(
            &mission.combat.as_ref().unwrap().battles,
            id,
        )?,
        death_queue,
        spell_fields: empty_spell_fields(),
        field_used: empty_field_used(),
        attack_packet,
        attack_identity,
        weapon_work: state.weapon_work,
        unavailable,
    });
    if let Some(packet) = state.combat.as_mut().unwrap().attack_packet.as_mut() {
        packet[19] = 0;
    }
    state.phase = Phase::Combat;
    state.bandage_ready = Some(1);
    state.party_mask.fill(0);
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
fn initial_cursor(order: &[i16]) -> Result<usize, RuleError> {
    order
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, body)| **body >= 0)
        .map(|(index, _)| index)
        .ok_or_else(|| RuleError::Invalid("no initial combatant".into()))
}

#[cfg(all(test, feature = "original-fixtures"))]
mod initial_turn_tests {
    use super::*;
    #[test]
    fn sparse_initial_order_matches_original_getup_execution() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../../evidence/fixtures/sparse-turn.json"))
                .unwrap();
        fn bytes(value: &serde_json::Value) -> Vec<u8> {
            let hex = value.as_str().unwrap();
            (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect()
        }
        for case in fixture["cases"].as_array().unwrap() {
            let body = case["body"].as_i64().unwrap() as i16;
            let first = case["first_slot"].as_u64().unwrap() as usize;
            let mut order = vec![-1; 110];
            order[first] = body;
            let cursor = initial_cursor(&order).unwrap();
            assert_eq!(cursor, case["cursor"].as_u64().unwrap() as usize);
            let mut warriors = bytes(&case["warriors"])
                .as_chunks::<3138>()
                .0
                .iter()
                .map(|w| Warrior::read(w).unwrap())
                .collect::<Vec<_>>();
            let mut enemies = vec![MonsterRecord {
                bytes: bytes(&case["enemy"]),
            }];
            let mut random = RandomState::new(1);
            initialize_turn(
                order[cursor],
                &mut warriors,
                &mut enemies,
                &[12, 14, 12, 14, 10, 12, 10, 10, 14, 16, 0, 12],
                &mut random,
            )
            .unwrap();
            assert_eq!(
                warriors
                    .iter()
                    .flat_map(|w| w.bytes.clone())
                    .collect::<Vec<_>>(),
                bytes(&case["after_warriors"])
            );
            assert_eq!(enemies[0].bytes, bytes(&case["after_enemy"]));
            assert_eq!(random.seed, case["seed"].as_u64().unwrap() as u32);
            assert_eq!(random.draws, case["draws"].as_u64().unwrap());
        }
    }
}
use centurions_content::word;

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

fn put_word(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

pub fn place_enemies(
    map: &mut TacticalMap,
    mission: &centurions_content::mission::OpeningMission,
    battle: &BattleRecord,
    definitions: &[u8],
    base: (i16, i16),
    random: &mut RandomState,
) -> Result<EnemySetup, RuleError> {
    place_enemies_with_allies(
        map,
        mission,
        battle,
        definitions,
        base,
        random,
        EnemySetup {
            records: vec![],
            positions: vec![],
        },
    )
}

fn place_enemies_with_allies(
    map: &mut TacticalMap,
    mission: &centurions_content::mission::OpeningMission,
    battle: &BattleRecord,
    definitions: &[u8],
    base: (i16, i16),
    random: &mut RandomState,
    mut result: EnemySetup,
) -> Result<EnemySetup, RuleError> {
    for y in 0..26 {
        for x in 0..26 {
            let id = battle.cells[x * 26 + y];
            if id == 0 || result.records.len() == 100 {
                continue;
            }
            let mut monster = MonsterRecord::read(definitions, id)?;
            let bytes = &mut monster.bytes;
            let dice = word(bytes, 0)?;
            if dice == 0 {
                continue;
            }
            if !(1..=127).contains(&dice) || word(bytes, 0x4e)? < 0 {
                return Err(RuleError::Unsupported {
                    action: "combat.enemies",
                    dependency: "enemy level or random weapon tables",
                });
            }
            put_word(bytes, 0x10c, -1);
            bytes[0x82] = 0;
            bytes[0x104] = 0;
            let placed = footprint::find_placement(
                map,
                &monster,
                mission,
                result.records.len() as i16 + 10,
                (x as i16, y as i16),
                base,
                random,
            )?;
            *map = placed.map;
            monster = placed.monster;
            let position = placed.position;
            let bytes = &mut monster.bytes;
            bytes[0x107] = 1;
            bytes[0x108] = 1;
            let armor = word(bytes, 8)?;
            if armor != 0 {
                put_word(bytes, 8, armor.wrapping_add(random.bound(3) - 2));
            }
            let armor = word(bytes, 8)?.max(0);
            put_word(bytes, 8, armor);
            let speed = word(bytes, 4)?.wrapping_add(random.bound(3) - 2).max(1);
            put_word(bytes, 4, speed);
            let psi = word(bytes, 0x52)?;
            let spread = psi / 10;
            let psi = psi.wrapping_add(random.range(-spread, spread));
            put_word(bytes, 0x52, psi);
            put_word(bytes, 0x10e, psi);
            let mut stamina = word(bytes, 2)?;
            for _ in 0..dice {
                stamina = stamina.wrapping_add(random.bound(8));
            }
            stamina = stamina.max(1);
            put_word(bytes, 0x110, stamina);
            put_word(bytes, 0x112, stamina);
            if id < 0 {
                let hostile = i16::from(word(bytes, 0xe)? == 0);
                put_word(bytes, 0xe, hostile);
            }
            if word(bytes, 0x4e)? == 0 && word(bytes, 0x28)? > 0 {
                let weapon = word(bytes, 0x28)?;
                put_word(bytes, 0x4e, weapon);
            }
            result.records.push(monster);
            result.positions.push(position);
        }
    }
    Ok(result)
}

pub fn place_squad(
    map: &mut TacticalMap,
    mission: &centurions_content::mission::OpeningMission,
    content: &Content,
    warriors: &mut [Warrior],
    start: (i16, i16),
    direction: i16,
    race_moves: &[i8],
) -> Result<SquadPlacement, RuleError> {
    if warriors.len() != 6 || !(1..=360).contains(&direction) {
        return Err(RuleError::Invalid(
            "formation requires six warriors and a direction".into(),
        ));
    }
    let vertical = !(45..=315).contains(&direction) || (direction > 135 && direction < 225);
    let shift_lr = if direction > 90 && direction < 270 {
        1
    } else {
        -1
    };
    let shift_ud = if direction < 180 { -1 } else { 1 };
    let origin = (map.shift.0.wrapping_add(5), map.shift.1.wrapping_add(5));
    let mut result = SquadPlacement {
        positions: vec![],
        under_tiles: vec![],
    };
    for (index, warrior) in warriors.iter_mut().enumerate() {
        let bytes = &mut warrior.bytes;
        for offset in [0x16, 0xacc, 0xac8, 0x10, 0x14, 0x40] {
            bytes[offset..offset + 2].fill(0);
        }
        bytes[0x9de] = 0;
        bytes[0x9dd] = 0;
        bytes[0x9e0] = 1;
        let weapon = word(bytes, 0xca)?;
        let sound = if weapon != 0 {
            word(
                content
                    .items
                    .get(weapon.unsigned_abs() as usize)
                    .ok_or_else(|| RuleError::Invalid("weapon definition missing".into()))?,
                14,
            )?
        } else {
            if word(bytes, 0xe4)? != 0 {
                bytes[0x9dd] = 1;
            }
            word(bytes, 0x38)?.wrapping_mul(8).wrapping_add(30)
        };
        bytes[0x128..0x12a].copy_from_slice(&sound.to_be_bytes());
        bytes[0x18..0x1a].copy_from_slice(&(-1_i16).to_be_bytes());
        let row = index as i16 / 3;
        let mut found = None;
        'search: for ring in 1..=180_i16 {
            for dy in -ring..ring {
                for dx in -ring..ring {
                    let relative = if vertical {
                        (
                            dx + row * shift_lr + start.0 * 3,
                            dy + row * shift_ud + index as i16 + start.1 * 3 - row * 3,
                        )
                    } else {
                        (
                            dx + row * shift_lr + index as i16 + start.0 * 3 - row * 3,
                            dy + row * shift_ud + start.1 * 3,
                        )
                    };
                    let absolute = (relative.0 + origin.0, relative.1 + origin.1);
                    if !(5..85).contains(&absolute.0) || !(5..85).contains(&absolute.1) {
                        continue;
                    }
                    let cell = absolute.0 as usize * 90 + absolute.1 as usize;
                    let tile = map.tiles[cell];
                    if tile > 999 && mission.tile_word(tile - 1000, 4)? == 0 {
                        found = Some((relative, cell, tile));
                        break 'search;
                    }
                }
            }
        }
        let (position, cell, tile) =
            found.ok_or_else(|| RuleError::Invalid("no free squad placement cell".into()))?;
        map.tiles[cell] = index as i16;
        result.positions.push(position);
        result.under_tiles.push(tile);
        if warrior.stamina()? > 0 {
            let race = word(&warrior.bytes, 0x32)?;
            let movement = *race_moves
                .get(race.wrapping_sub(1) as usize)
                .ok_or_else(|| RuleError::Invalid("race movement missing".into()))?;
            calculate_movement(warrior, movement)?;
            let value = word(&warrior.bytes, 0x3e)?;
            warrior.bytes[0x3c..0x3e].copy_from_slice(&value.to_be_bytes());
        }
    }
    Ok(result)
}

pub fn build_tactical_map(
    mission: &centurions_content::mission::OpeningMission,
    position: (i32, i32),
    edge_word: i16,
    random: &mut RandomState,
) -> Result<TacticalMap, RuleError> {
    let properties = &mission.tile_properties;
    let property = |tile: i16, offset: usize| -> Result<i16, RuleError> {
        if !(0..300).contains(&tile) {
            return Err(RuleError::Invalid(format!("tactical property tile {tile}")));
        }
        Ok(word(properties, tile as usize * 40 + offset)?)
    };
    centurions_content::mission::OpeningMission::index(position.0, position.1)?;
    let requested = (position.0 - 15, position.1 - 15);
    let origin = (requested.0.clamp(0, 60), requested.1.clamp(0, 60));
    let shift = (
        ((requested.0 - origin.0) * 3) as i16,
        ((requested.1 - origin.1) * 3) as i16,
    );
    let mut map = TacticalMap {
        tiles: vec![0; 8100],
        edge_word,
        shift,
    };
    for y in 0..30 {
        for x in 0..30 {
            let index =
                centurions_content::mission::OpeningMission::index(origin.0 + x, origin.1 + y)?;
            let floor = mission.floor[index];
            let tile = if floor <= 0 || floor >= 600 || mission.events[index] != 0 {
                mission.default_tile
            } else {
                floor
            };
            for dy in 0..3 {
                for dx in 0..3 {
                    map.tiles[((x * 3 + dx) * 90 + y * 3 + dy) as usize] =
                        property(tile, (20 + dy * 6 + dx * 2) as usize)?;
                }
            }
        }
    }
    if mission.minutes_per_tick == 5 {
        let foliage_mode = if property(206, 12)? == 0 { 2 } else { 1 };
        for y in 0..90 {
            for x in 0..90 {
                let index = x * 90 + y;
                let tile = map.tiles[index];
                let above = if index == 0 {
                    map.edge_word
                } else {
                    map.tiles[index - 1]
                };
                let sum = |tile, offsets: &[usize]| -> Result<i16, RuleError> {
                    offsets.iter().try_fold(0_i16, |sum, offset| {
                        Ok(sum.wrapping_add(property(tile, *offset)?))
                    })
                };
                let blocked = sum(tile, &[4, 6, 8, 12, 10])? != 0
                    || sum(above, &[4, 6, 8, 12])?.wrapping_add(property(tile, 10)?) != 0;
                let foliage = property(tile, 16)? != 0;
                if blocked {
                    if foliage {
                        map.tiles[index] = mission.default_tile;
                    }
                    continue;
                }
                if !foliage && random.bound(100) >= 5 {
                    continue;
                }
                if foliage_mode == 1 {
                    if random.bound(100) < 20 {
                        map.tiles[index] = random.range(207, 211);
                        let replacement = if (201..=211).contains(&above) {
                            206
                        } else {
                            random.range(201, 205)
                        };
                        if index == 0 {
                            map.edge_word = replacement;
                        } else {
                            map.tiles[index - 1] = replacement;
                        }
                        continue;
                    }
                } else if random.bound(100) < 20 && above > 113 {
                    map.tiles[index] = random.range(207, 211);
                    let replacement = map.tiles[index].wrapping_sub(6);
                    if index == 0 {
                        map.edge_word = replacement;
                    } else {
                        map.tiles[index - 1] = replacement;
                    }
                    continue;
                }
                map.tiles[index] = if random.bound(100) < 10 {
                    random.range(212, 222)
                } else {
                    mission.default_tile
                };
            }
        }
    }
    for tile in &mut map.tiles {
        *tile = tile.wrapping_add(1000);
    }
    Ok(map)
}

/// CODE 1 movecalc. Each SANE-to-single conversion is an explicit f32 rounding.
pub fn calculate_movement(warrior: &mut Warrior, race_movement: i8) -> Result<(), RuleError> {
    let bytes = &warrior.bytes;
    let maximum = 1000_i16
        .wrapping_add(word(bytes, 0x9e2)?.wrapping_mul(100))
        .wrapping_add(word(bytes, 0xace)?.wrapping_mul(10));
    let available = maximum.wrapping_sub(word(bytes, 0x9d6)?) as u16;
    if maximum == 0 {
        return Err(RuleError::Unsupported {
            action: "combat.movement",
            dependency: "SANE division by zero",
        });
    }
    let mut movement =
        ((available as f64 / (maximum as u16) as f64) as f32 as f64 * race_movement as f64) as f32;
    movement = (movement as f64 - word(bytes, 0x62)? as f64) as f32;
    if word(bytes, 0x6a)? != 0 {
        movement = (movement as f64 / 2.0) as f32;
    }
    if word(bytes, 0x8c)? != 0 {
        movement = (movement as f64 * 2.0) as f32;
    }
    movement = (movement as f64 + word(bytes, 0x30)? as f64) as f32;
    movement = ((movement as f64 + 0.5) as f32).max(5.0);
    if !movement.is_finite() || movement as f64 > i32::MAX as f64 {
        return Err(RuleError::Unsupported {
            action: "combat.movement",
            dependency: "SANE integer overflow",
        });
    }
    let movement = (movement.trunc() as i32) as i16;
    warrior.bytes[0x9d8..0x9da].copy_from_slice(&maximum.to_be_bytes());
    warrior.bytes[0x3e..0x40].copy_from_slice(&movement.to_be_bytes());
    Ok(())
}

/// CODE 1 getbattleorder, including its ordered adjacent comparisons.
pub fn initiative(
    warrior_speeds: &[i16],
    enemy_speeds: &[i16],
    surprise: i16,
    random: &mut RandomState,
) -> Result<Vec<i16>, RuleError> {
    if warrior_speeds.len() != 6 || enemy_speeds.len() > 100 || !(-1..=1).contains(&surprise) {
        return Err(RuleError::Invalid(
            "initiative requires the living starter squad and at most 100 enemies".into(),
        ));
    }
    let total = warrior_speeds.len() + enemy_speeds.len();
    let mut order = vec![-1; 110];
    match surprise {
        1 => {
            for index in 0..6 {
                order[index + 1] = index as i16;
            }
        }
        -1 => {
            for index in 0..6 {
                order[109 - index] = index as i16;
            }
        }
        _ => {
            let mut left = 6;
            while left != 0 {
                let slot = random.bound(total as i16) as usize;
                if order[slot] == -1 {
                    left -= 1;
                    order[slot] = left as i16;
                }
            }
        }
    }
    let mut left = enemy_speeds.len();
    while left != 0 {
        let slot = random.bound(total as i16) as usize;
        if order[slot] == -1 {
            order[slot] = left as i16 + 9;
            left -= 1;
        }
    }
    if surprise == 0 {
        let speed = |body: i16| {
            if body < 10 {
                warrior_speeds[body as usize]
            } else {
                enemy_speeds[(body - 10) as usize]
            }
        };
        for start in 1..total {
            for slot in start..total {
                if order[slot] >= 0
                    && order[slot + 1] >= 0
                    && speed(order[slot]) < speed(order[slot + 1])
                {
                    order.swap(slot, slot + 1);
                }
            }
        }
    }
    Ok(order)
}
