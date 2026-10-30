//! Spell fields that act on bodies entering or standing in them.
use crate::combat::map_alias::MapAliases;
use crate::combat::spells;
use crate::combat::spells::DirectSpell;
use crate::combat::spells::FieldCollision;
use crate::combat::spells::FieldUsage;
use crate::combat::spells::RestrictionMemoryView;
use crate::combat::spells::SpellContext;
use crate::combat::spells::out_of_combat::Arm;
use crate::combat::spells::summon::ItemRefresh;
use centurions_content::records::{CharacterStatisticsMut, Statistic};

use super::{append_spell_outcomes, spell_identity};
use crate::combat::Combat;
use crate::{Event, RandomState, RuleError};
use centurions_content::Warrior;

/// Resolves the fields under `actor`, the moving body.
///
/// Original: collide
pub fn collide_fields(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    count: &mut i16,
    random: &mut RandomState,
    (body, selected, gate): (i16, usize, &mut i16),
) -> Result<Vec<Event>, RuleError> {
    collide_fields_with_refresh(
        combat,
        warriors,
        mission,
        count,
        random,
        (body, None, selected, gate),
        &mut [0; 6],
        None,
    )
}

/// Resolves the fields under `actor`, the moving body. `items` is the item work
/// refreshed when a full roster refuses a summon.
///
/// Original: collide
#[allow(clippy::too_many_arguments)]
pub(crate) fn collide_fields_with_refresh(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    count: &mut i16,
    random: &mut RandomState,
    actor: (i16, Option<i16>, usize, &mut i16),
    party_track: &mut [i16; 6],
    items: ItemRefresh<'_>,
) -> Result<Vec<Event>, RuleError> {
    if actor.2 >= warriors.len() {
        return Err("field collision for an inactive selected warrior".into());
    }
    let mut next_gate = *actor.3;
    let mut next = combat.clone();
    let mut next_warriors = warriors.to_vec();
    let mut next_count = *count;
    let mut next_random = random.clone();
    let mut next_mask = *party_track;
    let mut next_buffers = items.as_ref().map(|(_, buffers)| (**buffers).clone());
    let events = collide_fields_impl(
        &mut next,
        &mut next_warriors,
        mission,
        &mut next_count,
        &mut next_random,
        FieldActor {
            body: actor.0,
            char_select_new: actor.2,
            item_restriction: actor.1,
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
    *combat = next;
    warriors.clone_from_slice(&next_warriors);
    *count = next_count;
    *random = next_random;
    *party_track = next_mask;
    Ok(events)
}

/// The body whose cell is checked for fields, with the selected warrior, the item restriction and
/// the movement-spell marker its effects read.
pub(super) struct FieldActor<'a> {
    pub(super) body: i16,
    pub(super) char_select_new: usize,
    pub(super) item_restriction: Option<i16>,
    /// The movement-spell marker.
    pub(super) movement_gate: &'a mut i16,
    /// The item work refreshed when a full roster refuses a summon.
    pub(super) items: ItemRefresh<'a>,
}

/// Resolves every spell field that reaches the body in its cell, one slot at a time. The field's
/// centre becomes the first target; a summon field summons at its centre, a move field moves the
/// selected body, any other casts its spell on the targets it covers. Nothing happens while no
/// field is active.
///
/// Original: collide
pub(super) fn collide_fields_impl(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    count: &mut i16,
    random: &mut RandomState,
    mut actor: FieldActor<'_>,
    party_track: &mut [i16; 6],
) -> Result<Vec<Event>, RuleError> {
    let body = actor.body;
    if !combat
        .spell_queue
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
    let mut usage = FieldUsage {
        slot: 0,
        used: combat.collide_check.clone(),
    };
    for slot in 0..60 {
        usage.slot = slot;
        let Some(collision) = spells::collision_field(
            combat,
            warriors,
            &combat.spell_queue,
            &usage,
            body as usize,
            &tables.attacks,
        )?
        else {
            continue;
        };
        // The field's centre becomes the first target.
        combat.target_picks.clear();
        combat
            .target_picks
            .write(std::slice::from_ref(&collision.target));
        *count = collision.count;
        combat.spell_info = collision.packet.clone();
        let mut identity = spell_identity(collision.id);
        identity[1] = collision.level;
        combat.cast_identity = identity;
        // in_spell is set while the field resolves, so a death it causes queues its
        // story; in_spell is clear afterwards. A summon field summons at its centre.
        if centurions_content::records::SpellInfoRecord(&collision.packet)
            .effect()?
            .wrapping_abs()
            == 58
        {
            combat.in_spell = 1;
            events.extend(super::live_arm(
                combat,
                warriors,
                (mission, random),
                (Arm::Summon, actor.char_select_new),
                (collision.id, &collision.packet, *count),
                &mut *actor.movement_gate,
                (None, None),
                actor
                    .items
                    .as_mut()
                    .map(|(items, buffers)| (*items, &mut **buffers)),
            )?);
            combat.in_spell = 0;
            combat.target_picks.resolved(0);
            continue;
        }
        let context = SpellContext {
            packet: &collision.packet,
            id: collision.id,
            count: *count,
            caster: body,
            initiative: combat.battle_order_index as i16,
            collision: true,
        };
        let mask = spells::target_mask_rotated_with_origin(
            combat,
            warriors,
            &mission.areas,
            (&collision.target, &collision.target),
            &context,
            &combat.spell_queue,
            random,
            (collision.rotation, MapAliases::of_mission(mission)),
        )?;
        party_track.copy_from_slice(&mask.mask[..6]);
        // The field's target marks stay until a later cast clears them.
        combat.track_tail = mask.mask[6..].to_vec();
        let previous_hp = warriors
            .iter()
            .map(Warrior::stamina)
            .collect::<Result<Vec<_>, _>>()?;
        combat.mark_monster_spell_pick(&collision.packet, &mask.mask);
        combat.in_spell = 1;
        let effect = centurions_content::records::SpellInfoRecord(&collision.packet)
            .effect()?
            .wrapping_abs();
        if matches!(effect, 55 | 56) {
            events.extend(move_selected(
                combat,
                warriors,
                (mission, random),
                &mut actor,
                &collision,
            )?);
            combat.in_spell = 0;
            combat.target_picks.resolved(0);
            continue;
        }
        let memory =
            RestrictionMemoryView::ordinary_startup(&mission.item_cache, actor.item_restriction)
                .with_tables(mission)
                .with_char_up(actor.char_select_new as i16)
                .with_live_caster(combat, warriors);
        let result = spells::resolve_collision_with_memory(
            warriors,
            &combat.roster.records,
            &mask.mask,
            &DirectSpell {
                packet: &collision.packet,
                level: collision.level,
                count: *count,
                armor_filter: combat.weapon_traits.armor_filter(),
                weapon_sound: combat.weapon_traits.sound(),
            },
            &mut usage,
            (combat.active_body(), memory),
            random,
        )?;
        combat.kill_party = combat.kill_party.wrapping_sub(result.recovered as i8);
        if result.healed {
            centurions_content::records::SpellInfoRecordMut(&mut combat.spell_info)
                .set_health_effect(4);
        }
        let result = result.body;
        warriors.clone_from_slice(&result.warriors);
        combat.kill_counts.add_num_enemy(result.total_delta);
        combat.roster.records = result.enemies;
        if combat
            .active_body()
            .is_some_and(|active| (0..6).contains(&active))
        {
            let member = actor.char_select_new;
            let warrior = warriors
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
            combat,
            warriors,
            result.targets,
            result.race_excluded,
            (None, collision.id, &collision.packet, &previous_hp),
            true,
            &mut events,
        )?;
        combat.in_spell = 0;
        combat.target_picks.resolved(0);
    }
    combat.collide_check = usage.used;
    Ok(events)
}

/// A movement field moves the acting warrior (charup) to its centre, after the
/// duration draws and the movement-marker test. No body is resolved and the field's
/// usage is not marked.
///
/// Original: resolvespell
fn move_selected(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    (mission, random): (
        &centurions_content::mission::OpeningMission,
        &mut RandomState,
    ),
    actor: &mut FieldActor<'_>,
    collision: &FieldCollision,
) -> Result<Vec<Event>, RuleError> {
    super::live_arm(
        combat,
        warriors,
        (mission, random),
        (Arm::Movement, actor.char_select_new),
        (collision.id, &collision.packet, collision.count),
        &mut *actor.movement_gate,
        (None, None),
        None,
    )
}
