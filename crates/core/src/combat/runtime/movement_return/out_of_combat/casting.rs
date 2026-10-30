//! Firing at an occupied cell, through the same cast as any other shot.
use super::occupied::CastRequest;
use crate::combat::runtime::death_return;
use crate::combat::spells::{spell_level, wrapping_spell_descriptor};
use crate::combat::{
    runtime::{TurnFlow, out_of_combat_spell},
    spells,
};
use crate::{Content, Event, RuleError, Session};
use centurions_content::records::{Condition, SpellField, SpellFieldMut, SpellInfoRecord};

/// Fires the targets prepared at an occupied cell through the same cast as any other shot. An
/// emptied weapon is dropped, an attack action is spent, and each target in turn is flown at and
/// resolved; a ray is walked cell by cell. Returns Terminated when the battle ended.
///
/// Original: combatchoice
pub(super) fn apply(
    state: &mut Session,
    content: &Content,
    request: CastRequest,
    events: &mut Vec<Event>,
) -> Result<TurnFlow, RuleError> {
    // The emptied weapon in slot 0 drops silently.
    if request.discard_empty {
        crate::combat::shooting::drop_empty_weapon(state, content, request.actor, events)?;
    }
    if request.targets.is_empty() {
        return Err("retained cast has no prepared targets".into());
    }
    let warrior = state
        .warriors
        .get_mut(request.actor)
        .ok_or_else(|| RuleError::from("selected warrior outside the squad"))?;
    crate::combat::records::spend_attack_action(warrior)?;
    state.globals.round_scripts.blank_round = Some(0);
    let packet = state.globals.spell_info.clone();
    let attack = SpellInfoRecord(&packet);
    let identity = state.globals.cast_identity;
    if state.globals.attack_count < 1 {
        state.globals.attack_count = 1;
    }
    let count = state.globals.attack_count;
    let id = wrapping_spell_descriptor(identity);
    // A warrior's shot does not turn; the rotation stays 0.
    if attack.can_rotate() && !attack.is_direct_weapon_damage()? {
        state.random.bound(4);
    }
    let rotation = 0;
    // Movement and summon spells read the first target.
    state.globals.target_picks.write(&request.targets);
    let mut first = true;
    let total = request.targets.len();
    let mut flights = vec![];
    for (index, target) in request.targets.iter().enumerate() {
        state.globals.target_picks.enter(index, total);
        let aliased;
        let target = if super::target_list::aliased(index) {
            // A blocked ray marks its target -1, which here lands in the attack packet.
            if attack.is_ray() && spells::ray_walks(&packet) {
                return Err(unavailable("ray through an aliased target triple"));
            }
            aliased = super::target_list::load(state, index)
                .ok_or("aliased target triple outside the triples firing stored")?;
            &aliased
        } else {
            target
        };
        if skip_dead(state, request.actor, target, &attack)? {
            continue;
        }
        let jitter = spells::flight_jitter(&packet, &mut state.random);
        flights.push((
            events.len(),
            out_of_combat_flight(
                state,
                request.actor as i16,
                target,
                &packet,
                (total, jitter),
            ),
        ));
        events.push(Event::Sound {
            resource_id: attack.launch_sound()?,
        });
        if attack.is_ray() {
            let invocation = (id, identity[1], count);
            let (flow, stop) = out_of_combat_ray(
                state,
                content,
                request.actor,
                target,
                &packet,
                invocation,
                events,
            )?;
            spells::stop_flight(&mut flights, stop);
            if flow == TurnFlow::Terminated {
                crate::event::Flight::attach(events, &flights);
                return Ok(TurnFlow::Terminated);
            }
            continue;
        }
        let field = FieldOrigin {
            origin: if first {
                (target.first, target.second)
            } else {
                (-1, -1)
            },
            id,
            rotation,
            collision: usize::from(state.globals.battlefield.collide_check[62]) != 0,
        };
        first = false;
        let mask = field_target_mask(state, content, target, &packet, field)?;
        if resolve(
            state,
            content,
            &mask,
            (id, identity[1], count, (target.first, target.second)),
            true,
            events,
        )? == TurnFlow::Terminated
        {
            crate::event::Flight::attach(events, &flights);
            return Ok(TurnFlow::Terminated);
        }
        state.globals.target_picks.resolved(index as i16);
        if state.globals.target_picks.loop_ended(index) {
            break;
        }
    }
    crate::event::Flight::attach(events, &flights);
    state.globals.target_picks.leave();
    state.clear_targets();
    let actor = state.char_select_new;
    if state
        .warriors
        .get(actor)
        .ok_or_else(|| RuleError::from("selected warrior outside the squad"))?
        .record()
        .action_points()?
        > 0
    {
        death_return::refresh_body(state, content, actor as i16)?;
    }
    if state.warriors[actor].stamina()? < 1 {
        state.warriors[actor].record_mut().set_action_points(0);
    }
    Ok(TurnFlow::Returned)
}

