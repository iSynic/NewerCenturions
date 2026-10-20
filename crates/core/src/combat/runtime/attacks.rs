//! Attacks in the original combat execution order.
use super::{has_active_warrior, is_melee_weapon, load_melee_work, load_natural_work, position};
use crate::combat::{Combat, attack, lifecycle};
use crate::{Event, RandomState, RuleError};
use centurions_content::{Warrior, word};

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_attack(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    a: i16,
    mut b: i16,
    index: usize,
    actor: i16,
    items: Option<&[Vec<u8>]>,
    attacks: Option<&[u8]>,
    buffs: &[u8],
    buffers: &mut crate::ammunition::WorkBuffers,
) -> Result<Vec<Event>, RuleError> {
    if a >= 10 && !has_active_warrior(w) {
        c.attack_packet = Some(if let Some(attacks) = attacks {
            attack::attack_packet(attacks)?
        } else {
            attack::default_attack_packet()
        });
        return Ok(vec![]);
    }
    c.inactive_rounds = 0;
    let previous_faction = if a >= 10 && (0..w.len() as i16).contains(&b) {
        Some((b as usize, w[b as usize].record().faction()?))
    } else {
        None
    };
    let retained = c.attack_packet.clone().unwrap_or_else(|| vec![0; 58]);
    // Warrior turns leave WORD447196 at the most recently selected NPC.
    let actor = if actor < 10 {
        c.selected_enemy as i16 + 10
    } else {
        actor
    };
    let result = if a < 10 && b >= 10 {
        let weapon =
            crate::combat::warrior_attack::weapon(&w[a as usize].bytes, items.unwrap_or(&[]))?;
        if w[a as usize].record().attacks_used()? < w[a as usize].record().attack_limit()? {
            if let Some(weapon) = weapon {
                c.weapon_work.load_weapon(weapon)?;
                buffers.load_weapon(items.unwrap(), w[a as usize].record().melee_weapon()?)?;
            } else {
                load_natural_work(c, items, buffers)?;
            }
        }
        if let Some(attacks) = attacks {
            let copies_packet =
                w[a as usize].record().attacks_used()? < w[a as usize].record().attack_limit()?;
            let (result, packet, reflected) = attack::warrior_attack_with_packet(
                &mut w[a as usize],
                &mut c.enemies.records[(b - 10) as usize],
                random,
                attacks,
                &retained,
                c.difficulty,
                weapon,
            )?;
            if reflected {
                b = a;
            }
            if copies_packet {
                c.attack_packet = Some(packet);
            }
            result
        } else {
            attack::unarmed_attack(
                &mut w[a as usize],
                &mut c.enemies.records[(b - 10) as usize],
                random,
                c.difficulty,
            )?
        }
    } else if a >= 10 && b < 10 {
        let attacker_x = position(c, a).0;
        let target_x = position(c, b).0;
        let facing = &mut c.enemies.records[(a - 10) as usize].bytes[0x107];
        if attacker_x < target_x && *facing == 0 {
            *facing = 1;
        } else if target_x < attacker_x && *facing == 1 {
            *facing = 0;
        }
        let melee_id = c.enemies.records[(a - 10) as usize]
            .record()
            .melee_weapon()?;
        if is_melee_weapon(items, melee_id)? {
            let weapon = items.and_then(|items| items.get(melee_id as usize)).ok_or(
                RuleError::Unsupported {
                    action: "combat.enemy_melee",
                    dependency: "melee weapon content missing",
                },
            )?;
            load_melee_work(c, a, weapon, items, buffers)?;
            let (result, packet, reflected) =
                crate::combat::enemy_weapons::melee_warrior_with_target(
                    c,
                    w,
                    random,
                    (a, b, index, actor),
                    weapon,
                    c.difficulty,
                    attacks,
                    &retained,
                    buffs,
                )?;
            if reflected {
                b = a;
            }
            c.attack_packet = Some(packet);
            result
        } else {
            load_natural_work(c, items, buffers)?;
            let (result, packet, reflected) = attack::natural_attack_with_target(
                &mut w[b as usize],
                &mut c.enemies.records[(a - 10) as usize],
                index,
                random,
                attacks,
                &retained,
                buffs,
                c.difficulty,
            )?;
            if reflected {
                b = a;
            }
            c.attack_packet = Some(packet);
            result
        }
    } else if a >= 10 && b >= 10 {
        let melee_id = c.enemies.records[(a - 10) as usize]
            .record()
            .melee_weapon()?;
        if is_melee_weapon(items, melee_id)? {
            let weapon = items.and_then(|items| items.get(melee_id as usize)).ok_or(
                RuleError::Unsupported {
                    action: "combat.enemy_melee",
                    dependency: "melee weapon content missing",
                },
            )?;
            load_melee_work(c, a, weapon, items, buffers)?;
            let (result, packet, reflected) =
                crate::combat::enemy_weapons::melee_enemy_with_target(
                    c,
                    random,
                    (a, b, index, actor),
                    weapon,
                    c.difficulty,
                    attacks,
                    &retained,
                )?;
            if reflected {
                b = a;
            }
            c.attack_packet = Some(packet);
            result
        } else {
            load_natural_work(c, items, buffers)?;
            let (result, packet, reflected) =
                crate::combat::ally_turns::natural_attack_enemy_with_target(
                    c,
                    w,
                    random,
                    (a, b, index, actor),
                    attacks,
                    &retained,
                )?;
            if reflected {
                b = a;
            }
            c.attack_packet = Some(packet);
            result
        }
    } else {
        let weapon =
            crate::combat::warrior_attack::weapon(&w[a as usize].bytes, items.unwrap_or(&[]))?;
        let copies_packet =
            w[a as usize].record().attacks_used()? < w[a as usize].record().attack_limit()?;
        if copies_packet {
            if let Some(weapon) = weapon {
                c.weapon_work.load_weapon(weapon)?;
                buffers.load_weapon(items.unwrap(), w[a as usize].record().melee_weapon()?)?;
            } else {
                load_natural_work(c, items, buffers)?;
            }
        }
        let (result, packet, actual) = crate::combat::warrior_attack::attack(
            w,
            a as usize,
            b as usize,
            attacks.ok_or("warrior melee attack table missing")?,
            items.unwrap_or(&[]),
            c.difficulty,
            random,
        )?;
        if copies_packet {
            c.attack_packet = Some(packet);
        }
        b = actual as i16;
        result
    };
    if let Some((member, 0)) = previous_faction
        && w[member].record().faction()? != 0
    {
        c.unavailable = c.unavailable.wrapping_add(1);
    }
    if result.killed {
        let direct = c.direct_deaths;
        c.direct_deaths &= a < 6;
        lifecycle::kill_body(c, w, b)?;
        c.direct_deaths = direct;
    }
    let feedback = result.feedback.map(|feedback| {
        let attack::AttackFeedback {
            code,
            amount,
            recipient,
        } = feedback;
        Event::Feedback(crate::feedback::NativeFeedback::CombatResult {
            body: if recipient == attack::AttackRecipient::Attacker {
                a
            } else {
                b
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
        attacker: a,
        target: b,
        damage: result.damage,
        defeated: result.killed,
    });
    if a >= 10 {
        let melee_id = c.enemies.records[(a - 10) as usize]
            .record()
            .melee_weapon()?;
        let impact = if c.enemies.records[(a - 10) as usize]
            .record()
            .selected_item()?
            != 0
            && is_melee_weapon(items, melee_id)?
        {
            word(&items.unwrap()[melee_id as usize], 0xe)?
        } else {
            let monster = &c.enemies.records[(a - 10) as usize].bytes;
            let attack = if monster[0x93 + index * 4] == 0 {
                0
            } else {
                index
            };
            i16::from(monster[0x95 + attack * 4] as i8)
        };
        events.push(Event::Sound {
            resource_id: if result.damage == 0 { 650 } else { impact },
        });
    } else if result.damage == 0 {
        events.push(Event::Sound { resource_id: 650 });
    } else if !result.killed {
        events.push(Event::Sound {
            resource_id: w[a as usize].record().melee_hit_sound()?,
        });
    }
    Ok(events)
}
