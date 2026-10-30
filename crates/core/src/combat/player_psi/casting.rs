//! A warrior casting a psi power at its targets.
use super::{
    CastRequest, Combat, Event, RandomState, RuleError, Session, cast_ray, cost, move_caster,
    packet, resolve_mask, selected_body, spells, validate_target,
};
use crate::combat::map_alias;
use crate::combat::map_alias::MapAliases;
use crate::combat::player_psi::selection::Selection;
use crate::combat::spells::spell_identity;
use crate::combat::spells::summon::ItemRefresh;
use crate::combat::warrior_alias;
use crate::combat::warrior_alias::Owners;
use centurions_content::Warrior;
use centurions_content::mission::OpeningMission;
use centurions_content::records::{PsiSelectionMemoryMut, SpellInfoRecord};

/// A warrior casts a psi power in battle. A cast that lasts on the whole party is refused here.
/// Nothing changes if the cast fails.
pub fn cast(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    random: &mut RandomState,
    memory: &mut Vec<u8>,
    disabled: u8,
    request: CastRequest,
) -> Result<Vec<Event>, RuleError> {
    if SpellInfoRecord(&packet(mission, request.spell)?).is_party_duration() {
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

/// A warrior casts a psi power in battle with the party's buffs in force. The cast is checked
/// first, then committed only if every effect succeeds.
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

/// How the cast is paid for.
pub(super) enum CastPayment {
    /// The cost is paid now.
    Charge,
    /// The cost was paid when the power was chosen.
    Prepared,
    /// A further target of a power already paid for; no new cost and no new action.
    AdditionalTarget,
}

/// What a cast needs beyond the request: the movement gate, the body aimed at, the animation delay,
/// the item restriction, and the records that effects past the warrior table reach.
#[derive(Default)]
pub(crate) struct CastContext<'a> {
    pub movement_gate: i16,
    pub body: Option<i16>,
    pub animation_delay: i16,
    pub item_restriction: Option<i16>,
    /// Item work, shop work and map header, where effects 26 and 64 on a monster
    /// land past the warrior table.
    pub owners: Owners<'a>,
    /// The current land arrays beside the map, read by spelltargets past its edges.
    pub land: MapAliases<'a>,
    /// The item work combatupdate2 reloads when a full roster refuses a summon.
    /// When present it is also the item work in `owners`.
    pub refresh: ItemRefresh<'a>,
}

/// Casts the power chosen in advance at a cell, after the cast is checked again with its cost
/// refunded. A movement effect (55 or 56) clears the bandage-ready flag afterwards.
#[allow(clippy::too_many_arguments)]
pub(crate) fn cast_prepared_at(
    state: &mut Session,
    (mission, items): (&OpeningMission, &[Vec<u8>]),
    pending: &Selection,
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
    // Admission sees the cost refunded; the cast itself never charges twice.
    let mut admission = state.warriors.clone();
    let warrior = &mut admission[pending.actor];
    let psi = warrior
        .record()
        .psi_points()?
        .wrapping_add(cost(&packet, pending.power)?);
    let casts = warrior.record().spells_so_far()?.wrapping_sub(1);
    warrior.record_mut().set_psi_points(psi);
    warrior.record_mut().set_spells_so_far(casts);
    validate_target(
        combat,
        &admission,
        mission,
        state.psi_disabled,
        request,
        (body, map_alias::session_map_aliases!(state, mission)),
    )?;
    let context = CastContext {
        movement_gate: state.bandage_ready,
        body,
        animation_delay: state.command_preferences.animation_delay,
        item_restriction: spells::loaded_item_restriction(&state.globals.loaded_items)?,
        owners: warrior_alias::backup_owners!(state),
        land: map_alias::session_map_aliases!(state),
        refresh: Some((items, &mut state.globals.loaded_items)),
    };
    // A cast at a single point.
    state.combat.as_mut().unwrap().target_picks.enter(0, 1);
    let events = cast_inner(
        state.combat.as_mut().unwrap(),
        &mut state.warriors,
        mission,
        &mut state.random,
        &mut state.globals.define_spells,
        &mut state.party_condition,
        request,
        CastPayment::Prepared,
        context,
    )?;
    state.combat.as_mut().unwrap().target_picks.leave();
    state.commit_alias_writes()?;
    if matches!(SpellInfoRecord(&packet).effect()?, 55 | 56) {
        state.bandage_ready = 0;
    }
    Ok(events)
}

/// Casts the power chosen in advance at each body picked for it, in turn. The first pays the chosen
/// cost and the others cost nothing more; the loop ends early when the target picks say so.
pub(crate) fn cast_selection(
    state: &mut Session,
    (mission, items): (&OpeningMission, &[Vec<u8>]),
    selection: &Selection,
) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    let combat = state.combat.as_mut().ok_or("psi combat missing")?;
    let memory = &mut state.globals.define_spells;
    let total = selection.targets.len();
    for (index, body) in selection.targets.iter().copied().enumerate() {
        combat.target_picks.enter(index, total);
        let target = combat.body_position(body);
        events.extend(cast_inner(
            combat,
            &mut state.warriors,
            mission,
            &mut state.random,
            memory,
            &mut state.party_condition,
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
                item_restriction: spells::loaded_item_restriction(&state.globals.loaded_items)?,
                owners: warrior_alias::backup_owners!(state),
                land: map_alias::session_map_aliases!(state),
                refresh: Some((items, &mut state.globals.loaded_items)),
                ..CastContext::default()
            },
        )?);
        warrior_alias::commit_pending!(state, combat)?;
        if combat.target_picks.loop_ended(index) {
            break;
        }
    }
    combat.target_picks.leave();
    Ok(events)
}