/// Skips a weapon shot at a target already out of stamina. On a warrior's turn the rounds the shot
/// would have used, with their weight, are returned to the weapon.
fn skip_dead(
    state: &mut Session,
    actor: usize,
    target: &spells::SpellTarget,
    attack: &SpellInfoRecord<'_>,
) -> Result<bool, RuleError> {
    if attack.defense_category() != 9 || attack.target_type() >= 2 {
        return Ok(false);
    }
    let stamina = match target.kind {
        // Party targets past the squad read the warrior table.
        1 => match usize::try_from(target.first)
            .ok()
            .and_then(|body| state.table_record(body))
        {
            Some(warrior) => warrior.stamina()?,
            None => {
                return Err(unavailable(
                    "retained cast party target outside the warrior table",
                ));
            }
        },
        2 => state
            .globals
            .monsters
            .slot(monster_index(target.first)?)?
            .record
            .stamina()?,
        _ => return Ok(false),
    };
    if stamina > 0 {
        return Ok(false);
    }
    let current = state
        .globals
        .battle_order
        .current(state.globals.monster_up)?;
    if current < 9 {
        let weapon = state
            .globals
            .loaded_items
            .weapon
            .as_deref()
            .ok_or_else(|| RuleError::from("WEAPON work missing"))?;
        let ammo = state
            .globals
            .loaded_items
            .ammo
            .as_deref()
            .ok_or_else(|| RuleError::from("AMMO work missing"))?;
        let rounds = centurions_content::records::ItemDefinition(weapon).rounds_per_shot()?;
        let weight = centurions_content::records::ItemDefinition(ammo)
            .charge_weight()?
            .wrapping_mul(rounds);
        let warrior = &mut state.warriors[actor];
        let quantity = warrior.record().inventory_quantity(0)?.wrapping_add(rounds);
        let weight = warrior.record().load()?.wrapping_add(weight);
        warrior.record_mut().set_inventory_quantity(0, quantity);
        warrior.record_mut().set_load(weight);
    }
    Ok(true)
}

/// Where a new field goes: the first target's raw cell (later targets pass -1, -1), the spell,
/// rotation and whether a collision already used the field slot.
///
/// Original: spelltargets
pub(in crate::combat::runtime) struct FieldOrigin {
    pub(in crate::combat::runtime) origin: (i16, i16),
    pub(in crate::combat::runtime) id: i16,
    pub(in crate::combat::runtime) rotation: i16,
    pub(in crate::combat::runtime) collision: bool,
}

