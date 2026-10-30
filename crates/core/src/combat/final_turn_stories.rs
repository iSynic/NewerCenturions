use crate::combat::{Combat, death_scripts, footprint};
use crate::{RandomState, RuleError};
use centurions_content::WARRIOR_BYTES;
use centurions_content::combat::MONSTER_BYTES;
use centurions_content::{
    Warrior,
    combat::MonsterRecord,
    mission::OpeningMission,
    records::{EnemyRecord, EnemyRecordMut, FootprintCell, StoryRecord},
    span, word,
};

/// What a monster's turn script did, in order: a sound or text, a monster summoned, or the monster
/// removing itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TurnEffect {
    Sound(i16),
    Text(i16),
    Spawn { body: i16, monster: i16 },
    SelfRemoval { body: i16 },
}

/// The result of a monster's turn script: the extra record it ended on, its effects, the hostile
/// monsters it added, any kill to count and whether the monster's turn was skipped.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FinalTurn {
    pub extra_record: Option<Vec<u8>>,
    pub effects: Vec<TurnEffect>,
    pub hostile_additions: i16,
    pub increment_kills: bool,
    pub skipped_turn: bool,
}

/// Reports a turn-script operation this build cannot yet run, without changing anything.
fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.turn_story",
        dependency,
    }
}

/// Builds a scripted monster, refusing summon IDs outside the authored range.
///
/// Original: buildmonster
fn build_monster(
    definitions: &[u8],
    id: i16,
    faction: i16,
    difficulty: i16,
    random: &mut RandomState,
) -> Result<MonsterRecord, RuleError> {
    let mut monster = MonsterRecord::read(definitions, id)?;
    monster.record_mut().set_target(-1);
    let mut monster = super::monster::summon(monster, faction, difficulty, random)?;
    let chance = monster.record().turn_macro_chance()?;
    monster
        .record_mut()
        .set_turn_macro_chance(chance.wrapping_neg());
    Ok(monster)
}

/// Replaces a body with a new monster. A replacement of a different size is refused (returns true)
/// after its record is built, and the old monster's turn-script chance is already negated.
///
/// Original: newland
pub fn replace_body(
    combat: &mut Combat,
    definitions: &[u8],
    replacement: i16,
    active: usize,
    random: &mut RandomState,
) -> Result<bool, RuleError> {
    let faction = combat
        .roster
        .records
        .get(active)
        .ok_or("replacement actor missing")?
        .record()
        .faction()?;
    // The monster template stays changed even when the size test refuses it.
    combat.monster_pick_written = true;
    let mut transformed = super::monster::summon(
        MonsterRecord::read(definitions, replacement)?,
        faction,
        combat.difficulty,
        random,
    )?;
    let old = &mut combat.roster.records[active].bytes;
    let chance = EnemyRecord::from_bytes(old).turn_macro_chance()?;
    EnemyRecordMut::from_bytes(old).set_turn_macro_chance(chance.wrapping_neg());
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
    combat.roster.records[active] = transformed;
    Ok(false)
}

/// Whether the new monster's whole body stands on open ground. Its anchor cell is blocked by
/// obstruction 2, or by any obstruction when its size is zero. Each other cell of its footprint is
/// blocked by obstruction of two or more, and a tile below 1000 is never open.
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

