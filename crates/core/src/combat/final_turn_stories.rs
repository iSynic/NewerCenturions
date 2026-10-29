use crate::combat::{Combat, death_scripts, footprint};
use crate::{RandomState, RuleError};
use centurions_content::{
    Warrior,
    combat::MonsterRecord,
    mission::OpeningMission,
    records::{EnemyRecord, EnemyRecordMut, FootprintCell, StoryRecord},
    span, word,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TurnEffect {
    Sound(i16),
    Text(i16),
    Spawn { body: i16, monster: i16 },
    SelfRemoval { body: i16 },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FinalTurn {
    pub extra_record: Option<Vec<u8>>,
    pub effects: Vec<TurnEffect>,
    pub hostile_additions: i16,
    pub increment_kills: bool,
    pub skipped_turn: bool,
}

fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.turn_story",
        dependency,
    }
}

/// Preserve the authored summon-ID boundary before invoking buildmonster.
fn build_monster(
    definitions: &[u8],
    id: i16,
    faction: i16,
    difficulty: i16,
    random: &mut RandomState,
) -> Result<MonsterRecord, RuleError> {
    let mut monster = MonsterRecord::read(definitions, id)?;
    monster.record_mut().set_target_body(-1);
    let mut monster = super::monster::summon(monster, faction, difficulty, random)?;
    let chance = monster.record().turn_script_chance()?;
    monster
        .record_mut()
        .set_turn_script_chance(chance.wrapping_neg());
    Ok(monster)
}

/// Opcode120 preserves buildmonster's chance/context fields and its partial size refusal.
pub fn replace_body(
    combat: &mut Combat,
    definitions: &[u8],
    replacement: i16,
    active: usize,
    random: &mut RandomState,
) -> Result<bool, RuleError> {
    let faction = combat
        .enemies
        .records
        .get(active)
        .ok_or("replacement actor missing")?
        .record()
        .faction()?;
    // The loaded work remains changed even when the later size test refuses copy-out.
    combat.monster_work_written = true;
    let mut transformed = super::monster::summon(
        MonsterRecord::read(definitions, replacement)?,
        faction,
        combat.difficulty,
        random,
    )?;
    let old = &mut combat.enemies.records[active].bytes;
    let chance = EnemyRecord::from_bytes(old).turn_script_chance()?;
    EnemyRecordMut::from_bytes(old).set_turn_script_chance(chance.wrapping_neg());
    for cell in [
        FootprintCell::Northwest,
        FootprintCell::West,
        FootprintCell::North,
        FootprintCell::Anchor,
    ] {
        transformed
            .record_mut()
            .set_footprint_tile(cell, EnemyRecord::from_bytes(old).footprint_tile(cell)?);
    }
    if transformed.record().size()? != EnemyRecord::from_bytes(old).size()? {
        return Ok(true);
    }
    combat.enemies.records[active] = transformed;
    Ok(false)
}

