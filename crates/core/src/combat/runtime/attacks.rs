//! Melee and natural attacks during combat.
use super::{has_active_warrior, is_melee_weapon, load_melee_weapon, load_natural_attack};
use crate::combat::death_scripts::DeathProcessing;
use crate::combat::enemy_weapons;
use crate::combat::warrior_attack::{self, WarriorMelee};
use crate::combat::{Combat, attack, lifecycle};
use crate::{Event, RandomState, RuleError};
use centurions_content::Warrior;
use std::ops::ControlFlow;

/// What a strike leaves behind: its events, whether it killed the target and so returns control
/// (`returned`), and whether it ended the battle (`terminated`, set only by out-of-combat
/// strikes).
#[derive(Default)]
pub(super) struct AttackOutcome {
    pub events: Vec<Event>,
    pub returned: bool,
    pub terminated: bool,
}

/// Resolves one attack, running the death story for a kill, and returns just its events.
#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_attack_with_death(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    attacker: i16,
    recipient: i16,
    attack_index: usize,
    turn_actor: i16,
    items: Option<&[Vec<u8>]>,
    attacks: Option<&[u8]>,
    buffs: &[u8],
    buffers: &mut crate::ammunition::LoadedItems,
    death_processing: Option<DeathProcessing>,
) -> Result<Vec<Event>, RuleError> {
    resolve_attack_result(
        combat,
        warriors,
        random,
        attacker,
        recipient,
        attack_index,
        turn_actor,
        items,
        attacks,
        buffs,
        buffers,
        death_processing,
    )
    .map(|outcome| outcome.events)
}

