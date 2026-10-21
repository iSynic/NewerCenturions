//! Round-script spawning: placement, initiative insertion and authored feedback.
use super::{
    Combat, OpeningMission, RandomState, RoundEffect, RoundOutcome, RuleError, unsupported,
};
use crate::combat::footprint;
use centurions_content::{combat::MonsterRecord, records::FootprintCell, word};

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
    let mut monster = crate::combat::monster::summon(monster, faction, difficulty, random)?;
    let chance = monster.record().turn_script_chance()?;
    monster
        .record_mut()
        .set_turn_script_chance(chance.wrapping_neg());
    Ok(monster)
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
    center: (i16, i16),
    body: i16,
) -> Result<footprint::PlacedBody, RuleError> {
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

pub(super) fn spawn(
    combat: &mut Combat,
    mission: &OpeningMission,
    definitions: &[u8],
    extra: &[u8],
    active: usize,
    random: &mut RandomState,
    result: &mut RoundOutcome,
) -> Result<(), RuleError> {
    let center = combat.story_position(active)?;
    spawn_at(
        combat,
        mission,
        definitions,
        extra,
        center,
        -1,
        random,
        result,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_at(
    combat: &mut Combat,
    mission: &OpeningMission,
    definitions: &[u8],
    extra: &[u8],
    center: (i16, i16),
    faction: i16,
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
    for _ in 0..count {
        let monster = build_monster(definitions, id, faction, combat.difficulty, random)?;
        let body = combat.enemies.records.len() as i16 + 10;
        let placed = place_spawn(combat, mission, &monster, center, body)?;
        combat.map = placed.map;
        if placed.monster.record().faction()? != 0 {
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
