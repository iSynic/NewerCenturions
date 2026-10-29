//! Shooting in the original combat execution order.
use super::{
    alive, finish_player_action, position, refresh_active_item_context, refusal, spell_identity,
};
use crate::combat::party_buffs;
use crate::combat::shooting::Discard;
use crate::combat::{Combat, firearm, lifecycle};
use crate::{Event, RuleError, Session};
use centurions_content::Content;
use centurions_content::records::{AttackPacket, ItemDefinition};

pub fn player_shoot(
    state: &mut Session,
    content: &Content,
    target: i16,
) -> Result<Vec<Event>, RuleError> {
    match crate::combat::shooting::select(state, content, target) {
        Err(RuleError::Refused { feedback }) => Ok(if feedback.is_empty() {
            vec![Event::Feedback(crate::feedback::NativeFeedback::Silent)]
        } else {
            feedback.into_iter().map(Event::Feedback).collect()
        }),
        result => result,
    }
}

pub fn player_shoot_immediate(
    state: &mut Session,
    content: &Content,
    target: i16,
) -> Result<Vec<Event>, RuleError> {
    let body = state
        .combat
        .as_ref()
        .and_then(Combat::active_body)
        .filter(|body| (0..6).contains(body))
        .ok_or_else(|| RuleError::Invalid("active warrior missing".into()))?;
    let events =
        match player_shoot_targets_action(state, content, &[target], false, Discard::AfterLoad) {
            Ok(mut events) => {
                events.insert(0, Event::Sound { resource_id: 145 });
                events.extend(finish_player_action(state, content, body)?);
                events
            }
            Err(RuleError::Refused { feedback }) if feedback.is_empty() => {
                vec![Event::Feedback(crate::feedback::NativeFeedback::Silent)]
            }
            Err(RuleError::Refused { feedback }) => {
                feedback.into_iter().map(Event::Feedback).collect()
            }
            Err(error) => return Err(error),
        };
    // Getup's combatupdate2 refreshes the new actor independently of the old weapon.
    if state.combat.as_ref().and_then(Combat::active_body) != Some(body) {
        refresh_active_item_context(state, content)?;
    }
    Ok(events)
}

pub fn player_shoot_action(
    state: &mut Session,
    content: &Content,
    target: i16,
) -> Result<Vec<Event>, RuleError> {
    player_shoot_targets_action(state, content, &[target], false, Discard::Never)
}

pub(crate) fn player_shoot_targets_action(
    state: &mut Session,
    content: &Content,
    targets: &[i16],
    paid: bool,
    discard: Discard,
) -> Result<Vec<Event>, RuleError> {
    player_shoot_action_at(state, content, targets, paid, &[], discard)
}

pub(crate) fn player_shoot_at_action(
    state: &mut Session,
    content: &Content,
    center: (i16, i16),
) -> Result<Vec<Event>, RuleError> {
    let body = state
        .combat
        .as_ref()
        .and_then(Combat::active_body)
        .ok_or("active warrior missing")?;
    player_shoot_action_at(
        state,
        content,
        &[body],
        false,
        &[center],
        Discard::AfterLoad,
    )
}

/// The fire key over collected kind-3 triples (combatchoice 11ae08), already charged.
pub(crate) fn player_shoot_locations_action(
    state: &mut Session,
    content: &Content,
    centers: &[(i16, i16)],
    discard: Discard,
) -> Result<Vec<Event>, RuleError> {
    let body = state
        .combat
        .as_ref()
        .and_then(Combat::active_body)
        .ok_or("active warrior missing")?;
    player_shoot_action_at(
        state,
        content,
        &vec![body; centers.len()],
        true,
        centers,
        discard,
    )
}

