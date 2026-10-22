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
    let weapon = items
        .get(weapon_id as usize)
        .ok_or(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "weapon content missing",
        })?;
    let attacks = &mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat content missing".into()))?
        .attacks;
    let item_context = match work.item_context {
        Some(context) => context,
        None if c.enemies.records[e].record().ammunition()? == 0 => 0,
        None => {
            return Err(RuleError::Unsupported {
                action: "combat.enemy_firearm",
                dependency: "unreconstructable legacy work-item context",
            }
            .with_context(|| {
                // The guard above has already read this field successfully.
                let ammunition = c.enemies.records[e].record().ammunition().unwrap_or(0);
                format!("enemy body {body} weapon {weapon_id} ammunition {ammunition}")
            }));
        }
    };
    let ammo_id = crate::combat::enemy_weapons::ammo_id(&c.enemies.records[e], item_context)?;
    let ammo = items
        .get(ammo_id.unsigned_abs() as usize)
        .ok_or_else(|| RuleError::Invalid("enemy ammunition outside content".into()))?;
    work.buffers.prepare_pair(weapon, ammo)?;
    let mut prepared = if weapon_id == 0 {
        let packet = c.attack_packet.clone().ok_or(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "unreconstructable retained attack packet",
        })?;
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
    } else if matches!(
        ItemDefinition(weapon).spell()?,
        firearm::POINT_TARGET_ATTACK | firearm::AREA_TARGET_ATTACK
    ) {
        crate::combat::enemy_weapons::prepare(&mut c.enemies.records[e], weapon, ammo, attacks)?
    } else {
        firearm::prepare_enemy(&mut c.enemies.records[e], weapon, attacks)?
    };
    if weapon_id != 0 {
        c.weapon_work.prepare(weapon, ammo)?;
        c.attack_packet = Some(prepared.packet.clone());
        c.attack_identity = Some(spell_identity(ItemDefinition(weapon).spell()?));
        work.attack_count = prepared.shots;
    }
    prepared.armor_filter = c.weapon_work.armor_filter()?;
    prepared.sound = c.weapon_work.sound()?;
    let selection = if weapon_id == 0 {
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
        crate::combat::spells::select_firearm_targets(
            c,
            w,
            mission,
            &mission.areas,
            e,
            &prepared,
            random,
        )?
    };
    work.attack_count = selection.count;
    if !selection.cast {
        return Ok(FireOutcome::Failed);
    }
    c.inactive_rounds = 0;
    c.enemies.records[e] = selection.caster;
    if work.display_writers {
        refresh_enemy_item_alias(c, e, items, work)?;
    }
    let physical = AttackPacket(&prepared.packet).plays_impact_when_wounded();
    work.attack_count = if physical {
        1
    } else {
        work.attack_count.max(1)
    };
    prepared.shots = work.attack_count;
    let id = if weapon_id == 0 {
        let identity = c.attack_identity.ok_or(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "unreconstructable retained spell identity",
        })?;
        (identity[0] + 1) * 1000 + (identity[1] + 1) * 100 + identity[2] + 1
    } else {
        ItemDefinition(weapon).spell()?
    };
    if AttackPacket(&prepared.packet).has_randomized_animation() {
        random.bound(4);
    }
    let mut origin = selection.targets.first().cloned();
    for target in selection.targets {
        if !AttackPacket(&prepared.packet).requires_area_mask() {
            let target_body = target.first;
            if !alive(c, w, target_body) {
                continue;
            }
        }
        events.push(Event::Sound {
            resource_id: AttackPacket(&prepared.packet).launch_sound()?,
        });
        let context = crate::combat::spells::SpellContext {
            packet: &prepared.packet,
            id,
            count: work.attack_count,
            caster: body,
            initiative: c.cursor as i16,
            collision: false,
        };
        let mask = crate::combat::spells::target_mask_with_origin(
            c,
            w,
            &mission.areas,
            (&target, origin.as_ref().unwrap()),
            &context,
            &c.spell_fields,
            random,
        )?;
        if let Some(origin) = origin.as_mut() {
            origin.first = -1;
            origin.second = -1;
        }
        c.spell_fields = mask.fields;
        if !physical {
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
                ),
            )?);
            continue;
        }
        let (minimum, maximum) = AttackPacket(&prepared.packet).duration_bounds();
        random.range(minimum, maximum);
        for (target, flag) in mask.mask.iter().enumerate() {
            if *flag <= 0 {
                continue;
            }
            let result = if target < 6 {
                firearm::resolve_enemy_after_duration(
                    &mut w[target],
                    &mut c.enemies.records[e],
                    &prepared,
                    random,
                    c.difficulty,
                )?
            } else if target >= 10 {
                let caster = c.enemies.records[e].clone();
                crate::combat::spells::resolve_enemy_firearm_body_after_duration(
                    &mut c.enemies.records[target - 10],
                    &caster,
                    &prepared,
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
            });
        }
    }
    if physical {
        Ok(FireOutcome::Physical)
    } else {
        work.spell_attempt = work.spell_attempt.wrapping_add(1);
        enemy_spell_continue(c, w, mission, random, (items, work), body, events)?;
        Ok(FireOutcome::Spell)
    }
}
