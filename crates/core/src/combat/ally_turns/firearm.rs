//! Friendly enemy-record firearm caller. Its range gate considers the furthest
//! living opponent, even when another visible opponent is in range. Keep this
//! distinct from player shooting and the hostile enemy caller.
use super::{alive, faction, position, result_audio};
use crate::combat::{Combat, damage, enemy_weapons, firearm, lifecycle, runtime};
use crate::{Event, RandomState, RuleError};
use centurions_content::{
    Warrior,
    records::{AttackPacket, EnemyRecord, ItemDefinition},
};
use damage::{DamageFlags, DamagePacket, resolve_damage};

fn packet(weapon: &[u8], ammo: &[u8], attacks: &[u8]) -> Result<Vec<u8>, RuleError> {
    use centurions_content::records::{WeaponAttackModifiers, WeaponAttackParameters};
    let mut packet = centurions_content::span(attacks, 3 * 2842, 58)?.to_vec();
    let mut parameters = WeaponAttackParameters::read(&packet)?;
    firearm::compose_weapon_attack(
        &mut parameters,
        &WeaponAttackModifiers::read(weapon)?,
        &WeaponAttackModifiers::read(ammo)?,
        0,
    );
    parameters.write(&mut packet);
    Ok(packet)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn fire(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    events: &mut Vec<Event>,
    actor: (i16, i16),
    buffers: &mut crate::ammunition::WorkBuffers,
    display_writers: bool,
    attack_count: &mut i16,
) -> Result<bool, RuleError> {
    let (item_context, body) = actor;
    let a = (body - 10) as usize;
    let source_bytes = c.enemies.records[a].bytes.clone();
    let source = EnemyRecord::from_bytes(&source_bytes);
    let weapon_id = source.primary_weapon()?;
    if weapon_id != 1 {
        return Err(RuleError::Unsupported {
            action: "combat.scientist_firearm",
            dependency: "ally rifle caller requires firearm one",
        });
    }
    let weapon = items
        .get(weapon_id as usize)
        .ok_or_else(|| RuleError::Invalid("ally weapon missing".into()))?;
    let ammo_id = enemy_weapons::ammo_id(&c.enemies.records[a], item_context)?;
    let ammo = items
        .get(ammo_id.unsigned_abs() as usize)
        .ok_or_else(|| RuleError::Invalid("ally ammunition missing".into()))?;
    let attacks = &mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat content missing".into()))?
        .attacks;
    c.enemies.records[a].record_mut().set_selected_item(1);
    let p = packet(weapon, ammo, attacks)?;
    c.weapon_work.prepare(weapon, ammo)?;
    buffers.prepare_pair(weapon, ammo)?;
    *attack_count = AttackPacket(&p).attack_count();
    c.attack_packet = Some(p.clone());
    c.attack_identity = Some([3, 0, 0]);
    random.bound(7);
    let shots = i16::from(ItemDefinition(weapon).selected_attack_count());
    let range = AttackPacket(&p)
        .attack_count()
        .wrapping_mul(shots)
        .wrapping_add(AttackPacket(&p).range_modifier())
        .wrapping_abs();
    let start = position(c, body);
    let mut distances = [127_i16; 110];
    let mut count = 0;
    let mut maximum = 0;
    for target in 0..10 + c.enemies.records.len() as i16 {
        if !alive(c, w, target) || faction(c, w, body)? == faction(c, w, target)? {
            continue;
        }
        let end = position(c, target);
        let dx = f32::from(start.0 - end.0);
        let dy = f32::from(start.1 - end.1);
        let distance = (dx * dx + dy * dy).sqrt().trunc() as i16;
        maximum = maximum.max(distance);
        if distance <= range && lifecycle::can_see(c, mission, start, end, 0)? {
            distances[target as usize] = distance;
            count += 1;
        }
    }
    if count == 0 || range < maximum {
        return Ok(false);
    }
    let mut selected = None;
    for _ in 0..100 {
        let target = (random.bound(c.enemies.records.len() as i16 + 10) - 1) as usize;
        if distances[target] <= range {
            selected = Some(target);
            break;
        }
    }
    let Some(target) = selected else {
        return Ok(false);
    };
    let actor = &mut c.enemies.records[a];
    let action_points = actor.record().action_points().wrapping_sub(2);
    let movement_points = actor.record().movement_points().wrapping_sub(3);
    actor.record_mut().set_action_points(action_points);
    actor.record_mut().set_movement_points(movement_points);
    actor.record_mut().set_reaction_ready(action_points >= 1);
    if display_writers {
        runtime::refresh_body_item(c, w, items, body, buffers)?;
    }
    events.push(Event::Sound {
        resource_id: AttackPacket(&p).launch_sound()?,
    });
    let (minimum_duration, maximum_duration) = AttackPacket(&p).duration_bounds();
    random.range(minimum_duration, maximum_duration);
    if target < 6 {
        let prepared = firearm::PreparedFirearm {
            sound: AttackPacket(&p).impact_sound()?,
            packet: p,
            slot: 0,
            shots: 1,
            rounds: ItemDefinition(weapon).rounds_per_shot()?,
            weight: 0,
            armor_filter: c.weapon_work.armor_filter()?,
        };
        let result = firearm::resolve_enemy_after_duration(
            &mut w[target],
            &mut c.enemies.records[a],
            &prepared,
            random,
            c.difficulty,
        )?;
        if result.killed {
            lifecycle::kill_body(c, w, target as i16)?;
        }
        events.extend(
            result
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
        if result.damage != 0 {
            if !result.killed {
                events.push(Event::Sound {
                    resource_id: result_audio(
                        &c.enemies.records[a],
                        0,
                        result.damage,
                        prepared.sound,
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
        return Ok(true);
    }
    let t = target - 10;
    let victim = c.enemies.records[t].record();
    let mut hit = 33_i16;
    if c.active_body().is_some_and(|actor| actor > 9) {
        hit = hit
            .wrapping_add(source.level()?.wrapping_mul(5))
            .wrapping_add(source.missile_accuracy_bonus()?);
    }
    if victim.turn_skip_duration() == 0 {
        hit = hit
            .wrapping_sub(victim.missile_dodge_bonus()?)
            .wrapping_sub(random.bound(victim.speed()? / 3))
            .wrapping_sub(victim.level()?.wrapping_mul(3))
            .wrapping_sub(i16::from(victim.guarding()).wrapping_mul(victim.level()?));
        hit = hit.wrapping_add(match victim.size()?.wrapping_abs() {
            0 => -5,
            1 | 2 => 5,
            3 => 10,
            _ => 0,
        });
        if random.bound(100)
            > hit.clamp(
                15_i16.wrapping_sub(c.difficulty.wrapping_mul(5)),
                90_i16.wrapping_sub(c.difficulty.wrapping_mul(5)),
            )
        {
            return Ok(true);
        }
    }
    let result = resolve_damage(
        &mut c.enemies.records[t].bytes,
        true,
        &DamagePacket {
            bytes: p.clone(),
            attack_count: shots,
        },
        DamageFlags {
            shield: true,
            armor: true,
            saves: true,
            ..Default::default()
        },
        random,
    )?;
    if result.shield_sound {
        events.push(Event::Sound { resource_id: 206 });
    }
    if result.total != 0 {
        let hp = c.enemies.records[t]
            .record()
            .stamina()?
            .wrapping_sub(result.total);
        c.enemies.records[t].record_mut().set_stamina(hp);
        if hp < 1 {
            lifecycle::kill_body(c, w, target as i16)?;
        } else {
            events.push(Event::Sound {
                resource_id: AttackPacket(&p).impact_sound()?,
            });
            events.push(Event::Sound {
                resource_id: result_audio(
                    &c.enemies.records[a],
                    0,
                    result.total,
                    AttackPacket(&p).impact_sound()?,
                )?,
            });
        }
        events.push(Event::AttackResolved {
            attacker: body,
            target: target as i16,
            damage: result.total,
            defeated: hp < 1,
        });
    }
    Ok(true)
}
