//! Fields in the original combat execution order.
use centurions_content::records::{CharacterStatisticsMut, Statistic};

use super::{append_spell_outcomes, spell_identity};
use crate::combat::Combat;
use crate::{Event, RandomState, RuleError};
use centurions_content::Warrior;

/// `actor` is the moving body, the selected warrior WORD4435d2 and the movement
/// marker WORD438786 that resolvespell reads.
pub fn collide_fields(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    count: &mut i16,
    random: &mut RandomState,
    (body, selected, gate): (i16, usize, &mut i16),
) -> Result<Vec<Event>, RuleError> {
    collide_fields_with_work(
        c,
        w,
        mission,
        count,
        random,
        (body, None, selected, gate),
        &mut [0; 6],
        None,
    )
}

/// `actor` is the moving body, the ITEM work alias, the selected warrior WORD4435d2
/// and the movement marker WORD438786 that resolvespell reads. `items` is the ITEM
/// work combatupdate2 refreshes after a full-roster summon refusal.
#[allow(clippy::too_many_arguments)]
pub(crate) fn collide_fields_with_work(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    count: &mut i16,
    random: &mut RandomState,
    actor: (i16, Option<i16>, usize, &mut i16),
    party_mask: &mut [i16; 6],
    items: crate::combat::spell_summon::ItemRefresh<'_>,
) -> Result<Vec<Event>, RuleError> {
    if actor.2 >= w.len() {
        return Err("field collision for an inactive selected warrior".into());
    }
    let mut next_gate = *actor.3;
    let mut next = c.clone();
    let mut next_warriors = w.to_vec();
    let mut next_count = *count;
    let mut next_random = random.clone();
    let mut next_mask = *party_mask;
    let mut next_buffers = items.as_ref().map(|(_, buffers)| (**buffers).clone());
    let events = collide_fields_impl(
        &mut next,
        &mut next_warriors,
        mission,
        &mut next_count,
        &mut next_random,
        FieldActor {
            body: actor.0,
            selected_warrior: actor.2,
            item_work_alias: actor.1,
            movement_gate: &mut next_gate,
            items: items
                .as_ref()
                .map(|(items, _)| *items)
                .zip(next_buffers.as_mut()),
        },
        &mut next_mask,
    )?;
    if let (Some((_, buffers)), Some(next)) = (items, next_buffers) {
        *buffers = next;
    }
    *actor.3 = next_gate;
    *c = next;
    w.clone_from_slice(&next_warriors);
    *count = next_count;
    *random = next_random;
    *party_mask = next_mask;
    Ok(events)
}

pub(super) struct FieldActor<'a> {
    pub(super) body: i16,
    pub(super) selected_warrior: usize,
    pub(super) item_work_alias: Option<i16>,
    /// WORD438786.
    pub(super) movement_gate: &'a mut i16,
    /// combatupdate2's ITEM work for a full-roster summon refusal.
    pub(super) items: crate::combat::spell_summon::ItemRefresh<'a>,
}