/// Casts a power for the active warrior from the mission's attack table.
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

/// Casts an item's power. The item has already paid its charge, and an item cast
/// is not remembered as the warrior's last power.
///
/// Original: cast
pub(crate) fn cast_item(
    state: &mut Session,
    (mission, items): (&OpeningMission, &[Vec<u8>]),
    pending: &Selection,
    target: (i16, i16),
    rotation: u8,
    body: Option<i16>,
    first: bool,
) -> Result<Vec<Event>, RuleError> {
    let packet = state
        .combat
        .as_ref()
        .map(|c| c.spell_info.clone())
        .ok_or("prepared item packet missing")?;
    let context = CastContext {
        movement_gate: state.bandage_ready,
        body,
        animation_delay: state.command_preferences.animation_delay,
        item_restriction: spells::loaded_item_restriction(&state.globals.loaded_items)?,
        owners: warrior_alias::backup_owners!(state),
        land: map_alias::session_map_aliases!(state),
        refresh: Some((items, &mut state.globals.loaded_items)),
    };
    // The power is at least 1, applied once targets are chosen.
    let power = pending.power.max(1);
    state.globals.attack_count = power;
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
        &mut state.party_condition,
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
    state.combat.as_mut().unwrap().item_condition_caller = previous;
    let events = result?;
    state.commit_alias_writes()?;
    Ok(events)
}