/// Marks the bodies a cast reaches: the shape comes from the packet (an area, the attack count or a
/// single cell), a rotating packet adds its rotation, and a new field is placed at the first
/// target.
pub(in crate::combat::runtime) fn field_target_mask(
    state: &mut Session,
    content: &Content,
    target: &spells::SpellTarget,
    packet: &[u8],
    field: FieldOrigin,
) -> Result<[i16; 110], RuleError> {
    let attack = SpellInfoRecord(packet);
    let mut shape = match attack.target_type() {
        3 => i16::from(attack.area_size()),
        4 => state.globals.attack_count,
        _ => 1,
    };
    // A rotating packet adds the rotation to its shape.
    if attack.can_rotate() {
        shape = shape.wrapping_add(field.rotation);
    }
    // Shape zero marks nothing, as in spell selection; a rotation that makes the
    // shape negative reads before the area table.
    let empty = [0; 49];
    let offset = usize::try_from(i32::from(shape.wrapping_sub(1)) * 49)
        .map_err(|_| unavailable("rotated cast shape reads before the land's area patterns"));
    let mission = state
        .land(content)
        .ok_or("retained cast area table missing")?;
    let pattern = if shape == 0 {
        &empty[..]
    } else {
        centurions_content::span(&mission.areas, offset?, 49)
            .map_err(|_| unavailable("retained cast area outside represented patterns"))?
    };
    if attack.field_kind() != 0 && !field.collision {
        create_field(state, packet, shape, &field)?;
    }
    let battlefield = &state.globals.battlefield;
    let map = &battlefield.field;
    let point = match target.kind {
        // A cell target already holds padded map coordinates.
        3 => (
            target.first.wrapping_sub(map.map_shift.0).wrapping_sub(5),
            target.second.wrapping_sub(map.map_shift.1).wrapping_sub(5),
        ),
        1 => party_position(state, target.first)?,
        2 => monster_cell(state, monster_index(target.first)?)?,
        _ => return Err(unavailable("retained occupied cast nonbody target")),
    };
    let center = crate::combat::records::wrapping_grid_point(point, map.map_shift);
    let current = state
        .globals
        .battle_order
        .current(state.globals.monster_up)?;
    let aliases = crate::combat::map_alias::session_map_aliases!(state, mission);
    let mut mask = [0; 110];
    for y in 0..7 {
        for x in 0..7 {
            if pattern[y * 7 + x] == 0 {
                continue;
            }
            let ax = center.0.wrapping_sub(3).wrapping_add(x as i16);
            let ay = center.1.wrapping_sub(3).wrapping_add(y as i16);
            let index = i32::from(ax as u16) * 90 + i32::from(ay);
            let body = aliases
                .read(map, index)?
                .ok_or_else(|| {
                    unavailable(
                        "retained cast area outside the map and the represented memory beside it",
                    )
                })?
                .wrapping_abs();
            if !(0..110).contains(&body) {
                continue;
            }
            let reflective = if attack.defense_category() == 9 {
                false
            } else if body < 9 {
                state
                    .table_record(body as usize)
                    .ok_or("retained cast body outside the warrior table")?
                    .record()
                    .condition(Condition::ReflectingScans)?
                    != 0
            } else if body > 9 {
                state
                    .globals
                    .monsters
                    .slot(monster_index(body)?)?
                    .record
                    .record()
                    .condition(Condition::ReflectingScans)
                    != 0
            } else {
                false
            };
            if reflective && state.random.bound(100) < 34 {
                let current = usize::try_from(current)
                    .ok()
                    .filter(|index| *index < 110)
                    .ok_or_else(|| {
                        unavailable("retained cast reflection to an order row outside 0..110")
                    })?;
                mask[current] = 1;
            } else if body < 10
                || state
                    .globals
                    .monsters
                    .slot(monster_index(body)?)?
                    .record
                    .record()
                    .psi_resistance()?
                    < 101
            {
                mask[body as usize] = 1;
            } else {
                mask[body as usize] = 0;
            }
        }
    }
    Ok(mask)
}

/// The monster slot of a body.
fn monster_index(body: i16) -> Result<usize, RuleError> {
    match body {
        10..110 => Ok((body - 10) as usize),
        110.. => Err(unavailable("retained cast body past the NPC array")),
        _ => Err(unavailable(
            "retained cast body below 10 before the NPC array",
        )),
    }
}

/// A monster's cell relative to the map origin.
fn monster_cell(state: &Session, index: usize) -> Result<(i16, i16), RuleError> {
    let backing = &state.globals.monsters;
    let position = backing.slot(index)?.position()?;
    Ok((
        position.0.wrapping_sub(backing.origin.0) as i16,
        position.1.wrapping_sub(backing.origin.1) as i16,
    ))
}

