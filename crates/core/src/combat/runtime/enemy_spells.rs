//! Monsters casting spells.
use super::{EnemyGlobals, resolve_spell_mask, spell_identity};
use crate::combat::Combat;
use crate::combat::firearm::PreparedFirearm;
use crate::combat::party_buffs;
use crate::combat::player_psi::{MoveArena, land_mover};
use crate::combat::spells::RayStep;
use crate::combat::spells::RayWalk;
use crate::combat::spells::RestrictionMemoryView;
use crate::combat::spells::SpellContext;
use crate::combat::spells::SpellTarget;
use crate::combat::spells::out_of_combat::Arm;
use crate::combat::spells::summon;
use crate::combat::spells::summon::ItemRefresh;
use crate::{Event, RandomState, RuleError};
use centurions_content::{Warrior, records::SpellInfoRecord};

/// Loads the monster's selected weapon into the item buffers and returns the restriction its item
/// carries.
pub(super) fn refresh_enemy_item_alias(
    combat: &Combat,
    monster: usize,
    items: &[Vec<u8>],
    globals: &mut EnemyGlobals,
) -> Result<Option<i16>, RuleError> {
    let selected = combat.roster.records[monster].record().weapon()?;
    refresh_selected_item_alias(
        selected,
        items,
        &mut globals.buffers,
        &mut globals.item_context,
    )
}

/// Loads the selected item into the item buffers and returns its restriction; with no item selected
/// the restriction of the item already loaded stands.
pub(super) fn refresh_selected_item_alias(
    selected: i16,
    items: &[Vec<u8>],
    buffers: &mut crate::ammunition::LoadedItems,
    item_context: &mut i16,
) -> Result<Option<i16>, RuleError> {
    if selected == 0 {
        return crate::combat::spells::loaded_item_restriction(buffers);
    }
    *item_context = buffers.load_item_from(items, selected)?;
    let index = usize::try_from(selected.wrapping_abs())
        .map_err(|_| RuleError::Invalid("invalid NPC selected item".into()))?;
    let item = items
        .get(index)
        .ok_or_else(|| RuleError::Invalid("NPC selected item missing".into()))?;
    Ok(Some(
        centurions_content::records::ItemDefinition(item).restriction_alias()?,
    ))
}

/// A monster tries to cast a psi power from its first attempt.
///
/// Original: combat
pub(crate) fn enemy_spell(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    resources: (&[Vec<u8>], &mut EnemyGlobals),
    body: i16,
    events: &mut Vec<Event>,
) -> Result<bool, RuleError> {
    resources.1.spell_attempt = 0;
    enemy_spell_continue(combat, warriors, mission, random, resources, body, events)
}

