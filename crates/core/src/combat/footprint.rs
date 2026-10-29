use crate::{RandomState, RuleError, combat, rules};
use centurions_content::{
    combat::MonsterRecord,
    mission::OpeningMission,
    records::{EnemyRecord, EnemyRecordMut, FootprintCell},
    span,
};

fn size(monster: &MonsterRecord) -> Result<i16, RuleError> {
    span(&monster.bytes, 0, 288)?;
    let size = monster.record().size()?.wrapping_abs();
    if !(0..=3).contains(&size) {
        return Err(RuleError::Unsupported {
            action: "combat.footprint",
            dependency: "body size outside the original four layouts",
        });
    }
    Ok(size)
}

fn index(
    map: &combat::TacticalMap,
    position: (i16, i16),
    dx: i16,
    dy: i16,
) -> Result<usize, RuleError> {
    OpeningMission::index(
        i32::from(position.0) + i32::from(map.shift.0) + 5 + i32::from(dx),
        i32::from(position.1) + i32::from(map.shift.1) + 5 + i32::from(dy),
    )
    .map_err(Into::into)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlacedBody {
    pub map: combat::TacticalMap,
    pub monster: MonsterRecord,
    pub position: (i16, i16),
}

/// placemonster stores the anchor first, followed by northwest, north and west.
pub fn place(
    map: &combat::TacticalMap,
    monster: &MonsterRecord,
    position: (i16, i16),
    body: i16,
) -> Result<PlacedBody, RuleError> {
    if !(10..110).contains(&body) || map.tiles.len() != 8100 {
        return Err(RuleError::Invalid(
            "body placement state unavailable".into(),
        ));
    }
    let size = size(monster)?;
    let mut result = PlacedBody {
        map: map.clone(),
        monster: monster.clone(),
        position,
    };
    for (dx, dy, part, occupied) in [
        (0, 0, FootprintCell::Anchor, true),
        (-1, -1, FootprintCell::Northwest, size == 3),
        (0, -1, FootprintCell::North, size == 1 || size == 3),
        (-1, 0, FootprintCell::West, size > 1),
    ] {
        if occupied {
            let cell = index(map, position, dx, dy)?;
            result
                .monster
                .record_mut()
                .set_footprint_tile(part, result.map.tiles[cell]);
            result.map.tiles[cell] = if part == FootprintCell::Anchor {
                body
            } else {
                -body
            };
        } else {
            result.monster.record_mut().set_footprint_tile(part, 0);
        }
    }
    Ok(result)
}

/// bodyfield has a signed size check on the north cell, independent of its other cells.
pub fn restore_body(combat: &combat::Combat, body: i16) -> Result<combat::TacticalMap, RuleError> {
    if body < 10 {
        let index = usize::try_from(body)
            .map_err(|_| RuleError::Invalid("body-field warrior position unavailable".into()))?;
        let position =
            *combat.squad.positions.get(index).ok_or_else(|| {
                RuleError::Invalid("body-field warrior position unavailable".into())
            })?;
        let under = *combat.squad.under_tiles.get(index).ok_or_else(|| {
            RuleError::Invalid("body-field warrior under-tile unavailable".into())
        })?;
        restore_warrior(&combat.map, position, under)
    } else {
        let index = (body - 10) as usize;
        let monster = combat
            .enemies
            .records
            .get(index)
            .ok_or_else(|| RuleError::Invalid("body-field enemy record unavailable".into()))?;
        let position =
            *combat.enemies.positions.get(index).ok_or_else(|| {
                RuleError::Invalid("body-field enemy position unavailable".into())
            })?;
        restore(&combat.map, monster, position)
    }
}

/// bodyfield's warrior arm: the cell under a party body gets its saved tile back.
pub(crate) fn restore_warrior(
    map: &combat::TacticalMap,
    position: (i16, i16),
    under: i16,
) -> Result<combat::TacticalMap, RuleError> {
    let cell = index(map, position, 0, 0)?;
    let mut result = map.clone();
    *result
        .tiles
        .get_mut(cell)
        .ok_or_else(|| RuleError::Invalid("body-field map cell unavailable".into()))? = under;
    Ok(result)
}

/// Original bodyfield enemy suffix; signed sizes have no eligibility gate.
pub fn restore(
    map: &combat::TacticalMap,
    monster: &MonsterRecord,
    position: (i16, i16),
) -> Result<combat::TacticalMap, RuleError> {
    span(&monster.bytes, 0, 288)?;
    let signed_size = monster.record().size()?;
    let size = signed_size.wrapping_abs();
    let mut result = map.clone();
    for (dx, dy, part, occupied) in [
        (-1, -1, FootprintCell::Northwest, size == 3),
        (-1, 0, FootprintCell::West, size > 1),
        (0, -1, FootprintCell::North, size == 3 || signed_size == 1),
        (0, 0, FootprintCell::Anchor, true),
    ] {
        if occupied {
            let cell = index(map, position, dx, dy)?;
            *result
                .tiles
                .get_mut(cell)
                .ok_or_else(|| RuleError::Invalid("body restoration tile unavailable".into()))? =
                monster.record().footprint_tile(part)?;
        }
    }
    Ok(result)
}

pub(crate) fn placement_passable(
    body: &PlacedBody,
    mission: &OpeningMission,
) -> Result<bool, RuleError> {
    let size = size(&body.monster)?;
    for (part, occupied, anchor) in [
        (FootprintCell::Anchor, true, true),
        (FootprintCell::North, size == 1 || size == 3, false),
        (FootprintCell::Northwest, size == 3, false),
        (FootprintCell::West, size > 1, false),
    ] {
        if !occupied {
            continue;
        }
        let tile = body.monster.record().footprint_tile(part)?;
        if tile < 1000 {
            return Ok(false);
        }
        let obstruction = mission.tile_word(tile - 1000, 4)?;
        if (anchor && obstruction == 2)
            || (!anchor && obstruction > 1)
            || (anchor && size == 0 && obstruction != 0)
        {
            return Ok(false);
        }
    }
    Ok(true)
}

/// combatsetup searches complete rings and consumes position jitter only for size zero.
pub fn find_placement(
    map: &combat::TacticalMap,
    monster: &MonsterRecord,
    mission: &OpeningMission,
    body: i16,
    battle_cell: (i16, i16),
    base: (i16, i16),
    random: &mut RandomState,
) -> Result<PlacedBody, RuleError> {
    let size = size(monster)?;
    let origin = (map.shift.0.wrapping_add(5), map.shift.1.wrapping_add(5));
    let mut next_random = random.clone();
    let mut current_map = map.clone();
    for ring in 1..=180_i16 {
        for dy in -ring..ring {
            for dx in -ring..ring {
                let mut position = (
                    battle_cell
                        .0
                        .wrapping_mul(3)
                        .wrapping_add(base.0)
                        .wrapping_add(dx)
                        .wrapping_add(3),
                    battle_cell
                        .1
                        .wrapping_mul(3)
                        .wrapping_add(base.1)
                        .wrapping_add(dy)
                        .wrapping_add(3),
                );
                let absolute = (
                    position.0.wrapping_add(origin.0),
                    position.1.wrapping_add(origin.1),
                );
                if !(5..85).contains(&absolute.0) || !(5..85).contains(&absolute.1) {
                    continue;
                }
                if size == 0 {
                    position.0 = position
                        .0
                        .wrapping_add(next_random.bound(3))
                        .wrapping_sub(2);
                    position.1 = position
                        .1
                        .wrapping_add(next_random.bound(3))
                        .wrapping_sub(2);
                }
                let candidate = place(&current_map, monster, position, body)?;
                if placement_passable(&candidate, mission)? {
                    *random = next_random;
                    return Ok(candidate);
                }
                current_map = restore(&candidate.map, &candidate.monster, position)?;
            }
        }
    }
    Err(RuleError::Invalid(
        "no free enemy placement footprint".into(),
    ))
}

/// checkforenemy expands its adjacent scan by one cell for each extra footprint dimension.
pub fn neighbor_scan(monster: &MonsterRecord) -> Result<(i16, i16), RuleError> {
    let size = size(monster)?;
    Ok((
        if size > 1 { -2 } else { -1 },
        if size == 1 || size == 3 { -2 } else { -1 },
    ))
}

pub(crate) fn shift_direction(
    position: (i16, i16),
    target: (i16, i16),
    dx: &mut i8,
    dy: &mut i8,
    random: &mut RandomState,
) {
    let x = (target.0 as i8 as i16 - position.0 as i8 as i16).abs() as i8;
    let y = (target.1 as i8 as i16 - position.1 as i8 as i16).abs() as i8;
    if x > y {
        if *dy != 0 && random.bound(2) == 1 {
            *dy = 0;
            return;
        }
        *dy = match random.bound(3) {
            1 => 1,
            2 => 0,
            _ => -1,
        };
        if random.bound(3) == 3 && *dy != 0 {
            *dx = 0;
            return;
        }
        *dx = if random.bound(2) == 1 { -1 } else { 1 };
    } else if *dx != 0 {
        *dx = 0;
    } else {
        *dx = match random.bound(3) {
            1 => 1,
            2 => 0,
            _ => -1,
        };
        *dy = match random.bound(3) {
            1 => -1,
            2 => 1,
            _ => 0,
        };
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodyMovement {
    pub combat: combat::Combat,
    pub moved: bool,
    pub sounds: Vec<i16>,
}

pub fn move_enemy(
    combat: &combat::Combat,
    mission: &OpeningMission,
    enemy_index: usize,
    random: &mut RandomState,
) -> Result<BodyMovement, RuleError> {
    move_enemy_before_place(
        combat,
        (mission, super::map_alias::MapAliases::of_mission(mission)),
        enemy_index,
        random,
        &mut |_, _, _| Ok(()),
    )
}

/// The position slot movemonster13e276 reads for a target body.
pub(crate) enum TargetSlot {
    Npc(usize),
    Party(usize),
}

/// Targets below 9 index the party positions 0x44709a and others the NPC positions
/// 0x446f0a, four bytes a slot. The NPC array's 100 slots end at 0x44709a, so targets
/// -100..-1 alias NPC slots 0..99 and targets from 110 the party slots.
pub(crate) fn target_slot(target: i16) -> Option<TargetSlot> {
    const NPCS: i32 = 0x446f0a;
    const PARTY: i32 = 0x44709a;
    let address = if target < 9 {
        PARTY + 4 * i32::from(target)
    } else {
        NPCS + 4 * (i32::from(target) - 10)
    };
    if (NPCS..PARTY).contains(&address) {
        Some(TargetSlot::Npc(((address - NPCS) / 4) as usize))
    } else if address >= PARTY {
        Some(TargetSlot::Party(((address - PARTY) / 4) as usize))
    } else {
        None
    }
}

type DepartureReaction<'a> =
    dyn FnMut(&mut combat::Combat, &mut RandomState, (i16, i16)) -> Result<(), RuleError> + 'a;

pub(crate) fn move_enemy_before_place(
    combat: &combat::Combat,
    (mission, aliases): (&OpeningMission, super::map_alias::MapAliases<'_>),
    enemy_index: usize,
    random: &mut RandomState,
    before_place: &mut DepartureReaction<'_>,
) -> Result<BodyMovement, RuleError> {
    let monster = combat
        .enemies
        .records
        .get(enemy_index)
        .ok_or_else(|| RuleError::Invalid("enemy movement record unavailable".into()))?;
    let size = size(monster)?;
    let fleeing = monster.record().fleeing_duration() != 0;
    let target = monster.record().target_body()?;
    let outside = || RuleError::Unsupported {
        action: "combat.move",
        dependency: "NPC movement target outside the represented position arrays",
    };
    let target_position = match target_slot(target).ok_or_else(outside)? {
        TargetSlot::Npc(slot) => combat.story_position(slot)?,
        TargetSlot::Party(slot) => *combat.squad.positions.get(slot).ok_or_else(outside)?,
    };
    let position = *combat
        .enemies
        .positions
        .get(enemy_index)
        .ok_or_else(|| RuleError::Invalid("enemy movement position unavailable".into()))?;
    let body = enemy_index as i16 + 10;
    let mut result = BodyMovement {
        combat: combat.clone(),
        moved: false,
        sounds: vec![],
    };
    let mut next_random = random.clone();
    let mut dx = (target_position.0 - position.0).signum() as i8;
    let mut dy = (target_position.1 - position.1).signum() as i8;
    if fleeing {
        dx = -dx;
        dy = -dy;
    }
    result.combat.enemies.records[enemy_index]
        .record_mut()
        .set_movement_direction(dx, dy);
    let mut cost = 0;
    for _ in 0..21 {
        let next = (
            position.0.wrapping_add(i16::from(dx)),
            position.1.wrapping_add(i16::from(dy)),
        );
        let mut blocked = dx == 0 && dy == 0;
        for (ox, oy, occupied) in [
            (0, 0, true),
            (0, -1, size == 1 || size == 3),
            (-1, 0, size > 1),
            (-1, -1, size == 3),
        ] {
            if !occupied || blocked {
                continue;
            }
            let mut tile = super::map::movement_tile(&combat.map, aliases, next, (ox, oy))?;
            tile = tile.wrapping_abs();
            if tile == body {
                tile = mission.default_tile.wrapping_add(1000);
            }
            if tile < 1000 {
                blocked = true;
                break;
            }
            let obstruction = mission.tile_word(tile - 1000, 4)?;
            if obstruction != 0
                && ((size == 0 && monster.record().flying_flag() == 0)
                    || (tile > 1000 && obstruction > 1))
            {
                blocked = true;
                break;
            }
            cost = cost.max(rules::movement_cost(
                mission.tile_word(tile - 1000, 2)?,
                dx,
                dy,
            ));
            if cost > i16::from(monster.record().movement_points()) {
                break;
            }
        }
        if blocked {
            shift_direction(
                position,
                target_position,
                &mut dx,
                &mut dy,
                &mut next_random,
            );
            continue;
        }
        if cost > i16::from(monster.record().movement_points()) {
            continue;
        }
        if fleeing {
            before_place(&mut result.combat, &mut next_random, next)?;
        }
        let map = restore(
            &result.combat.map,
            &result.combat.enemies.records[enemy_index],
            position,
        )?;
        let placed = place(
            &map,
            &result.combat.enemies.records[enemy_index],
            next,
            body,
        )?;
        result.combat.map = placed.map;
        result.combat.enemies.records[enemy_index] = placed.monster;
        result.combat.enemies.positions[enemy_index] = next;
        let bytes = &mut result.combat.enemies.records[enemy_index].bytes;
        if i16::from(EnemyRecord::from_bytes(bytes).movement_points())
            == EnemyRecord::from_bytes(bytes).movement_per_round()?
            && EnemyRecord::from_bytes(bytes).movement_sound()? != 0
        {
            result
                .sounds
                .push(EnemyRecord::from_bytes(bytes).movement_sound()?);
        }
        let remaining = EnemyRecord::from_bytes(bytes)
            .movement_points()
            .wrapping_sub(cost as i8);
        EnemyRecordMut::from_bytes(bytes).set_movement_points(remaining);
        result
            .sounds
            .push(mission.tile_word(EnemyRecord::from_bytes(bytes).underlying_tile()? - 1000, 0)?);
        let absolute = (
            next.0.wrapping_add(combat.map.shift.0).wrapping_add(5),
            next.1.wrapping_add(combat.map.shift.1).wrapping_add(5),
        );
        if !(2..=87).contains(&absolute.0) || !(2..=87).contains(&absolute.1) {
            if EnemyRecord::from_bytes(bytes).survivor_marker()? == -1 {
                EnemyRecordMut::from_bytes(bytes)
                    .set_movement_direction(dx.wrapping_neg(), dy.wrapping_neg());
            } else {
                EnemyRecordMut::from_bytes(bytes).set_reaction_ready(false);
                EnemyRecordMut::from_bytes(bytes).set_movement_points(0);
                EnemyRecordMut::from_bytes(bytes).set_fleeing_chance(101);
                result.combat.cursor = result.combat.cursor.saturating_sub(1);
            }
        }
        result.moved = true;
        *random = next_random;
        return Ok(result);
    }
    let bytes = &mut result.combat.enemies.records[enemy_index].bytes;
    EnemyRecordMut::from_bytes(bytes).set_target_body(-1);
    EnemyRecordMut::from_bytes(bytes).set_movement_points(0);
    *random = next_random;
    Ok(result)
}
