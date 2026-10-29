//! Enemy fire in the original combat execution order.
use super::{
    EnemyWork, FireOutcome, alive, enemy_spell_continue, refresh_enemy_item_alias,
    resolve_spell_mask, spell_identity,
};
use crate::combat::{Combat, firearm, lifecycle};
use crate::{Event, RandomState, RuleError};
use centurions_content::{
    Warrior,
    records::{AttackPacket, ItemDefinition},
};

pub(super) fn enemy_fire(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items_context: (&[Vec<u8>], &mut EnemyWork),
    body: i16,
    events: &mut Vec<Event>,
) -> Result<FireOutcome, RuleError> {
    let (items, work) = items_context;
    let e = (body - 10) as usize;
    let weapon_id = c.enemies.records[e].record().primary_weapon()?;
    // loadweapon 130d38 caches the definition at abs(id).
    let weapon = items
        .get(weapon_id.unsigned_abs() as usize)
        .ok_or(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "weapon content missing",
        })?;
    let attacks = &mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat content missing".into()))?
        .attacks;
    let item_context = work.item_context;
    let ammo_id = crate::combat::enemy_weapons::ammo_id(&c.enemies.records[e], item_context)?;
    let ammo = items
        .get(ammo_id.unsigned_abs() as usize)
        .ok_or_else(|| RuleError::Invalid("enemy ammunition outside content".into()))?;
    work.buffers.prepare_pair(weapon, ammo)?;
    let mut prepared = if weapon_id == 0 {
        let packet = c.attack_packet.clone();
        c.weapon_work.load_pair(weapon, ammo)?;
        c.enemies.records[e].record_mut().set_selected_item(0);
        firearm::PreparedFirearm {
            sound: AttackPacket(&packet).impact_sound()?,
            packet,
            slot: 0,
            shots: i16::from(ItemDefinition(weapon).selected_attack_count()),
            rounds: ItemDefinition(weapon).rounds_per_shot()?,
            weight: 0,
            armor_filter: ItemDefinition(weapon).bypasses_hit_roll()?,
        }
    } else if ItemDefinition(weapon).spell()? >= 1101 {
        crate::combat::enemy_weapons::prepare(&mut c.enemies.records[e], weapon, ammo, attacks)?
    } else {
        // combat 11477c calls prepattack for the NPC, whose fallback below 1101 reads
        // the enhancement BYTE 448be4 + D3 * 22 with D3 left by combat's loops.
        return Err(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "enemy fallback descriptor reads combat's loop register D3 as its row",
        });
    };
    if weapon_id != 0 {
        c.weapon_work.prepare(weapon, ammo)?;
        c.attack_packet = prepared.packet.clone();
        c.attack_identity = spell_identity(ItemDefinition(weapon).spell()?);
        work.attack_count = prepared.shots;
    }
    prepared.armor_filter = c.weapon_work.armor_filter();
    prepared.sound = c.weapon_work.sound();
    let selection = if weapon_id == 0 || AttackPacket(&prepared.packet).defense_category() != 9 {
        crate::combat::spells::select_retained_targets(
            c,
            w,
            mission,
            &mission.areas,
            e,
            &prepared,
            random,
        )?
    } else {
        crate::combat::spells::select_firearm_targets_beside(
            c,
            w,
            (mission, work.land.as_ref()),
            e,
            &prepared,
            random,
        )?
    };
    work.attack_count = selection.count;
    // combat 114b7e and 115110 call cleartarget around the decision's triples and
    // the shots; no movement or summon arm reads them in between.
    c.target_work.clear();
    if !selection.cast {
        return Ok(FireOutcome::Failed);
    }
    c.inactive_rounds = 0;
    c.enemies.records[e] = selection.caster;
    if work.display_writers {
        refresh_enemy_item_alias(c, e, items, work)?;
    }
    let weapon_result = AttackPacket(&prepared.packet).plays_impact_when_wounded();
    let physical = AttackPacket(&prepared.packet).is_direct_weapon_damage()?;
    work.attack_count = if weapon_result {
        1
    } else {
        work.attack_count.max(1)
    };
    prepared.shots = work.attack_count;
    let id = if weapon_id == 0 {
        let identity = c.attack_identity;
        (identity[0] + 1) * 1000 + (identity[1] + 1) * 100 + identity[2] + 1
    } else {
        ItemDefinition(weapon).spell()?
    };
    // combat1150a4 stores Rand(4)-1 in WORD44331a for a byte-7 packet.
    let rotation = if AttackPacket(&prepared.packet).has_randomized_animation() {
        (random.bound(4) - 1) as u8
    } else {
        0
    };
    let mut origin = selection.targets.first().cloned();
    let count = selection.targets.len();
    let mut flights = vec![];
    // combat 11509e calls showspellinfo before cast.
    let info = super::enemy_spells::spell_info(c, &prepared.packet, work);
    let start = events.len();
    for target in selection.targets {
        if !AttackPacket(&prepared.packet).requires_area_mask() {
            let target_body = target.first;
            if !alive(c, w, target_body) {
                continue;
            }
        }
        let jitter = crate::combat::spells::flight_jitter(&prepared.packet, random);
        flights.push((
            events.len(),
            crate::combat::spells::flight(c, body, &target, &prepared.packet, count, jitter),
        ));
        events.push(Event::Sound {
            resource_id: AttackPacket(&prepared.packet).launch_sound()?,
        });
        // The NPC decision fires through cast10d4fe, whose ray walks for a mode-6 packet;
        // its category-9 tests (10d890, 10d96a) only skip dead targets below mode 2.
        if physical && AttackPacket(&prepared.packet).targeting_mode() == 6 {
            let stop = super::enemy_spells::enemy_ray(
                c,
                w,
                (mission, random),
                (e, &target),
                (&prepared.packet, work.animation_delay),
                &mut |c, w, random, mask| {
                    let mut events = vec![];
                    physical_bodies(c, w, random, (e, &prepared), mask, &mut events)?;
                    Ok(events)
                },
                events,
            )?;
            crate::combat::spells::stop_flight(&mut flights, stop);
            continue;
        }
        if AttackPacket(&prepared.packet).targeting_mode() == 6 {
            let snapshot = work.alias_snapshot();
            let memory = crate::combat::spells::RestrictionMemoryView::ordinary_startup(
                &mission.item_cache,
                None,
            )
            .with_tables(mission)
            .with_definitions(items)
            .with_selected_warrior(work.selected_warrior as i16)
            .with_alias_owners(snapshot.owners())
            .with_live_caster(c, w);
            let invocation = (id, &prepared.packet[..], work.attack_count);
            let stop = super::enemy_spells::enemy_ray(
                c,
                w,
                (mission, random),
                (e, &target),
                (&prepared.packet, work.animation_delay),
                &mut |c, w, random, mask| {
                    resolve_spell_mask(c, w, random, invocation, mask, body, memory)
                },
                events,
            )?;
            crate::combat::spells::stop_flight(&mut flights, stop);
            work.take_alias_writes(c)?;
            continue;
        }
        let context = crate::combat::spells::SpellContext {
            packet: &prepared.packet,
            id,
            count: work.attack_count,
            caster: body,
            initiative: c.cursor as i16,
            collision: false,
        };
        let mask = crate::combat::spells::target_mask_rotated_with_origin(
            c,
            w,
            &mission.areas,
            (&target, origin.as_ref().unwrap()),
            &context,
            &c.spell_fields,
            random,
            (
                rotation,
                crate::combat::map_alias::land_aliases(mission, work.land.as_ref()),
            ),
        )?;
        if let Some(origin) = origin.as_mut() {
            origin.first = -1;
            origin.second = -1;
        }
        c.spell_fields = mask.fields;
        if !physical {
            let snapshot = work.alias_snapshot();
            let body_loop = events.len();
            events.extend(resolve_spell_mask(
                c,
                w,
                random,
                (id, &prepared.packet, work.attack_count),
                &mask.mask,
                body,
                crate::combat::spells::RestrictionMemoryView::ordinary_startup(
                    &mission.item_cache,
                    None,
                )
                .with_tables(mission)
                .with_definitions(items)
                .with_selected_warrior(work.selected_warrior as i16)
                .with_alias_owners(snapshot.owners())
                .with_live_caster(c, w),
            )?);
            crate::event::SpellArea::attach(events, body_loop, &mask.mask, &prepared.packet);
            work.take_alias_writes(c)?;
            continue;
        }
        let body_loop = events.len();
        physical_bodies(c, w, random, (e, &prepared), &mask.mask, events)?;
        crate::event::SpellArea::attach(events, body_loop, &mask.mask, &prepared.packet);
    }
    crate::event::Flight::attach(events, &flights);
    crate::event::SpellInfo::attach(events, start, info);
    if weapon_result {
        Ok(FireOutcome::Physical)
    } else {
        work.spell_attempt = work.spell_attempt.wrapping_add(1);
        enemy_spell_continue(c, w, mission, random, (items, work), body, events)?;
        Ok(FireOutcome::Spell)
    }
}

