use crate::{RandomState, RuleError, combat, rules};
use centurions_content::combat::MONSTER_BYTES;
use centurions_content::{
    combat::MonsterRecord,
    mission::OpeningMission,
    records::{EnemyRecord, EnemyRecordMut, FootprintCell},
    span,
};

/// The body size of a monster, 0 to 3, ignoring the sign.
fn size(monster: &MonsterRecord) -> Result<i16, RuleError> {
    span(&monster.bytes, 0, MONSTER_BYTES)?;
    let size = monster.record().size()?.wrapping_abs();
    if !(0..=3).contains(&size) {
        return Err(RuleError::Unsupported {
            action: "combat.footprint",
            dependency: "body size outside the original four layouts",
        });
    }
    Ok(size)
}

/// The index of the map cell at offset `dx`, `dy` from a body's position.
fn index(
    map: &combat::CombatField,
    position: (i16, i16),
    dx: i16,
    dy: i16,
) -> Result<usize, RuleError> {
    let (x, y) = crate::combat::records::grid_point(position, map.map_shift);
    OpeningMission::index(x + i32::from(dx), y + i32::from(dy)).map_err(Into::into)
}

/// A monster set down on the map: the map with its tiles written, the monster with its footprint
/// tiles remembered and its position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlacedBody {
    pub map: combat::CombatField,
    pub monster: MonsterRecord,
    pub position: (i16, i16),
}

