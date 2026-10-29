//! Effects for combat/player_psi.
use super::{CastRequest, Combat, Event, RandomState, RuleError, runtime, spells};
use centurions_content::Warrior;
use centurions_content::mission::OpeningMission;
use centurions_content::records::{AttackPacket, AttackPacketMut};

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_mask(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    random: &mut RandomState,
    actor: usize,
    request: CastRequest,
    packet: &[u8],
    mask: &[i16; 110],
    (item_work_alias, owners, refresh): (
        Option<i16>,
        crate::combat::warrior_alias::Owners<'_>,
        crate::combat::spell_summon::ItemRefresh<'_>,
    ),
) -> Result<Vec<Event>, RuleError> {
    // resolvespell's 0x3a arm replaces the body loop and its NPC copies; a full
    // roster reloads ITEM work through combatupdate2.
    if AttackPacket(packet).effect()?.wrapping_abs() == 58 {
        return crate::combat::spell_summon::summon(
            combat,
            warriors,
            mission,
            random,
            actor,
            request.target,
            request.power,
            packet,
            refresh,
        );
    }
    // A caller that lends the ITEM work for the refusal reads its aliases there too.
    let owners = crate::combat::warrior_alias::Owners {
        item_work: match &refresh {
            Some((_, buffers)) => buffers.item.as_deref(),
            None => owners.item_work,
        },
        ..owners
    };
    combat.mark_monster_spell_work(packet, mask);
    let mut events = vec![];
    let condition_caller = combat.item_condition_caller.unwrap_or(actor as i16);
    let memory_view =
        spells::RestrictionMemoryView::ordinary_startup(&mission.item_cache, item_work_alias)
            .with_tables(mission)
            .with_alias_owners(owners)
            .with_live_caster(combat, warriors);
    let memory_view = if combat.item_condition_caller.is_some() {
        memory_view.with_selected_party_caster(actor as i16)
    } else {
        memory_view
    };
    let previous_hp = warriors
        .iter()
        .map(Warrior::stamina)
        .collect::<Result<Vec<_>, _>>()?;
    if spells::is_support(packet)? {
        let pending = std::mem::take(&mut combat.alias_writes);
        let result = spells::resolve_support_for_caster(
            warriors,
            &combat.enemies.records,
            mask,
            request.spell,
            &spells::DirectSpell {
                packet,
                level: request.spell % 1000 / 100 - 1,
                count: request.power,
                armor_filter: combat.weapon_work.armor_filter(),
                weapon_sound: combat.weapon_work.sound(),
            },
            (
                Some(condition_caller),
                Some(memory_view.with_pending(&pending)),
            ),
            random,
        )?;
        combat.alias_writes = pending;
        combat.alias_writes.extend(result.alias_writes);
        warriors.clone_from_slice(&result.warriors);
        combat.fatigue_reset |= result.fatigue_reset;
        if let Some(shape) = result.area_shape {
            AttackPacketMut(&mut combat.attack_packet).set_area_shape(shape);
        }
        combat.victory_work.add_total(result.total_delta);
        combat.enemies.records = result.enemies;
        combat.unavailable = combat.unavailable.wrapping_sub(result.recovered as i8);
        if result.healed {
            AttackPacketMut(&mut combat.attack_packet).set_health_effect(4);
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
            result.race_excluded,
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
        armor_filter: combat.weapon_work.armor_filter(),
        weapon_sound: combat.weapon_work.sound(),
    };
    let result = if matches!(AttackPacket(packet).effect()?, 60 | 61) {
        spells::resolve_psi_drain(
            warriors,
            &combat.enemies.records,
            mask,
            &context,
            condition_caller,
            (memory_view, None),
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
    combat.victory_work.add_total(result.total_delta);
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
    (alias, owners, mut refresh): (
        Option<i16>,
        crate::combat::warrior_alias::Owners<'_>,
        crate::combat::spell_summon::ItemRefresh<'_>,
    ),
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
        if AttackPacket(packet).range_per_power() + AttackPacket(packet).base_range() > 0
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
                combat,
                warriors,
                mission,
                random,
                actor,
                request,
                packet,
                &mask,
                (
                    alias,
                    owners,
                    refresh
                        .as_mut()
                        .map(|(items, buffers)| (*items, &mut **buffers)),
                ),
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
    gate: i16,
) -> Result<Vec<Event>, RuleError> {
    let (minimum, maximum) = AttackPacket(packet).duration_bounds();
    random.range(minimum, maximum);
    let (additional_minimum, additional_maximum) =
        AttackPacket(packet).additional_duration_bounds();
    if additional_minimum != 0 {
        for _ in 0..request.power {
            random.range(additional_minimum, additional_maximum);
        }
    }
    if gate == 0 {
        return Ok(vec![Event::Feedback(
            crate::feedback::NativeFeedback::Warning { index: 87 },
        )]);
    }
    let target = request.target.ok_or("movement spell destination missing")?;
    let (mut events, defeated) = land_mover(
        MoveArena {
            map: &mut combat.map,
            positions: &mut combat.squad.positions,
            under_tiles: &mut combat.squad.under_tiles,
            cursor: &mut combat.cursor,
        },
        warriors,
        mission,
        actor,
        packet,
        target,
    )?;
    if defeated {
        crate::combat::lifecycle::kill_body(combat, warriors, actor as i16)?;
    }
    events.push(Event::SpellResolved {
        caster: Some(actor as i16),
        spell: request.spell,
        target: actor as i16,
        damage: 0,
        defeated,
        icon: crate::event::PresentationIcon(0),
        flight: Default::default(),
    });
    Ok(events)
}

/// The tactical arrays resolvespell's movement arm writes, live or after teardown.
pub(crate) struct MoveArena<'a> {
    pub map: &'a mut crate::combat::TacticalMap,
    pub positions: &'a mut [(i16, i16)],
    pub under_tiles: &'a mut [i16],
    /// WORD447194, the initiative cursor.
    pub cursor: &'a mut usize,
}

/// resolvespell151418..1516e6 after its duration draws and WORD438786 test: moves
/// WORD4435d2 to the target cell. A defeated landing leaves killbody to the caller.
pub(crate) fn land_mover(
    arena: MoveArena<'_>,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    actor: usize,
    packet: &[u8],
    target: (i16, i16),
) -> Result<(Vec<Event>, bool), RuleError> {
    if AttackPacket(packet).effect()? == 55 {
        warriors[actor].record_mut().set_movement_points(0);
        let maximum = warriors[actor].record().scan_capacity()?;
        warriors[actor].record_mut().set_scans_used(maximum);
    }
    let position = *arena
        .positions
        .get(actor)
        .ok_or_else(|| RuleError::Invalid("body-field warrior position unavailable".into()))?;
    let under = *arena
        .under_tiles
        .get(actor)
        .ok_or_else(|| RuleError::Invalid("body-field warrior under-tile unavailable".into()))?;
    *arena.map = crate::combat::footprint::restore_warrior(arena.map, position, under)?;
    let cell = OpeningMission::index(
        i32::from(target.0) + i32::from(arena.map.shift.0) + 5,
        i32::from(target.1) + i32::from(arena.map.shift.1) + 5,
    )?;
    arena.positions[actor] = target;
    let ground = arena.map.tiles[cell];
    arena.under_tiles[actor] = ground;
    arena.map.tiles[cell] = actor as i16;
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
    } else {
        // 1516b4: a repeated-initiative packet steps the initiative cursor back
        // so the caster acts again; a defeated landing does not.
        if packet[31] != 0 {
            *arena.cursor = usize::from((*arena.cursor as u16).wrapping_sub(1));
        }
        warriors[actor].record_mut().mark_damaged();
    }
    Ok((events, defeated))
}
