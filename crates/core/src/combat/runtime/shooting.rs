//! Warriors firing weapons.
use super::{finish_player_action, refresh_active_item_context, refusal, spell_identity};
use crate::combat::party_buffs;
use crate::combat::player_psi::CastRequest;
use crate::combat::shooting::Discard;
use crate::combat::spells::RayStep;
use crate::combat::spells::RayWalk;
use crate::combat::spells::SpellContext;
use crate::combat::spells::SpellTarget;
use crate::combat::spells::out_of_combat;
use crate::combat::spells::{FIRST_SPELL_DESCRIPTOR, effects, spell_descriptor};
use crate::combat::warrior_alias;
use crate::combat::{Combat, firearm, lifecycle};
use crate::{Event, RuleError, Session};
use centurions_content::Content;
use centurions_content::mission::OpeningMission;
use centurions_content::records::{ItemDefinition, SpellInfoRecord};

/// The active warrior fires at a target; a refusal shows its message or stays silent.
///
/// Original: combatchoice
pub fn player_shoot(
    state: &mut Session,
    content: &Content,
    target: i16,
) -> Result<Vec<Event>, RuleError> {
    match crate::combat::shooting::select(state, content, target) {
        Err(RuleError::Refused { feedback }) => Ok(if feedback.is_empty() {
            vec![Event::Feedback(crate::feedback::GameFeedback::Silent)]
        } else {
            feedback.into_iter().map(Event::Feedback).collect()
        }),
        result => result,
    }
}

/// The active warrior fires at once at a target, with the fire sound; an emptied weapon is dropped
/// after loading and the action ends.
///
/// Original: combatchoice
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
                vec![Event::Feedback(crate::feedback::GameFeedback::Silent)]
            }
            Err(RuleError::Refused { feedback }) => {
                feedback.into_iter().map(Event::Feedback).collect()
            }
            Err(error) => return Err(error),
        };
    // getup refreshes the new actor's weapon whatever the last actor held.
    if state.combat.as_ref().and_then(Combat::active_body) != Some(body) {
        refresh_active_item_context(state, content)?;
    }
    Ok(events)
}

/// Fires at one target; the action is not ended and an emptied weapon is not dropped.
pub fn player_shoot_action(
    state: &mut Session,
    content: &Content,
    target: i16,
) -> Result<Vec<Event>, RuleError> {
    player_shoot_targets_action(state, content, &[target], false, Discard::Never)
}

/// Fires at the given targets; `paid` says the rounds were already taken, and `discard` when an
/// emptied weapon is dropped.
pub(crate) fn player_shoot_targets_action(
    state: &mut Session,
    content: &Content,
    targets: &[i16],
    paid: bool,
    discard: Discard,
) -> Result<Vec<Event>, RuleError> {
    player_shoot_action_at(state, content, targets, paid, &[], discard)
}

/// Fires at a chosen location, which becomes the centre of the shot.
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

/// The fire key over chosen locations, already charged.
///
/// Original: combatchoice
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