/// Resolves one attack by a warrior or monster against a body. The attacker's weapon or natural
/// attack is loaded, the blow is rolled, a kill runs the death of the target, and the sounds,
/// result message and resolved attack are reported. Nothing is struck once no warrior of the party
/// is left in the battle.
///
/// Original: attack, attack2
#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_attack_result(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    attacker: i16,
    mut recipient: i16,
    attack_index: usize,
    turn_actor: i16,
    items: Option<&[Vec<u8>]>,
    attacks: Option<&[u8]>,
    buffs: &[u8],
    buffers: &mut crate::ammunition::LoadedItems,
    death_processing: Option<DeathProcessing>,
) -> Result<AttackOutcome, RuleError> {
    let presents_result = attacker >= 10
        || warriors[attacker as usize].record().attacks_used()?
            < warriors[attacker as usize].record().attack_limit()?;
    if presents_result {
        combat.did_attack_write = Some(1);
    }
    if !has_active_warrior(warriors) {
        combat.kill_counts.finish();
        if attacker >= 10 {
            combat.spell_info = if let Some(attacks) = attacks {
                attack::spell_info(attacks)?
            } else {
                attack::default_spell_info()
            };
        }
        return Ok(AttackOutcome::default());
    }
    combat.blank_round = 0;
    let info = combat.spell_info.clone();
    // During a warrior's turn, monster_up still names the last monster that acted.
    let turn_actor = if turn_actor < 10 {
        combat.monster_up as i16 + 10
    } else {
        turn_actor
    };
    let swing = Swing {
        attacker,
        attack_index,
        turn_actor,
        items,
        attacks,
        buffs,
        info: &info,
        death_processing,
    };
    let mut death_handled = false;
    let result = if attacker < 10 {
        match warrior_strike(combat, warriors, random, &swing, &mut recipient, buffers)? {
            ControlFlow::Continue(result) => result,
            ControlFlow::Break(halted) => return Ok(halted),
        }
    } else if recipient < 10 {
        enemy_strikes_warrior(combat, warriors, random, &swing, &mut recipient, buffers)?
    } else {
        let (result, handled) =
            enemy_strikes_enemy(combat, warriors, random, &swing, &mut recipient, buffers)?;
        death_handled = handled;
        result
    };
    if presents_result {
        combat.neighbors.departure = 0;
    }
    if result.killed && !death_handled {
        let direct = combat.direct_deaths;
        combat.direct_deaths &= attacker < 6;
        lifecycle::kill_body_with_processing(combat, warriors, recipient, death_processing)?;
        combat.direct_deaths = direct;
    }
    let killed = result.killed;
    let mut events = strike_events(combat, warriors, &swing, recipient, result)?;
    if death_processing.is_some() && killed {
        if recipient >= 10 {
            events.extend(super::death::enemy_death_feedback(
                &combat.roster.records[(recipient - 10) as usize],
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
        returned: killed,
        terminated: false,
    })
}

/// One attack's inputs that every arm reads unchanged.
struct Swing<'a> {
    attacker: i16,
    attack_index: usize,
    turn_actor: i16,
    items: Option<&'a [Vec<u8>]>,
    attacks: Option<&'a [u8]>,
    buffs: &'a [u8],
    /// The attack packet as it stood before this attack.
    info: &'a Vec<u8>,
    death_processing: Option<DeathProcessing>,
}

/// A warrior's strike: its weapon work while attacks remain, then the unarmed or melee
/// attack. A halted spell weapon breaks out with the finished outcome.
fn warrior_strike(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    swing: &Swing,
    recipient: &mut i16,
    buffers: &mut crate::ammunition::LoadedItems,
) -> Result<ControlFlow<AttackOutcome, attack::AttackResult>, RuleError> {
    let attacker = swing.attacker;
    let items = swing.items;
    let weapon = warrior_attack::weapon(&warriors[attacker as usize].bytes, items.unwrap_or(&[]))?;
    if warriors[attacker as usize].record().attacks_used()?
        < warriors[attacker as usize].record().attack_limit()?
    {
        if let Some(weapon) = weapon {
            combat.weapon_traits.load_weapon(weapon)?;
            buffers.load_weapon(
                items.unwrap(),
                warriors[attacker as usize].record().melee_weapon()?,
            )?;
        } else {
            load_natural_attack(combat, items, buffers)?;
        }
    }
    Ok(ControlFlow::Continue(match swing.attacks {
        None if *recipient >= 10 => attack::unarmed_attack(
            &mut warriors[attacker as usize],
            &mut combat.roster.records[(*recipient - 10) as usize],
            random,
            combat.difficulty,
        )?,
        attacks => match warrior_attack::warrior_melee(
            combat,
            warriors,
            random,
            attacker,
            *recipient,
            items.unwrap_or(&[]),
            attacks.ok_or("warrior melee attack table missing")?,
            swing.info,
            swing.buffs,
        )? {
            WarriorMelee::Halted { packet, warned } => {
                return Ok(ControlFlow::Break(AttackOutcome {
                    events: halted_spell_weapon(combat, packet, warned),
                    returned: false,
                    terminated: false,
                }));
            }
            WarriorMelee::Struck {
                result,
                packet,
                target,
            } => {
                *recipient = target;
                if let Some(packet) = packet {
                    combat.spell_info = packet;
                }
                result
            }
        },
    }))
}

/// The monster's melee weapon when it holds one, or `None` for a natural attack.
fn enemy_melee_weapon<'a>(
    combat: &Combat,
    swing: &Swing<'a>,
) -> Result<Option<&'a Vec<u8>>, RuleError> {
    let items = swing.items;
    let melee_id = combat.roster.records[(swing.attacker - 10) as usize]
        .record()
        .melee_weapon()?;
    if !is_melee_weapon(items, melee_id)? {
        return Ok(None);
    }
    items
        .and_then(|items| items.get(melee_id as usize))
        .ok_or(RuleError::Unsupported {
            action: "combat.enemy_melee",
            dependency: "melee weapon content missing",
        })
        .map(Some)
}

/// A monster turning to strike a warrior with its melee weapon or a natural attack.
fn enemy_strikes_warrior(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    swing: &Swing,
    recipient: &mut i16,
    buffers: &mut crate::ammunition::LoadedItems,
) -> Result<attack::AttackResult, RuleError> {
    let (attacker, items) = (swing.attacker, swing.items);
    let attacker_x = combat.body_position(attacker).0;
    let target_x = combat.body_position(*recipient).0;
    combat.roster.records[(attacker - 10) as usize]
        .record_mut()
        .face_towards(attacker_x, target_x);
    if let Some(weapon) = enemy_melee_weapon(combat, swing)? {
        load_melee_weapon(combat, attacker, weapon, items, buffers)?;
        let (result, packet, reflected) = enemy_weapons::melee_warrior_with_target(
            combat,
            warriors,
            random,
            (attacker, *recipient, swing.attack_index, swing.turn_actor),
            weapon,
            combat.difficulty,
            swing.attacks,
            swing.info,
            swing.buffs,
        )?;
        if reflected {
            *recipient = attacker;
        }
        combat.spell_info = packet;
        Ok(result)
    } else {
        load_natural_attack(combat, items, buffers)?;
        let (result, packet, reflected, total_delta) = attack::natural_attack_with_target(
            &mut warriors[*recipient as usize],
            &mut combat.roster.records[(attacker - 10) as usize],
            swing.attack_index,
            random,
            swing.attacks,
            swing.info,
            swing.buffs,
            combat.difficulty,
        )?;
        combat.kill_counts.add_num_enemy(total_delta);
        if !reflected {
            combat.kill_party = combat.kill_party.wrapping_add(total_delta as i8);
        }
        if reflected {
            *recipient = attacker;
        }
        combat.spell_info = packet;
        Ok(result)
    }
}

