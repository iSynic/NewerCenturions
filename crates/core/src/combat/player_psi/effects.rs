//! Effects for combat/player_psi.
use super::{CastRequest, Combat, Event, RandomState, RuleError, put_word, runtime, spells};
use centurions_content::mission::OpeningMission;
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
    if packet[9] <= 3
        && matches!(packet[19], 7 | 8 | 10)
        && matches!(word(packet, 40)?, 0..=41 | 51 | 57 | 64)
        && (word(packet, 40)? == 57
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
            combat.attack_packet.as_mut().unwrap()[18] = 4;
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
                let w = &mut warriors[actor].bytes;
                let total = i32::from_be_bytes(w[0xa46..0xa4a].try_into().unwrap())
                    .wrapping_add(i32::from(outcome.damage));
                w[0xa46..0xa4a].copy_from_slice(&total.to_be_bytes());
                if outcome.killed {
                    let kills =
                        i32::from_be_bytes(w[0xa5a..0xa5e].try_into().unwrap()).wrapping_add(1);
                    w[0xa5a..0xa5e].copy_from_slice(&kills.to_be_bytes());
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
    let result = if matches!(word(packet, 40)?, 60 | 61) {
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
            let w = &mut warriors[actor].bytes;
            let total = i32::from_be_bytes(w[0xa46..0xa4a].try_into().unwrap())
                .wrapping_add(i32::from(outcome.damage));
            w[0xa46..0xa4a].copy_from_slice(&total.to_be_bytes());
            if outcome.killed {
                let kills = i32::from_be_bytes(w[0xa5a..0xa5e].try_into().unwrap()).wrapping_add(1);
                w[0xa5a..0xa5e].copy_from_slice(&kills.to_be_bytes());
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
        if i16::from(packet[0] as i8) + i16::from(packet[2] as i8) > 0
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
    random.range(i16::from(packet[13] as i8), i16::from(packet[14] as i8));
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
    if word(packet, 40)? == 55 {
        put_word(&mut warriors[actor].bytes, 0x3c, 0);
        let maximum = word(&warriors[actor].bytes, 0xac6)?;
        put_word(&mut warriors[actor].bytes, 0xac8, maximum);
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
        put_word(&mut warriors[actor].bytes, 0x50, -10);
        crate::combat::lifecycle::kill_body(combat, warriors, actor as i16)?;
    } else {
        put_word(&mut warriors[actor].bytes, 0x14, 1);
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