/// Fires the prepared firearm at the targets or locations. The shot is admitted, the rounds are
/// taken, an emptied weapon may be dropped, the action costs two attacks and three movement, and
/// each target is flown at and resolved in turn.
///
/// Original: combatchoice
pub(super) fn player_shoot_action_at(
    state: &mut Session,
    content: &Content,
    targets: &[i16],
    paid: bool,
    centers: &[(i16, i16)],
    discard: Discard,
) -> Result<Vec<Event>, RuleError> {
    let mut prepared = crate::combat::shooting::prepare(state, content)?;
    admit_targets(state, content, targets, paid, centers, &mut prepared)?;
    let combat = state.combat.as_mut().unwrap();
    if !centers.is_empty() {
        combat.in_spell = 1;
    }
    combat.blank_round = 0;
    state.target_quantity = None;
    let body = combat.active_body().unwrap();
    let spell = load_firearm_shot(state, content, body, &prepared)?;
    if !paid {
        charge_rounds(&mut state.warriors[body as usize], &prepared, targets.len())?;
    }
    let mut events = vec![];
    // The fire key drops the emptied item before the charges and the cast.
    if discards(state, body, &prepared, targets, discard)? {
        crate::combat::shooting::drop_empty_weapon(state, content, body as usize, &mut events)?;
    }
    let warrior = &mut state.warriors[body as usize];
    let actions = warrior.record().attacks()?.wrapping_sub(2);
    let movement = warrior.record().movement()?.wrapping_sub(3).max(0);
    warrior.record_mut().set_attacks(actions);
    warrior.record_mut().set_movement(movement);
    if SpellInfoRecord(&prepared.packet).divides_shots_between_targets() {
        prepared.shots = 1;
    }
    let mission = crate::Session::land_in(
        content,
        state.mission.as_deref(),
        state.scenario_level,
        state.exploration.as_ref(),
    )
    .unwrap();
    let physical = SpellInfoRecord(&prepared.packet).is_direct_weapon_damage()?;
    // A warrior's shot does not turn: only the rotate button changes the rotation.
    if !physical && SpellInfoRecord(&prepared.packet).can_rotate() {
        state.random.bound(4);
    }
    let shot = Shot {
        content,
        mission,
        prepared: &prepared,
        body,
        spell,
        physical,
    };
    let mut indexed = vec![];
    let mut flights = vec![];
    for (index, &selected) in targets.iter().enumerate() {
        let combat = state.combat.as_mut().unwrap();
        combat.target_picks.enter(index, targets.len());
        if !combat.body_alive(&state.warriors, selected) {
            continue;
        }
        // The flight draws come before the area is built.
        let jitter = crate::combat::spells::flight_jitter(&prepared.packet, &mut state.random);
        let mask = target_mask(state, &shot, centers.get(index).copied(), selected)?;
        let combat = state.combat.as_mut().unwrap();
        let triple = SpellTarget::body(selected);
        flights.push((
            events.len(),
            crate::combat::spells::flight(
                combat,
                body,
                &triple,
                &prepared.packet,
                targets.len(),
                jitter,
            ),
        ));
        events.push(Event::Sound {
            resource_id: SpellInfoRecord(&prepared.packet).launch_sound()?,
        });
        if SpellInfoRecord(&prepared.packet).is_ray() {
            walk_ray(state, &shot, selected, &mut events, &mut flights)?;
        } else if !physical && SpellInfoRecord(&prepared.packet).is_party_duration() {
            party_durations(state, &shot, &mut events, &mut indexed)?;
        } else {
            resolve_target_bodies(state, &shot, &mask, &mut events)?;
        }
    }
    let combat = state.combat.as_mut().unwrap();
    crate::event::Flight::attach(&mut events, &flights);
    // The target marks clear after each target.
    combat.clear_track_tail();
    combat.target_picks.leave();
    // Effect 110 and the indexed durations touch nothing the later targets read.
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

/// The loaded shot every target of one fire action shares.
struct Shot<'a> {
    content: &'a Content,
    mission: &'a OpeningMission,
    prepared: &'a firearm::PreparedFirearm,
    body: i16,
    spell: i16,
    physical: bool,
}

/// The target-count ceiling, then admission of the location or of each body.
fn admit_targets(
    state: &mut Session,
    content: &Content,
    targets: &[i16],
    paid: bool,
    centers: &[(i16, i16)],
    prepared: &mut firearm::PreparedFirearm,
) -> Result<(), RuleError> {
    let ceiling = crate::combat::shooting::ceiling(state, content)?;
    if targets.is_empty()
        || targets.len()
            > if centers.is_empty() && SpellInfoRecord(&prepared.packet).requires_area_mask() {
                1
            } else {
                ceiling.max(1) as usize
            }
    {
        return Err(RuleError::Invalid("invalid firearm target count".into()));
    }
    // Chosen locations are not admitted again.
    if let [location] = centers
        && !paid
    {
        crate::combat::shooting::admit_location(state, content, *location, prepared)?;
    } else if centers.is_empty() {
        for &target in targets {
            crate::combat::shooting::admit(state, content, target, prepared, paid)?;
        }
    }
    if paid && centers.is_empty() && targets.len() == 8 {
        crate::combat::shooting::overwrite_with_eighth_body(&mut prepared.packet, targets[7]);
    }
    Ok(())
}

/// Loads the weapon and ammunition work globals and the attack packet, returning the
/// spell descriptor the shot resolves with.
fn load_firearm_shot(
    state: &mut Session,
    content: &Content,
    body: i16,
    prepared: &firearm::PreparedFirearm,
) -> Result<i16, RuleError> {
    let combat = state.combat.as_mut().unwrap();
    let warrior = &state.warriors[body as usize];
    let weapon_id = warrior.record().equipped_weapon()?;
    let weapon = &content.items[weapon_id.unsigned_abs() as usize];
    let ammunition = &content.items[warrior
        .record()
        .inventory_ammunition(prepared.slot)?
        .unsigned_abs() as usize];
    combat.weapon_traits.prepare(weapon, ammunition)?;
    state
        .globals
        .loaded_items
        .prepare_pair(weapon, ammunition)?;
    state.globals.loaded_items.item = Some(crate::ammunition::definition(content, weapon_id)?);
    state.globals.loaded_item_field32 = ItemDefinition(weapon).ammunition_caliber()?;
    state.globals.attack_count = prepared.shots;
    combat.spell_info = prepared.packet.clone();
    // A plain weapon shot keeps the previous spell identity.
    Ok(match ItemDefinition(weapon).spell()? {
        descriptor if descriptor >= FIRST_SPELL_DESCRIPTOR => {
            combat.cast_identity = spell_identity(descriptor);
            descriptor
        }
        _ => {
            let identity = combat.cast_identity;
            spell_descriptor(identity)
        }
    })
}