/// resolvespell's per-body weapon damage for a physical packet: the duration draw,
/// then each selected body in order.
pub(super) fn physical_bodies(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    (e, prepared): (usize, &firearm::PreparedFirearm),
    mask: &[i16; 110],
    events: &mut Vec<Event>,
) -> Result<(), RuleError> {
    let body = e as i16 + 10;
    let (minimum, maximum) = AttackPacket(&prepared.packet).duration_bounds();
    random.range(minimum, maximum);
    for (target, flag) in mask.iter().enumerate() {
        if *flag <= 0 {
            continue;
        }
        let result = if target < 6 {
            firearm::resolve_enemy_after_duration(
                &mut w[target],
                &mut c.enemies.records[e],
                prepared,
                random,
                c.difficulty,
            )?
        } else if target >= 10 {
            let caster = c.enemies.records[e].clone();
            crate::combat::spells::resolve_enemy_firearm_body_after_duration(
                &mut c.enemies.records[target - 10],
                &caster,
                prepared,
                c.difficulty,
                random,
            )?
        } else {
            return Err(RuleError::Invalid("firearm selected empty body".into()));
        };
        if result.killed {
            lifecycle::kill_body(c, w, target as i16)?;
        }
        events.extend(
            result
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
        if result.damage != 0
            && !result.killed
            && AttackPacket(&prepared.packet).plays_impact_when_wounded()
        {
            events.push(Event::Sound {
                resource_id: crate::combat::ally_turns::result_audio(
                    &c.enemies.records[e],
                    0,
                    result.damage,
                    AttackPacket(&prepared.packet).impact_sound()?,
                )?,
            });
        }
        events.push(Event::AttackResolved {
            attacker: body,
            target: target as i16,
            damage: result.damage,
            defeated: result.killed,
            icon: crate::event::PresentationIcon(result.icon),
            flight: Default::default(),
        });
    }
    Ok(())
}