/// A monster striking another monster. The flag says the natural adapter already ran
/// killbody, including after reflection.
fn enemy_strikes_enemy(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    swing: &Swing,
    recipient: &mut i16,
    buffers: &mut crate::ammunition::LoadedItems,
) -> Result<(attack::AttackResult, bool), RuleError> {
    let (attacker, items) = (swing.attacker, swing.items);
    if let Some(weapon) = enemy_melee_weapon(combat, swing)? {
        load_melee_weapon(combat, attacker, weapon, items, buffers)?;
        let (result, packet, reflected) = enemy_weapons::melee_enemy_with_target(
            combat,
            random,
            (attacker, *recipient, swing.attack_index, swing.turn_actor),
            weapon,
            combat.difficulty,
            swing.attacks,
            swing.info,
        )?;
        if reflected {
            *recipient = attacker;
        }
        combat.spell_info = packet;
        Ok((result, false))
    } else {
        load_natural_attack(combat, items, buffers)?;
        let (result, packet, reflected) =
            crate::combat::ally_turns::natural_attack_enemy_with_processing(
                combat,
                warriors,
                random,
                (attacker, *recipient, swing.attack_index, swing.turn_actor),
                swing.attacks,
                swing.info,
                swing.death_processing,
            )?;
        if reflected {
            *recipient = attacker;
        }
        combat.spell_info = packet;
        // The monster natural adapter owns killbody, including reflection.
        let handled = result.killed;
        Ok((result, handled))
    }
}

/// The strike's sounds, result feedback, resolved event and impact cue.
fn strike_events(
    combat: &Combat,
    warriors: &[Warrior],
    swing: &Swing,
    recipient: i16,
    result: attack::AttackResult,
) -> Result<Vec<Event>, RuleError> {
    let (attacker, items) = (swing.attacker, swing.items);
    let feedback = result.feedback.map(|feedback| {
        let attack::AttackFeedback {
            code,
            amount,
            recipient: feedback_recipient,
        } = feedback;
        Event::Feedback(crate::feedback::GameFeedback::CombatResult {
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
        let melee_id = combat.roster.records[(attacker - 10) as usize]
            .record()
            .melee_weapon()?;
        let impact = if combat.roster.records[(attacker - 10) as usize]
            .record()
            .weapon()?
            != 0
            && is_melee_weapon(items, melee_id)?
        {
            centurions_content::records::ItemDefinition(&items.unwrap()[melee_id as usize])
                .impact_sound()?
        } else {
            combat.roster.records[(attacker - 10) as usize]
                .record()
                .natural_attack(swing.attack_index)
                .impact_sound
        };
        events.push(Event::Sound {
            resource_id: if result.damage == 0 { 650 } else { impact },
        });
    } else if result.damage == 0 {
        events.push(Event::Sound { resource_id: 650 });
    } else if !result.killed {
        events.push(Event::Sound {
            resource_id: warriors[attacker as usize].record().weapon_sound()?,
        });
    }
    Ok(events)
}

/// A halted spell-weapon attack leaves the spell's attack packet and no strike events; the warning
/// shows only when the halt was warned.
fn halted_spell_weapon(combat: &mut Combat, packet: Vec<u8>, warned: bool) -> Vec<Event> {
    combat.spell_info = packet;
    if warned {
        vec![Event::Feedback(crate::feedback::GameFeedback::Warning {
            index: 39,
        })]
    } else {
        vec![]
    }
}