/// Continues a monster's casting up to its number of magic attacks. Each attempt picks one of its
/// powers; a global power resolves directly on its targets, any other has its targets chosen and is
/// cast. Returns whether a spell was cast.
///
/// Original: combat
pub(super) fn enemy_spell_continue(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    resources: (&[Vec<u8>], &mut EnemyGlobals),
    body: i16,
    events: &mut Vec<Event>,
) -> Result<bool, RuleError> {
    let (items, globals) = resources;
    let monster_index = (body - 10) as usize;
    let tables = mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("spell content missing".into()))?;
    let attempts = combat.roster.records[monster_index]
        .record()
        .magic_attack_count();
    let mut cast = false;
    while globals.spell_attempt < attempts {
        let Some((id, packet)) = crate::combat::spells::select_packet(
            &combat.roster.records[monster_index],
            &tables.attacks,
            random,
        )?
        else {
            break;
        };
        combat.spell_info = packet.clone();
        combat.cast_identity = spell_identity(id);
        if matches!(packet[1], 9 | 10 | 12) {
            let result =
                crate::combat::spells::global_targets(combat, warriors, monster_index, &packet)?;
            combat.roster.records[monster_index] = result.caster;
            let alias = refresh_enemy_item_alias(combat, monster_index, items, globals)?;
            let memory = RestrictionMemoryView::ordinary_startup(&mission.item_cache, alias)
                .with_tables(mission)
                .with_definitions(items)
                .with_selected_party_caster(globals.char_select_new as i16)
                .with_live_caster(combat, warriors);

            combat.in_spell = 1;
            let snapshot = globals.alias_snapshot();
            let info = spell_info(combat, &packet, globals);
            let start = events.len();
            events.extend(resolve_monster_mask(
                combat,
                warriors,
                random,
                (id, &packet, globals.attack_count),
                &result.mask,
                body,
                memory.with_alias_owners(snapshot.owners()),
            )?);
            crate::event::SpellInfo::attach(events, start, info);
            globals.take_alias_writes(combat)?;
            cast = true;
            globals.spell_attempt = globals.spell_attempt.wrapping_add(1);
            continue;
        }
        let selection = crate::combat::spells::select_targets_beside(
            combat,
            warriors,
            (mission, globals.land.as_ref()),
            monster_index,
            &packet,
            random,
        )?;
        globals.attack_count = selection.count;
        combat.target_picks.clear();
        combat.target_picks.write(&selection.targets);
        if selection.targets.is_empty() {
            combat.target_picks.clear();
            break;
        }
        combat.roster.records[monster_index] = selection.caster;
        combat.in_spell = 1;
        let alias = refresh_enemy_item_alias(combat, monster_index, items, globals)?;
        let memory = RestrictionMemoryView::ordinary_startup(&mission.item_cache, alias)
            .with_tables(mission)
            .with_definitions(items)
            .with_selected_party_caster(globals.char_select_new as i16)
            .with_live_caster(combat, warriors);

        globals.attack_count = globals.attack_count.max(1);
        let arm = crate::combat::spells::out_of_combat::arm_of(&packet)?;
        let mut origin = selection.targets[0].clone();
        let total = selection.targets.len();
        let mut flights = vec![];
        let info = spell_info(combat, &packet, globals);
        let start = events.len();
        for (index, target) in selection.targets.into_iter().enumerate() {
            combat.target_picks.enter(index, total);
            let context = SpellContext {
                packet: &packet,
                id,
                count: globals.attack_count,
                caster: body,
                initiative: combat.battle_order_index as i16,
                collision: false,
            };
            let jitter = crate::combat::spells::flight_jitter(&packet, random);
            flights.push((
                events.len(),
                crate::combat::spells::flight(combat, body, &target, &packet, total, jitter),
            ));
            events.push(Event::Sound {
                resource_id: SpellInfoRecord(&packet).launch_sound()?,
            });
            let snapshot = globals.alias_snapshot();
            let memory = memory.with_alias_owners(snapshot.owners());
            if SpellInfoRecord(&packet).is_ray() {
                let invocation = (id, &packet[..], globals.attack_count);
                let stop = enemy_ray(
                    combat,
                    warriors,
                    (mission, random),
                    (monster_index, &target),
                    (&packet, globals.animation_delay),
                    &mut |combat, warriors, random, mask| {
                        resolve_monster_mask(
                            combat, warriors, random, invocation, mask, body, memory,
                        )
                    },
                    events,
                )?;
                crate::combat::spells::stop_flight(&mut flights, stop);
                globals.take_alias_writes(combat)?;
                continue;
            }
            let mask = crate::combat::spells::target_mask_rotated_with_origin(
                combat,
                warriors,
                &mission.areas,
                (&target, &origin),
                &context,
                &combat.spell_queue,
                random,
                (
                    selection.rotation,
                    crate::combat::map_alias::land_aliases(mission, globals.land.as_ref()),
                ),
            )?;
            origin.first = -1;
            origin.second = -1;
            combat.spell_queue = mask.fields;
            combat.in_spell = 1;
            if SpellInfoRecord(&packet).is_party_duration() {
                events.extend(global_durations(combat, random, &packet, globals)?);
                combat.target_picks.resolved(index as i16);
                continue;
            }
            if let Some(arm) = arm {
                combat.mark_monster_spell_pick(&packet, &mask.mask);
                combat.blank_round = 0;
                let invocation = (id, &packet[..], globals.attack_count);
                events.extend(enemy_arm(
                    combat,
                    warriors,
                    (mission, items),
                    random,
                    globals,
                    (arm, body, (target.first, target.second)),
                    invocation,
                )?);
                combat.in_spell = 0;
                combat.target_picks.resolved(index as i16);
                if combat.target_picks.loop_ended(index) {
                    break;
                }
                continue;
            }
            let body_loop = events.len();
            events.extend(resolve_monster_mask(
                combat,
                warriors,
                random,
                (id, &packet, globals.attack_count),
                &mask.mask,
                body,
                memory,
            )?);
            crate::event::SpellArea::attach(events, body_loop, &mask.mask, &combat.spell_info);
            globals.take_alias_writes(combat)?;
            if !SpellInfoRecord(&packet).is_ray() {
                combat.in_spell = 0;
            }
            combat.target_picks.resolved(index as i16);
        }
        crate::event::Flight::attach(events, &flights);
        crate::event::SpellInfo::attach(events, start, info);
        combat.target_picks.leave();
        cast = true;
        globals.spell_attempt = globals.spell_attempt.wrapping_add(1);
    }
    Ok(cast)
}

