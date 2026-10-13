use crate::{RandomState, RuleError};
use centurions_content::{combat::MonsterRecord, mission::OpeningMission, span, word};

use super::{Combat, footprint};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RoundEffect {
    Sound(i16),
    Text(i16),
    Spawn { body: i16, monster: i16 },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RoundOutcome {
    pub effects: Vec<RoundEffect>,
    pub hostile_additions: i16,
}

fn put_word(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.round_story",
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
    if ![113, 114, 115, 116, 119, 121].contains(&id) {
        return Err(unsupported("unverified summoned monster"));
    }
    let mut monster = MonsterRecord::read(definitions, id)?;
    put_word(&mut monster.bytes, 0x10c, -1);
    let mut monster = super::monster::summon(monster, faction, difficulty, random)?;
    let chance = word(&monster.bytes, 0x5c)?;
    put_word(&mut monster.bytes, 0x5c, chance.wrapping_neg());
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
    result: &mut RoundOutcome,
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
    let faction = -1;
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
            result.effects.push(RoundEffect::Sound(sound));
        }
        result
            .effects
            .push(RoundEffect::Spawn { body, monster: id });
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RoundScripts {
    pub counter: i8,
    pub slots: [i16; 5],
}

impl Default for RoundScripts {
    fn default() -> Self {
        Self {
            counter: 1,
            slots: [0; 5],
        }
    }
}

impl RoundScripts {
    pub fn read(battles: &[u8], battle: i16) -> Result<Self, RuleError> {
        let record = span(battles, usize::from(battle.unsigned_abs()) * 1372, 1372)?;
        Ok(Self {
            counter: 1,
            slots: [
                word(record, 1360)?,
                word(record, 1362)?,
                word(record, 1364)?,
                word(record, 1366)?,
                word(record, 1368)?,
            ],
        })
    }
}

fn expected_script(id: i16) -> Result<&'static [(i16, i16)], RuleError> {
    match id {
        363 => Ok(&[(126, 661)]),
        364 => Ok(&[(126, 660)]),
        338 => Ok(&[(126, 620)]),
        340 => Ok(&[(126, 623)]),
        332 => Ok(&[(0, 0), (1, 1245), (124, 608), (124, 609), (124, 610)]),
        334 => Ok(&[
            (0, 0),
            (124, 612),
            (1, 1246),
            (124, 608),
            (124, 609),
            (124, 610),
        ]),
        339 => Ok(&[(1, 1249), (124, 622), (124, 621)]),
        341 => Ok(&[(9, 550), (1, 1250)]),
        _ => Err(unsupported("unverified battle round script")),
    }
}

fn extra(extras: &[u8], id: i16) -> Result<&[u8], RuleError> {
    let expected = match id {
        661 => [0, 1, 0, 332, 0],
        660 => [0, 2, 0, 334, 0],
        620 => [0, 5, 0, 339, 0],
        623 => [0, 4, 0, 341, 0],
        608 => [0, 113, 2, 10136, 0],
        609 => [0, 114, 3, 10136, 0],
        610 => [0, 115, 1, 10136, 0],
        612 => [0, 119, 1, 1103, 0],
        622 => [0, 121, 1, 1101, 0],
        621 => [0, 116, 8, 10136, 0],
        _ => return Err(unsupported("unverified battle round extra")),
    };
    let bytes = span(extras, id as usize * 10, 10)?;
    for (index, value) in expected.into_iter().enumerate() {
        if word(bytes, index * 2)? != value {
            return Err(unsupported(
                "battle round extra differs from verified fields",
            ));
        }
    }
    Ok(bytes)
}

fn verify_script(mission: &OpeningMission, id: i16) -> Result<&'static [(i16, i16)], RuleError> {
    let expected = expected_script(id)?;
    let script = span(&mission.door_scripts, id as usize * 42, 42)?;
    for i in 0..8 {
        if (word(script, 8 + i * 2)?, word(script, 24 + i * 2)?)
            != expected.get(i).copied().unwrap_or((0, 0))
        {
            return Err(unsupported(
                "battle round script differs from verified actions",
            ));
        }
    }
    Ok(expected)
}

/// getup125d34..125db8 dispatches BD round slots before hourly reduction.
pub fn advance(
    combat: &mut Combat,
    state: &mut RoundScripts,
    mission: &OpeningMission,
    definitions: &[u8],
    extras: &[u8],
    center: usize,
    random: &mut RandomState,
) -> Result<RoundOutcome, RuleError> {
    if combat.enemies.records.len() > 100
        || center >= combat.enemies.records.len()
        || combat.enemies.positions.len() != combat.enemies.records.len()
        || combat
            .enemies
            .records
            .iter()
            .any(|record| record.bytes.len() != 288)
        || combat.order.len() != 110
        || combat.map.tiles.len() != 8100
    {
        return Err(RuleError::Invalid(
            "invalid battle round story input".into(),
        ));
    }
    if combat.death_queue.processing {
        return Err(unsupported(
            "battle round invoked during death-script processing",
        ));
    }
    let mut next = combat.clone();
    let mut next_state = state.clone();
    let mut next_random = random.clone();
    let mut outcome = RoundOutcome::default();
    next_state.counter = next_state.counter.wrapping_add(1);
    for slot in 0..5 {
        let id = next_state.slots[slot];
        if id == 0 {
            continue;
        }
        let actions = verify_script(mission, id)?;
        if actions.len() != 1 || actions[0].0 != 126 {
            return Err(unsupported("battle slot is not a verified round predicate"));
        }
        let predicate = extra(extras, actions[0].1)?;
        if i16::from(next_state.counter) - 1 < word(predicate, 2)? {
            continue;
        }
        next_state.slots[slot] = 0;
        let continuation = word(predicate, 6)?;
        for &(opcode, value) in verify_script(mission, continuation)? {
            match opcode {
                0 => {}
                1 => outcome.effects.push(RoundEffect::Text(value)),
                9 => outcome.effects.push(RoundEffect::Sound(value)),
                124 => spawn(
                    &mut next,
                    mission,
                    definitions,
                    extra(extras, value)?,
                    center,
                    &mut next_random,
                    &mut outcome,
                )?,
                _ => return Err(unsupported("unverified battle round continuation")),
            }
        }
    }
    *combat = next;
    *state = next_state;
    *random = next_random;
    Ok(outcome)
}
