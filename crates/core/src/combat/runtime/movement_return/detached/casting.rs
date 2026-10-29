//! Occupied firing returns through combatchoice's shared cast suffix.
use super::occupied::CastRequest;
use crate::combat::{
    runtime::{TurnFlow, retained_spell},
    spells,
};
use crate::{Content, Event, RuleError, Session};
use centurions_content::records::{AttackPacket, Condition, SpellField, SpellFieldMut};

pub(super) fn apply(
    state: &mut Session,
    content: &Content,
    request: CastRequest,
    events: &mut Vec<Event>,
) -> Result<TurnFlow, RuleError> {
    // combatchoice 11ae0e..11ae1c: silent dropitem of slot 0 after loaditem(WORD44719a).
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
    let movement = warrior.record().movement_points()?.wrapping_sub(3).max(0);
    let actions = warrior.record().action_points()?.wrapping_sub(2);
    warrior.record_mut().set_movement_points(movement);
    warrior.record_mut().set_action_points(actions);
    state.native_work.round_work.inactive_rounds = Some(0);
    let packet = state.native_work.attack_packet.clone();
    let attack = AttackPacket(&packet);
    let identity = state.native_work.attack_identity;
    if state.native_work.attack_count < 1 {
        state.native_work.attack_count = 1;
    }
    let count = state.native_work.attack_count;
    let id = identity[0]
        .wrapping_mul(1000)
        .wrapping_add(identity[1].wrapping_mul(100))
        .wrapping_add(identity[2])
        .wrapping_add(1101);
    // The player-side draw for a randomized nonbody packet leaves WORD44331a at the
    // combatchoice reset value 0.
    if attack.has_randomized_animation() && !attack.is_direct_weapon_damage()? {
        state.random.bound(4);
    }
    let rotation = 0;
    // combatchoice stored the triples from 443722; the arms read triple 0's WORDs.
    state.native_work.target_work.write(&request.targets);
    let mut first = true;
    let total = request.targets.len();
    let mut flights = vec![];
    for (index, target) in request.targets.iter().enumerate() {
        state.native_work.target_work.enter(index, total);
        let aliased;
        let target = if super::target_list::aliased(index) {
            // A ray's blocked target writes -1 over its triple (10e14a), here the packet.
            if attack.targeting_mode() == 6 && spells::ray_walks(&packet) {
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
            retained_flight(
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
        if attack.targeting_mode() == 6 {
            let invocation = (id, identity[1], count);
            let (flow, stop) = retained_ray(
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
            collision: usize::from(state.native_work.tactical_work.field_used[62]) != 0,
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
        state.native_work.target_work.resolved(index as i16);
        if state.native_work.target_work.loop_ended(index) {
            break;
        }
    }
    crate::event::Flight::attach(events, &flights);
    // cast ends with cleartarget (10e678), and combatchoice 11af78 calls it again.
    state.native_work.target_work.leave();
    state.clear_targets();
    let actor = state.selected_warrior;
    if state
        .warriors
        .get(actor)
        .ok_or_else(|| RuleError::from("selected warrior outside the squad"))?
        .record()
        .action_points()?
        > 0
    {
        crate::combat::runtime::death_return::refresh_body(state, content, actor as i16)?;
    }
    if state.warriors[actor].stamina()? < 1 {
        state.warriors[actor].record_mut().set_action_points(0);
    }
    Ok(TurnFlow::Returned)
}

fn skip_dead(
    state: &mut Session,
    actor: usize,
    target: &spells::SpellTarget,
    attack: &AttackPacket<'_>,
) -> Result<bool, RuleError> {
    if attack.defense_category() != 9 || attack.targeting_mode() >= 2 {
        return Ok(false);
    }
    let stamina = match target.kind {
        // Party targets read 0x448aa0 + first * 0xc42, past the squad from the table.
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
            .native_work
            .npc_backing
            .slot(npc_index(target.first)?)?
            .record
            .stamina()?,
        _ => return Ok(false),
    };
    if stamina > 0 {
        return Ok(false);
    }
    let current = state
        .native_work
        .initiative_work
        .current(state.native_work.selected_enemy)?;
    if current < 9 {
        let weapon = state
            .native_work
            .ammunition_work
            .weapon
            .as_deref()
            .ok_or_else(|| RuleError::from("WEAPON work missing"))?;
        let ammo = state
            .native_work
            .ammunition_work
            .ammo
            .as_deref()
            .ok_or_else(|| RuleError::from("AMMO work missing"))?;
        let rounds = centurions_content::records::ItemDefinition(weapon).rounds_per_shot()?;
        let weight = centurions_content::records::ItemDefinition(ammo)
            .unit_weight()?
            .wrapping_mul(rounds);
        let warrior = &mut state.warriors[actor];
        let quantity = warrior.record().inventory_quantity(0)?.wrapping_add(rounds);
        let weight = warrior.record().carried_weight()?.wrapping_add(weight);
        warrior.record_mut().set_inventory_quantity(0, quantity);
        warrior.record_mut().set_carried_weight(weight);
    }
    Ok(true)
}

/// spelltargets15cb52 places a new field at target 0's raw WORDs before the mask.
pub(in crate::combat::runtime) struct FieldOrigin {
    pub(in crate::combat::runtime) origin: (i16, i16),
    pub(in crate::combat::runtime) id: i16,
    pub(in crate::combat::runtime) rotation: i16,
    pub(in crate::combat::runtime) collision: bool,
}

pub(in crate::combat::runtime) fn field_target_mask(
    state: &mut Session,
    content: &Content,
    target: &spells::SpellTarget,
    packet: &[u8],
    field: FieldOrigin,
) -> Result<[i16; 110], RuleError> {
    let attack = AttackPacket(packet);
    let mut shape = match attack.targeting_mode() {
        3 => i16::from(attack.area_shape()),
        4 => state.native_work.attack_count,
        _ => 1,
    };
    // spelltargets15cb3a adds WORD44331a to the shape when packet byte 7 is set.
    if attack.has_randomized_animation() {
        shape = shape.wrapping_add(field.rotation);
    }
    // Shape zero takes the empty-pattern policy of spell selection; a shape the
    // rotation WORD44331a = Rand(4) - 1 makes negative reads before the area table.
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
    let work = &state.native_work.tactical_work;
    let map = &work.map;
    let point = match target.kind {
        // A cell target already holds padded map WORDs (spelltargets15cbf2).
        3 => (
            target.first.wrapping_sub(map.shift.0).wrapping_sub(5),
            target.second.wrapping_sub(map.shift.1).wrapping_sub(5),
        ),
        1 => party_position(state, target.first)?,
        2 => npc_cell(state, npc_index(target.first)?)?,
        _ => return Err(unavailable("retained occupied cast nonbody target")),
    };
    let center = (
        point.0.wrapping_add(map.shift.0).wrapping_add(5),
        point.1.wrapping_add(map.shift.1).wrapping_add(5),
    );
    let current = state
        .native_work
        .initiative_work
        .current(state.native_work.selected_enemy)?;
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
                    .native_work
                    .npc_backing
                    .slot(npc_index(body)?)?
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
                    .native_work
                    .npc_backing
                    .slot(npc_index(body)?)?
                    .record
                    .record()
                    .magic_immunity()?
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

/// NPC records at 0x439d3c + (body - 10) * 0x120.
fn npc_index(body: i16) -> Result<usize, RuleError> {
    match body {
        10..110 => Ok((body - 10) as usize),
        110.. => Err(unavailable("retained cast body past the NPC array")),
        _ => Err(unavailable(
            "retained cast body below 10 before the NPC array",
        )),
    }
}

/// An NPC's cell relative to the retained map origin.
fn npc_cell(state: &Session, index: usize) -> Result<(i16, i16), RuleError> {
    let backing = &state.native_work.npc_backing;
    let position = backing.slot(index)?.position()?;
    Ok((
        position.0.wrapping_sub(backing.origin.0) as i16,
        position.1.wrapping_sub(backing.origin.1) as i16,
    ))
}

/// Party positions 0x44709a, four bytes a slot. The NPC positions 0x446f0a end where
/// they begin, so slots -100..-1 read NPC slots 0..99.
fn party_position(state: &Session, body: i16) -> Result<(i16, i16), RuleError> {
    let positions = &state.native_work.tactical_work.positions;
    if let Some(point) = usize::try_from(body)
        .ok()
        .and_then(|body| positions.get(body))
    {
        return Ok(*point);
    }
    match body {
        -100..0 => npc_cell(state, (body + 100) as usize),
        0..8 => Err(unavailable(
            "retained cast party position in an inactive party slot",
        )),
        // Slot n's WORDs are initiative rows 2n - 15 and 2n - 14 (0x4470b8 + 2 * row),
        // then the cursor 0x447194 and the selected NPC 0x447196. Slot 63 reaches the
        // dispatch mode WORD 0x447198, which the session does not keep.
        8..63 => {
            let work = &state.native_work.initiative_work;
            let word = |row: usize| match row {
                0..110 => work.order[row],
                110 => work.cursor as i16,
                _ => state.native_work.selected_enemy as i16,
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

fn unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.retained_cast",
        dependency,
    }
}

/// The first expired record receives the cast's geometry, identity and drawn duration.
fn create_field(
    state: &mut Session,
    packet: &[u8],
    shape: i16,
    field: &FieldOrigin,
) -> Result<(), RuleError> {
    let attack = AttackPacket(packet);
    let initiative = state.native_work.initiative_work.cursor as i16;
    let count = state.native_work.attack_count;
    let work = &mut state.native_work.tactical_work;
    let Some(slot) = work
        .spell_fields
        .as_chunks::<20>()
        .0
        .iter()
        .position(|record| SpellField(record).duration() == 0)
    else {
        return Ok(());
    };
    let (minimum, maximum) = attack.duration_bounds();
    let mut duration = state.random.range(minimum, maximum) as u8;
    let (minimum, maximum) = attack.additional_duration_bounds();
    for _ in 0..count {
        duration = duration.wrapping_add(state.random.range(minimum, maximum) as u8);
    }
    let mut record = SpellFieldMut(&mut work.spell_fields[slot * 20..slot * 20 + 20]);
    record.set_initiative(initiative);
    record.set_power(count);
    record.set_level(field.id % 1000 / 100 - 1);
    record.set_rotation(field.rotation as u8);
    record.set_shape(usize::from(shape as u16));
    record.set_center(field.origin);
    record.set_spell(field.id);
    record.set_duration(duration);
    record.set_kind(attack.field_kind());
    Ok(())
}

/// One retained body resolution; spelltargets wraps non-ray resolution with Ready 1/0.
fn resolve(
    state: &mut Session,
    content: &Content,
    mask: &[i16; 110],
    (id, level, count, current): (i16, i16, i16, (i16, i16)),
    ready: bool,
    events: &mut Vec<Event>,
) -> Result<TurnFlow, RuleError> {
    state.publish_spell_targets(mask);
    let work = &state.native_work.tactical_work;
    let index = usize::from(work.field_used[62]);
    if index >= work.field_used.len() {
        return Err(unavailable(
            "retained cast usage index outside represented bytes",
        ));
    }
    let mut usage = spells::FieldUsage {
        slot: index.saturating_sub(1),
        used: work.field_used.clone(),
    };
    if ready {
        state.set_ready(1);
    }
    let invocation = retained_spell::Invocation {
        id,
        level,
        count,
        collision: index != 0,
        current: Some(current),
    };
    if retained_spell::apply(state, content, mask, &mut usage, invocation, events)?
        == TurnFlow::Terminated
    {
        return Ok(TurnFlow::Terminated);
    }
    if ready {
        state.set_ready(0);
    }
    state.native_work.party_mask.fill(0);
    Ok(TurnFlow::Returned)
}

/// cast10d4fe's ray over retained records: the caster's and target body's pixels, then each
/// newly crossed body resolves without Ready before the next step. Returns the walk
/// step at which a blocking cell stopped the ray (0x10e110..0x10e198) with the flow.
pub(in crate::combat::runtime) fn retained_ray(
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
            let map = &state.native_work.tactical_work.map;
            let mission = state.land(content).ok_or("retained ray land missing")?;
            ray.next(&map.tiles, map.shift, mission, packet)?
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

/// cast's target pixel for a retained triple (0x10d870..0x10dada).
fn target_pixel(state: &Session, target: &spells::SpellTarget) -> Result<(i16, i16), RuleError> {
    Ok(match target.kind {
        1 | 2 => ray_pixel(state, target.first)?,
        // cast10da96: a cell holds padded map WORDs; the origin LONG 442780 is the shift plus 5.
        _ => {
            let shift = state.native_work.tactical_work.map.shift;
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

/// cast's flight for a retained caster and triple; none when a body is missing.
pub(in crate::combat::runtime) fn retained_flight(
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

/// A retained body's ray pixel: party cells from tactical work, NPC cells from the backing
/// slot relative to the map origin.
fn ray_pixel(state: &Session, body: i16) -> Result<(i16, i16), RuleError> {
    if body < 10 {
        return Ok(spells::body_pixel(party_position(state, body)?, None));
    }
    let slot = state.native_work.npc_backing.slot(npc_index(body)?)?;
    let origin = state.native_work.npc_backing.origin;
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