/// A monster's movement or summon spell. Both act for the acting warrior (charup):
/// movement lands on the first target, a summon at `current`, the cast's target.
///
/// Original: resolvespell
#[allow(clippy::too_many_arguments)]
fn enemy_arm(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    (mission, items): (&centurions_content::mission::OpeningMission, &[Vec<u8>]),
    random: &mut RandomState,
    globals: &mut EnemyGlobals,
    (arm, caster, current): (Arm, i16, (i16, i16)),
    (id, packet, count): (i16, &[u8], i16),
) -> Result<Vec<Event>, RuleError> {
    let actor = globals.char_select_new;
    if actor >= warriors.len() {
        // The session admits only a selected warrior inside the squad.
        return Err("NPC movement or summon arm for an inactive selected warrior".into());
    }
    live_arm(
        combat,
        warriors,
        (mission, random),
        (arm, actor),
        (id, packet, count),
        &mut globals.movement_gate,
        (Some(caster), Some(current)),
        Some((items, &mut globals.buffers)),
    )
}

/// The summon spell, or the movement spell after its duration draws and the
/// movement-marker test, for the acting warrior at the first target's cell. Showing
/// the movement result clears the marker. `current` is the cast's target, which the
/// summon reads.
///
/// Original: resolvespell
#[allow(clippy::too_many_arguments)]
pub(crate) fn live_arm(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    (mission, random): (
        &centurions_content::mission::OpeningMission,
        &mut RandomState,
    ),
    (arm, actor): (Arm, usize),
    (id, packet, count): (i16, &[u8], i16),
    gate: &mut i16,
    (caster, current): (Option<i16>, Option<(i16, i16)>),
    refresh: ItemRefresh<'_>,
) -> Result<Vec<Event>, RuleError> {
    let shift = combat.field.map_shift;
    let relative = move |cell: (i16, i16)| (cell.0 - shift.0 - 5, cell.1 - shift.1 - 5);
    if arm == Arm::Summon {
        // A full roster refuses the summon before its target is read.
        let target = combat.target_picks.summon_cell(current).map(relative);
        return summon::summon_at(
            combat, warriors, mission, random, actor, target, count, packet, refresh,
        );
    }
    crate::combat::spells::out_of_combat::duration(packet, count, random);
    if *gate == 0 {
        return Ok(vec![Event::Feedback(
            crate::feedback::GameFeedback::Warning { index: 87 },
        )]);
    }
    let target = relative(combat.target_picks.arm_cell()?);
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
    *gate = 0;
    if defeated {
        crate::combat::lifecycle::kill_body(combat, warriors, actor as i16)?;
    }
    events.push(Event::SpellResolved {
        caster,
        spell: id,
        target: actor as i16,
        damage: 0,
        defeated,
        icon: crate::event::PresentationIcon(0),
        flight: Default::default(),
    });
    Ok(events)
}

/// A monster's global (mode 7) spell writes the same party duration table.
///
/// Original: resolvespell
fn global_durations(
    combat: &mut Combat,
    random: &mut RandomState,
    packet: &[u8],
    globals: &mut EnemyGlobals,
) -> Result<Vec<Event>, RuleError> {
    let buffs = globals
        .party_condition
        .as_mut()
        .ok_or_else(|| RuleError::from("NPC global spell without party durations"))?;
    combat.in_spell = 0;
    combat.blank_round = 0;
    let attack = SpellInfoRecord(packet);
    let (minimum, maximum) = attack.duration_range();
    let mut duration = random.range(minimum, maximum);
    let (minimum, maximum) = attack.power_duration_range();
    if minimum != 0 {
        for _ in 0..globals.attack_count {
            duration = duration.wrapping_add(random.range(minimum, maximum));
        }
    }
    let mut events = vec![];
    match attack.effect()?.wrapping_abs() {
        50 => {
            events.push(Event::Sound { resource_id: 601 });
            party_buffs::extend_from_count(buffs, globals.attack_count)?;
        }
        effect @ 0..=9 => party_buffs::extend(buffs, effect, duration)?,
        // Effect 110's duration lies past the scenario directory's terminator.
        110 => {
            let tail = globals.scenario_tail.as_mut().ok_or_else(|| {
                unavailable("NPC effect110 tail WORD overlaps scenario-directory text")
            })?;
            *tail = (*tail).max(duration);
        }
        effect => {
            let owned = globals.flags.as_mut().is_some_and(|flags| {
                party_buffs::extend_indexed_owners(
                    buffs,
                    &mut combat.difficulty,
                    flags,
                    effect,
                    duration,
                )
            });
            if !owned {
                return Err(unavailable(
                    "NPC indexed duration outside the party durations",
                ));
            }
        }
    }
    Ok(events)
}

/// The refusal for a monster spell that depends on an unrecovered part of the game.
fn unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.npc_spell",
        dependency,
    }
}

