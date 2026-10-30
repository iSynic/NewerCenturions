//! What a psi power does to its targets: bodies, rays and moving the caster.
use super::{CastRequest, Combat, Event, RandomState, RuleError, runtime, spells};
use crate::combat::spells::spell_level;
use centurions_content::Warrior;
use centurions_content::mission::OpeningMission;
use centurions_content::records::{SpellInfoRecord, SpellInfoRecordMut};

/// Applies the spell to every body its mask picks. A summon replaces the loop over bodies.
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
    (item_restriction, owners, refresh): (
        Option<i16>,
        crate::combat::warrior_alias::Owners<'_>,
        crate::combat::spells::summon::ItemRefresh<'_>,
    ),
) -> Result<Vec<Event>, RuleError> {
    // A summon replaces the loop over bodies; a full roster refreshes the panel.
    if SpellInfoRecord(packet).effect()?.wrapping_abs() == 58 {
        return crate::combat::spells::summon::summon(
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
    // The item work lent for the refusal also serves monster reads past the table.
    let owners = crate::combat::warrior_alias::Owners {
        loaded_item: match &refresh {
            Some((_, buffers)) => buffers.item.as_deref(),
            None => owners.loaded_item,
        },
        ..owners
    };
    combat.mark_monster_spell_pick(packet, mask);
    let mut events = vec![];
    let condition_caller = combat.item_condition_caller.unwrap_or(actor as i16);
    let memory_view =
        spells::RestrictionMemoryView::ordinary_startup(&mission.item_cache, item_restriction)
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
            &combat.roster.records,
            mask,
            request.spell,
            &spells::DirectSpell {
                packet,
                level: spell_level(request.spell),
                count: request.power,
                armor_filter: combat.weapon_traits.armor_filter(),
                weapon_sound: combat.weapon_traits.sound(),
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
            SpellInfoRecordMut(&mut combat.spell_info).set_area_size(shape);
        }
        combat.kill_counts.add_num_enemy(result.total_delta);
        combat.roster.records = result.enemies;
        combat.kill_party = combat.kill_party.wrapping_sub(result.recovered as i8);
        if result.healed {
            SpellInfoRecordMut(&mut combat.spell_info).set_health_effect(4);
        }
        if !result.targets.is_empty() {
            combat.collide_check[0] = 1;
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
        level: spell_level(request.spell),
        count: request.power,
        armor_filter: combat.weapon_traits.armor_filter(),
        weapon_sound: combat.weapon_traits.sound(),
    };
    let result = if matches!(SpellInfoRecord(packet).effect()?, 60 | 61) {
        spells::resolve_psi_drain(
            warriors,
            &combat.roster.records,
            mask,
            &context,
            condition_caller,
            (memory_view, None),
            random,
        )?
    } else {
        spells::resolve_bodies_with_memory(
            warriors,
            &combat.roster.records,
            mask,
            &context,
            memory_view,
            random,
        )?
    };
    warriors.clone_from_slice(&result.warriors);
    combat.kill_counts.add_num_enemy(result.total_delta);
    combat.roster.records = result.enemies;
    if !result.targets.is_empty() {
        combat.collide_check[0] = 1;
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

/// A ray resolves each body it crosses before travelling on. Its steps add and
/// truncate in single precision.
///
/// Original: cast
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
        crate::combat::spells::summon::ItemRefresh<'_>,
    ),
) -> Result<Vec<Event>, RuleError> {
    let start = combat.squad.pos[actor];
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
            i32::from(x) + i32::from(combat.field.map_shift.0) + 5,
            i32::from(y) + i32::from(combat.field.map_shift.1) + 5,
        )?;
        let body = combat.field.tiles[cell].wrapping_abs();
        if SpellInfoRecord(packet).range_per_power() + SpellInfoRecord(packet).range_base() > 0
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

/// Moves the caster to the target cell; the destination is not tested for
/// resistance.
///
/// Original: resolvespell
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
    let (minimum, maximum) = SpellInfoRecord(packet).duration_range();
    random.range(minimum, maximum);
    let (additional_minimum, additional_maximum) = SpellInfoRecord(packet).power_duration_range();
    if additional_minimum != 0 {
        for _ in 0..request.power {
            random.range(additional_minimum, additional_maximum);
        }
    }
    if gate == 0 {
        return Ok(vec![Event::Feedback(
            crate::feedback::GameFeedback::Warning { index: 87 },
        )]);
    }
    let target = request.target.ok_or("movement spell destination missing")?;
    let (mut events, defeated) = land_mover(
        MoveArena {
            map: &mut combat.field,
            positions: &mut combat.squad.pos,
            under_tiles: &mut combat.squad.char_under,
            cursor: &mut combat.battle_order_index,
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

/// What a moving spell writes, during or after a battle.
pub(crate) struct MoveArena<'a> {
    pub map: &'a mut crate::combat::CombatField,
    pub positions: &'a mut [(i16, i16)],
    pub under_tiles: &'a mut [i16],
    /// The battle-order cursor.
    pub cursor: &'a mut usize,
}

/// Moves the selected warrior to the target cell, after the duration draws. A
/// landing that kills leaves killbody to the caller.
///
/// Original: resolvespell
pub(crate) fn land_mover(
    arena: MoveArena<'_>,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    actor: usize,
    packet: &[u8],
    target: (i16, i16),
) -> Result<(Vec<Event>, bool), RuleError> {
    if SpellInfoRecord(packet).effect()? == 55 {
        warriors[actor].record_mut().set_movement(0);
        let maximum = warriors[actor].record().max_spells_attacks()?;
        warriors[actor].record_mut().set_spells_so_far(maximum);
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
    let cell = crate::combat::records::grid_index(target, arena.map.map_shift)?;
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
        events.push(Event::Feedback(crate::feedback::GameFeedback::Warning {
            index: 123,
        }));
        warriors[actor].record_mut().set_stamina(-10);
    } else {
        // A packet with repeated initiative steps the battle-order cursor back so
        // the caster acts again; a landing that kills does not.
        if packet[31] != 0 {
            *arena.cursor = usize::from((*arena.cursor as u16).wrapping_sub(1));
        }
        warriors[actor].record_mut().mark_been_attacked();
    }
    Ok((events, defeated))
}