fn passable(body: &footprint::PlacedBody, mission: &OpeningMission) -> Result<bool, RuleError> {
    let signed_size = body.monster.record().size()?;
    let size = signed_size.wrapping_abs();
    for (part, occupied, anchor) in [
        (FootprintCell::Anchor, true, true),
        (FootprintCell::North, size == 1 || size == 3, false),
        (FootprintCell::Northwest, size == 3, false),
        (FootprintCell::West, size >= 2, false),
    ] {
        if !occupied {
            continue;
        }
        let tile = body.monster.record().footprint_tile(part)?;
        if tile <= 999 {
            return Ok(false);
        }
        let obstruction = mission.tile_word(tile - 1000, 4)?;
        if anchor {
            if obstruction == 2 || (signed_size == 0 && obstruction != 0) {
                return Ok(false);
            }
        } else if obstruction >= 2 {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Opcode124 searches full expanding squares without combatsetup's size-zero jitter.
fn place_spawn(
    combat: &Combat,
    mission: &OpeningMission,
    monster: &MonsterRecord,
    active: usize,
    body: i16,
) -> Result<footprint::PlacedBody, RuleError> {
    let center = combat.enemies.positions[active];
    let mut map = combat.map.clone();
    // The original widens the square without limit. Past this ring every
    // in-window cell has been tried, so finding none means it never ends.
    let reach = |axis: i16, shift: i16| {
        let absolute = i32::from(axis) + i32::from(shift) + 5;
        (absolute - 7).max(83 - absolute).max(1)
    };
    let last = reach(center.0, map.shift.0).max(reach(center.1, map.shift.1));
    for ring in 1..=last.min(i32::from(i16::MAX)) as i16 {
        for dy in -ring..ring {
            for dx in -ring..ring {
                let position = (center.0.wrapping_add(dx), center.1.wrapping_add(dy));
                let absolute = (
                    position.0.wrapping_add(map.shift.0).wrapping_add(5),
                    position.1.wrapping_add(map.shift.1).wrapping_add(5),
                );
                if !(7..83).contains(&absolute.0) || !(7..83).contains(&absolute.1) {
                    continue;
                }
                let candidate = footprint::place(&map, monster, position, body)?;
                if passable(&candidate, mission)? {
                    return Ok(candidate);
                }
                map = footprint::restore(&candidate.map, &candidate.monster, position)?;
            }
        }
    }
    Err(unsupported(
        "no passable summoning placement; the original search never ends",
    ))
}

fn spawn(
    combat: &mut Combat,
    mission: &OpeningMission,
    definitions: &[u8],
    extra: &[u8],
    active: usize,
    random: &mut RandomState,
    result: &mut FinalTurn,
) -> Result<(), RuleError> {
    // The original checks capacity before it draws a negative count.
    if combat.enemies.records.len() >= 100 {
        return Ok(());
    }
    let id = word(extra, 2)?;
    let count = word(extra, 4)?;
    let count = if count < 0 {
        random.bound(count.wrapping_abs())
    } else {
        count
    };
    if count < 0 || combat.enemies.records.len() + count as usize > 100 {
        // Native 99 + 2 creates body110, aliasing warrior0's position. Keep
        // this unrepresented corruption atomic rather than clamping its count.
        return Err(unsupported(
            "summon count exceeds the native roster capacity",
        ));
    }
    let faction = combat.enemies.records[active].record().faction()?;
    for _ in 0..count {
        combat.monster_work_written = true;
        let monster = build_monster(definitions, id, faction, combat.difficulty, random)?;
        let body = combat.enemies.records.len() as i16 + 10;
        let placed = place_spawn(combat, mission, &monster, active, body)?;
        combat.map = placed.map;
        if placed.monster.record().faction()? != 0 {
            result.hostile_additions = result.hostile_additions.wrapping_add(1);
            combat.victory_work.add_total(1);
        }
        combat.enemies.records.push(placed.monster);
        combat.enemies.positions.push(placed.position);
        if let Some(slot) = combat.order[1..].iter_mut().rev().find(|id| **id == -1) {
            *slot = body;
        }
        let sound = word(extra, 6)?;
        if sound != 0 {
            result.effects.push(TurnEffect::Sound(sound));
        }
        result.effects.push(TurnEffect::Spawn { body, monster: id });
    }
    Ok(())
}

pub(crate) struct TurnProgram {
    pub script: i16,
    pub cursor: u8,
}

/// Original final encounter getup126938..126c3e; errors preserve all state and RNG.
pub fn initialize(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    definitions: &[u8],
    extras: &[u8],
    random: &mut RandomState,
) -> Result<FinalTurn, RuleError> {
    initialize_impl(
        combat,
        warriors,
        mission,
        definitions,
        extras,
        random,
        false,
    )
    .map(|(result, _)| result)
}

pub(crate) fn initialize_for_session(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    definitions: &[u8],
    extras: &[u8],
    random: &mut RandomState,
) -> Result<(FinalTurn, Option<TurnProgram>), RuleError> {
    initialize_impl(combat, warriors, mission, definitions, extras, random, true)
}

fn initialize_impl(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    definitions: &[u8],
    extras: &[u8],
    random: &mut RandomState,
    dispatch: bool,
) -> Result<(FinalTurn, Option<TurnProgram>), RuleError> {
    if combat.enemies.records.len() > 100
        || combat.enemies.positions.len() != combat.enemies.records.len()
        || combat.enemies.records.iter().any(|m| m.bytes.len() != 288)
        || !(1..=6).contains(&warriors.len())
        || warriors.iter().any(|w| w.bytes.len() != 3138)
        || combat.order.len() != 110
        || combat.cursor >= 110
        || combat.map.tiles.len() != 8100
    {
        return Err(RuleError::Invalid("invalid enemy turn story input".into()));
    }
    let body = combat.order[combat.cursor];
    let active = usize::try_from(body - 10)
        .ok()
        .filter(|i| *i < combat.enemies.records.len())
        .ok_or_else(|| RuleError::Invalid("turn story body outside roster".into()))?;
    let bytes = &combat.enemies.records[active].bytes;
    let script_id = EnemyRecord::from_bytes(bytes).final_turn_mode()?;
    if EnemyRecord::from_bytes(bytes).turn_skip_duration() != 0
        || EnemyRecord::from_bytes(bytes).stamina()? <= 0
    {
        return Err(unsupported("inactive enemy turn"));
    }
    let mut next = combat.clone();
    next.selected_enemy = active;
    let mut next_warriors = warriors.to_vec();
    let mut next_random = random.clone();
    let mut result = FinalTurn::default();
    let mut program = None;
    let chance = EnemyRecord::from_bytes(bytes).turn_script_chance()?;
    // getup125a3a's NPC turn entry runs the turn macro without testing 438584/438585.
    if next_random.bound(100) <= chance {
        next.script_enemy = active;
        let script = span(&mission.door_scripts, script_id as usize * 42, 42)?;
        for i in 0..8 {
            let (opcode, value) = StoryRecord(script).instruction(i)?;
            match opcode {
                0 => {}
                9 => result.effects.push(TurnEffect::Sound(value)),
                1 => result.effects.push(TurnEffect::Text(value)),
                124 | 120 => {
                    let extra = span(extras, value as usize * 10, 10)?;
                    result.extra_record = Some(extra.to_vec());
                    if opcode == 124 {
                        spawn(
                            &mut next,
                            mission,
                            definitions,
                            extra,
                            active,
                            &mut next_random,
                            &mut result,
                        )?;
                    } else {
                        let text = word(extra, 6)?;
                        if text != 0 {
                            result.effects.push(TurnEffect::Text(text));
                        }
                        let replacement = word(extra, 0)?;
                        if replacement == -1 {
                            let bytes = &mut next.enemies.records[active].bytes;
                            EnemyRecordMut::from_bytes(bytes).set_stamina(0);
                            EnemyRecordMut::from_bytes(bytes).set_death_script(0);
                            let cleanup =
                                death_scripts::cleanup_enemy(&mut next, &mut next_warriors, body)?;
                            result.increment_kills = cleanup.increment_kills;
                            result.effects.push(TurnEffect::SelfRemoval { body });
                            result.skipped_turn = true;
                            break;
                        }
                        if replace_body(
                            &mut next,
                            definitions,
                            replacement,
                            active,
                            &mut next_random,
                        )? {
                            break;
                        }
                        let sound = word(extra, 4)?;
                        if sound != 0 {
                            result.effects.push(TurnEffect::Sound(sound));
                        }
                        let text = word(extra, 8)?;
                        if text != 0 {
                            result.effects.push(TurnEffect::Text(text));
                        }
                    }
                }
                _ if dispatch => {
                    program = Some(TurnProgram {
                        script: script_id,
                        cursor: i as u8,
                    });
                    break;
                }
                _ => return Err(unsupported("enemy turn operation lacks a rule adapter")),
            }
        }
        if !result.skipped_turn && program.is_none() {
            next.script_enemy = 0;
        }
    }
    if !result.skipped_turn && program.is_none() {
        let bytes = &mut next.enemies.records[active].bytes;
        let (skipped, hostile_change) = super::initialize_enemy_record(bytes, &mut next_random)?;
        result.skipped_turn = skipped;
        result.hostile_additions = result.hostile_additions.wrapping_add(hostile_change);
        next.victory_work.add_total(hostile_change);
    }
    *combat = next;
    warriors.clone_from_slice(&next_warriors);
    *random = next_random;
    Ok((result, program))
}
