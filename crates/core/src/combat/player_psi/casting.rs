//! Casting for combat/player_psi.
use super::{
    CastRequest, Combat, Event, RandomState, RuleError, Session, cast_ray, cost, move_caster,
    packet, resolve_mask, selected_body, spells, validate_target,
};
use centurions_content::Warrior;
use centurions_content::mission::OpeningMission;
use centurions_content::records::{AttackPacket, PsiSelectionMemoryMut};

pub fn cast(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    random: &mut RandomState,
    memory: &mut Vec<u8>,
    disabled: u8,
    request: CastRequest,
) -> Result<Vec<Event>, RuleError> {
    if AttackPacket(&packet(mission, request.spell)?).targeting_mode() == 7 {
        return Err(RuleError::Unsupported {
            action: "combat.player_psi",
            dependency: "party duration state required",
        });
    }
    cast_with_buffs(
        combat,
        warriors,
        mission,
        random,
        memory,
        &mut vec![0; 20],
        disabled,
        request,
        CastContext::default(),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cast_with_buffs(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    random: &mut RandomState,
    memory: &mut Vec<u8>,
    buffs: &mut Vec<u8>,
    disabled: u8,
    request: CastRequest,
    context: CastContext<'_>,
) -> Result<Vec<Event>, RuleError> {
    validate_target(
        combat,
        warriors,
        mission,
        disabled,
        request,
        (context.body, context.land.with_tutorial(mission)),
    )?;
    if memory.len() != 480 {
        return Err(RuleError::Invalid("psi memory size".into()));
    }
    let mut next_combat = combat.clone();
    let mut next_warriors = warriors.to_vec();
    let mut next_random = random.clone();
    let mut next_memory = memory.clone();
    let mut next_buffs = buffs.clone();
    let events = cast_inner(
        &mut next_combat,
        &mut next_warriors,
        mission,
        &mut next_random,
        &mut next_memory,
        &mut next_buffs,
        request,
        CastPayment::Charge,
        context,
    )?;
    *combat = next_combat;
    warriors.clone_from_slice(&next_warriors);
    *random = next_random;
    *memory = next_memory;
    *buffs = next_buffs;
    Ok(events)
}

pub(super) enum CastPayment {
    Charge,
    Prepared,
    AdditionalTarget,
}

#[derive(Default)]
pub(crate) struct CastContext<'a> {
    pub movement_gate: i16,
    pub body: Option<i16>,
    pub animation_delay: i16,
    pub item_work_alias: Option<i16>,
    /// ITEM work, shop work and map header for an effect26/64 NPC alias.
    pub owners: crate::combat::warrior_alias::Owners<'a>,
    /// The current land arrays beside the map, read by spelltargets past its edges.
    pub land: crate::combat::map_alias::MapAliases<'a>,
    /// The session ITEM work combatupdate2 reloads after a full-roster summon
    /// refusal. When present it is also the alias owners' ITEM work.
    pub refresh: crate::combat::spell_summon::ItemRefresh<'a>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cast_prepared_at(
    state: &mut Session,
    (mission, items): (&OpeningMission, &[Vec<u8>]),
    pending: &crate::combat::psi_selection::Selection,
    target: (i16, i16),
    rotation: u8,
    body: Option<i16>,
) -> Result<Vec<Event>, RuleError> {
    let packet = packet(mission, pending.spell)?;
    let request = CastRequest {
        rotation,
        spell: pending.spell,
        power: pending.power,
        target: Some(target),
    };
    let combat = state
        .combat
        .as_ref()
        .ok_or("prepared scan combat missing")?;
    // Admission sees the prepayment credited back; execution uses Prepared and never charges twice.
    let mut admission = state.warriors.clone();
    let warrior = &mut admission[pending.actor];
    let psi = warrior
        .record()
        .psi_points()?
        .wrapping_add(cost(&packet, pending.power)?);
    let casts = warrior.record().scans_used()?.wrapping_sub(1);
    warrior.record_mut().set_psi_points(psi);
    warrior.record_mut().set_scans_used(casts);
    validate_target(
        combat,
        &admission,
        mission,
        state.psi_disabled,
        request,
        (
            body,
            crate::combat::map_alias::session_map_aliases!(state, mission),
        ),
    )?;
    let context = CastContext {
        movement_gate: state.bandage_ready,
        body,
        animation_delay: state.command_preferences.animation_delay,
        item_work_alias: spells::retained_item_alias(&state.native_work.ammunition_work)?,
        owners: crate::combat::warrior_alias::Owners::new(
            None,
            &state.native_work.shop_work,
            state
                .exploration
                .as_ref()
                .map(|position| position.event_headers.as_slice()),
        )
        .with_backup(&state.native_work.warrior_table),
        land: crate::combat::map_alias::session_map_aliases!(state),
        refresh: Some((items, &mut state.native_work.ammunition_work)),
    };
    // cast's loop over combatchoice's single point triple (10d632..10e678).
    state.combat.as_mut().unwrap().target_work.enter(0, 1);
    let events = cast_inner(
        state.combat.as_mut().unwrap(),
        &mut state.warriors,
        mission,
        &mut state.random,
        &mut state.native_work.psi_memory,
        &mut state.party_buffs,
        request,
        CastPayment::Prepared,
        context,
    )?;
    state.combat.as_mut().unwrap().target_work.leave();
    state.commit_alias_writes()?;
    if matches!(AttackPacket(&packet).effect()?, 55 | 56) {
        state.bandage_ready = 0;
    }
    Ok(events)
}

pub(crate) fn cast_selection(
    state: &mut Session,
    (mission, items): (&OpeningMission, &[Vec<u8>]),
    selection: &crate::combat::psi_selection::Selection,
) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    let combat = state.combat.as_mut().ok_or("psi combat missing")?;
    let memory = &mut state.native_work.psi_memory;
    let total = selection.targets.len();
    for (index, body) in selection.targets.iter().copied().enumerate() {
        // cast's loop index WORD442bd4 over combatchoice's WORD4435c0 triples.
        combat.target_work.enter(index, total);
        let target = if body < 10 {
            combat.squad.positions[body as usize]
        } else {
            combat.enemies.positions[(body - 10) as usize]
        };
        events.extend(cast_inner(
            combat,
            &mut state.warriors,
            mission,
            &mut state.random,
            memory,
            &mut state.party_buffs,
            CastRequest {
                spell: selection.spell,
                power: selection.power,
                target: Some(target),
                rotation: 0,
            },
            if index == 0 {
                CastPayment::Prepared
            } else {
                CastPayment::AdditionalTarget
            },
            CastContext {
                item_work_alias: spells::retained_item_alias(&state.native_work.ammunition_work)?,
                owners: crate::combat::warrior_alias::Owners::new(
                    None,
                    &state.native_work.shop_work,
                    state
                        .exploration
                        .as_ref()
                        .map(|position| position.event_headers.as_slice()),
                )
                .with_backup(&state.native_work.warrior_table),
                land: crate::combat::map_alias::session_map_aliases!(state),
                refresh: Some((items, &mut state.native_work.ammunition_work)),
                ..CastContext::default()
            },
        )?);
        crate::combat::warrior_alias::commit_pending!(state, combat)?;
        if combat.target_work.loop_ended(index) {
            break;
        }
    }
    // cast's last increment and closing cleartarget (10e658..10e678).
    combat.target_work.leave();
    Ok(events)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn cast_inner(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    random: &mut RandomState,
    memory: &mut [u8],
    buffs: &mut [u8],
    request: CastRequest,
    payment: CastPayment,
    context: CastContext<'_>,
) -> Result<Vec<Event>, RuleError> {
    let packet = packet(mission, request.spell)?;
    let actor = combat.active_body().unwrap() as usize;
    cast_packet(
        combat,
        warriors,
        mission,
        random,
        Some(memory),
        buffs,
        request,
        payment,
        context,
        packet,
        actor,
    )
}

/// Items has already consumed its charge and preserves the prepared attack.
/// BYTE435662 is clear, so this caller does not write learned Psi memory.
pub(crate) fn cast_item(
    state: &mut Session,
    (mission, items): (&OpeningMission, &[Vec<u8>]),
    pending: &crate::combat::psi_selection::Selection,
    target: (i16, i16),
    rotation: u8,
    body: Option<i16>,
    first: bool,
) -> Result<Vec<Event>, RuleError> {
    let packet = state
        .combat
        .as_ref()
        .map(|c| c.attack_packet.clone())
        .ok_or("prepared item packet missing")?;
    let context = CastContext {
        movement_gate: state.bandage_ready,
        body,
        animation_delay: state.command_preferences.animation_delay,
        item_work_alias: spells::retained_item_alias(&state.native_work.ammunition_work)?,
        owners: crate::combat::warrior_alias::Owners::new(
            None,
            &state.native_work.shop_work,
            state
                .exploration
                .as_ref()
                .map(|position| position.event_headers.as_slice()),
        )
        .with_backup(&state.native_work.warrior_table),
        land: crate::combat::map_alias::session_map_aliases!(state),
        refresh: Some((items, &mut state.native_work.ammunition_work)),
    };
    // cast clamps the retained WORD after Combatchoice has collected targets.
    let power = pending.power.max(1);
    state.native_work.attack_count = power;
    let combat = state
        .combat
        .as_mut()
        .ok_or("prepared item combat missing")?;
    let previous = combat.item_condition_caller;
    combat.item_condition_caller = combat.active_body();
    let result = cast_packet(
        state
            .combat
            .as_mut()
            .ok_or("prepared item combat missing")?,
        &mut state.warriors,
        mission,
        &mut state.random,
        None,
        &mut state.party_buffs,
        CastRequest {
            spell: pending.spell,
            power,
            target: Some(target),
            rotation,
        },
        if first {
            CastPayment::Prepared
        } else {
            CastPayment::AdditionalTarget
        },
        context,
        packet,
        pending.actor,
    );
    // The shared kernels borrow this existing Combat and cannot replace Session.
    state.combat.as_mut().unwrap().item_condition_caller = previous;
    let events = result?;
    state.commit_alias_writes()?;
    Ok(events)
}

#[allow(clippy::too_many_arguments)]
fn cast_packet(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    random: &mut RandomState,
    memory: Option<&mut [u8]>,
    buffs: &mut [u8],
    request: CastRequest,
    payment: CastPayment,
    context: CastContext<'_>,
    packet: Vec<u8>,
    actor: usize,
) -> Result<Vec<Event>, RuleError> {
    // castspell10b9bc / combatchoice1191e6: prepared and remembered casts arm Ready.
    combat.ready = 1;
    combat.inactive_rounds = 0;
    let descriptor = [
        request.spell / 1000 - 1,
        request.spell % 1000 / 100 - 1,
        request.spell % 100 - 1,
    ];
    combat.attack_packet = packet.clone();
    combat.attack_identity = descriptor;
    let warrior = &mut warriors[actor];
    if matches!(payment, CastPayment::Charge) {
        let remaining = warrior
            .record()
            .psi_points()?
            .wrapping_sub(cost(&packet, request.power)?);
        warrior.record_mut().set_psi_points(remaining);
        warrior.record_mut().count_scan_cast();
        let count = warrior.record().scans_used()?.wrapping_add(1);
        warrior.record_mut().set_scans_used(count);
    }
    if !matches!(payment, CastPayment::AdditionalTarget) {
        let movement = warrior.record().movement_points()?.wrapping_sub(3).max(0);
        let points = warrior.record().action_points()?.wrapping_sub(2);
        warrior.record_mut().set_movement_points(movement);
        warrior.record_mut().set_action_points(points);
    }
    let mut events = vec![];
    let mut flights = vec![];
    if matches!(AttackPacket(&packet).targeting_mode(), 9 | 10) {
        events.push(Event::Feedback(crate::feedback::NativeFeedback::Flash {
            address: if AttackPacket(&packet).targeting_mode() == 9 {
                0x401840
            } else {
                0x401820
            },
            x: 325,
            y: 322,
            wait: 60,
            sound: 0,
        }));
    }
    if let Some(memory) = memory {
        PsiSelectionMemoryMut(memory).remember_cast(actor, descriptor, request.power);
    }
    if matches!(AttackPacket(&packet).effect()?, 55 | 56) {
        let jitter = spells::flight_jitter(&packet, random);
        let flight = request.target.and_then(|cell| {
            let target = spells::SpellTarget {
                first: cell.0.wrapping_add(combat.map.shift.0).wrapping_add(5),
                second: cell.1.wrapping_add(combat.map.shift.1).wrapping_add(5),
                kind: 3,
            };
            let targets = usize::try_from(combat.target_work.count).unwrap_or(0);
            spells::flight(combat, actor as i16, &target, &packet, targets, jitter)
        });
        flights.push((events.len(), flight));
        events.push(Event::Sound {
            resource_id: AttackPacket(&packet).launch_sound()?,
        });
        events.extend(move_caster(
            combat,
            warriors,
            mission,
            random,
            actor,
            &packet,
            request,
            context.movement_gate,
        )?);
        crate::event::Flight::attach(&mut events, &flights);
        return Ok(events);
    }
    if AttackPacket(&packet).targeting_mode() == 6 {
        let jitter = spells::flight_jitter(&packet, random);
        let flight = request.target.and_then(|cell| {
            let target = spells::SpellTarget {
                first: cell.0.wrapping_add(combat.map.shift.0).wrapping_add(5),
                second: cell.1.wrapping_add(combat.map.shift.1).wrapping_add(5),
                kind: 3,
            };
            spells::flight(combat, actor as i16, &target, &packet, 1, jitter)
        });
        flights.push((events.len(), flight));
        events.push(Event::Sound {
            resource_id: AttackPacket(&packet).launch_sound()?,
        });
        if !spells::ray_walks(&packet) {
            request.target.ok_or("ray destination missing")?;
        } else {
            let start = combat.squad.positions[actor];
            let end = request.target.ok_or("ray destination missing")?;
            let crossings = spells::crossings(
                &combat.map.tiles,
                combat.map.shift,
                mission,
                &packet,
                (
                    (start.0.wrapping_mul(32), start.1.wrapping_mul(32)),
                    (end.0.wrapping_mul(32), end.1.wrapping_mul(32)),
                    128 + context.animation_delay,
                    actor,
                ),
            )?;
            let walk = events.len();
            events.extend(cast_ray(
                combat,
                warriors,
                mission,
                random,
                actor,
                request,
                &packet,
                context.animation_delay,
                (context.item_work_alias, context.owners, context.refresh),
            )?);
            spells::mark_crossings(&mut events[walk..], &crossings.0, crossings.1.is_some());
            spells::stop_flight(&mut flights, crossings.1);
            crate::event::Flight::attach(&mut events, &flights);
            return Ok(events);
        }
        crate::event::Flight::attach(&mut events, &flights);
        return Ok(events);
    }
    let mask = if AttackPacket(&packet).effect()? == 64
        && let Some(body) = context.body
    {
        let mut mask = [0; 110];
        mask[body as usize] = 1;
        let target = spells::SpellTarget {
            first: body,
            second: 0,
            kind: if body < 10 { 1 } else { 2 },
        };
        let jitter = spells::flight_jitter(&packet, random);
        flights.push((
            events.len(),
            spells::flight(
                combat,
                actor as i16,
                &target,
                &packet,
                usize::try_from(combat.target_work.count).unwrap_or(0),
                jitter,
            ),
        ));
        events.push(Event::Sound {
            resource_id: AttackPacket(&packet).launch_sound()?,
        });
        mask
    } else if matches!(AttackPacket(&packet).targeting_mode(), 9 | 10 | 12) {
        let mut mask = [0; 110];
        for (body, warrior) in warriors.iter().enumerate() {
            if warrior.record().active_flag() != 0
                && (AttackPacket(&packet).targeting_mode() == 12
                    || (warrior.record().faction()? == 0)
                        == (AttackPacket(&packet).targeting_mode() == 9))
            {
                mask[body] = 1;
            }
        }
        for (index, enemy) in combat.enemies.records.iter().enumerate() {
            if enemy.record().stamina()? > 0
                && (AttackPacket(&packet).targeting_mode() == 12
                    || (enemy.record().faction()? == 0)
                        == (AttackPacket(&packet).targeting_mode() == 9))
            {
                mask[index + 10] = 1;
            }
        }
        mask
    } else {
        let target = request.target.unwrap_or(combat.squad.positions[actor]);
        let target = if AttackPacket(&packet).targeting_mode() < 2 {
            let body = match context.body {
                Some(body) => body,
                None => selected_body(combat, warriors, target)?,
            };
            spells::SpellTarget {
                first: body,
                second: 0,
                kind: if body < 10 { 1 } else { 2 },
            }
        } else {
            spells::SpellTarget {
                first: target.0.wrapping_add(combat.map.shift.0).wrapping_add(5),
                second: target.1.wrapping_add(combat.map.shift.1).wrapping_add(5),
                kind: 3,
            }
        };
        let jitter = spells::flight_jitter(&packet, random);
        // combatchoice pushes the triple count WORD4435c0 as cast's count
        // (0x118e78, 0x11af66), which the short-walk test reads at 0x10d5c6.
        let targets = usize::try_from(combat.target_work.count).unwrap_or(0);
        flights.push((
            events.len(),
            spells::flight(combat, actor as i16, &target, &packet, targets, jitter),
        ));
        events.push(Event::Sound {
            resource_id: AttackPacket(&packet).launch_sound()?,
        });
        // spelltargets15cb8c offsets effect58's shape past the eighteen patterns;
        // that read is an empty pattern (docs/spell-variant-boundaries.md).
        if AttackPacket(&packet).effect()? == 58 {
            combat.spell_fields = spells::summon_field(
                &combat.spell_fields,
                &spells::SpellContext {
                    packet: &packet,
                    id: request.spell,
                    count: request.power,
                    caster: actor as i16,
                    initiative: combat.cursor as i16,
                    collision: false,
                },
                &target,
                random,
                request.rotation,
            )?;
            [0; 110]
        } else {
            let mask = spells::target_mask_rotated_with_origin(
                combat,
                warriors,
                &mission.areas,
                (&target, &target),
                &spells::SpellContext {
                    packet: &packet,
                    id: request.spell,
                    count: request.power,
                    caster: actor as i16,
                    initiative: combat.cursor as i16,
                    collision: false,
                },
                &combat.spell_fields,
                random,
                (request.rotation, context.land.with_tutorial(mission)),
            )?;
            combat.spell_fields = mask.fields;
            mask.mask
        }
    };
    // cast pushes WORD442bd4 as spelltargets' triple argument (10e634).
    let slot = combat.target_work.index;
    if AttackPacket(&packet).targeting_mode() == 7 {
        combat.ready = 0;
        let (minimum, maximum) = AttackPacket(&packet).duration_bounds();
        let mut duration = random.range(minimum, maximum);
        let (additional_minimum, additional_maximum) =
            AttackPacket(&packet).additional_duration_bounds();
        if additional_minimum != 0 {
            for _ in 0..request.power {
                duration =
                    duration.wrapping_add(random.range(additional_minimum, additional_maximum));
            }
        }
        crate::combat::party_buffs::extend(buffs, AttackPacket(&packet).effect()?, duration)?;
        combat.target_work.resolved(slot);
        crate::event::Flight::attach(&mut events, &flights);
        return Ok(events);
    }
    let body_loop = events.len();
    events.extend(resolve_mask(
        combat,
        warriors,
        mission,
        random,
        actor,
        request,
        &packet,
        &mask,
        (context.item_work_alias, context.owners, context.refresh),
    )?);
    crate::event::SpellArea::attach(&mut events, body_loop, &mask, &combat.attack_packet);
    crate::event::Flight::attach(&mut events, &flights);
    // spelltargets15d13c..15d1bc wraps a non-ray body resolution with Ready1/0.
    combat.ready = 0;
    combat.target_work.resolved(slot);
    Ok(events)
}
