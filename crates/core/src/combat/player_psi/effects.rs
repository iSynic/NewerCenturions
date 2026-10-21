//! Effects for combat/player_psi.
use super::{CastRequest, Combat, Event, RandomState, RuleError, runtime, spells};
use centurions_content::mission::OpeningMission;
use centurions_content::records::{AttackPacket, AttackPacketMut};
use centurions_content::{Warrior, word};

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_mask(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    random: &mut RandomState,
    actor: usize,
    request: CastRequest,
    packet: &[u8],
    mask: &[i16; 110],
) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    let previous_hp = warriors
        .iter()
        .map(Warrior::stamina)
        .collect::<Result<Vec<_>, _>>()?;
    if AttackPacket(packet).saving_throw_mode() <= 3
        && matches!(AttackPacket(packet).defense_category(), 7 | 8 | 10)
        && matches!(AttackPacket(packet).effect()?, 0..=41 | 51 | 57 | 64)
        && (AttackPacket(packet).effect()? == 57
            || (packet[12] == 0
                && word(packet, 34)? == 0
                && packet[23..30].iter().all(|v| *v == 0)))
    {
        let result = spells::resolve_support_for_caster(
            warriors,
            &combat.enemies.records,
            mask,
            request.spell,
            &spells::DirectSpell {
                packet,
                level: request.spell % 1000 / 100 - 1,
                count: request.power,
                armor_filter: combat.weapon_work.armor_filter()?,
                weapon_sound: combat.weapon_work.sound()?,
            },
            (
                Some(actor as i16),
                Some(spells::RestrictionMemoryView::ordinary_startup(
                    &mission.item_cache,
                    None,
                )),
            ),
            random,
        )?;
        warriors.clone_from_slice(&result.warriors);
        combat.enemies.records = result.enemies;
        combat.unavailable = combat.unavailable.wrapping_sub(result.recovered as i8);
        if result.healed {
            AttackPacketMut(combat.attack_packet.as_mut().unwrap()).set_health_effect(4);
        }
        if !result.targets.is_empty() {
            combat.field_used[0] = 1;
        }
        events.extend(
            result
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
        for (target, outcome) in &result.targets {
            if *target >= 10 && outcome.damage != 0 {
                let mut record = warriors[actor].record_mut();
                record.add_damage_dealt(outcome.damage);
                if outcome.killed {
                    record.count_kill();
                }
            }
        }
        runtime::append_spell_outcomes(
            combat,
            warriors,
            result.targets,
            vec![],
            (Some(actor as i16), request.spell, packet, &previous_hp),
            false,
            &mut events,
        )?;
        return Ok(events);
    }
    let context = spells::DirectSpell {
        packet,
        level: request.spell % 1000 / 100 - 1,
        count: request.power,
        armor_filter: combat.weapon_work.armor_filter()?,
        weapon_sound: combat.weapon_work.sound()?,
    };
    let memory_view = spells::RestrictionMemoryView::ordinary_startup(&mission.item_cache, None);
    let result = if matches!(AttackPacket(packet).effect()?, 60 | 61) {
        spells::resolve_psi_drain(
            warriors,
            &combat.enemies.records,
            mask,
            &context,
            actor as i16,
            memory_view,
            random,
        )?
    } else {
        spells::resolve_bodies_with_memory(
            warriors,
            &combat.enemies.records,
            mask,
            &context,
            memory_view,
            random,
        )?
    };
    warriors.clone_from_slice(&result.warriors);
    combat.enemies.records = result.enemies;
    if !result.targets.is_empty() {
        combat.field_used[0] = 1;
    }
    events.extend(
        result
            .sounds
            .into_iter()
            .map(|resource_id| Event::Sound { resource_id }),
    );
    for (target, outcome) in &result.targets {
        if *target >= 10 && outcome.damage != 0 {
            let mut record = warriors[actor].record_mut();
            record.add_damage_dealt(outcome.damage);
            if outcome.killed {
                record.count_kill();
            }
        }
    }
    runtime::append_spell_outcomes(
        combat,
        warriors,
        result.targets,
        result.race_excluded,
        (Some(actor as i16), request.spell, packet, &previous_hp),
        false,
        &mut events,
    )?;
    Ok(events)
}