pub(super) fn player_shoot_action_at(
    state: &mut Session,
    content: &Content,
    targets: &[i16],
    paid: bool,
    centers: &[(i16, i16)],
    discard: Discard,
) -> Result<Vec<Event>, RuleError> {
    let mut prepared = crate::combat::shooting::prepare(state, content)?;
    let ceiling = crate::combat::shooting::ceiling(state, content)?;
    if targets.is_empty()
        || targets.len()
            > if centers.is_empty() && AttackPacket(&prepared.packet).requires_area_mask() {
                1
            } else {
                ceiling.max(1) as usize
            }
    {
        return Err(RuleError::Invalid("invalid firearm target count".into()));
    }
    // cast does not admit collected location triples again.
    if let [location] = centers
        && !paid
    {
        crate::combat::shooting::admit_location(state, content, *location, &prepared)?;
    } else if centers.is_empty() {
        for &target in targets {
            crate::combat::shooting::admit(state, content, target, &prepared, paid)?;
        }
    }
    if paid && centers.is_empty() && targets.len() == 8 {
        crate::combat::shooting::overwrite_with_eighth_body(&mut prepared.packet, targets[7]);
    }
    let c = state.combat.as_mut().unwrap();
    if !centers.is_empty() {
        c.ready = 1;
    }
    c.inactive_rounds = 0;
    state.target_quantity = None;
    let body = c.active_body().unwrap();
    let warrior = &mut state.warriors[body as usize];
    let weapon_id = warrior.record().equipped_weapon()?;
    let weapon = &content.items[weapon_id.unsigned_abs() as usize];
    let ammunition = &content.items[warrior
        .record()
        .inventory_ammunition(prepared.slot)?
        .unsigned_abs() as usize];
    c.weapon_work.prepare(weapon, ammunition)?;
    state
        .native_work
        .ammunition_work
        .prepare_pair(weapon, ammunition)?;
    state.native_work.ammunition_work.item =
        Some(crate::ammunition::definition(content, weapon_id)?);
    state.native_work.loaded_item_field32 = ItemDefinition(weapon).retained_field32()?;
    state.native_work.attack_count = prepared.shots;
    c.attack_packet = prepared.packet.clone();
    // A fallback descriptor skips loadspell2, leaving the previous spell identity.
    let spell = match ItemDefinition(weapon).spell()? {
        descriptor if descriptor >= 1101 => {
            c.attack_identity = spell_identity(descriptor);
            descriptor
        }
        _ => {
            let identity = c.attack_identity;
            (identity[0] + 1) * 1000 + (identity[1] + 1) * 100 + identity[2] + 1
        }
    };
    if !paid {
        let rounds = prepared.rounds.wrapping_mul(targets.len() as i16);
        let slot = prepared.slot;
        if warrior.record().inventory_quantity(slot)? < rounds {
            return Err(refusal(crate::feedback::CombatRefusal::NotEnoughAmmo));
        }
        let loaded = warrior
            .record()
            .inventory_quantity(slot)?
            .wrapping_sub(rounds);
        warrior.record_mut().set_inventory_quantity(slot, loaded);
        let weight = warrior
            .record()
            .carried_weight()?
            .wrapping_sub(rounds.wrapping_mul(prepared.weight));
        warrior.record_mut().set_carried_weight(weight);
    }
    let mut events = vec![];
    let discard = match discard {
        Discard::Never => false,
        Discard::Collected(flag) => flag,
        // An area packet takes the location arm, which tests in any mode (11b218); the
        // body arm tests only enemies in firing mode (11b41a..11b482).
        Discard::AfterLoad => {
            (AttackPacket(&prepared.packet).requires_area_mask()
                || (targets[0] >= 10 && warrior.record().firing_mode()))
                && crate::combat::shooting::empties_discarded(state, body as usize)?
        }
    };
    // The fire key drops the emptied item before its charges and cast (11ae08..11af6c).
    if discard {
        crate::combat::shooting::drop_empty_weapon(state, content, body as usize, &mut events)?;
    }
    let c = state.combat.as_mut().unwrap();
    let warrior = &mut state.warriors[body as usize];
    let actions = warrior.record().action_points()?.wrapping_sub(2);
    let movement = warrior.record().movement_points()?.wrapping_sub(3).max(0);
    warrior.record_mut().set_action_points(actions);
    warrior.record_mut().set_movement_points(movement);
    if AttackPacket(&prepared.packet).divides_shots_between_targets() {
        prepared.shots = 1;
    }
    let mission = crate::Session::land_in(
        content,
        state.mission.as_deref(),
        state.scenario_level,
        state.exploration.as_ref(),
    )
    .unwrap();
    let physical = AttackPacket(&prepared.packet).is_direct_weapon_damage()?;
    // This draw does not write WORD44331a: combatchoice resets the player rotation to 0
    // and only its rotate button changes it.
    if !physical && AttackPacket(&prepared.packet).has_randomized_animation() {
        state.random.bound(4);
    }
    let mut indexed = vec![];
    let mut flights = vec![];
    for (index, &selected) in targets.iter().enumerate() {
        // cast's loop index WORD442bd4 over combatchoice's WORD4435c0 triples.
        c.target_work.enter(index, targets.len());
        if !alive(c, &state.warriors, selected) {
            continue;
        }
        // cast's flight draws come before spelltargets builds the area (10e63a).
        let jitter = crate::combat::spells::flight_jitter(&prepared.packet, &mut state.random);
        let mut mask = [0_i16; 110];
        // A ray never reaches spelltargets (0x10e61c), so it builds no area.
        if !AttackPacket(&prepared.packet).requires_area_mask()
            || AttackPacket(&prepared.packet).targeting_mode() == 6
        {
            mask[selected as usize] = 1;
        } else {
            let center = centers
                .get(index)
                .copied()
                .unwrap_or_else(|| position(c, selected));
            let target = crate::combat::spells::SpellTarget {
                first: center.0 + c.map.shift.0 + 5,
                second: center.1 + c.map.shift.1 + 5,
                kind: 3,
            };
            let resolved = crate::combat::spells::target_mask_rotated_with_origin(
                c,
                &state.warriors,
                &mission.areas,
                (&target, &target),
                &crate::combat::spells::SpellContext {
                    packet: &prepared.packet,
                    id: spell,
                    count: prepared.shots,
                    caster: body,
                    initiative: c.cursor as i16,
                    collision: false,
                },
                &c.spell_fields,
                &mut state.random,
                (
                    0,
                    crate::combat::map_alias::session_map_aliases!(state, mission),
                ),
            )?;
            mask = resolved.mask;
            c.spell_fields = resolved.fields;
        }
        let triple = crate::combat::spells::SpellTarget {
            first: selected,
            second: 0,
            kind: if selected < 10 { 1 } else { 2 },
        };
        flights.push((
            events.len(),
            crate::combat::spells::flight(
                c,
                body,
                &triple,
                &prepared.packet,
                targets.len(),
                jitter,
            ),
        ));
        events.push(Event::Sound {
            resource_id: AttackPacket(&prepared.packet).launch_sound()?,
        });
        if AttackPacket(&prepared.packet).targeting_mode() == 6 {
            if !crate::combat::spells::ray_walks(&prepared.packet) {
                continue;
            }
            // cast10d4fe walks a ray from the shooter to the target body's pixel and
            // resolves each crossed body before the next step, physical or not: its
            // category-9 tests (10d890, 10d96a) only skip dead targets below mode 2.
            let pixel = |c: &Combat, body: i16| -> Result<(i16, i16), RuleError> {
                Ok(if body < 10 {
                    crate::combat::spells::body_pixel(c.squad.positions[body as usize], None)
                } else {
                    let index = (body - 10) as usize;
                    let size = c.enemies.records[index].record().size()?.wrapping_abs();
                    crate::combat::spells::body_pixel(c.enemies.positions[index], Some(size))
                })
            };
            let steps = 128 + state.command_preferences.animation_delay;
            let mut ray = crate::combat::spells::RayWalk::new(
                pixel(c, body)?,
                pixel(c, selected)?,
                steps,
                body as usize,
            );
            while let Some(step) = ray.next(&c.map.tiles, c.map.shift, mission, &prepared.packet)? {
                let crate::combat::spells::RayStep::Body(crossed) = step else {
                    events.push(Event::Sound {
                        resource_id: -10082,
                    });
                    crate::combat::spells::stop_flight(&mut flights, Some(ray.step()));
                    break;
                };
                let mut mask = [0_i16; 110];
                mask[crossed] = 1;
                let crossing = events.len();
                if physical {
                    physical_bodies(
                        c,
                        &mut state.warriors,
                        (&mut state.random, &mut state.native_work.health_context),
                        (body, &prepared),
                        &mask,
                        &mut events,
                    )?;
                    crate::combat::spells::mark_step(&mut events[crossing..], ray.step());
                    continue;
                }
                events.extend(crate::combat::player_psi::resolve_mask(
                    c,
                    &mut state.warriors,
                    mission,
                    &mut state.random,
                    body as usize,
                    crate::combat::player_psi::CastRequest {
                        rotation: 0,
                        spell,
                        power: prepared.shots,
                        target: None,
                    },
                    &prepared.packet,
                    &mask,
                    (
                        crate::combat::spells::retained_item_alias(
                            &state.native_work.ammunition_work,
                        )?,
                        crate::combat::warrior_alias::Owners::new(
                            None,
                            &state.native_work.shop_work,
                            state
                                .exploration
                                .as_ref()
                                .map(|position| position.event_headers.as_slice()),
                        )
                        .with_backup(&state.native_work.warrior_table),
                        Some((&content.items, &mut state.native_work.ammunition_work)),
                    ),
                )?);
                crate::combat::spells::mark_step(&mut events[crossing..], ray.step());
                crate::combat::warrior_alias::commit_pending!(state, c)?;
            }
            continue;
        }
        if !physical && AttackPacket(&prepared.packet).targeting_mode() == 7 {
            // resolvespell's mode-7 arm (15108c) writes the party durations, after
            // spelltargets arms Ready, and clears Ready again.
            let packet = &prepared.packet;
            let duration = crate::combat::spells::detached::duration(
                packet,
                prepared.shots,
                &mut state.random,
            );
            match AttackPacket(packet).effect()?.wrapping_abs() {
                50 => {
                    events.push(Event::Sound { resource_id: 601 });
                    party_buffs::extend_from_count(&mut state.party_buffs, prepared.shots)?;
                }
                effect @ 0..=9 => party_buffs::extend(&mut state.party_buffs, effect, duration)?,
                effect => indexed.push((effect, duration)),
            }
            c.ready = 0;
            continue;
        }
        let body_loop = events.len();
        if !physical {
            if AttackPacket(&prepared.packet).targeting_mode() != 6 {
                c.ready = 1;
            }
            events.extend(crate::combat::player_psi::resolve_mask(
                c,
                &mut state.warriors,
                mission,
                &mut state.random,
                body as usize,
                crate::combat::player_psi::CastRequest {
                    rotation: 0,
                    spell,
                    power: prepared.shots,
                    target: None,
                },
                &prepared.packet,
                &mask,
                (
                    crate::combat::spells::retained_item_alias(&state.native_work.ammunition_work)?,
                    crate::combat::warrior_alias::Owners::new(
                        None,
                        &state.native_work.shop_work,
                        state
                            .exploration
                            .as_ref()
                            .map(|position| position.event_headers.as_slice()),
                    )
                    .with_backup(&state.native_work.warrior_table),
                    Some((&content.items, &mut state.native_work.ammunition_work)),
                ),
            )?);
            crate::combat::warrior_alias::commit_pending!(state, c)?;
            crate::event::SpellArea::attach(&mut events, body_loop, &mask, &prepared.packet);
            if AttackPacket(&prepared.packet).targeting_mode() != 6 {
                c.ready = 0;
            }
            continue;
        }
        // spelltargets15d13c arms Ready after mask selection, before resolvespell.
        if AttackPacket(&prepared.packet).targeting_mode() != 6 {
            c.ready = 1;
        }
        physical_bodies(
            c,
            &mut state.warriors,
            (&mut state.random, &mut state.native_work.health_context),
            (body, &prepared),
            &mask,
            &mut events,
        )?;
        crate::event::SpellArea::attach(&mut events, body_loop, &mask, &prepared.packet);
        if AttackPacket(&prepared.packet).targeting_mode() != 6 {
            c.ready = 0;
        }
    }
    crate::event::Flight::attach(&mut events, &flights);
    // cast10e642 clears the whole mask after each target and cleartarget follows.
    c.clear_target_tail();
    c.target_work.leave();
    // Effect 110's tail and the indexed WORDs touch nothing the later targets read.
    for (effect, duration) in indexed {
        if effect == 110 {
            party_buffs::extend_scenario_tail(state, duration)?;
        } else if !party_buffs::extend_indexed(state, effect, duration)? {
            return Err(RuleError::Unsupported {
                action: "combat.firearm",
                dependency: "indexed global firearm duration outside retained owners",
            });
        }
    }
    Ok(events)
}

