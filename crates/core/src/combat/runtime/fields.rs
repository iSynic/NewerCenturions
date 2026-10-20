//! Fields in the original combat execution order.
use super::{append_spell_outcomes, increment_stat, spell_identity};
use crate::combat::Combat;
use crate::{Event, RandomState, RuleError};
use centurions_content::Warrior;

pub fn collide_fields(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    count: &mut i16,
    random: &mut RandomState,
    body: i16,
) -> Result<Vec<Event>, RuleError> {
    collide_fields_with_mask(c, w, mission, count, random, body, &mut [0; 6])
}

pub fn collide_fields_with_mask(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    count: &mut i16,
    random: &mut RandomState,
    body: i16,
    party_mask: &mut [i16; 6],
) -> Result<Vec<Event>, RuleError> {
    let mut next = c.clone();
    let mut next_warriors = w.to_vec();
    let mut next_count = *count;
    let mut next_random = random.clone();
    let mut next_mask = *party_mask;
    let events = collide_fields_impl(
        &mut next,
        &mut next_warriors,
        mission,
        &mut next_count,
        &mut next_random,
        FieldActor {
            body,
            selected_warrior: None,
        },
        &mut next_mask,
    )?;
    *c = next;
    w.clone_from_slice(&next_warriors);
    *count = next_count;
    *random = next_random;
    *party_mask = next_mask;
    Ok(events)
}

pub(super) struct FieldActor {
    pub(super) body: i16,
    pub(super) selected_warrior: Option<usize>,
}

pub(super) fn collide_fields_impl(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    count: &mut i16,
    random: &mut RandomState,
    actor: FieldActor,
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
        *count = collision.count;
        c.attack_packet = Some(collision.packet.clone());
        let mut identity = spell_identity(collision.id);
        identity[1] = collision.level;
        c.attack_identity = Some(identity);
        let context = crate::combat::spells::SpellContext {
            packet: &collision.packet,
            id: collision.id,
            count: *count,
            caster: body,
            initiative: c.cursor as i16,
            collision: true,
        };
        let mask = crate::combat::spells::target_mask_rotated(
            c,
            w,
            &mission.areas,
            &collision.target,
            &context,
            &c.spell_fields,
            random,
            collision.rotation,
        )?;
        party_mask.copy_from_slice(&mask.mask[..6]);
        let previous_hp = w
            .iter()
            .map(Warrior::stamina)
            .collect::<Result<Vec<_>, _>>()?;
        let result = crate::combat::spells::resolve_collision_with_item_cache(
            w,
            &c.enemies.records,
            &mask.mask,
            &crate::combat::spells::DirectSpell {
                packet: &collision.packet,
                level: collision.level,
                count: *count,
                armor_filter: c.weapon_work.armor_filter()?,
                weapon_sound: c.weapon_work.sound()?,
            },
            &mut usage,
            &mission.item_cache,
            random,
        )?;
        w.clone_from_slice(&result.warriors);
        c.enemies.records = result.enemies;
        if let Some(member) = actor.selected_warrior
            && c.active_body()
                .is_some_and(|active| (0..6).contains(&active))
        {
            let warrior = w
                .get_mut(member)
                .ok_or_else(|| RuleError::Invalid("field credit warrior missing".into()))?;
            for (target, outcome) in &result.targets {
                if *target >= 10 && outcome.damage > 0 {
                    increment_stat(&mut warrior.bytes, 0xa46, i32::from(outcome.damage));
                    if collision.packet[19] == 9 {
                        increment_stat(&mut warrior.bytes, 0xa4a, 1);
                    }
                    if outcome.killed {
                        increment_stat(&mut warrior.bytes, 0xa5a, 1);
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
            actor.selected_warrior.is_some(),
            &mut events,
        )?;
    }
    c.field_used = usage.used;
    Ok(events)
}