/// Places a monster's footprint: the anchor cell, then northwest, north and west.
///
/// Original: placemonster
pub fn place(
    map: &combat::CombatField,
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

/// Restores the tiles under a body. The north cell has its own signed size test.
///
/// Original: bodyfield
pub fn restore_body(combat: &combat::Combat, body: i16) -> Result<combat::CombatField, RuleError> {
    if body < 10 {
        let index = usize::try_from(body)
            .map_err(|_| RuleError::Invalid("body-field warrior position unavailable".into()))?;
        let position =
            *combat.squad.pos.get(index).ok_or_else(|| {
                RuleError::Invalid("body-field warrior position unavailable".into())
            })?;
        let under = *combat.squad.char_under.get(index).ok_or_else(|| {
            RuleError::Invalid("body-field warrior under-tile unavailable".into())
        })?;
        restore_warrior(&combat.field, position, under)
    } else {
        let index = (body - 10) as usize;
        let monster = combat
            .roster
            .records
            .get(index)
            .ok_or_else(|| RuleError::Invalid("body-field enemy record unavailable".into()))?;
        let position =
            *combat.roster.mon_pos.get(index).ok_or_else(|| {
                RuleError::Invalid("body-field enemy position unavailable".into())
            })?;
        restore(&combat.field, monster, position)
    }
}

/// The cell under a warrior gets its saved tile back.
///
/// Original: bodyfield
pub(crate) fn restore_warrior(
    map: &combat::CombatField,
    position: (i16, i16),
    under: i16,
) -> Result<combat::CombatField, RuleError> {
    let cell = index(map, position, 0, 0)?;
    let mut result = map.clone();
    *result
        .tiles
        .get_mut(cell)
        .ok_or_else(|| RuleError::Invalid("body-field map cell unavailable".into()))? = under;
    Ok(result)
}

/// Restores the tiles under a monster; any size, even negative, is restored.
///
/// Original: bodyfield
pub fn restore(
    map: &combat::CombatField,
    monster: &MonsterRecord,
    position: (i16, i16),
) -> Result<combat::CombatField, RuleError> {
    span(&monster.bytes, 0, MONSTER_BYTES)?;
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

/// Whether the whole body stands on open ground: its anchor cell is blocked by obstruction 2, or by
/// any obstruction if it has size zero, each other cell by obstruction of two or more, and a
/// footprint tile below 1000 is never open.
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

/// Finds a place for a monster in widening rings; only size-zero monsters get a
/// random nudge.
///
/// Original: combatsetup
pub fn find_placement(
    map: &combat::CombatField,
    monster: &MonsterRecord,
    mission: &OpeningMission,
    body: i16,
    battle_cell: (i16, i16),
    base: (i16, i16),
    random: &mut RandomState,
) -> Result<PlacedBody, RuleError> {
    let size = size(monster)?;
    let origin = (
        map.map_shift.0.wrapping_add(5),
        map.map_shift.1.wrapping_add(5),
    );
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

/// How far a monster looks for neighbours: one more cell for each extra footprint
/// dimension.
///
/// Original: checkforenemy
pub fn neighbor_scan(monster: &MonsterRecord) -> Result<(i16, i16), RuleError> {
    let size = size(monster)?;
    Ok((
        if size > 1 { -2 } else { -1 },
        if size == 1 || size == 3 { -2 } else { -1 },
    ))
}

/// Chooses a new heading when a monster's step is blocked, drawing random numbers. When the target
/// is farther along x than y, a heading with a y part has an even chance of dropping it; otherwise
/// a new y part is drawn and then possibly a new x part. When it is not, a heading with an x part
/// drops it, and one without draws new x and y parts.
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

/// A monster after its move: the battle with the new position, whether it moved and the sounds its
/// steps made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodyMovement {
    pub combat: combat::Combat,
    pub moved: bool,
    pub sounds: Vec<i16>,
}

/// Moves a monster toward its target, or away from it when it flees.
///
/// Original: movemonster
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

/// The position slot a monster reads for its target body.
pub(crate) enum TargetSlot {
    Monster(usize),
    Party(usize),
}

/// Targets below 9 read the party positions and others the monster positions. The
/// monster positions run straight into the party positions, so targets -100..-1
/// read monster slots and targets from 110 read party slots.
///
/// Original: movemonster
pub(crate) fn target_slot(target: i16) -> Option<TargetSlot> {
    /// Where the monster positions begin.
    const MON_POS: i32 = 0x446f0a;
    /// Where the party positions begin, which the monster positions run into.
    const PARTY: i32 = 0x44709a;
    let address = if target < 9 {
        PARTY + 4 * i32::from(target)
    } else {
        MON_POS + 4 * (i32::from(target) - 10)
    };
    if (MON_POS..PARTY).contains(&address) {
        Some(TargetSlot::Monster(((address - MON_POS) / 4) as usize))
    } else if address >= PARTY {
        Some(TargetSlot::Party(((address - PARTY) / 4) as usize))
    } else {
        None
    }
}

/// What happens when a fleeing monster leaves its cell, called with the cell it steps to before the
/// body is placed there.
type DepartureReaction<'a> =
    dyn FnMut(&mut combat::Combat, &mut RandomState, (i16, i16)) -> Result<(), RuleError> + 'a;

/// Moves a monster one cell toward its target (away when it flees) in up to twenty-one tries. A
/// blocked step changes heading; a step costs the terrain's movement cost and must fit the
/// monster's movement. If no try succeeds the monster loses its target and its movement.
///
/// Original: movemonster
pub(crate) fn move_enemy_before_place(
    combat: &combat::Combat,
    (mission, aliases): (&OpeningMission, super::map_alias::MapAliases<'_>),
    enemy_index: usize,
    random: &mut RandomState,
    before_place: &mut DepartureReaction<'_>,
) -> Result<BodyMovement, RuleError> {
    let monster = combat
        .roster
        .records
        .get(enemy_index)
        .ok_or_else(|| RuleError::Invalid("enemy movement record unavailable".into()))?;
    let size = size(monster)?;
    let fleeing = monster.record().fleeing_duration() != 0;
    let target = monster.record().target()?;
    let outside = || RuleError::Unsupported {
        action: "combat.move",
        dependency: "NPC movement target outside the represented position arrays",
    };
    let target_position = match target_slot(target).ok_or_else(outside)? {
        TargetSlot::Monster(slot) => combat.story_position(slot)?,
        TargetSlot::Party(slot) => *combat.squad.pos.get(slot).ok_or_else(outside)?,
    };
    let position = *combat
        .roster
        .mon_pos
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
    result.combat.roster.records[enemy_index]
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
            let mut tile = super::map::movement_tile(&combat.field, aliases, next, (ox, oy))?;
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
            if cost > i16::from(monster.record().movement()) {
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
        if cost > i16::from(monster.record().movement()) {
            continue;
        }
        if fleeing {
            before_place(&mut result.combat, &mut next_random, next)?;
        }
        let map = restore(
            &result.combat.field,
            &result.combat.roster.records[enemy_index],
            position,
        )?;
        let placed = place(&map, &result.combat.roster.records[enemy_index], next, body)?;
        result.combat.field = placed.map;
        result.combat.roster.records[enemy_index] = placed.monster;
        result.combat.roster.mon_pos[enemy_index] = next;
        let bytes = &mut result.combat.roster.records[enemy_index].bytes;
        if i16::from(EnemyRecord::from_bytes(bytes).movement())
            == EnemyRecord::from_bytes(bytes).movement_max()?
            && EnemyRecord::from_bytes(bytes).movement_sound()? != 0
        {
            result
                .sounds
                .push(EnemyRecord::from_bytes(bytes).movement_sound()?);
        }
        let remaining = EnemyRecord::from_bytes(bytes)
            .movement()
            .wrapping_sub(cost as i8);
        EnemyRecordMut::from_bytes(bytes).set_movement(remaining);
        result
            .sounds
            .push(mission.tile_word(EnemyRecord::from_bytes(bytes).underlying_tile()? - 1000, 0)?);
        let absolute = crate::combat::records::wrapping_grid_point(next, combat.field.map_shift);
        if !(2..=87).contains(&absolute.0) || !(2..=87).contains(&absolute.1) {
            if EnemyRecord::from_bytes(bytes).survivor_marker()? == -1 {
                EnemyRecordMut::from_bytes(bytes)
                    .set_movement_direction(dx.wrapping_neg(), dy.wrapping_neg());
            } else {
                EnemyRecordMut::from_bytes(bytes).set_reaction_ready(false);
                EnemyRecordMut::from_bytes(bytes).set_movement(0);
                EnemyRecordMut::from_bytes(bytes).set_surrender_percent(101);
                result.combat.battle_order_index =
                    result.combat.battle_order_index.saturating_sub(1);
            }
        }
        result.moved = true;
        *random = next_random;
        return Ok(result);
    }
    let bytes = &mut result.combat.roster.records[enemy_index].bytes;
    EnemyRecordMut::from_bytes(bytes).set_target(-1);
    EnemyRecordMut::from_bytes(bytes).set_movement(0);
    *random = next_random;
    Ok(result)
}