/// Finds a place for a spawned monster in widening squares, without the random
/// nudge battle setup gives size-zero monsters.
fn place_spawn(
    combat: &Combat,
    mission: &OpeningMission,
    monster: &MonsterRecord,
    active: usize,
    body: i16,
) -> Result<footprint::PlacedBody, RuleError> {
    let center = combat.roster.mon_pos[active];
    let mut map = combat.field.clone();
    // The search widens without limit; past this ring every cell has been tried,
    // so finding none means the game never returns.
    let reach = |axis: i16, shift: i16| {
        let absolute = i32::from(axis) + i32::from(shift) + 5;
        (absolute - 7).max(83 - absolute).max(1)
    };
    let last = reach(center.0, map.map_shift.0).max(reach(center.1, map.map_shift.1));
    for ring in 1..=last.min(i32::from(i16::MAX)) as i16 {
        for dy in -ring..ring {
            for dx in -ring..ring {
                let position = (center.0.wrapping_add(dx), center.1.wrapping_add(dy));
                let absolute = crate::combat::records::wrapping_grid_point(position, map.map_shift);
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

/// Summons the monsters an extra record names around the acting monster, on its side. A negative
/// count in the record is a random count up to its size. A full roster summons none, and a count
/// that would overflow it is refused whole.
fn spawn(
    combat: &mut Combat,
    mission: &OpeningMission,
    definitions: &[u8],
    extra: &[u8],
    active: usize,
    random: &mut RandomState,
    result: &mut FinalTurn,
) -> Result<(), RuleError> {
    // A full roster refuses before a negative count is drawn.
    if combat.roster.records.len() >= 100 {
        return Ok(());
    }
    let id = word(extra, 2)?;
    let count = word(extra, 4)?;
    let count = if count < 0 {
        random.bound(count.wrapping_abs())
    } else {
        count
    };
    if count < 0 || combat.roster.records.len() + count as usize > 100 {
        // A 101st monster would overwrite the first warrior's position; refuse it
        // whole rather than clamp the count.
        return Err(unsupported(
            "summon count exceeds the native roster capacity",
        ));
    }
    let faction = combat.roster.records[active].record().faction()?;
    for _ in 0..count {
        combat.monster_pick_written = true;
        let monster = build_monster(definitions, id, faction, combat.difficulty, random)?;
        let body = combat.roster.records.len() as i16 + 10;
        let placed = place_spawn(combat, mission, &monster, active, body)?;
        combat.field = placed.map;
        if placed.monster.record().faction()? != 0 {
            result.hostile_additions = result.hostile_additions.wrapping_add(1);
            combat.kill_counts.add_num_enemy(1);
        }
        combat.roster.records.push(placed.monster);
        combat.roster.mon_pos.push(placed.position);
        if let Some(slot) = combat.battle_order[1..]
            .iter_mut()
            .rev()
            .find(|id| **id == -1)
        {
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

/// A turn script handed to the session to run, and the instruction it resumes at.
pub(crate) struct TurnProgram {
    pub script: i16,
    pub cursor: u8,
}

/// Sets up a turn in the final encounter. An error changes nothing.
///
/// Original: getup
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

/// Sets up a monster's turn, handing back any script the session must run.
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

/// Sets up a monster's turn: its turn script runs by chance, and with `dispatch` an operation this
/// module does not carry out is handed back instead of failing.
fn initialize_impl(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    definitions: &[u8],
    extras: &[u8],
    random: &mut RandomState,
    dispatch: bool,
) -> Result<(FinalTurn, Option<TurnProgram>), RuleError> {
    if combat.roster.records.len() > 100
        || combat.roster.mon_pos.len() != combat.roster.records.len()
        || combat
            .roster
            .records
            .iter()
            .any(|m| m.bytes.len() != MONSTER_BYTES)
        || !(1..=6).contains(&warriors.len())
        || warriors.iter().any(|w| w.bytes.len() != WARRIOR_BYTES)
        || combat.battle_order.len() != 110
        || combat.battle_order_index >= 110
        || combat.field.tiles.len() != 8100
    {
        return Err(RuleError::Invalid("invalid enemy turn story input".into()));
    }
    let body = combat.battle_order[combat.battle_order_index];
    let active = usize::try_from(body - 10)
        .ok()
        .filter(|i| *i < combat.roster.records.len())
        .ok_or_else(|| RuleError::Invalid("turn story body outside roster".into()))?;
    let bytes = &combat.roster.records[active].bytes;
    let script_id = EnemyRecord::from_bytes(bytes).turn_macro()?;
    if EnemyRecord::from_bytes(bytes).turn_skip_duration() != 0
        || EnemyRecord::from_bytes(bytes).stamina()? <= 0
    {
        return Err(unsupported("inactive enemy turn"));
    }
    let mut next = combat.clone();
    next.monster_up = active;
    let mut next_warriors = warriors.to_vec();
    let mut next_random = random.clone();
    let mut result = FinalTurn::default();
    let mut program = None;
    let chance = EnemyRecord::from_bytes(bytes).turn_macro_chance()?;
    // A monster's turn script runs even while death scripts are running.
    if next_random.bound(100) <= chance {
        next.macro_monster = active;
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
                            let bytes = &mut next.roster.records[active].bytes;
                            EnemyRecordMut::from_bytes(bytes).set_stamina(0);
                            EnemyRecordMut::from_bytes(bytes).set_todo_on_death(0);
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
            next.macro_monster = 0;
        }
    }
    if !result.skipped_turn && program.is_none() {
        let bytes = &mut next.roster.records[active].bytes;
        let (skipped, hostile_change) = super::initialize_enemy_record(bytes, &mut next_random)?;
        result.skipped_turn = skipped;
        result.hostile_additions = result.hostile_additions.wrapping_add(hostile_change);
        next.kill_counts.add_num_enemy(hostile_change);
    }
    *combat = next;
    warriors.clone_from_slice(&next_warriors);
    *random = next_random;
    Ok((result, program))
}
