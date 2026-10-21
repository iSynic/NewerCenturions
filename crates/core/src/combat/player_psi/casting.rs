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
    context: CastContext,
) -> Result<Vec<Event>, RuleError> {
    validate_target(combat, warriors, mission, disabled, request, context.body)?;
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

#[derive(Clone, Copy, Default)]
pub(crate) struct CastContext {
    pub movement_gate: Option<i16>,
    pub body: Option<i16>,
    pub animation_delay: i16,
}

pub(crate) fn cast_prepared_at(
    state: &mut Session,
    mission: &OpeningMission,
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
        body,
    )?;
    let context = CastContext {
        movement_gate: state.bandage_ready,
        body,
        animation_delay: state.command_preferences.animation_delay,
    };
    let events = cast_inner(
        state.combat.as_mut().unwrap(),
        &mut state.warriors,
        mission,
        &mut state.random,
        state
            .psi_memory
            .as_mut()
            .ok_or("prepared scan memory missing")?,
        &mut state.party_buffs,
        request,
        CastPayment::Prepared,
        context,
    )?;
    if matches!(AttackPacket(&packet).effect()?, 55 | 56) {
        state.bandage_ready = Some(0);
    }
    Ok(events)
}

pub(crate) fn cast_selection(
    state: &mut Session,
    mission: &OpeningMission,
    selection: &crate::combat::psi_selection::Selection,
) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    let combat = state.combat.as_mut().ok_or("psi combat missing")?;
    let memory = state.psi_memory.as_mut().ok_or("psi memory missing")?;
    for (index, body) in selection.targets.iter().copied().enumerate() {
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
            CastContext::default(),
        )?);
    }
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
    context: CastContext,
) -> Result<Vec<Event>, RuleError> {
    let actor = combat.active_body().unwrap() as usize;
    let packet = packet(mission, request.spell)?;
    combat.inactive_rounds = 0;
    let descriptor = [
        request.spell / 1000 - 1,
        request.spell % 1000 / 100 - 1,
        request.spell % 100 - 1,
    ];
    combat.attack_packet = Some(packet.clone());
    combat.attack_identity = Some(descriptor);
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
    PsiSelectionMemoryMut(memory).remember_cast(actor, descriptor, request.power);
    if matches!(AttackPacket(&packet).effect()?, 55 | 56) {
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
        return Ok(events);
    }
    if AttackPacket(&packet).targeting_mode() == 6 {
        events.push(Event::Sound {
            resource_id: AttackPacket(&packet).launch_sound()?,
        });
        events.extend(cast_ray(
            combat,
            warriors,
            mission,
            random,
            actor,
            request,
            &packet,
            context.animation_delay,
        )?);
        return Ok(events);
    }
    let mask = if AttackPacket(&packet).effect()? == 64
        && let Some(body) = context.body
    {
        let mut mask = [0; 110];
        mask[body as usize] = 1;
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
        events.push(Event::Sound {
            resource_id: AttackPacket(&packet).launch_sound()?,
        });
        let mask = spells::target_mask_rotated(
            combat,
            warriors,
            &mission.areas,
            &target,
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
            request.rotation,
        )?;
        combat.spell_fields = mask.fields;
        mask.mask
    };
    if AttackPacket(&packet).targeting_mode() == 7 {
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
        return Ok(events);
    }
    events.extend(resolve_mask(
        combat, warriors, mission, random, actor, request, &packet, &mask,
    )?);
    Ok(events)
}
