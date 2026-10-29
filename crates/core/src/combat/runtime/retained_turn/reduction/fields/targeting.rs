//! collide's literal body filter and spelltargets' tactical area scan.
use crate::{RuleError, Session};
use centurions_content::records::{AttackPacket, Condition, SpellField};

pub(super) fn intersects(
    state: &Session,
    body: i16,
    field: &SpellField<'_>,
) -> Result<bool, RuleError> {
    let work = &state.native_work.tactical_work;
    let map = &work.map;
    let (point, size) = if body < 10 {
        let point = work.positions[body as usize];
        (
            (
                point.0.wrapping_add(map.shift.0).wrapping_add(5),
                point.1.wrapping_add(map.shift.1).wrapping_add(5),
            ),
            0,
        )
    } else {
        let slot = state.native_work.npc_backing.slot((body - 10) as usize)?;
        let position = slot.position()?;
        let x = position
            .0
            .wrapping_sub(state.native_work.npc_backing.origin.0) as i16;
        let y = position
            .1
            .wrapping_sub(state.native_work.npc_backing.origin.1) as i16;
        (
            (
                x.wrapping_add(map.shift.0).wrapping_add(5),
                y.wrapping_add(map.shift.1).wrapping_add(5),
            ),
            i16::from(slot.record.record().size()?.wrapping_abs() as i8),
        )
    };
    Ok(field.contains(point)?
        || ((size == 1 || size == 3) && field.contains((point.0, point.1.wrapping_sub(1)))?)
        || ((size == 2 || size == 3) && field.contains((point.0.wrapping_sub(1), point.1))?)
        || (size == 3 && field.contains((point.0.wrapping_sub(1), point.1.wrapping_sub(1)))?))
}

pub(super) fn mask(
    state: &mut Session,
    body: i16,
    field: &SpellField<'_>,
    packet: &[u8],
    areas: &[u8],
) -> Result<[i16; 110], RuleError> {
    let attack = AttackPacket(packet);
    let mut shape = match attack.targeting_mode() {
        3 => i16::from(attack.area_shape()),
        4 => field.power(),
        _ => 1,
    };
    if attack.effect()? == 58 {
        shape = i16::from(attack.area_shape()).wrapping_add(0x2007);
    }
    if attack.has_randomized_animation() {
        shape = shape.wrapping_add(i16::from(field.rotation() as i8));
    }
    // Shape zero takes the empty-pattern policy of spell selection; a negative
    // rotated shape still reads before the area table.
    let empty = [0; 49];
    let offset = usize::try_from(i32::from(shape.wrapping_sub(1)) * 49)
        .map_err(|_| unavailable("retained field area before represented patterns"));
    let pattern = if shape == 0 {
        &empty[..]
    } else {
        centurions_content::span(areas, offset?, 49)
            .map_err(|_| unavailable("retained field area outside represented patterns"))?
    };
    let work = &state.native_work.tactical_work;
    let map = &work.map;
    let center = if attack.targeting_mode() >= 2 {
        field.center()
    } else {
        let point = party_position(state, field.center().0)?;
        (
            point.0.wrapping_add(map.shift.0).wrapping_add(5),
            point.1.wrapping_add(map.shift.1).wrapping_add(5),
        )
    };
    let mut mask = [0; 110];
    for y in 0..7 {
        for x in 0..7 {
            if pattern[y * 7 + x] == 0 {
                continue;
            }
            let ax = center.0.wrapping_sub(3).wrapping_add(x as i16);
            let ay = center.1.wrapping_sub(3).wrapping_add(y as i16);
            let index = i32::from(ax as u16) * 90 + i32::from(ay);
            let tile = crate::combat::map_alias::session_map_aliases!(state)
                .read(map, index)?
                .ok_or_else(|| {
                    unavailable(
                        "retained field area outside the map, its edge WORD and land arrays",
                    )
                })?
                .wrapping_abs();
            if tile != body {
                continue;
            }
            let reflective = if body < 9 {
                state.warriors[body as usize]
                    .record()
                    .condition(Condition::ReflectingScans)?
                    != 0
            } else {
                state
                    .native_work
                    .npc_backing
                    .slot((body - 10) as usize)?
                    .record
                    .record()
                    .condition(Condition::ReflectingScans)
                    != 0
            };
            if attack.defense_category() != 9 && reflective {
                state.random.bound(100);
            }
            let eligible = body < 10
                || state
                    .native_work
                    .npc_backing
                    .slot((body - 10) as usize)?
                    .record
                    .record()
                    .magic_immunity()?
                    < 101;
            mask[body as usize] = i16::from(eligible);
        }
    }
    Ok(mask)
}

fn party_position(state: &Session, first: i16) -> Result<(i16, i16), RuleError> {
    let work = &state.native_work.tactical_work;
    if let Ok(index) = usize::try_from(first)
        && index < 6
    {
        return Ok(work.positions[index]);
    }
    if first == 72 {
        return Ok(if state.command_preferences.small_combat_field {
            (320, 377)
        } else {
            (480, 485)
        });
    }
    // The NPC positions 0x446f0a end where the party positions 0x44709a begin.
    if (-100..0).contains(&first) {
        let backing = &state.native_work.npc_backing;
        let point = backing.slot((100 + first) as usize)?.position()?;
        let shift = work.map.shift;
        return Ok((
            point.0.wrapping_sub(i32::from(shift.0) + 5) as i16,
            point.1.wrapping_sub(i32::from(shift.1) + 5) as i16,
        ));
    }
    if first < 0 {
        return Err(unavailable(
            "retained field position before the NPC position array",
        ));
    }
    if first < 8 {
        return Err(unavailable(
            "retained field position in an inactive party slot",
        ));
    }
    if first > 63 {
        return Err(unavailable(
            "retained field position past the initiative work",
        ));
    }
    let order = &state.native_work.initiative_work;
    let index = (first as usize * 4 - 30) / 2;
    let word = |index| match index {
        0..110 => order.order.get(index).copied(),
        110 => Some(order.cursor as i16),
        111 => Some(state.native_work.selected_enemy as i16),
        112 => Some(0),
        _ => None,
    };
    let missing = || RuleError::from("retained field position outside the initiative work");
    Ok((
        word(index).ok_or_else(missing)?,
        word(index + 1).ok_or_else(missing)?,
    ))
}

fn unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.retained_field",
        dependency,
    }
}
