//! Attacks in the original combat execution order.
use super::{has_active_warrior, is_melee_weapon, load_melee_work, load_natural_work, position};
use crate::combat::warrior_attack::{self, WarriorMelee};
use crate::combat::{Combat, attack, lifecycle};
use crate::{Event, RandomState, RuleError};
use centurions_content::Warrior;

#[derive(Default)]
pub(super) struct AttackOutcome {
    pub events: Vec<Event>,
    pub returned: bool,
    pub terminated: bool,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_attack_with_death(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    attacker: i16,
    recipient: i16,
    attack_index: usize,
    retained_actor: i16,
    items: Option<&[Vec<u8>]>,
    attacks: Option<&[u8]>,
    buffs: &[u8],
    buffers: &mut crate::ammunition::WorkBuffers,
    death_processing: Option<crate::combat::death_scripts::DeathProcessing>,
) -> Result<Vec<Event>, RuleError> {
    resolve_attack_result(
        combat,
        warriors,
        random,
        attacker,
        recipient,
        attack_index,
        retained_actor,
        items,
        attacks,
        buffs,
        buffers,
        death_processing,
    )
    .map(|outcome| outcome.events)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_attack_result(
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
    death_processing: Option<crate::combat::death_scripts::DeathProcessing>,
) -> Result<AttackOutcome, RuleError> {
    let presents_result = attacker >= 10
        || warriors[attacker as usize].record().attacks_used()?
            < warriors[attacker as usize].record().attack_limit()?;
    if presents_result {
        combat.npc_inhibit_write = Some(1);
    }
    if !has_active_warrior(warriors) {
        combat.victory_work.finish();
        if attacker >= 10 {
            combat.attack_packet = if let Some(attacks) = attacks {
                attack::attack_packet(attacks)?
            } else {
                attack::default_attack_packet()
            };
        }
        return Ok(AttackOutcome::default());
    }
    combat.inactive_rounds = 0;
    let retained = combat.attack_packet.clone();
    // Warrior turns leave WORD447196 at the most recently selected NPC.
    let retained_actor = if retained_actor < 10 {
        combat.selected_enemy as i16 + 10
    } else {
        retained_actor
    };
    let mut death_handled = false;
    let result = if attacker < 10 {
        let weapon =
            warrior_attack::weapon(&warriors[attacker as usize].bytes, items.unwrap_or(&[]))?;
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
        match attacks {
            None if recipient >= 10 => attack::unarmed_attack(
                &mut warriors[attacker as usize],
                &mut combat.enemies.records[(recipient - 10) as usize],
                random,
                combat.difficulty,
            )?,
            attacks => match warrior_attack::warrior_melee(
                combat,
                warriors,
                random,
                attacker,
                recipient,
                items.unwrap_or(&[]),
                attacks.ok_or("warrior melee attack table missing")?,
                &retained,
                buffs,
            )? {
                WarriorMelee::Halted { packet, warned } => {
                    return Ok(AttackOutcome {
                        events: halted_spell_weapon(combat, packet, warned),
                        returned: false,
                        terminated: false,
                    });
                }
                WarriorMelee::Struck {
                    result,
                    packet,
                    target,
                } => {
                    recipient = target;
                    if let Some(packet) = packet {
                        combat.attack_packet = packet;
                    }
                    result
                }
            },
        }
    } else if recipient < 10 {
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
            combat.attack_packet = packet;
            result
        } else {
            load_natural_work(combat, items, buffers)?;
            let (result, packet, reflected, total_delta) = attack::natural_attack_with_target(
                &mut warriors[recipient as usize],
                &mut combat.enemies.records[(attacker - 10) as usize],
                attack_index,
                random,
                attacks,
                &retained,
                buffs,
                combat.difficulty,
            )?;
            combat.victory_work.add_total(total_delta);
            if !reflected {
                combat.unavailable = combat.unavailable.wrapping_add(total_delta as i8);
            }
            if reflected {
                recipient = attacker;
            }
            combat.attack_packet = packet;
            result
        }
    } else {
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
            combat.attack_packet = packet;
            result
        } else {
            load_natural_work(combat, items, buffers)?;
            let (result, packet, reflected) =
                crate::combat::ally_turns::natural_attack_enemy_with_processing(
                    combat,
                    warriors,
                    random,
                    (attacker, recipient, attack_index, retained_actor),
                    attacks,
                    &retained,
                    death_processing,
                )?;
            if reflected {
                recipient = attacker;
            }
            combat.attack_packet = packet;
            // The NPC natural adapter owns killbody, including reflection.
            death_handled = result.killed;
            result
        }
    };
    if presents_result {
        combat.neighbor_work.departure = 0;
    }
    if result.killed && !death_handled {
        let direct = combat.direct_deaths;
        combat.direct_deaths &= attacker < 6;
        lifecycle::kill_body_with_processing(combat, warriors, recipient, death_processing)?;
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
        icon: crate::event::PresentationIcon(result.icon),
        flight: Default::default(),
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
    if death_processing.is_some() && result.killed {
        if recipient >= 10 {
            events.extend(super::death::enemy_death_feedback(
                &combat.enemies.records[(recipient - 10) as usize],
                recipient,
            )?);
        }
        if combat
            .direct_death
            .as_ref()
            .is_none_or(|(body, _)| *body != recipient)
        {
            events.push(Event::Delay { ticks: 15 });
        }
    }
    Ok(AttackOutcome {
        events,
        returned: result.killed,
        terminated: false,
    })
}

/// A halted spell-weapon attack leaves loadspell2's packet and no strike events.
fn halted_spell_weapon(combat: &mut Combat, packet: Vec<u8>, warned: bool) -> Vec<Event> {
    combat.attack_packet = packet;
    if warned {
        vec![Event::Feedback(crate::feedback::NativeFeedback::Warning {
            index: 39,
        })]
    } else {
        vec![]
    }
}