/// A party position. Slots -100..-1 read monster positions 0..99, which lie just
/// before the party's (see docs/memory-alias-audit.md).
fn party_position(state: &Session, body: i16) -> Result<(i16, i16), RuleError> {
    let positions = &state.globals.battlefield.pos;
    if let Some(point) = usize::try_from(body)
        .ok()
        .and_then(|body| positions.get(body))
    {
        return Ok(*point);
    }
    match body {
        -100..0 => monster_cell(state, (body + 100) as usize),
        0..8 => Err(unavailable(
            "retained cast party position in an inactive party slot",
        )),
        // Slot n reads battle order rows 2n - 15 and 2n - 14, then the order cursor
        // and monster_up; slot 63 reaches the dispatch mode, which is not kept.
        8..63 => {
            let order = &state.globals.battle_order;
            let word = |row: usize| match row {
                0..110 => order.order[row],
                110 => order.index as i16,
                _ => state.globals.monster_up as i16,
            };
            let row = body as usize * 2 - 15;
            Ok((word(row), word(row + 1)))
        }
        63.. => Err(unavailable(
            "retained cast party position over the initiative work",
        )),
        _ => Err(unavailable(
            "retained cast party position before the NPC position array",
        )),
    }
}

/// The refusal for a cast that depends on an unrecovered part of the game.
fn unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.retained_cast",
        dependency,
    }
}

/// The first expired field record receives the cast's geometry, identity and drawn duration; with
/// none free, no field is made.
fn create_field(
    state: &mut Session,
    packet: &[u8],
    shape: i16,
    field: &FieldOrigin,
) -> Result<(), RuleError> {
    let attack = SpellInfoRecord(packet);
    let initiative = state.globals.battle_order.index as i16;
    let count = state.globals.attack_count;
    let battlefield = &mut state.globals.battlefield;
    let Some(slot) = battlefield
        .spell_queue
        .as_chunks::<20>()
        .0
        .iter()
        .position(|record| SpellField(record).duration() == 0)
    else {
        return Ok(());
    };
    let (minimum, maximum) = attack.duration_range();
    let mut duration = state.random.range(minimum, maximum) as u8;
    let (minimum, maximum) = attack.power_duration_range();
    for _ in 0..count {
        duration = duration.wrapping_add(state.random.range(minimum, maximum) as u8);
    }
    let mut record = SpellFieldMut(&mut battlefield.spell_queue[slot * 20..slot * 20 + 20]);
    record.set_initiative(initiative);
    record.set_power(count);
    record.set_level(spell_level(field.id));
    record.set_rotation(field.rotation as u8);
    record.set_shape(usize::from(shape as u16));
    record.set_center(field.origin);
    record.set_spell(field.id);
    record.set_duration(duration);
    record.set_kind(attack.field_kind());
    Ok(())
}

/// Resolves the cast on the bodies marked in `mask`. `in_spell` is set while it resolves, except
/// for the bodies a ray crosses.
fn resolve(
    state: &mut Session,
    content: &Content,
    mask: &[i16; 110],
    (id, level, count, current): (i16, i16, i16, (i16, i16)),
    in_spell: bool,
    events: &mut Vec<Event>,
) -> Result<TurnFlow, RuleError> {
    state.publish_spell_targets(mask);
    let battlefield = &state.globals.battlefield;
    let index = usize::from(battlefield.collide_check[62]);
    if index >= battlefield.collide_check.len() {
        return Err(unavailable(
            "retained cast usage index outside represented bytes",
        ));
    }
    let mut usage = spells::FieldUsage {
        slot: index.saturating_sub(1),
        used: battlefield.collide_check.clone(),
    };
    if in_spell {
        state.set_in_spell(1);
    }
    let invocation = out_of_combat_spell::Invocation {
        id,
        level,
        count,
        collision: index != 0,
        current: Some(current),
    };
    if out_of_combat_spell::apply(state, content, mask, &mut usage, invocation, events)?
        == TurnFlow::Terminated
    {
        return Ok(TurnFlow::Terminated);
    }
    if in_spell {
        state.set_in_spell(0);
    }
    state.globals.party_track.fill(0);
    Ok(TurnFlow::Returned)
}