/// Casts a power: marks the spell under way, pays for it, and resolves it as a move, a ray or a
/// spell on the picked bodies according to its effect.
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
    // Prepared and remembered casts mark the spell as under way.
    combat.in_spell = 1;
    combat.blank_round = 0;
    let descriptor = spell_identity(request.spell);
    combat.spell_info = packet.clone();
    combat.cast_identity = descriptor;
    pay_cast(&mut warriors[actor], &packet, request, payment)?;
    let mut events = vec![];
    let mut flights = vec![];
    if matches!(SpellInfoRecord(&packet).target_type(), 9 | 10) {
        events.push(global_flash(&packet));
    }
    if let Some(memory) = memory {
        PsiSelectionMemoryMut(memory).remember_cast(actor, descriptor, request.power);
    }
    if matches!(SpellInfoRecord(&packet).effect()?, 55 | 56) {
        let jitter = spells::flight_jitter(&packet, random);
        let flight = request.target.and_then(|cell| {
            let (x, y) = crate::combat::records::wrapping_grid_point(cell, combat.field.map_shift);
            let target = spells::SpellTarget::cell(x, y);
            let targets = usize::try_from(combat.target_picks.count).unwrap_or(0);
            spells::flight(combat, actor as i16, &target, &packet, targets, jitter)
        });
        flights.push((events.len(), flight));
        events.push(Event::Sound {
            resource_id: SpellInfoRecord(&packet).launch_sound()?,
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
    if SpellInfoRecord(&packet).is_ray() {
        let jitter = spells::flight_jitter(&packet, random);
        let flight = request.target.and_then(|cell| {
            let (x, y) = crate::combat::records::wrapping_grid_point(cell, combat.field.map_shift);
            let target = spells::SpellTarget::cell(x, y);
            spells::flight(combat, actor as i16, &target, &packet, 1, jitter)
        });
        flights.push((events.len(), flight));
        events.push(Event::Sound {
            resource_id: SpellInfoRecord(&packet).launch_sound()?,
        });
        if !spells::ray_walks(&packet) {
            request.target.ok_or("ray destination missing")?;
        } else {
            let start = combat.squad.pos[actor];
            let end = request.target.ok_or("ray destination missing")?;
            let crossings = spells::crossings(
                &combat.field.tiles,
                combat.field.map_shift,
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
                (context.item_restriction, context.owners, context.refresh),
            )?);
            spells::mark_crossings(&mut events[walk..], &crossings.0, crossings.1.is_some());
            spells::stop_flight(&mut flights, crossings.1);
            crate::event::Flight::attach(&mut events, &flights);
            return Ok(events);
        }
        crate::event::Flight::attach(&mut events, &flights);
        return Ok(events);
    }
    let mask = if SpellInfoRecord(&packet).effect()? == 64
        && let Some(body) = context.body
    {
        let mut mask = [0; 110];
        mask[body as usize] = 1;
        let target = spells::SpellTarget::body(body);
        launch(
            combat,
            random,
            actor,
            &target,
            &packet,
            &mut events,
            &mut flights,
        )?;
        mask
    } else if SpellInfoRecord(&packet).is_global() {
        global_mask(combat, warriors, &packet)?
    } else {
        let target = cast_target(combat, warriors, actor, request, &packet, context.body)?;
        // The target count passes to cast, whose short-walk test reads it.
        launch(
            combat,
            random,
            actor,
            &target,
            &packet,
            &mut events,
            &mut flights,
        )?;
        let spell = spells::SpellContext {
            packet: &packet,
            id: request.spell,
            count: request.power,
            caster: actor as i16,
            initiative: combat.battle_order_index as i16,
            collision: false,
        };
        area_mask(
            combat,
            warriors,
            mission,
            random,
            &spell,
            &target,
            (request.rotation, context.land),
        )?
    };
    // spelltargets receives the target cast is resolving.
    let slot = combat.target_picks.index;
    if SpellInfoRecord(&packet).is_party_duration() {
        combat.in_spell = 0;
        extend_party_duration(random, buffs, &packet, request.power)?;
        combat.target_picks.resolved(slot);
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
        (context.item_restriction, context.owners, context.refresh),
    )?);
    crate::event::SpellArea::attach(&mut events, body_loop, &mask, &combat.spell_info);
    crate::event::Flight::attach(&mut events, &flights);
    // spelltargets marks the spell as under way only while it resolves a body.
    combat.in_spell = 0;
    combat.target_picks.resolved(slot);
    Ok(events)
}

/// The flash a faction-wide cast shows over its own side.
const OWN_SIDE_FLASH: u32 = 0x401840;
/// The flash a faction-wide cast shows over the other side.
const OTHER_SIDE_FLASH: u32 = 0x401820;
/// Where a faction-wide cast's flash is drawn.
const GLOBAL_FLASH_POSITION: (i16, i16) = (325, 322);

/// The flash before a faction-wide (mode 9 or 10) cast.
fn global_flash(packet: &[u8]) -> Event {
    Event::Feedback(crate::feedback::GameFeedback::Flash {
        address: if SpellInfoRecord(packet).is_global_own_side() {
            OWN_SIDE_FLASH
        } else {
            OTHER_SIDE_FLASH
        },
        x: GLOBAL_FLASH_POSITION.0,
        y: GLOBAL_FLASH_POSITION.1,
        wait: 60,
        sound: 0,
    })
}

/// A charged cast spends the packet's psi cost and counts the scan; every cast but an
/// additional target spends the attack action.
fn pay_cast(
    warrior: &mut Warrior,
    packet: &[u8],
    request: CastRequest,
    payment: CastPayment,
) -> Result<(), RuleError> {
    if matches!(payment, CastPayment::Charge) {
        let remaining = warrior
            .record()
            .psi_points()?
            .wrapping_sub(cost(packet, request.power)?);
        warrior.record_mut().set_psi_points(remaining);
        warrior.record_mut().count_scan_cast();
        let count = warrior.record().spells_so_far()?.wrapping_add(1);
        warrior.record_mut().set_spells_so_far(count);
    }
    if !matches!(payment, CastPayment::AdditionalTarget) {
        crate::combat::records::spend_attack_action(warrior)?;
    }
    Ok(())
}

