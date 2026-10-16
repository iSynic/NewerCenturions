use crate::{RandomState, RuleError};
use centurions_content::{Warrior, combat::MonsterRecord, mission::OpeningMission, span, word};

use super::{Combat, death_scripts, footprint};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TurnEffect {
    Sound(i16),
    Text(i16),
    Spawn { body: i16, monster: i16 },
    SelfRemoval { body: i16 },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FinalTurn {
    pub effects: Vec<TurnEffect>,
    pub hostile_additions: i16,
    pub increment_kills: bool,
    pub skipped_turn: bool,
}

fn put_word(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
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
    put_word(&mut monster.bytes, 0x10c, -1);
    let mut monster = super::monster::summon(monster, faction, difficulty, random)?;
    let chance = word(&monster.bytes, 0x5c)?;
    put_word(&mut monster.bytes, 0x5c, chance.wrapping_neg());
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
    let faction = word(
        &combat
            .enemies
            .records
            .get(active)
            .ok_or("replacement actor missing")?
            .bytes,
        14,
    )?;
    let mut transformed = super::monster::summon(
        MonsterRecord::read(definitions, replacement)?,
        faction,
        combat.difficulty,
        random,
    )?;
    let old = &mut combat.enemies.records[active].bytes;
    let chance = word(old, 0x5c)?;
    put_word(old, 0x5c, chance.wrapping_neg());
    for offset in [0x118, 0x11a, 0x11c, 0x11e] {
        transformed.bytes[offset..offset + 2].copy_from_slice(&old[offset..offset + 2]);
    }
    if word(&transformed.bytes, 0x10)? != word(old, 0x10)? {
        return Ok(true);
    }
    combat.enemies.records[active] = transformed;
    Ok(false)
}

fn passable(body: &footprint::PlacedBody, mission: &OpeningMission) -> Result<bool, RuleError> {
    let signed_size = word(&body.monster.bytes, 0x10)?;
    let size = signed_size.wrapping_abs();
    for (offset, occupied, anchor) in [
        (0x11e, true, true),
        (0x11c, size == 1 || size == 3, false),
        (0x118, size == 3, false),
        (0x11a, size >= 2, false),
    ] {
        if !occupied {
            continue;
        }
        let tile = word(&body.monster.bytes, offset)?;
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
    for ring in 1..=180_i16 {
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
    Err(unsupported("summoning placement budget exhausted"))
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
        return Err(unsupported(
            "summon count exceeds the native roster capacity",
        ));
    }
    let faction = word(&combat.enemies.records[active].bytes, 0xe)?;
    for _ in 0..count {
        let monster = build_monster(definitions, id, faction, combat.difficulty, random)?;
        let body = combat.enemies.records.len() as i16 + 10;
        let placed = place_spawn(combat, mission, &monster, active, body)?;
        combat.map = placed.map;
        if word(&placed.monster.bytes, 0xe)? != 0 {
            result.hostile_additions = result.hostile_additions.wrapping_add(1);
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

/// Original final encounter getup126938..126c3e; errors preserve all state and RNG.
pub fn initialize(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    definitions: &[u8],
    extras: &[u8],
    random: &mut RandomState,
) -> Result<FinalTurn, RuleError> {
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
    if combat.death_queue.processing {
        return Err(unsupported(
            "turn story invoked during death-script processing",
        ));
    }
    let body = combat.order[combat.cursor];
    let active = usize::try_from(body - 10)
        .ok()
        .filter(|i| *i < combat.enemies.records.len())
        .ok_or_else(|| RuleError::Invalid("turn story body outside roster".into()))?;
    let bytes = &combat.enemies.records[active].bytes;
    let script_id = word(bytes, 0x5e)?;
    if bytes[0xb3] != 0 || word(bytes, 0x110)? <= 0 {
        return Err(unsupported("inactive enemy turn"));
    }
    let mut next = combat.clone();
    next.selected_enemy = active;
    let mut next_warriors = warriors.to_vec();
    let mut next_random = random.clone();
    let mut result = FinalTurn::default();
    let chance = word(bytes, 0x5c)?;
    if next_random.bound(100) <= chance {
        next.script_enemy = active;
        let script = span(&mission.door_scripts, script_id as usize * 42, 42)?;
        for i in 0..8 {
            let opcode = word(script, 8 + i * 2)?;
            let value = word(script, 24 + i * 2)?;
            match opcode {
                0 => {}
                9 => result.effects.push(TurnEffect::Sound(value)),
                1 => result.effects.push(TurnEffect::Text(value)),
                124 | 120 => {
                    let extra = span(extras, value as usize * 10, 10)?;
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
                            put_word(bytes, 0x110, 0);
                            put_word(bytes, 0x60, 0);
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
                _ => return Err(unsupported("enemy turn operation lacks a rule adapter")),
            }
        }
        if !result.skipped_turn {
            next.script_enemy = 0;
        }
    }
    if !result.skipped_turn {
        let bytes = &mut next.enemies.records[active].bytes;
        let (skipped, hostile_change) = super::initialize_enemy_record(bytes, &mut next_random)?;
        result.skipped_turn = skipped;
        result.hostile_additions = result.hostile_additions.wrapping_add(hostile_change);
    }
    *combat = next;
    warriors.clone_from_slice(&next_warriors);
    *random = next_random;
    Ok(result)
}