/// A ray from the caster to the target body; each newly crossed body resolves,
/// without in_spell, before the next step. Returns the step at which a blocking cell
/// stopped the ray, with the flow.
///
/// Original: cast
pub(in crate::combat::runtime) fn out_of_combat_ray(
    state: &mut Session,
    content: &Content,
    actor: usize,
    target: &spells::SpellTarget,
    packet: &[u8],
    invocation: (i16, i16, i16),
    events: &mut Vec<Event>,
) -> Result<(TurnFlow, Option<i16>), RuleError> {
    if !spells::ray_walks(packet) {
        return Ok((TurnFlow::Returned, None));
    }
    let end = target_pixel(state, target)?;
    let start = ray_pixel(state, actor as i16)?;
    let steps = 128 + state.command_preferences.animation_delay;
    let mut ray = spells::RayWalk::new(start, end, steps, actor);
    loop {
        let step = {
            let map = &state.globals.battlefield.field;
            let mission = state.land(content).ok_or("retained ray land missing")?;
            ray.next(&map.tiles, map.map_shift, mission, packet)?
        };
        match step {
            None => return Ok((TurnFlow::Returned, None)),
            Some(spells::RayStep::Blocked) => {
                events.push(Event::Sound {
                    resource_id: -10082,
                });
                return Ok((TurnFlow::Returned, Some(ray.step())));
            }
            Some(spells::RayStep::Body(body)) => {
                let mut mask = [0; 110];
                mask[body] = 1;
                let (id, level, count) = invocation;
                let current = (target.first, target.second);
                let crossing = events.len();
                let flow = resolve(
                    state,
                    content,
                    &mask,
                    (id, level, count, current),
                    false,
                    events,
                )?;
                spells::mark_step(&mut events[crossing..], ray.step());
                if flow == TurnFlow::Terminated {
                    return Ok((TurnFlow::Terminated, None));
                }
            }
        }
    }
}

/// The pixel a cast aims at for a target.
///
/// Original: cast
fn target_pixel(state: &Session, target: &spells::SpellTarget) -> Result<(i16, i16), RuleError> {
    Ok(match target.kind {
        1 | 2 => ray_pixel(state, target.first)?,
        // A cell holds padded map coordinates; the map origin is the shift plus 5.
        _ => {
            let shift = state.globals.battlefield.field.map_shift;
            (
                target
                    .first
                    .wrapping_sub(shift.0)
                    .wrapping_sub(5)
                    .wrapping_mul(32),
                target
                    .second
                    .wrapping_sub(shift.1)
                    .wrapping_sub(5)
                    .wrapping_mul(32),
            )
        }
    })
}

/// The flight from caster to target; none when a body is missing.
///
/// Original: cast
pub(in crate::combat::runtime) fn out_of_combat_flight(
    state: &Session,
    caster: i16,
    target: &spells::SpellTarget,
    packet: &[u8],
    (targets, jitter): (usize, [i8; 16]),
) -> Option<crate::Flight> {
    let from = ray_pixel(state, caster).ok()?;
    let to = target_pixel(state, target).ok()?;
    spells::flight_between(packet, from, to, targets, jitter)
}

/// An out-of-combat body's ray pixel: party cells from tactical work, monster cells from the
/// backing slot relative to the map origin.
fn ray_pixel(state: &Session, body: i16) -> Result<(i16, i16), RuleError> {
    if body < 10 {
        return Ok(spells::body_pixel(party_position(state, body)?, None));
    }
    let slot = state.globals.monsters.slot(monster_index(body)?)?;
    let origin = state.globals.monsters.origin;
    let position = slot.position()?;
    let cell = (
        position.0.wrapping_sub(origin.0) as i16,
        position.1.wrapping_sub(origin.1) as i16,
    );
    Ok(spells::body_pixel(
        cell,
        Some(slot.record.record().size()?.wrapping_abs()),
    ))
}
