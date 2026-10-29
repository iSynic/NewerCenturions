//! Enemy spells in the original combat execution order.
use super::{EnemyWork, resolve_spell_mask, spell_identity};
use crate::combat::Combat;
use crate::combat::firearm::PreparedFirearm;
use crate::combat::player_psi::{MoveArena, land_mover};
use crate::combat::spells::detached::Arm;
use crate::{Event, RandomState, RuleError};
use centurions_content::{Warrior, records::AttackPacket};

pub(super) fn refresh_enemy_item_alias(
    c: &Combat,
    enemy: usize,
    items: &[Vec<u8>],
    work: &mut EnemyWork,
) -> Result<Option<i16>, RuleError> {
    let selected = c.enemies.records[enemy].record().selected_item()?;
    refresh_selected_item_alias(selected, items, &mut work.buffers, &mut work.item_context)
}

pub(super) fn refresh_selected_item_alias(
    selected: i16,
    items: &[Vec<u8>],
    buffers: &mut crate::ammunition::WorkBuffers,
    item_context: &mut i16,
) -> Result<Option<i16>, RuleError> {
    if selected == 0 {
        return crate::combat::spells::retained_item_alias(buffers);
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

pub(crate) fn enemy_spell(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    resources: (&[Vec<u8>], &mut EnemyWork),
    body: i16,
    events: &mut Vec<Event>,
) -> Result<bool, RuleError> {
    resources.1.spell_attempt = 0;
    enemy_spell_continue(c, w, mission, random, resources, body, events)
}

pub(super) fn enemy_spell_continue(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    resources: (&[Vec<u8>], &mut EnemyWork),
    body: i16,
    events: &mut Vec<Event>,
) -> Result<bool, RuleError> {
    let (items, work) = resources;
    let e = (body - 10) as usize;
    let tables = mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("spell content missing".into()))?;
    let attempts = c.enemies.records[e].record().spell_attempts();
    let mut cast = false;
    while work.spell_attempt < attempts {
        let Some((id, packet)) =
            crate::combat::spells::select_packet(&c.enemies.records[e], &tables.attacks, random)?
        else {
            break;
        };
        c.attack_packet = packet.clone();
        c.attack_identity = spell_identity(id);
        if matches!(packet[1], 9 | 10 | 12) {
            let result = crate::combat::spells::global_targets(c, w, e, &packet)?;
            c.enemies.records[e] = result.caster;
            let alias = refresh_enemy_item_alias(c, e, items, work)?;
            let memory = crate::combat::spells::RestrictionMemoryView::ordinary_startup(
                &mission.item_cache,
                alias,
            )
            .with_tables(mission)
            .with_definitions(items)
            .with_selected_party_caster(work.selected_warrior as i16)
            .with_live_caster(c, w);

            c.ready = 1;
            let snapshot = work.alias_snapshot();
            // combat 11495e, 114a4c and 114b50 call showspellinfo before resolvespell.
            let info = spell_info(c, &packet, work);
            let start = events.len();
            events.extend(resolve_npc_mask(
                c,
                w,
                random,
                (id, &packet, work.attack_count),
                &result.mask,
                body,
                memory.with_alias_owners(snapshot.owners()),
            )?);
            crate::event::SpellInfo::attach(events, start, info);
            work.take_alias_writes(c)?;
            cast = true;
            work.spell_attempt = work.spell_attempt.wrapping_add(1);
            continue;
        }
        let selection = crate::combat::spells::select_targets_beside(
            c,
            w,
            (mission, work.land.as_ref()),
            e,
            &packet,
            random,
        )?;
        work.attack_count = selection.count;
        // combat 114b7e calls cleartarget before the decision writes its triples.
        c.target_work.clear();
        c.target_work.write(&selection.targets);
        if selection.targets.is_empty() {
            // 115110 calls cleartarget after a failed search.
            c.target_work.clear();
            break;
        }
        c.enemies.records[e] = selection.caster;
        c.ready = 1;
        let alias = refresh_enemy_item_alias(c, e, items, work)?;
        let memory = crate::combat::spells::RestrictionMemoryView::ordinary_startup(
            &mission.item_cache,
            alias,
        )
        .with_tables(mission)
        .with_definitions(items)
        .with_selected_party_caster(work.selected_warrior as i16)
        .with_live_caster(c, w);

        work.attack_count = work.attack_count.max(1);
        let arm = crate::combat::spells::detached::arm_of(&packet)?;
        let mut origin = selection.targets[0].clone();
        let total = selection.targets.len();
        let mut flights = vec![];
        // combat 11509e calls showspellinfo before cast.
        let info = spell_info(c, &packet, work);
        let start = events.len();
        for (index, target) in selection.targets.into_iter().enumerate() {
            c.target_work.enter(index, total);
            let context = crate::combat::spells::SpellContext {
                packet: &packet,
                id,
                count: work.attack_count,
                caster: body,
                initiative: c.cursor as i16,
                collision: false,
            };
            let jitter = crate::combat::spells::flight_jitter(&packet, random);
            flights.push((
                events.len(),
                crate::combat::spells::flight(c, body, &target, &packet, total, jitter),
            ));
            events.push(Event::Sound {
                resource_id: AttackPacket(&packet).launch_sound()?,
            });
            let snapshot = work.alias_snapshot();
            let memory = memory.with_alias_owners(snapshot.owners());
            if AttackPacket(&packet).targeting_mode() == 6 {
                let invocation = (id, &packet[..], work.attack_count);
                let stop = enemy_ray(
                    c,
                    w,
                    (mission, random),
                    (e, &target),
                    (&packet, work.animation_delay),
                    &mut |c, w, random, mask| {
                        resolve_npc_mask(c, w, random, invocation, mask, body, memory)
                    },
                    events,
                )?;
                crate::combat::spells::stop_flight(&mut flights, stop);
                work.take_alias_writes(c)?;
                continue;
            }
            let mask = crate::combat::spells::target_mask_rotated_with_origin(
                c,
                w,
                &mission.areas,
                (&target, &origin),
                &context,
                &c.spell_fields,
                random,
                (
                    selection.rotation,
                    crate::combat::map_alias::land_aliases(mission, work.land.as_ref()),
                ),
            )?;
            origin.first = -1;
            origin.second = -1;
            c.spell_fields = mask.fields;
            c.ready = 1;
            if AttackPacket(&packet).targeting_mode() == 7 {
                events.extend(global_durations(c, random, &packet, work)?);
                c.target_work.resolved(index as i16);
                continue;
            }
            if let Some(arm) = arm {
                c.mark_monster_spell_work(&packet, &mask.mask);
                c.inactive_rounds = 0;
                let invocation = (id, &packet[..], work.attack_count);
                events.extend(enemy_arm(
                    c,
                    w,
                    (mission, items),
                    random,
                    work,
                    (arm, body, (target.first, target.second)),
                    invocation,
                )?);
                c.ready = 0;
                c.target_work.resolved(index as i16);
                if c.target_work.loop_ended(index) {
                    break;
                }
                continue;
            }
            let body_loop = events.len();
            events.extend(resolve_npc_mask(
                c,
                w,
                random,
                (id, &packet, work.attack_count),
                &mask.mask,
                body,
                memory,
            )?);
            crate::event::SpellArea::attach(events, body_loop, &mask.mask, &c.attack_packet);
            work.take_alias_writes(c)?;
            if AttackPacket(&packet).targeting_mode() != 6 {
                c.ready = 0;
            }
            c.target_work.resolved(index as i16);
        }
        crate::event::Flight::attach(events, &flights);
        crate::event::SpellInfo::attach(events, start, info);
        // cast ends with cleartarget, and 115110 calls it again.
        c.target_work.leave();
        cast = true;
        work.spell_attempt = work.spell_attempt.wrapping_add(1);
    }
    Ok(cast)
}

/// resolvespell's movement (151418) and summon (1516ea) arms for an NPC cast: both act
/// for WORD4435d2, the movement arm at triple 0 of the decision's target list
/// (WORD443722/443724) and the summon arm at `current`, the loop's triple.
#[allow(clippy::too_many_arguments)]
fn enemy_arm(
    c: &mut Combat,
    w: &mut [Warrior],
    (mission, items): (&centurions_content::mission::OpeningMission, &[Vec<u8>]),
    random: &mut RandomState,
    work: &mut EnemyWork,
    (arm, caster, current): (Arm, i16, (i16, i16)),
    (id, packet, count): (i16, &[u8], i16),
) -> Result<Vec<Event>, RuleError> {
    let actor = work.selected_warrior;
    if actor >= w.len() {
        // The session admits only a selected warrior inside the squad.
        return Err("NPC movement or summon arm for an inactive selected warrior".into());
    }
    live_arm(
        c,
        w,
        (mission, random),
        (arm, actor),
        (id, packet, count),
        &mut work.movement_gate,
        (Some(caster), Some(current)),
        Some((items, &mut work.buffers)),
    )
}

/// resolvespell's summon arm 1516ea, or its movement arm 151418 after the duration
/// draws and the WORD438786 test, for WORD4435d2 at the cell WORD443722/443724.
/// The movement arm's showresults clears the marker. `current` is the cast loop's
/// triple, which the summon arm reads at WORD442bd4.
#[allow(clippy::too_many_arguments)]
pub(crate) fn live_arm(
    c: &mut Combat,
    w: &mut [Warrior],
    (mission, random): (
        &centurions_content::mission::OpeningMission,
        &mut RandomState,
    ),
    (arm, actor): (Arm, usize),
    (id, packet, count): (i16, &[u8], i16),
    gate: &mut i16,
    (caster, current): (Option<i16>, Option<(i16, i16)>),
    refresh: crate::combat::spell_summon::ItemRefresh<'_>,
) -> Result<Vec<Event>, RuleError> {
    let shift = c.map.shift;
    let relative = move |cell: (i16, i16)| (cell.0 - shift.0 - 5, cell.1 - shift.1 - 5);
    if arm == Arm::Summon {
        // The full-roster branch (1516f0) runs before spelllist reads the triple.
        let target = c.target_work.summon_cell(current).map(relative);
        return crate::combat::spell_summon::summon_at(
            c, w, mission, random, actor, target, count, packet, refresh,
        );
    }
    crate::combat::spells::detached::duration(packet, count, random);
    if *gate == 0 {
        return Ok(vec![Event::Feedback(
            crate::feedback::NativeFeedback::Warning { index: 87 },
        )]);
    }
    let target = relative(c.target_work.arm_cell()?);
    let (mut events, defeated) = land_mover(
        MoveArena {
            map: &mut c.map,
            positions: &mut c.squad.positions,
            under_tiles: &mut c.squad.under_tiles,
            cursor: &mut c.cursor,
        },
        w,
        mission,
        actor,
        packet,
        target,
    )?;
    *gate = 0;
    if defeated {
        crate::combat::lifecycle::kill_body(c, w, actor as i16)?;
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

/// resolvespell's mode7 arm: an NPC caster writes the same party duration table.
fn global_durations(
    c: &mut Combat,
    random: &mut RandomState,
    packet: &[u8],
    work: &mut EnemyWork,
) -> Result<Vec<Event>, RuleError> {
    let buffs = work
        .party_buffs
        .as_mut()
        .ok_or_else(|| RuleError::from("NPC global spell without party durations"))?;
    c.ready = 0;
    c.inactive_rounds = 0;
    let attack = AttackPacket(packet);
    let (minimum, maximum) = attack.duration_bounds();
    let mut duration = random.range(minimum, maximum);
    let (minimum, maximum) = attack.additional_duration_bounds();
    if minimum != 0 {
        for _ in 0..work.attack_count {
            duration = duration.wrapping_add(random.range(minimum, maximum));
        }
    }
    let mut events = vec![];
    match attack.effect()?.wrapping_abs() {
        50 => {
            events.push(Event::Sound { resource_id: 601 });
            crate::combat::party_buffs::extend_from_count(buffs, work.attack_count)?;
        }
        effect @ 0..=9 => crate::combat::party_buffs::extend(buffs, effect, duration)?,
        // resolvespell15130e writes WORD44263e beyond the directory NUL.
        110 => {
            let tail = work.scenario_tail.as_mut().ok_or_else(|| {
                unavailable("NPC effect110 tail WORD overlaps scenario-directory text")
            })?;
            *tail = (*tail).max(duration);
        }
        effect => {
            let owned = work.flags.as_mut().is_some_and(|flags| {
                crate::combat::party_buffs::extend_indexed_owners(
                    buffs,
                    &mut c.difficulty,
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

fn unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.npc_spell",
        dependency,
    }
}

/// The NPC decision's cast10d4fe ray: from the caster's footprint pixel to the target body's,
/// resolving each newly crossed body with Ready kept.
#[allow(clippy::too_many_arguments)]
pub(super) fn enemy_ray(
    c: &mut Combat,
    w: &mut [Warrior],
    (mission, random): (
        &centurions_content::mission::OpeningMission,
        &mut RandomState,
    ),
    (e, target): (usize, &crate::combat::spells::SpellTarget),
    (packet, animation_delay): (&[u8], i16),
    resolve: &mut RayBody<'_>,
    events: &mut Vec<Event>,
) -> Result<Option<i16>, RuleError> {
    if !crate::combat::spells::ray_walks(packet) {
        return Ok(None);
    }
    let npc_pixel = |c: &Combat, index: usize| -> Result<(i16, i16), RuleError> {
        let cell = *c
            .enemies
            .positions
            .get(index)
            .ok_or_else(|| RuleError::Invalid("NPC ray body missing".into()))?;
        let size = c.enemies.records[index].record().size()?.wrapping_abs();
        Ok(crate::combat::spells::body_pixel(cell, Some(size)))
    };
    let end = match target.kind {
        1 => crate::combat::spells::body_pixel(
            *c.squad
                .positions
                .get(target.first as usize)
                .ok_or_else(|| RuleError::Invalid("NPC ray party target missing".into()))?,
            None,
        ),
        2 => npc_pixel(c, (target.first - 10) as usize)?,
        _ => return Err(unavailable("NPC ray cell target")),
    };
    let start = npc_pixel(c, e)?;
    let body = e + 10;
    let mut ray = crate::combat::spells::RayWalk::new(start, end, 128 + animation_delay, body);
    while let Some(step) = ray.next(&c.map.tiles, c.map.shift, mission, packet)? {
        let crate::combat::spells::RayStep::Body(crossed) = step else {
            events.push(Event::Sound {
                resource_id: -10082,
            });
            return Ok(Some(ray.step()));
        };
        let mut mask = [0_i16; 110];
        mask[crossed] = 1;
        let crossing = events.len();
        events.extend(resolve(c, w, random, &mask)?);
        crate::combat::spells::mark_step(&mut events[crossing..], ray.step());
    }
    Ok(None)
}

/// showspellinfo's drawing for an NPC's packet.
pub(super) fn spell_info(c: &Combat, packet: &[u8], work: &EnemyWork) -> crate::event::SpellInfo {
    npc_spell_info(packet, c.attack_identity, work.attack_count, &work.buffers)
}

/// showspellinfo's drawing for an NPC's packet, identity WORDs 0x4435c6..0x4435ca
/// and count WORD4435cc. combat 115088..11508e sets WORD4435cc to 1 for a
/// category-9 packet, whose number is WORD4435be instead. Only the party's
/// selections write WORD4435be: combat's repeat-Psi path (0x116a02, 0x116a22,
/// 0x116a2e), combatchoice's entry clear unless BYTE437a6a (0x117c40), its fire
/// button (0x118932, 0x1189ee, 0x118a0e, 0x118a1a, 0x118a2c), item button
/// (0x1190c2), castspell result (0x11921a, 0x119226) and quantity steps
/// (0x11a580, 0x11a702). One combatchoice entry spans several commands, and
/// commands do not record which entry they belong to, so the number stays unknown.
pub(super) fn npc_spell_info(
    packet: &[u8],
    identity: [i16; 3],
    count: i16,
    buffers: &crate::ammunition::WorkBuffers,
) -> crate::event::SpellInfo {
    let firearm = AttackPacket(packet).defense_category() == 9;
    crate::event::SpellInfo::new(
        packet,
        identity,
        if firearm { 1 } else { count },
        None,
        (buffers.weapon.as_deref(), buffers.item.as_deref()),
    )
}

/// resolvespell 151022 for one crossed body (cast 10e22a).
pub(super) type RayBody<'a> = dyn FnMut(
        &mut Combat,
        &mut [Warrior],
        &mut RandomState,
        &[i16; 110],
    ) -> Result<Vec<Event>, RuleError>
    + 'a;

/// resolvespell's body loop for an NPC caster. A direct weapon-damage packet takes
/// the per-body weapon damage NPC fire uses, with WEAPON work 453768's filter and
/// sound; every other packet goes through the shared spell resolver.
fn resolve_npc_mask(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    (id, packet, count): (i16, &[u8], i16),
    mask: &[i16; 110],
    body: i16,
    memory: crate::combat::spells::RestrictionMemoryView<'_>,
) -> Result<Vec<Event>, RuleError> {
    if !AttackPacket(packet).is_direct_weapon_damage()? {
        return resolve_spell_mask(c, w, random, (id, packet, count), mask, body, memory);
    }
    c.inactive_rounds = 0;
    let prepared = PreparedFirearm {
        packet: packet.to_vec(),
        slot: 0,
        shots: count,
        rounds: 0,
        weight: 0,
        sound: c.weapon_work.sound(),
        armor_filter: c.weapon_work.armor_filter(),
    };
    let mut events = vec![];
    super::enemy_fire::physical_bodies(
        c,
        w,
        random,
        ((body - 10) as usize, &prepared),
        mask,
        &mut events,
    )?;
    Ok(events)
}
