//! Attacks in the original combat execution order.
use super::{has_active_warrior, is_melee_weapon, load_melee_work, load_natural_work, position};
use crate::combat::{Combat, attack, lifecycle};
use crate::{Event, RandomState, RuleError};
use centurions_content::Warrior;

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_attack(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    attacker: i16,
    mut recipient: i16,
    attack_index: usize,
    retained_actor: i16,
    items: Option<&[Vec<u8>]>,
    attacks: Option<&[u8]>,
    buffs: &[u8],
    buffers: &mut crate::ammunition::WorkBuffers,
) -> Result<Vec<Event>, RuleError> {
    if attacker >= 10 && !has_active_warrior(warriors) {
        combat.attack_packet = Some(if let Some(attacks) = attacks {
            attack::attack_packet(attacks)?
        } else {
            attack::default_attack_packet()
        });
        return Ok(vec![]);
    }
    combat.inactive_rounds = 0;
    let previous_faction = if attacker >= 10 && (0..warriors.len() as i16).contains(&recipient) {
        Some((
            recipient as usize,
            warriors[recipient as usize].record().faction()?,
        ))
    } else {
        None
    };
    let retained = combat.attack_packet.clone().unwrap_or_else(|| vec![0; 58]);
    // Warrior turns leave WORD447196 at the most recently selected NPC.
    let retained_actor = if retained_actor < 10 {
        combat.selected_enemy as i16 + 10
    } else {
        retained_actor
    };
    let result = if attacker < 10 && recipient >= 10 {
        let weapon = crate::combat::warrior_attack::weapon(
            &warriors[attacker as usize].bytes,
            items.unwrap_or(&[]),
        )?;
        if warriors[attacker as usize].record().attacks_used()?
            < warriors[attacker as usize].record().attack_limit()?
        {
            if let Some(weapon) = weapon {
                combat.weapon_work.load_weapon(weapon)?;
                buffers.load_weapon(
                    items.unwrap(),
                    warriors[attacker as usize].record().melee_weapon()?,
                )?;
            } else {
                load_natural_work(combat, items, buffers)?;
            }
        }
        if let Some(attacks) = attacks {
            let copies_packet = warriors[attacker as usize].record().attacks_used()?
                < warriors[attacker as usize].record().attack_limit()?;
            let (result, packet, reflected) = attack::warrior_attack_with_packet(
                &mut warriors[attacker as usize],
                &mut combat.enemies.records[(recipient - 10) as usize],
                random,
                attacks,
                &retained,
                combat.difficulty,
                weapon,
            )?;
            if reflected {
                recipient = attacker;
            }
            if copies_packet {
                combat.attack_packet = Some(packet);
            }
            result
        } else {
            attack::unarmed_attack(
                &mut warriors[attacker as usize],
                &mut combat.enemies.records[(recipient - 10) as usize],
                random,
                combat.difficulty,
            )?
        }
    } else if attacker >= 10 && recipient < 10 {
        let attacker_x = position(combat, attacker).0;
        let target_x = position(combat, recipient).0;
        combat.enemies.records[(attacker - 10) as usize]
            .record_mut()
            .face_towards(attacker_x, target_x);
        let melee_id = combat.enemies.records[(attacker - 10) as usize]
            .record()
            .melee_weapon()?;
        if is_melee_weapon(items, melee_id)? {
            let weapon = items.and_then(|items| items.get(melee_id as usize)).ok_or(
                RuleError::Unsupported {
                    action: "combat.enemy_melee",
                    dependency: "melee weapon content missing",
                },
            )?;
            load_melee_work(combat, attacker, weapon, items, buffers)?;
            let (result, packet, reflected) =
                crate::combat::enemy_weapons::melee_warrior_with_target(
                    combat,
                    warriors,
                    random,
                    (attacker, recipient, attack_index, retained_actor),
                    weapon,
                    combat.difficulty,
                    attacks,
                    &retained,
                    buffs,
                )?;
            if reflected {
                recipient = attacker;
            }
            combat.attack_packet = Some(packet);
            result
        } else {
            load_natural_work(combat, items, buffers)?;
            let (result, packet, reflected) = attack::natural_attack_with_target(
                &mut warriors[recipient as usize],
                &mut combat.enemies.records[(attacker - 10) as usize],
                attack_index,
                random,
                attacks,
                &retained,
                buffs,
                combat.difficulty,
            )?;
            if reflected {
                recipient = attacker;
            }
            combat.attack_packet = Some(packet);
            result
        }
    } else if attacker >= 10 && recipient >= 10 {
        let melee_id = combat.enemies.records[(attacker - 10) as usize]
            .record()
            .melee_weapon()?;
        if is_melee_weapon(items, melee_id)? {
            let weapon = items.and_then(|items| items.get(melee_id as usize)).ok_or(
                RuleError::Unsupported {
                    action: "combat.enemy_melee",
                    dependency: "melee weapon content missing",
                },
            )?;
            load_melee_work(combat, attacker, weapon, items, buffers)?;
            let (result, packet, reflected) =
                crate::combat::enemy_weapons::melee_enemy_with_target(
                    combat,
                    random,
                    (attacker, recipient, attack_index, retained_actor),
                    weapon,
                    combat.difficulty,
                    attacks,
                    &retained,
                )?;
            if reflected {
                recipient = attacker;
            }
            combat.attack_packet = Some(packet);
            result
        } else {
            load_natural_work(combat, items, buffers)?;
            let (result, packet, reflected) =
                crate::combat::ally_turns::natural_attack_enemy_with_target(
                    combat,
                    warriors,
                    random,
                    (attacker, recipient, attack_index, retained_actor),
                    attacks,
                    &retained,
                )?;
            if reflected {
                recipient = attacker;
            }
            combat.attack_packet = Some(packet);
            result
        }
    } else {
        let weapon = crate::combat::warrior_attack::weapon(
            &warriors[attacker as usize].bytes,
            items.unwrap_or(&[]),
        )?;
        let copies_packet = warriors[attacker as usize].record().attacks_used()?
            < warriors[attacker as usize].record().attack_limit()?;
        if copies_packet {
            if let Some(weapon) = weapon {
                combat.weapon_work.load_weapon(weapon)?;
                buffers.load_weapon(
                    items.unwrap(),
                    warriors[attacker as usize].record().melee_weapon()?,
                )?;
            } else {
                load_natural_work(combat, items, buffers)?;
            }
        }
        let (result, packet, actual) = crate::combat::warrior_attack::attack(
            warriors,
            attacker as usize,
            recipient as usize,
            attacks.ok_or("warrior melee attack table missing")?,
            items.unwrap_or(&[]),
            combat.difficulty,
            random,
        )?;
        if copies_packet {
            combat.attack_packet = Some(packet);
        }
        recipient = actual as i16;
        result
    };
    if let Some((member, 0)) = previous_faction
        && warriors[member].record().faction()? != 0
    {
        combat.unavailable = combat.unavailable.wrapping_add(1);
    }
    if result.killed {
        let direct = combat.direct_deaths;
        combat.direct_deaths &= attacker < 6;
        lifecycle::kill_body(combat, warriors, recipient)?;
        combat.direct_deaths = direct;
    }
    let feedback = result.feedback.map(|feedback| {
        let attack::AttackFeedback {
            code,
            amount,
            recipient: feedback_recipient,
        } = feedback;
        Event::Feedback(crate::feedback::NativeFeedback::CombatResult {
            body: if feedback_recipient == attack::AttackRecipient::Attacker {
                attacker
            } else {
                recipient
            },
            code,
            amount,
            audible: crate::feedback::combat_result(code)
                .is_some_and(|(_, sound)| !result.sounds.contains(&sound)),
        })
    });
    let mut events: Vec<Event> = result
        .sounds
        .into_iter()
        .map(|resource_id| Event::Sound { resource_id })
        .collect();
    events.extend(feedback);
    events.push(Event::AttackResolved {
        attacker,
        target: recipient,
        damage: result.damage,
        defeated: result.killed,
    });
    if attacker >= 10 {
        let melee_id = combat.enemies.records[(attacker - 10) as usize]
            .record()
            .melee_weapon()?;
        let impact = if combat.enemies.records[(attacker - 10) as usize]
            .record()
            .selected_item()?
            != 0
            && is_melee_weapon(items, melee_id)?
        {
            centurions_content::records::ItemDefinition(&items.unwrap()[melee_id as usize])
                .impact_sound()?
        } else {
            combat.enemies.records[(attacker - 10) as usize]
                .record()
                .natural_attack(attack_index)
                .impact_sound
        };
        events.push(Event::Sound {
            resource_id: if result.damage == 0 { 650 } else { impact },
        });
    } else if result.damage == 0 {
        events.push(Event::Sound { resource_id: 650 });
    } else if !result.killed {
        events.push(Event::Sound {
            resource_id: warriors[attacker as usize].record().melee_hit_sound()?,
        });
    }
    Ok(events)
}