/// Takes every target's rounds, and their weight, from the loaded slot.
fn charge_rounds(
    warrior: &mut centurions_content::Warrior,
    prepared: &firearm::PreparedFirearm,
    targets: usize,
) -> Result<(), RuleError> {
    let rounds = prepared.rounds.wrapping_mul(targets as i16);
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
        .load()?
        .wrapping_sub(rounds.wrapping_mul(prepared.weight));
    warrior.record_mut().set_load(weight);
    Ok(())
}

/// Whether this action drops an emptied weapon before it fires.
fn discards(
    state: &Session,
    body: i16,
    prepared: &firearm::PreparedFirearm,
    targets: &[i16],
    discard: Discard,
) -> Result<bool, RuleError> {
    Ok(match discard {
        Discard::Never => false,
        Discard::Collected(flag) => flag,
        // An area shot drops the empty item in any mode; a body shot only for an
        // enemy in firing mode.
        Discard::AfterLoad => {
            (SpellInfoRecord(&prepared.packet).requires_area_mask()
                || (targets[0] >= 10 && state.warriors[body as usize].record().toggle()))
                && crate::combat::shooting::empties_discarded(state, body as usize)?
        }
    })
}

/// One target's mask: its body alone, or spelltargets' area around its center.
fn target_mask(
    state: &mut Session,
    shot: &Shot,
    center: Option<(i16, i16)>,
    selected: i16,
) -> Result<[i16; 110], RuleError> {
    let combat = state.combat.as_mut().unwrap();
    let packet = &shot.prepared.packet;
    let mut mask = [0_i16; 110];
    // A ray builds no area.
    if !SpellInfoRecord(packet).requires_area_mask() || SpellInfoRecord(packet).is_ray() {
        mask[selected as usize] = 1;
        return Ok(mask);
    }
    let center = center.unwrap_or_else(|| combat.body_position(selected));
    let target = SpellTarget::cell(
        center.0 + combat.field.map_shift.0 + 5,
        center.1 + combat.field.map_shift.1 + 5,
    );
    let resolved = crate::combat::spells::target_mask_rotated_with_origin(
        combat,
        &state.warriors,
        &shot.mission.areas,
        (&target, &target),
        &SpellContext {
            packet,
            id: shot.spell,
            count: shot.prepared.shots,
            caster: shot.body,
            initiative: combat.battle_order_index as i16,
            collision: false,
        },
        &combat.spell_queue,
        &mut state.random,
        (
            0,
            crate::combat::map_alias::session_map_aliases!(state, shot.mission),
        ),
    )?;
    combat.spell_queue = resolved.fields;
    Ok(resolved.mask)
}

/// Walks a ray from the shooter to the target body and resolves each crossed body
/// before the next step, physical or not; weapon fire only skips dead targets below
/// mode 2.
///
/// Original: cast
fn walk_ray(
    state: &mut Session,
    shot: &Shot,
    selected: i16,
    events: &mut Vec<Event>,
    flights: &mut [(usize, Option<crate::event::Flight>)],
) -> Result<(), RuleError> {
    if !crate::combat::spells::ray_walks(&shot.prepared.packet) {
        return Ok(());
    }
    let combat = state.combat.as_mut().unwrap();
    let steps = 128 + state.command_preferences.animation_delay;
    let mut ray = RayWalk::new(
        combat.body_pixel(shot.body)?,
        combat.body_pixel(selected)?,
        steps,
        shot.body as usize,
    );
    loop {
        let combat = state.combat.as_mut().unwrap();
        let Some(step) = ray.next(
            &combat.field.tiles,
            combat.field.map_shift,
            shot.mission,
            &shot.prepared.packet,
        )?
        else {
            break;
        };
        let RayStep::Body(crossed) = step else {
            events.push(Event::Sound {
                resource_id: -10082,
            });
            crate::combat::spells::stop_flight(flights, Some(ray.step()));
            break;
        };
        let mut mask = [0_i16; 110];
        mask[crossed] = 1;
        let crossing = events.len();
        if shot.physical {
            physical_bodies(
                combat,
                &mut state.warriors,
                (&mut state.random, &mut state.globals.health_context),
                (shot.body, shot.prepared),
                &mask,
                events,
            )?;
        } else {
            events.extend(resolve_psi_mask(state, shot, &mask)?);
        }
        crate::combat::spells::mark_step(&mut events[crossing..], ray.step());
    }
    Ok(())
}

