//! Monsters firing weapons.
use super::{
    EnemyGlobals, FireOutcome, enemy_spell_continue, refresh_enemy_item_alias, resolve_spell_mask,
    spell_identity,
};
use crate::combat::enemy_weapons;
use crate::combat::spells::RestrictionMemoryView;
use crate::combat::spells::SpellContext;
use crate::combat::spells::{FIRST_SPELL_DESCRIPTOR, spell_descriptor};
use crate::combat::warrior_alias::Snapshot;
use crate::combat::{Combat, firearm, lifecycle};
use crate::{Event, RandomState, RuleError};
use centurions_content::{
    Warrior,
    records::{ItemDefinition, SpellInfoRecord},
};

/// A monster fires its weapon. Targets are selected as for a spell packet, except that a firearm
/// packet of defense category 9 selects them beside the shooter; the outcome says whether the shot
/// failed, struck physically or cast a spell.
///
/// Original: combat
pub(super) fn enemy_fire(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items_context: (&[Vec<u8>], &mut EnemyGlobals),
    body: i16,
    events: &mut Vec<Event>,
) -> Result<FireOutcome, RuleError> {
    let (items, globals) = items_context;
    let monster_index = (body - 10) as usize;
    let (weapon_id, weapon, mut prepared) =
        load_firearm(combat, mission, items, globals, monster_index)?;
    let selection = if weapon_id == 0 || SpellInfoRecord(&prepared.packet).defense_category() != 9 {
        crate::combat::spells::select_spell_info_targets(
            combat,
            warriors,
            mission,
            &mission.areas,
            monster_index,
            &prepared,
            random,
        )?
    } else {
        crate::combat::spells::select_firearm_targets_beside(
            combat,
            warriors,
            (mission, globals.land.as_ref()),
            monster_index,
            &prepared,
            random,
        )?
    };
    globals.attack_count = selection.count;
    // The target list is cleared before the decision and after the shots; nothing
    // reads it in between.
    combat.target_picks.clear();
    if !selection.cast {
        return Ok(FireOutcome::Failed);
    }
    combat.blank_round = 0;
    combat.roster.records[monster_index] = selection.caster;
    if globals.display_writers {
        refresh_enemy_item_alias(combat, monster_index, items, globals)?;
    }
    let weapon_result = SpellInfoRecord(&prepared.packet).plays_impact_when_wounded();
    let physical = SpellInfoRecord(&prepared.packet).is_direct_weapon_damage()?;
    globals.attack_count = if weapon_result {
        1
    } else {
        globals.attack_count.max(1)
    };
    prepared.shots = globals.attack_count;
    let id = if weapon_id == 0 {
        let identity = combat.cast_identity;
        spell_descriptor(identity)
    } else {
        ItemDefinition(weapon).spell()?
    };
    // A rotating packet turns by Rand(4) - 1.
    let rotation = if SpellInfoRecord(&prepared.packet).can_rotate() {
        (random.bound(4) - 1) as u8
    } else {
        0
    };
    let mut origin = selection.targets.first().cloned();
    let count = selection.targets.len();
    let mut flights = vec![];
    let info = super::enemy_spells::spell_info(combat, &prepared.packet, globals);
    let start = events.len();
    for target in selection.targets {
        if !SpellInfoRecord(&prepared.packet).requires_area_mask() {
            let target_body = target.first;
            if !combat.body_alive(warriors, target_body) {
                continue;
            }
        }
        let jitter = crate::combat::spells::flight_jitter(&prepared.packet, random);
        flights.push((
            events.len(),
            crate::combat::spells::flight(combat, body, &target, &prepared.packet, count, jitter),
        ));
        events.push(Event::Sound {
            resource_id: SpellInfoRecord(&prepared.packet).launch_sound()?,
        });
        // A ray walks for every mode-6 packet; weapon fire only skips dead targets
        // below mode 2.
        if physical && SpellInfoRecord(&prepared.packet).is_ray() {
            let stop = super::enemy_spells::enemy_ray(
                combat,
                warriors,
                (mission, random),
                (monster_index, &target),
                (&prepared.packet, globals.animation_delay),
                &mut |combat, warriors, random, mask| {
                    let mut events = vec![];
                    physical_bodies(
                        combat,
                        warriors,
                        random,
                        (monster_index, &prepared),
                        mask,
                        &mut events,
                    )?;
                    Ok(events)
                },
                events,
            )?;
            crate::combat::spells::stop_flight(&mut flights, stop);
            continue;
        }
        if SpellInfoRecord(&prepared.packet).is_ray() {
            let snapshot = globals.alias_snapshot();
            let memory = spell_memory(
                combat,
                warriors,
                mission,
                items,
                globals.char_select_new,
                &snapshot,
            );
            let invocation = (id, &prepared.packet[..], globals.attack_count);
            let stop = super::enemy_spells::enemy_ray(
                combat,
                warriors,
                (mission, random),
                (monster_index, &target),
                (&prepared.packet, globals.animation_delay),
                &mut |combat, warriors, random, mask| {
                    resolve_spell_mask(combat, warriors, random, invocation, mask, body, memory)
                },
                events,
            )?;
            crate::combat::spells::stop_flight(&mut flights, stop);
            globals.take_alias_writes(combat)?;
            continue;
        }
        let context = SpellContext {
            packet: &prepared.packet,
            id,
            count: globals.attack_count,
            caster: body,
            initiative: combat.battle_order_index as i16,
            collision: false,
        };
        let mask = crate::combat::spells::target_mask_rotated_with_origin(
            combat,
            warriors,
            &mission.areas,
            (&target, origin.as_ref().unwrap()),
            &context,
            &combat.spell_queue,
            random,
            (
                rotation,
                crate::combat::map_alias::land_aliases(mission, globals.land.as_ref()),
            ),
        )?;
        if let Some(origin) = origin.as_mut() {
            origin.first = -1;
            origin.second = -1;
        }
        combat.spell_queue = mask.fields;
        if !physical {
            let snapshot = globals.alias_snapshot();
            let body_loop = events.len();
            events.extend(resolve_spell_mask(
                combat,
                warriors,
                random,
                (id, &prepared.packet, globals.attack_count),
                &mask.mask,
                body,
                spell_memory(
                    combat,
                    warriors,
                    mission,
                    items,
                    globals.char_select_new,
                    &snapshot,
                ),
            )?);
            crate::event::SpellArea::attach(events, body_loop, &mask.mask, &prepared.packet);
            globals.take_alias_writes(combat)?;
            continue;
        }
        let body_loop = events.len();
        physical_bodies(
            combat,
            warriors,
            random,
            (monster_index, &prepared),
            &mask.mask,
            events,
        )?;
        crate::event::SpellArea::attach(events, body_loop, &mask.mask, &prepared.packet);
    }
    crate::event::Flight::attach(events, &flights);
    crate::event::SpellInfo::attach(events, start, info);
    if weapon_result {
        Ok(FireOutcome::Physical)
    } else {
        globals.spell_attempt = globals.spell_attempt.wrapping_add(1);
        enemy_spell_continue(
            combat,
            warriors,
            mission,
            random,
            (items, globals),
            body,
            events,
        )?;
        Ok(FireOutcome::Spell)
    }
}

