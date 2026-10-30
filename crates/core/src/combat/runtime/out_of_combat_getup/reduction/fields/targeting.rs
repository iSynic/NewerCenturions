//! Which bodies a field collision touches, and the area a spell marks.
use crate::{RuleError, Session};
use centurions_content::records::{Condition, SpellField, SpellInfoRecord};

/// Whether a spell field covers the cell a body stands in, taking a monster's footprint into
/// account.
pub(super) fn intersects(
    state: &Session,
    body: i16,
    field: &SpellField<'_>,
) -> Result<bool, RuleError> {
    let battlefield = &state.globals.battlefield;
    let map = &battlefield.field;
    let (point, size) = if body < 10 {
        let point = battlefield.pos[body as usize];
        (
            crate::combat::records::wrapping_grid_point(point, map.map_shift),
            0,
        )
    } else {
        let slot = state.globals.monsters.slot((body - 10) as usize)?;
        let position = slot.position()?;
        let x = position.0.wrapping_sub(state.globals.monsters.origin.0) as i16;
        let y = position.1.wrapping_sub(state.globals.monsters.origin.1) as i16;
        (
            (
                x.wrapping_add(map.map_shift.0).wrapping_add(5),
                y.wrapping_add(map.map_shift.1).wrapping_add(5),
            ),
            i16::from(slot.record.record().size()?.wrapping_abs() as i8),
        )
    };
    Ok(field.contains(point)?
        || ((size == 1 || size == 3) && field.contains((point.0, point.1.wrapping_sub(1)))?)
        || ((size == 2 || size == 3) && field.contains((point.0.wrapping_sub(1), point.1))?)
        || (size == 3 && field.contains((point.0.wrapping_sub(1), point.1.wrapping_sub(1)))?))
}

/// Marks `body` when the area of a field's spell covers it: the shape comes from the packet (an
/// area, the field's power or a single cell), a summon uses its own shape, and a rotating packet
/// adds the field's rotation.
pub(super) fn mask(
    state: &mut Session,
    body: i16,
    field: &SpellField<'_>,
    packet: &[u8],
    areas: &[u8],
) -> Result<[i16; 110], RuleError> {
    let attack = SpellInfoRecord(packet);
    let mut shape = match attack.target_type() {
        3 => i16::from(attack.area_size()),
        4 => field.power(),
        _ => 1,
    };
    if attack.effect()? == 58 {
        shape = i16::from(attack.area_size()).wrapping_add(0x2007);
    }
    if attack.can_rotate() {
        shape = shape.wrapping_add(i16::from(field.rotation() as i8));
    }
    // Shape zero marks nothing, as in spell selection; a rotation that makes the
    // shape negative reads before the area table.
    let empty = [0; 49];
    let offset = usize::try_from(i32::from(shape.wrapping_sub(1)) * 49)
        .map_err(|_| unavailable("retained field area before represented patterns"));
    let pattern = if shape == 0 {
        &empty[..]
    } else {
        centurions_content::span(areas, offset?, 49)
            .map_err(|_| unavailable("retained field area outside represented patterns"))?
    };
    let battlefield = &state.globals.battlefield;
    let map = &battlefield.field;
    let center = if attack.target_type() >= 2 {
        field.center()
    } else {
        let point = party_position(state, field.center().0)?;
        crate::combat::records::wrapping_grid_point(point, map.map_shift)
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
                    .globals
                    .monsters
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
                    .globals
                    .monsters
                    .slot((body - 10) as usize)?
                    .record
                    .record()
                    .psi_resistance()?
                    < 101;
            mask[body as usize] = i16::from(eligible);
        }
    }
    Ok(mask)
}

/// The cell of a party body, or of the special positions a field target can name.
fn party_position(state: &Session, first: i16) -> Result<(i16, i16), RuleError> {
    let battlefield = &state.globals.battlefield;
    if let Ok(index) = usize::try_from(first)
        && index < 6
    {
        return Ok(battlefield.pos[index]);
    }
    if first == 72 {
        return Ok(if state.command_preferences.small_combat_field {
            (320, 377)
        } else {
            (480, 485)
        });
    }
    // Monster positions lie just before the party's (see docs/memory-alias-audit.md).
    if (-100..0).contains(&first) {
        let backing = &state.globals.monsters;
        let point = backing.slot((100 + first) as usize)?.position()?;
        let shift = battlefield.field.map_shift;
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
    let order = &state.globals.battle_order;
    let index = (first as usize * 4 - 30) / 2;
    let word = |index| match index {
        0..110 => order.order.get(index).copied(),
        110 => Some(order.index as i16),
        111 => Some(state.globals.monster_up as i16),
        112 => Some(0),
        _ => None,
    };
    let missing = || RuleError::from("retained field position outside the initiative work");
    Ok((
        word(index).ok_or_else(missing)?,
        word(index + 1).ok_or_else(missing)?,
    ))
}

/// The refusal for a field that depends on an unrecovered part of the game.
fn unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.retained_field",
        dependency,
    }
}