/// A global (mode 7) spell writes the party durations while in_spell is set, then
/// clears in_spell.
///
/// Original: resolvespell
fn party_durations(
    state: &mut Session,
    shot: &Shot,
    events: &mut Vec<Event>,
    indexed: &mut Vec<(i16, i16)>,
) -> Result<(), RuleError> {
    let packet = &shot.prepared.packet;
    let duration = out_of_combat::duration(packet, shot.prepared.shots, &mut state.random);
    match SpellInfoRecord(packet).effect()?.wrapping_abs() {
        effects::PARTY_DURATION_COUNT => {
            events.push(Event::Sound { resource_id: 601 });
            party_buffs::extend_from_count(&mut state.party_condition, shot.prepared.shots)?;
        }
        effect @ 0..=9 => party_buffs::extend(&mut state.party_condition, effect, duration)?,
        effect => indexed.push((effect, duration)),
    }
    state.combat.as_mut().unwrap().in_spell = 0;
    Ok(())
}

/// Sets in_spell once the targets are marked, then resolves each body as weapon
/// damage or as the packet's psi effect.
///
/// Original: spelltargets, resolvespell
fn resolve_target_bodies(
    state: &mut Session,
    shot: &Shot,
    mask: &[i16; 110],
    events: &mut Vec<Event>,
) -> Result<(), RuleError> {
    let packet = &shot.prepared.packet;
    let body_loop = events.len();
    let combat = state.combat.as_mut().unwrap();
    if !SpellInfoRecord(packet).is_ray() {
        combat.in_spell = 1;
    }
    if shot.physical {
        physical_bodies(
            combat,
            &mut state.warriors,
            (&mut state.random, &mut state.globals.health_context),
            (shot.body, shot.prepared),
            mask,
            events,
        )?;
    } else {
        events.extend(resolve_psi_mask(state, shot, mask)?);
    }
    crate::event::SpellArea::attach(events, body_loop, mask, packet);
    if !SpellInfoRecord(packet).is_ray() {
        state.combat.as_mut().unwrap().in_spell = 0;
    }
    Ok(())
}

/// resolvespell over a psi packet's mask, then the alias writes it left pending.
fn resolve_psi_mask(
    state: &mut Session,
    shot: &Shot,
    mask: &[i16; 110],
) -> Result<Vec<Event>, RuleError> {
    let combat = state.combat.as_mut().unwrap();
    let events = crate::combat::player_psi::resolve_mask(
        combat,
        &mut state.warriors,
        shot.mission,
        &mut state.random,
        shot.body as usize,
        CastRequest {
            rotation: 0,
            spell: shot.spell,
            power: shot.prepared.shots,
            target: None,
        },
        &shot.prepared.packet,
        mask,
        (
            crate::combat::spells::loaded_item_restriction(&state.globals.loaded_items)?,
            warrior_alias::backup_owners!(state),
            Some((&shot.content.items, &mut state.globals.loaded_items)),
        ),
    )?;
    warrior_alias::commit_pending!(state, combat)?;
    Ok(events)
}

/// Weapon damage for a physical packet: the duration draw, then each selected body
/// in order.
///
/// Original: resolvespell
fn physical_bodies(
    combat: &mut Combat,
    warriors: &mut [centurions_content::Warrior],
    (random, health): (
        &mut crate::RandomState,
        &mut crate::story::health::HealthContext,
    ),
    (body, prepared): (i16, &firearm::PreparedFirearm),
    mask: &[i16; 110],
    events: &mut Vec<Event>,
) -> Result<(), RuleError> {
    // The duration is drawn before any impact, even though it is not shown.
    let (minimum, maximum) = SpellInfoRecord(&prepared.packet).duration_range();
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
                combat.difficulty,
            )?
        } else {
            firearm::resolve_after_duration(
                &mut warriors[body as usize],
                &mut combat.roster.records[(target - 10) as usize],
                prepared,
                random,
                combat.difficulty,
            )?
        };
        combat.neighbors.departure = 0;
        if result.killed {
            lifecycle::kill_body(combat, warriors, target)?;
            // killbody remembers every body it kills.
            health.last_body = target;
        }
        events.extend(
            result
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
        // A killed monster plays its death cue whatever the weapon.
        if result.killed && target >= 10 {
            let sound = combat.roster.records[(target - 10) as usize]
                .record()
                .death_sound()?;
            if sound != 0 {
                events.push(Event::Sound { resource_id: sound });
            }
        }
        if result.damage != 0
            && !result.killed
            && SpellInfoRecord(&prepared.packet).plays_impact_when_wounded()
        {
            events.push(Event::Sound {
                resource_id: SpellInfoRecord(&prepared.packet).impact_sound()?,
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
