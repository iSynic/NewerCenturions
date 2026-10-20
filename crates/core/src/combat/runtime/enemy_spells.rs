//! Enemy spells in the original combat execution order.
use super::{EnemyWork, resolve_spell_mask, spell_identity};
use crate::combat::Combat;
use crate::{Event, RandomState, RuleError};
use centurions_content::{Warrior, word};

pub(super) fn refresh_enemy_item_alias(
    c: &Combat,
    enemy: usize,
    items: &[Vec<u8>],
    work: &mut EnemyWork,
) -> Result<Option<i16>, RuleError> {
    let selected = c.enemies.records[enemy].record().selected_item()?;
    if selected == 0 {
        return Ok(None);
    }
    work.item_context = Some(work.buffers.load_item_from(items, selected)?);
    let index = usize::try_from(selected.wrapping_abs())
        .map_err(|_| RuleError::Invalid("invalid NPC selected item".into()))?;
    let item = items
        .get(index)
        .ok_or_else(|| RuleError::Invalid("NPC selected item missing".into()))?;
    Ok(Some(word(item, 0x32)?))
}

pub(super) fn enemy_spell(
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
    let attempts = c.enemies.records[e].bytes[0x7e] as i8;
    let mut cast = false;
    while work.spell_attempt < attempts {
        let Some((id, packet)) =
            crate::combat::spells::select_packet(&c.enemies.records[e], &tables.attacks, random)?
        else {
            break;
        };
        c.attack_packet = Some(packet.clone());
        c.attack_identity = Some(spell_identity(id));
        if matches!(packet[1], 9 | 10 | 12) {
            let result = crate::combat::spells::global_targets(c, w, e, &packet)?;
            c.enemies.records[e] = result.caster;
            let alias = refresh_enemy_item_alias(c, e, items, work)?;
            let memory = crate::combat::spells::RestrictionMemoryView::ordinary_startup(
                &mission.item_cache,
                alias,
            );

            events.extend(resolve_spell_mask(
                c,
                w,
                random,
                (id, &packet, work.attack_count),
                &result.mask,
                body,
                memory,
            )?);
            cast = true;
            work.spell_attempt = work.spell_attempt.wrapping_add(1);
            continue;
        }
        let selection = crate::combat::spells::select_targets(
            c,
            w,
            mission,
            &mission.areas,
            e,
            &packet,
            random,
        )?;
        work.attack_count = selection.count;
        if selection.targets.is_empty() {
            break;
        }
        c.enemies.records[e] = selection.caster;
        let alias = refresh_enemy_item_alias(c, e, items, work)?;
        let memory = crate::combat::spells::RestrictionMemoryView::ordinary_startup(
            &mission.item_cache,
            alias,
        );

        work.attack_count = work.attack_count.max(1);
        let mut origin = selection.targets[0].clone();
        for target in selection.targets {
            let context = crate::combat::spells::SpellContext {
                packet: &packet,
                id,
                count: work.attack_count,
                caster: body,
                initiative: c.cursor as i16,
                collision: false,
            };
            events.push(Event::Sound {
                resource_id: word(&packet, 36)?,
            });
            let mask = crate::combat::spells::target_mask_rotated_with_origin(
                c,
                w,
                &mission.areas,
                (&target, &origin),
                &context,
                &c.spell_fields,
                random,
                selection.rotation,
            )?;
            origin.first = -1;
            origin.second = -1;
            c.spell_fields = mask.fields;
            events.extend(resolve_spell_mask(
                c,
                w,
                random,
                (id, &packet, work.attack_count),
                &mask.mask,
                body,
                memory,
            )?);
        }
        cast = true;
        work.spell_attempt = work.spell_attempt.wrapping_add(1);
    }
    Ok(cast)
}