pub(super) fn collide_fields_impl(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    count: &mut i16,
    random: &mut RandomState,
    mut actor: FieldActor<'_>,
    party_mask: &mut [i16; 6],
) -> Result<Vec<Event>, RuleError> {
    let body = actor.body;
    if !c
        .spell_fields
        .as_chunks::<20>()
        .0
        .iter()
        .any(|field| field[15] != 0)
    {
        return Ok(vec![]);
    }
    let tables = mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("spell collision tables missing".into()))?;
    let mut events = vec![];
    let mut usage = crate::combat::spells::FieldUsage {
        slot: 0,
        used: c.field_used.clone(),
    };
    for slot in 0..60 {
        usage.slot = slot;
        let Some(collision) = crate::combat::spells::collision_field(
            c,
            w,
            &c.spell_fields,
            &usage,
            body as usize,
            &tables.attacks,
        )?
        else {
            continue;
        };
        // collide 1142a6 calls cleartarget; 1142e0..1142fe store the centre as target 0.
        c.target_work.clear();
        c.target_work.write(std::slice::from_ref(&collision.target));
        *count = collision.count;
        c.attack_packet = collision.packet.clone();
        let mut identity = spell_identity(collision.id);
        identity[1] = collision.level;
        c.attack_identity = identity;
        // collide1140ae calls spelltargets15cb0e, which holds Ready at 1 around
        // resolvespell, so a collision killbody queues its script; Ready is 0 after.
        // Target 0 is the field centre and resolvespell's 0x3a arm summons there;
        // spelltargets' mask is never consulted.
        if centurions_content::records::AttackPacket(&collision.packet)
            .effect()?
            .wrapping_abs()
            == 58
        {
            c.ready = 1;
            events.extend(super::live_arm(
                c,
                w,
                (mission, random),
                (
                    crate::combat::spells::detached::Arm::Summon,
                    actor.selected_warrior,
                ),
                (collision.id, &collision.packet, *count),
                &mut *actor.movement_gate,
                (None, None),
                actor
                    .items
                    .as_mut()
                    .map(|(items, buffers)| (*items, &mut **buffers)),
            )?);
            c.ready = 0;
            c.target_work.resolved(0);
            continue;
        }
        let context = crate::combat::spells::SpellContext {
            packet: &collision.packet,
            id: collision.id,
            count: *count,
            caster: body,
            initiative: c.cursor as i16,
            collision: true,
        };
        let mask = crate::combat::spells::target_mask_rotated_with_origin(
            c,
            w,
            &mission.areas,
            (&collision.target, &collision.target),
            &context,
            &c.spell_fields,
            random,
            (
                collision.rotation,
                crate::combat::map_alias::MapAliases::of_mission(mission),
            ),
        )?;
        party_mask.copy_from_slice(&mask.mask[..6]);
        // collide114332 leaves spelltargets' marks; no cast clears them.
        c.target_tail = mask.mask[6..].to_vec();
        let previous_hp = w
            .iter()
            .map(Warrior::stamina)
            .collect::<Result<Vec<_>, _>>()?;
        c.mark_monster_spell_work(&collision.packet, &mask.mask);
        c.ready = 1;
        let effect = centurions_content::records::AttackPacket(&collision.packet)
            .effect()?
            .wrapping_abs();
        if matches!(effect, 55 | 56) {
            events.extend(move_selected(
                c,
                w,
                (mission, random),
                &mut actor,
                &collision,
            )?);
            c.ready = 0;
            c.target_work.resolved(0);
            continue;
        }
        let memory = crate::combat::spells::RestrictionMemoryView::ordinary_startup(
            &mission.item_cache,
            actor.item_work_alias,
        )
        .with_tables(mission)
        .with_selected_warrior(actor.selected_warrior as i16)
        .with_live_caster(c, w);
        let result = crate::combat::spells::resolve_collision_with_memory(
            w,
            &c.enemies.records,
            &mask.mask,
            &crate::combat::spells::DirectSpell {
                packet: &collision.packet,
                level: collision.level,
                count: *count,
                armor_filter: c.weapon_work.armor_filter(),
                weapon_sound: c.weapon_work.sound(),
            },
            &mut usage,
            (c.active_body(), memory),
            random,
        )?;
        c.unavailable = c.unavailable.wrapping_sub(result.recovered as i8);
        if result.healed {
            centurions_content::records::AttackPacketMut(&mut c.attack_packet).set_health_effect(4);
        }
        let result = result.body;
        w.clone_from_slice(&result.warriors);
        c.victory_work.add_total(result.total_delta);
        c.enemies.records = result.enemies;
        if c.active_body()
            .is_some_and(|active| (0..6).contains(&active))
        {
            let member = actor.selected_warrior;
            let warrior = w
                .get_mut(member)
                .ok_or_else(|| RuleError::Invalid("field credit warrior missing".into()))?;
            for (target, outcome) in &result.targets {
                if *target >= 10 && outcome.damage != 0 {
                    CharacterStatisticsMut(&mut warrior.bytes)
                        .add(Statistic::DamageDealt, i32::from(outcome.damage));
                    if collision.packet[19] == 9 {
                        CharacterStatisticsMut(&mut warrior.bytes).add(Statistic::Hits, 1);
                    }
                    if outcome.killed {
                        CharacterStatisticsMut(&mut warrior.bytes).add(Statistic::Kills, 1);
                    }
                }
            }
        }
        events.extend(
            result
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
        append_spell_outcomes(
            c,
            w,
            result.targets,
            result.race_excluded,
            (None, collision.id, &collision.packet, &previous_hp),
            true,
            &mut events,
        )?;
        c.ready = 0;
        // collide pushes 0 as spelltargets' triple argument (114326).
        c.target_work.resolved(0);
    }
    c.field_used = usage.used;
    Ok(events)
}

/// resolvespell151418 for a movement field: after its duration draws and the
/// WORD438786 test, WORD4435d2 lands on the field centre collide stored in
/// WORD443722/443724. The body loop and its usage byte do not run.
fn move_selected(
    c: &mut Combat,
    w: &mut [Warrior],
    (mission, random): (
        &centurions_content::mission::OpeningMission,
        &mut RandomState,
    ),
    actor: &mut FieldActor<'_>,
    collision: &crate::combat::spells::FieldCollision,
) -> Result<Vec<Event>, RuleError> {
    super::live_arm(
        c,
        w,
        (mission, random),
        (
            crate::combat::spells::detached::Arm::Movement,
            actor.selected_warrior,
        ),
        (collision.id, &collision.packet, collision.count),
        &mut *actor.movement_gate,
        (None, None),
        None,
    )
}