/// Loads the monster's primary weapon and ammunition and prepares its shot with the
/// weapon's armor filter and sound.
///
/// Original: loadweapon, prepattack
fn load_firearm<'a>(
    combat: &mut Combat,
    mission: &centurions_content::mission::OpeningMission,
    items: &'a [Vec<u8>],
    globals: &mut EnemyGlobals,
    e: usize,
) -> Result<(i16, &'a Vec<u8>, firearm::PreparedFirearm), RuleError> {
    let weapon_id = combat.roster.records[e].record().primary_weapon()?;
    // A negative weapon id loads the definition of its absolute value.
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
    let item_context = globals.item_context;
    let ammo_id = enemy_weapons::ammo_id(&combat.roster.records[e], item_context)?;
    let ammo = items
        .get(ammo_id.unsigned_abs() as usize)
        .ok_or_else(|| RuleError::Invalid("enemy ammunition outside content".into()))?;
    globals.buffers.prepare_pair(weapon, ammo)?;
    let mut prepared = if weapon_id == 0 {
        let packet = combat.spell_info.clone();
        combat.weapon_traits.load_pair(weapon, ammo)?;
        combat.roster.records[e].record_mut().set_weapon(0);
        firearm::PreparedFirearm {
            sound: SpellInfoRecord(&packet).impact_sound()?,
            packet,
            slot: 0,
            shots: i16::from(ItemDefinition(weapon).selected_attack_count()),
            rounds: ItemDefinition(weapon).rounds_per_shot()?,
            weight: 0,
            armor_filter: ItemDefinition(weapon).bypasses_hit_roll()?,
        }
    } else if ItemDefinition(weapon).spell()? >= FIRST_SPELL_DESCRIPTOR {
        enemy_weapons::prepare(&mut combat.roster.records[e], weapon, ammo, attacks)?
    } else {
        // The fallback shot reads an enhancement left over from an earlier loop
        // (see docs/source-notes/combat.md).
        return Err(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "enemy fallback descriptor reads combat's loop register D3 as its row",
        });
    };
    if weapon_id != 0 {
        combat.weapon_traits.prepare(weapon, ammo)?;
        combat.spell_info = prepared.packet.clone();
        combat.cast_identity = spell_identity(ItemDefinition(weapon).spell()?);
        globals.attack_count = prepared.shots;
    }
    prepared.armor_filter = combat.weapon_traits.armor_filter();
    prepared.sound = combat.weapon_traits.sound();
    Ok((weapon_id, weapon, prepared))
}

