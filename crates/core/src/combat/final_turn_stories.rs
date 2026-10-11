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

/// buildmonster109210 at difficulty zero consumes HP dice before armor, speed and psi.
fn build_monster(
    definitions: &[u8],
    id: i16,
    faction: i16,
    random: &mut RandomState,
) -> Result<MonsterRecord, RuleError> {
    if ![1, 2, 3, 5, 108, 109].contains(&id) {
        return Err(unsupported("unverified summoned monster"));
    }
    let mut monster = MonsterRecord::read(definitions, id)?;
    let bytes = &mut monster.bytes;
    let level = word(bytes, 0)?;
    if !(0..=127).contains(&level) || word(bytes, 0x4e)? < 0 {
        return Err(unsupported("summoned level or random weapon tables"));
    }
    put_word(bytes, 0x10c, -1);
    let mut hp = word(bytes, 2)?;
    for _ in 0..level {
        hp = hp.wrapping_add(random.bound(8));
    }
    hp = hp.max(1);
    put_word(bytes, 0x110, hp);
    put_word(bytes, 0x112, hp);
    if faction > -1 {
        put_word(bytes, 0xe, faction);
    }
    bytes[0x107] = 0;
    bytes[0x108] = 0;
    let armor = word(bytes, 8)?;
    let armor = if armor == 0 {
        armor
    } else {
        armor.wrapping_add(random.bound(3).wrapping_sub(2))
    };
    put_word(bytes, 8, armor.max(0));
    let speed = word(bytes, 4)?
        .wrapping_add(random.bound(3).wrapping_sub(2))
        .max(1);
    put_word(bytes, 4, speed);
    let psi = word(bytes, 0x52)?;
    let spread = psi / 10;
    let psi = psi.wrapping_add(random.range(spread.wrapping_neg(), spread));
    put_word(bytes, 0x52, psi);
    put_word(bytes, 0x10e, psi);
    let chance = word(bytes, 0x5c)?;
    put_word(bytes, 0x5c, chance.wrapping_neg());
    Ok(monster)
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
        let monster = build_monster(definitions, id, faction, random)?;
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

fn expected_script(id: i16) -> Result<&'static [(i16, i16)], RuleError> {
    match id {
        345 => Ok(&[(9, 431), (0, 0), (1, 1265), (124, 626), (124, 627)]),
        276 => Ok(&[(9, 509), (1, 1264), (124, 525), (124, 526), (120, 527)]),
        277 => Ok(&[(9, 509), (1, 1264), (124, 528), (124, 529), (120, 527)]),
        _ => Err(unsupported("unverified enemy turn script")),
    }
}

fn expected_extra(id: i16) -> Result<[i16; 5], RuleError> {
    match id {
        525 => Ok([0, 1, -2, 0, 500]),
        526 => Ok([0, 5, -2, 500, 0]),
        527 => Ok([-1, 0, 510, 0, 0]),
        528 => Ok([0, 2, -2, 500, 0]),
        529 => Ok([0, 3, -2, 500, 0]),
        626 => Ok([0, 108, 1, 431, 0]),
        627 => Ok([0, 109, 1, 431, 0]),
        _ => Err(unsupported("unverified enemy turn extra")),
    }
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
        || warriors.len() != 6
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
    let expected = expected_script(script_id)?;
    if [0xb3, 0xcf, 0xc9].iter().any(|offset| bytes[*offset] != 0) || word(bytes, 0x110)? <= 0 {
        return Err(unsupported("inactive, confused or hasted enemy turn"));
    }
    let mut next = combat.clone();
    let mut next_warriors = warriors.to_vec();
    let mut next_random = random.clone();
    let mut result = FinalTurn::default();
    let chance = word(bytes, 0x5c)?;
    if next_random.bound(100) <= chance {
        let script = span(&mission.door_scripts, script_id as usize * 42, 42)?;
        for i in 0..8 {
            let actual = (word(script, 8 + i * 2)?, word(script, 24 + i * 2)?);
            if actual != expected.get(i).copied().unwrap_or((0, 0)) {
                return Err(unsupported(
                    "enemy turn script differs from verified actions",
                ));
            }
        }
        for &(opcode, value) in expected {
            match opcode {
                0 => {}
                9 => result.effects.push(TurnEffect::Sound(value)),
                1 => result.effects.push(TurnEffect::Text(value)),
                124 | 120 => {
                    let extra = span(extras, value as usize * 10, 10)?;
                    let words = [
                        word(extra, 0)?,
                        word(extra, 2)?,
                        word(extra, 4)?,
                        word(extra, 6)?,
                        word(extra, 8)?,
                    ];
                    if words != expected_extra(value)? {
                        return Err(unsupported("enemy turn extra differs from verified fields"));
                    }
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
                        let bytes = &mut next.enemies.records[active].bytes;
                        put_word(bytes, 0x110, 0);
                        put_word(bytes, 0x60, 0);
                        let cleanup =
                            death_scripts::cleanup_enemy(&mut next, &mut next_warriors, body)?;
                        result.increment_kills = cleanup.increment_kills;
                        result.effects.push(TurnEffect::SelfRemoval { body });
                        result.skipped_turn = true;
                    }
                }
                _ => unreachable!(),
            }
        }
    }
    if !result.skipped_turn {
        let bytes = &mut next.enemies.records[active].bytes;
        put_word(bytes, 0x5c, chance.wrapping_abs());
        bytes[0x109] = (bytes[0x109] as i8).min(1).wrapping_add(bytes[0x8a] as i8) as u8;
        let mut movement = (word(bytes, 6)? as i8).wrapping_sub(bytes[0xb4] as i8);
        if bytes[0xb8] != 0 {
            movement /= 2;
        }
        bytes[0x106] = movement as u8;
    }
    *combat = next;
    warriors.clone_from_slice(&next_warriors);
    *random = next_random;
    Ok(result)
}
