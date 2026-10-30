//! Round-script spawning: placement, initiative insertion and authored feedback.
use super::{
    Combat, OpeningMission, RandomState, RoundEffect, RoundOutcome, RuleError, unsupported,
};
use crate::combat::footprint;
use centurions_content::{combat::MonsterRecord, records::FootprintCell, word};

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
    let mut monster = crate::combat::monster::summon(monster, faction, difficulty, random)?;
    let chance = monster.record().turn_macro_chance()?;
    monster
        .record_mut()
        .set_turn_macro_chance(chance.wrapping_neg());
    Ok(monster)
}

/// Whether every cell of the new monster's body stands on open ground. Each cell needs a map tile;
/// the anchor cell may not have obstruction 2 (or any obstruction, for a size-zero monster) and
/// every other cell needs an obstruction below 2.
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
    center: (i16, i16),
    body: i16,
) -> Result<footprint::PlacedBody, RuleError> {
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

/// Summons the monsters an extra record names around the active monster, each keeping the faction
/// its definition gives.
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

/// Summons the monsters an extra record names around a point; a negative faction keeps each
/// definition's own. A full roster summons none, and a count (a negative one is drawn at random)
/// that would overflow the roster is refused whole with a diagnostic.
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
        // whole.
        return Err(unsupported(
            "summon count exceeds the native roster capacity",
        ));
    }
    for _ in 0..count {
        combat.monster_pick_written = true;
        let monster = build_monster(definitions, id, faction, combat.difficulty, random)?;
        let body = combat.roster.records.len() as i16 + 10;
        let placed = place_spawn(combat, mission, &monster, center, body)?;
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
            result.effects.push(RoundEffect::Sound(sound));
        }
        result
            .effects
            .push(RoundEffect::Spawn { body, monster: id });
    }
    Ok(())
}