/// cast10e0c8..10e642 resolves each newly crossed body before continuing the ray.
/// SINGLE accumulators and truncation match the retained SANE boundary model.
#[allow(clippy::too_many_arguments)]
pub(super) fn cast_ray(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    random: &mut RandomState,
    actor: usize,
    request: CastRequest,
    packet: &[u8],
    animation_delay: i16,
) -> Result<Vec<Event>, RuleError> {
    let start = combat.squad.positions[actor];
    let end = request.target.ok_or("ray destination missing")?;
    let steps = 128 + animation_delay;
    let mut point = (
        f32::from(start.0.wrapping_mul(32)),
        f32::from(start.1.wrapping_mul(32)),
    );
    let increment = (
        f32::from(
            end.0
                .wrapping_mul(32)
                .wrapping_sub(start.0.wrapping_mul(32)),
        ) / steps as f32,
        f32::from(
            end.1
                .wrapping_mul(32)
                .wrapping_sub(start.1.wrapping_mul(32)),
        ) / steps as f32,
    );
    let mut seen = [false; 110];
    let mut events = vec![];
    for _ in 0..steps {
        let x = (point.0.trunc() as i16).wrapping_add(16) / 32;
        let y = (point.1.trunc() as i16).wrapping_add(16) / 32;
        let cell = OpeningMission::index(
            i32::from(x) + i32::from(combat.map.shift.0) + 5,
            i32::from(y) + i32::from(combat.map.shift.1) + 5,
        )?;
        let body = combat.map.tiles[cell].wrapping_abs();
        if AttackPacket(packet).attack_count() + AttackPacket(packet).range_modifier() > 0
            && body > 1000
            && mission.tile_word(body - 1000, 12)? != 0
        {
            events.push(Event::Sound {
                resource_id: -10082,
            });
            break;
        }
        if (0..110).contains(&body) && body as usize != actor && !seen[body as usize] {
            seen[body as usize] = true;
            let mut mask = [0; 110];
            mask[body as usize] = 1;
            events.extend(resolve_mask(
                combat, warriors, mission, random, actor, request, packet, &mask,
            )?);
        }
        point.0 += increment.0;
        point.1 += increment.1;
    }
    Ok(events)
}

/// resolvespell151418..1516e6 moves the caster without a destination resistance test.
#[allow(clippy::too_many_arguments)]
pub(super) fn move_caster(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    random: &mut RandomState,
    actor: usize,
    packet: &[u8],
    request: CastRequest,
    gate: Option<i16>,
) -> Result<Vec<Event>, RuleError> {
    let gate = gate.ok_or(RuleError::Unsupported {
        action: "combat.player_psi",
        dependency: "movement spell interaction marker absent from legacy context",
    })?;
    let (minimum, maximum) = AttackPacket(packet).duration_bounds();
    random.range(minimum, maximum);
    if packet[15] != 0 || packet[31] != 0 {
        return Err(RuleError::Unsupported {
            action: "combat.player_psi",
            dependency: "movement packet with duration scaling or repeated initiative",
        });
    }
    if gate == 0 {
        return Ok(vec![Event::Feedback(
            crate::feedback::NativeFeedback::Warning { index: 87 },
        )]);
    }
    if AttackPacket(packet).effect()? == 55 {
        warriors[actor].record_mut().set_movement_points(0);
        let maximum = warriors[actor].record().scan_capacity()?;
        warriors[actor].record_mut().set_scans_used(maximum);
    }
    let target = request.target.ok_or("movement spell destination missing")?;
    combat.map = crate::combat::footprint::restore_body(combat, actor as i16)?;
    let cell = OpeningMission::index(
        i32::from(target.0) + i32::from(combat.map.shift.0) + 5,
        i32::from(target.1) + i32::from(combat.map.shift.1) + 5,
    )?;
    combat.squad.positions[actor] = target;
    let ground = combat.map.tiles[cell];
    combat.squad.under_tiles[actor] = ground;
    combat.map.tiles[cell] = actor as i16;
    let mut events = vec![
        Event::Sound { resource_id: 699 },
        Event::TacticalMoved {
            body: actor as i16,
            x: target.0,
            y: target.1,
        },
        Event::Sound { resource_id: 658 },
    ];
    let defeated = ground < 999 || ground > 999 && mission.tile_word(ground - 1000, 4)? != 0;
    if defeated {
        events.push(Event::Sound { resource_id: 631 });
        events.push(Event::Feedback(crate::feedback::NativeFeedback::Warning {
            index: 123,
        }));
        warriors[actor].record_mut().set_stamina(-10);
        crate::combat::lifecycle::kill_body(combat, warriors, actor as i16)?;
    } else {
        warriors[actor].record_mut().mark_damaged();
    }
    events.push(Event::SpellResolved {
        caster: Some(actor as i16),
        spell: request.spell,
        target: actor as i16,
        damage: 0,
        defeated,
    });
    Ok(events)
}