/// A monster's ray from its footprint to the target body, resolving each newly
/// crossed body while in_spell stays set.
///
/// Original: cast
#[allow(clippy::too_many_arguments)]
pub(super) fn enemy_ray(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    (mission, random): (
        &centurions_content::mission::OpeningMission,
        &mut RandomState,
    ),
    (e, target): (usize, &SpellTarget),
    (packet, animation_delay): (&[u8], i16),
    resolve: &mut RayBody<'_>,
    events: &mut Vec<Event>,
) -> Result<Option<i16>, RuleError> {
    if !crate::combat::spells::ray_walks(packet) {
        return Ok(None);
    }
    let monster_pixel = |combat: &Combat, index: usize| -> Result<(i16, i16), RuleError> {
        let cell = *combat
            .roster
            .mon_pos
            .get(index)
            .ok_or_else(|| RuleError::Invalid("NPC ray body missing".into()))?;
        let size = combat.roster.records[index].record().size()?.wrapping_abs();
        Ok(crate::combat::spells::body_pixel(cell, Some(size)))
    };
    let end = match target.kind {
        1 => crate::combat::spells::body_pixel(
            *combat
                .squad
                .pos
                .get(target.first as usize)
                .ok_or_else(|| RuleError::Invalid("NPC ray party target missing".into()))?,
            None,
        ),
        2 => monster_pixel(combat, (target.first - 10) as usize)?,
        _ => return Err(unavailable("NPC ray cell target")),
    };
    let start = monster_pixel(combat, e)?;
    let body = e + 10;
    let mut ray = RayWalk::new(start, end, 128 + animation_delay, body);
    while let Some(step) = ray.next(&combat.field.tiles, combat.field.map_shift, mission, packet)? {
        let RayStep::Body(crossed) = step else {
            events.push(Event::Sound {
                resource_id: -10082,
            });
            return Ok(Some(ray.step()));
        };
        let mut mask = [0_i16; 110];
        mask[crossed] = 1;
        let crossing = events.len();
        events.extend(resolve(combat, warriors, random, &mask)?);
        crate::combat::spells::mark_step(&mut events[crossing..], ray.step());
    }
    Ok(None)
}

/// The spell information shown for a monster's packet.
///
/// Original: showspellinfo
pub(super) fn spell_info(
    combat: &Combat,
    packet: &[u8],
    globals: &EnemyGlobals,
) -> crate::event::SpellInfo {
    monster_spell_info(
        packet,
        combat.cast_identity,
        globals.attack_count,
        &globals.buffers,
    )
}

/// The spell information shown for a monster's packet: its identity and power
/// level. A weapon (category 9) packet shows the party's last target count instead,
/// which the reconstruction cannot know (see docs/source-notes/combat.md).
///
/// Original: showspellinfo
pub(super) fn monster_spell_info(
    packet: &[u8],
    identity: [i16; 3],
    count: i16,
    buffers: &crate::ammunition::LoadedItems,
) -> crate::event::SpellInfo {
    let firearm = SpellInfoRecord(packet).defense_category() == 9;
    crate::event::SpellInfo::new(
        packet,
        identity,
        if firearm { 1 } else { count },
        None,
        (buffers.weapon.as_deref(), buffers.item.as_deref()),
    )
}

/// Resolves the spell on one body the ray crosses.
pub(super) type RayBody<'a> = dyn FnMut(
        &mut Combat,
        &mut [Warrior],
        &mut RandomState,
        &[i16; 110],
    ) -> Result<Vec<Event>, RuleError>
    + 'a;

/// Resolves a monster's spell on each selected body. A direct weapon-damage packet
/// does weapon damage with the weapon's armor filter and sound; every other packet
/// goes through the shared spell resolver.
///
/// Original: resolvespell
fn resolve_monster_mask(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    (id, packet, count): (i16, &[u8], i16),
    mask: &[i16; 110],
    body: i16,
    memory: RestrictionMemoryView<'_>,
) -> Result<Vec<Event>, RuleError> {
    if !SpellInfoRecord(packet).is_direct_weapon_damage()? {
        return resolve_spell_mask(
            combat,
            warriors,
            random,
            (id, packet, count),
            mask,
            body,
            memory,
        );
    }
    combat.blank_round = 0;
    let prepared = PreparedFirearm {
        packet: packet.to_vec(),
        slot: 0,
        shots: count,
        rounds: 0,
        weight: 0,
        sound: combat.weapon_traits.sound(),
        armor_filter: combat.weapon_traits.armor_filter(),
    };
    let mut events = vec![];
    super::enemy_fire::physical_bodies(
        combat,
        warriors,
        random,
        ((body - 10) as usize, &prepared),
        mask,
        &mut events,
    )?;
    Ok(events)
}