/// The flight draws, flight and launch sound toward one target.
fn launch(
    combat: &Combat,
    random: &mut RandomState,
    actor: usize,
    target: &spells::SpellTarget,
    packet: &[u8],
    events: &mut Vec<Event>,
    flights: &mut Vec<(usize, Option<crate::event::Flight>)>,
) -> Result<(), RuleError> {
    let jitter = spells::flight_jitter(packet, random);
    let targets = usize::try_from(combat.target_picks.count).unwrap_or(0);
    flights.push((
        events.len(),
        spells::flight(combat, actor as i16, target, packet, targets, jitter),
    ));
    events.push(Event::Sound {
        resource_id: SpellInfoRecord(packet).launch_sound()?,
    });
    Ok(())
}

/// Every active warrior and living monster on the side a faction-wide packet reaches.
fn global_mask(
    combat: &Combat,
    warriors: &[Warrior],
    packet: &[u8],
) -> Result<[i16; 110], RuleError> {
    let mut mask = [0; 110];
    for (body, warrior) in warriors.iter().enumerate() {
        if warrior.record().in_battle() != 0
            && (SpellInfoRecord(packet).is_global_all()
                || (warrior.record().faction()? == 0)
                    == (SpellInfoRecord(packet).is_global_own_side()))
        {
            mask[body] = 1;
        }
    }
    for (index, monster) in combat.roster.records.iter().enumerate() {
        if monster.record().stamina()? > 0
            && (SpellInfoRecord(packet).is_global_all()
                || (monster.record().faction()? == 0)
                    == (SpellInfoRecord(packet).is_global_own_side()))
        {
            mask[index + 10] = 1;
        }
    }
    Ok(mask)
}

/// The selected body below mode 2, otherwise the selected or caster's cell.
fn cast_target(
    combat: &Combat,
    warriors: &[Warrior],
    actor: usize,
    request: CastRequest,
    packet: &[u8],
    body: Option<i16>,
) -> Result<spells::SpellTarget, RuleError> {
    let target = request.target.unwrap_or(combat.squad.pos[actor]);
    Ok(if SpellInfoRecord(packet).target_type() < 2 {
        let body = match body {
            Some(body) => body,
            None => selected_body(combat, warriors, target)?,
        };
        spells::SpellTarget::body(body)
    } else {
        let (x, y) = crate::combat::records::wrapping_grid_point(target, combat.field.map_shift);
        spells::SpellTarget::cell(x, y)
    })
}

/// The area around the target; effect 58 builds a summon field instead.
///
/// Original: spelltargets
fn area_mask(
    combat: &mut Combat,
    warriors: &[Warrior],
    mission: &OpeningMission,
    random: &mut RandomState,
    spell: &spells::SpellContext<'_>,
    target: &spells::SpellTarget,
    (rotation, land): (u8, MapAliases<'_>),
) -> Result<[i16; 110], RuleError> {
    // Effect 58's shape lies past the eighteen patterns and reads as empty
    // (docs/spell-variant-boundaries.md).
    if SpellInfoRecord(spell.packet).effect()? == 58 {
        combat.spell_queue =
            spells::summon_field(&combat.spell_queue, spell, target, random, rotation)?;
        return Ok([0; 110]);
    }
    let mask = spells::target_mask_rotated_with_origin(
        combat,
        warriors,
        &mission.areas,
        (target, target),
        spell,
        &combat.spell_queue,
        random,
        (rotation, land.with_tutorial(mission)),
    )?;
    combat.spell_queue = mask.fields;
    Ok(mask.mask)
}

/// A mode-7 spell draws its duration, then extends the party's.
///
/// Original: resolvespell
fn extend_party_duration(
    random: &mut RandomState,
    buffs: &mut [u8],
    packet: &[u8],
    power: i16,
) -> Result<(), RuleError> {
    let (minimum, maximum) = SpellInfoRecord(packet).duration_range();
    let mut duration = random.range(minimum, maximum);
    let (additional_minimum, additional_maximum) = SpellInfoRecord(packet).power_duration_range();
    if additional_minimum != 0 {
        for _ in 0..power {
            duration = duration.wrapping_add(random.range(additional_minimum, additional_maximum));
        }
    }
    crate::combat::party_buffs::extend(buffs, SpellInfoRecord(packet).effect()?, duration)
}