/// resolvespell's per-body weapon damage for a physical packet: the duration draw,
/// then each selected body in order.
fn physical_bodies(
    c: &mut Combat,
    warriors: &mut [centurions_content::Warrior],
    (random, health): (
        &mut crate::RandomState,
        &mut crate::story_health::HealthContext,
    ),
    (body, prepared): (i16, &firearm::PreparedFirearm),
    mask: &[i16; 110],
    events: &mut Vec<Event>,
) -> Result<(), RuleError> {
    // Native duration draw precedes every impact, even when the duration
    // itself is not rendered by this caller.
    let (minimum, maximum) = AttackPacket(&prepared.packet).duration_bounds();
    random.range(minimum, maximum);
    for (target, flag) in mask.iter().enumerate() {
        if *flag <= 0 {
            continue;
        }
        let target = target as i16;
        let result = if target < 10 {
            let attacker = warriors[body as usize].clone();
            firearm::resolve_friendly_after_duration(
                &attacker,
                &mut warriors[target as usize],
                prepared,
                random,
                c.difficulty,
            )?
        } else {
            firearm::resolve_after_duration(
                &mut warriors[body as usize],
                &mut c.enemies.records[(target - 10) as usize],
                prepared,
                random,
                c.difficulty,
            )?
        };
        c.neighbor_work.departure = 0;
        if result.killed {
            lifecycle::kill_body(c, warriors, target)?;
            // killbody12e506 retains every direct defeated body before returning.
            health.last_body = target;
        }
        events.extend(
            result
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
        // showresults158ee0 reads the defeated NPC's cue for every weapon.
        if result.killed && target >= 10 {
            let sound = c.enemies.records[(target - 10) as usize]
                .record()
                .death_sound()?;
            if sound != 0 {
                events.push(Event::Sound { resource_id: sound });
            }
        }
        if result.damage != 0
            && !result.killed
            && AttackPacket(&prepared.packet).plays_impact_when_wounded()
        {
            events.push(Event::Sound {
                resource_id: AttackPacket(&prepared.packet).impact_sound()?,
            });
        }
        events.push(Event::AttackResolved {
            attacker: body,
            target,
            damage: result.damage,
            defeated: result.killed,
            icon: crate::event::PresentationIcon(result.icon),
            flight: Default::default(),
        });
    }
    Ok(())
}