/// The restriction memory resolvespell reads for the monster's cast.
fn spell_memory<'a>(
    combat: &Combat,
    warriors: &[Warrior],
    mission: &'a centurions_content::mission::OpeningMission,
    items: &'a [Vec<u8>],
    char_select_new: usize,
    snapshot: &'a Snapshot,
) -> RestrictionMemoryView<'a> {
    RestrictionMemoryView::ordinary_startup(&mission.item_cache, None)
        .with_tables(mission)
        .with_definitions(items)
        .with_char_up(char_select_new as i16)
        .with_alias_owners(snapshot.owners())
        .with_live_caster(combat, warriors)
}

/// Weapon damage for a physical packet: the duration draw, then each selected body
/// in order.
///
/// Original: resolvespell
pub(super) fn physical_bodies(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    (e, prepared): (usize, &firearm::PreparedFirearm),
    mask: &[i16; 110],
    events: &mut Vec<Event>,
) -> Result<(), RuleError> {
    let body = e as i16 + 10;
    let (minimum, maximum) = SpellInfoRecord(&prepared.packet).duration_range();
    random.range(minimum, maximum);
    for (target, flag) in mask.iter().enumerate() {
        if *flag <= 0 {
            continue;
        }
        let result = if target < 6 {
            firearm::resolve_enemy_after_duration(
                &mut warriors[target],
                &mut combat.roster.records[e],
                prepared,
                random,
                combat.difficulty,
            )?
        } else if target >= 10 {
            let caster = combat.roster.records[e].clone();
            crate::combat::spells::resolve_enemy_firearm_body_after_duration(
                &mut combat.roster.records[target - 10],
                &caster,
                prepared,
                combat.difficulty,
                random,
            )?
        } else {
            return Err(RuleError::Invalid("firearm selected empty body".into()));
        };
        if result.killed {
            lifecycle::kill_body(combat, warriors, target as i16)?;
        }
        events.extend(
            result
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
        if result.damage != 0
            && !result.killed
            && SpellInfoRecord(&prepared.packet).plays_impact_when_wounded()
        {
            events.push(Event::Sound {
                resource_id: crate::combat::ally_turns::result_audio(
                    &combat.roster.records[e],
                    0,
                    result.damage,
                    SpellInfoRecord(&prepared.packet).impact_sound()?,
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
