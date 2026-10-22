//! Shooting in the original combat execution order.
use super::{
    alive, finish_player_action, position, refresh_active_item_context, refusal, spell_identity,
};
use crate::combat::{Combat, firearm, lifecycle};
use crate::{Event, RuleError, Session};
use centurions_content::Content;
use centurions_content::records::{AttackPacket, ItemDefinition};

pub fn player_shoot(
    state: &mut Session,
    content: &Content,
    target: i16,
) -> Result<Vec<Event>, RuleError> {
    match crate::combat::shooting::select(state, content, target) {
        Err(RuleError::Refused { feedback }) => Ok(if feedback.is_empty() {
            vec![Event::Feedback(crate::feedback::NativeFeedback::Silent)]
        } else {
            feedback.into_iter().map(Event::Feedback).collect()
        }),
        result => result,
    }
}

pub fn player_shoot_immediate(
    state: &mut Session,
    content: &Content,
    target: i16,
) -> Result<Vec<Event>, RuleError> {
    let body = state
        .combat
        .as_ref()
        .and_then(Combat::active_body)
        .filter(|body| (0..6).contains(body))
        .ok_or_else(|| RuleError::Invalid("active warrior missing".into()))?;
    let events = match player_shoot_action(state, content, target) {
        Ok(mut events) => {
            events.insert(0, Event::Sound { resource_id: 145 });
            events.extend(finish_player_action(state, content, body)?);
            events
        }
        Err(RuleError::Refused { feedback }) if feedback.is_empty() => {
            vec![Event::Feedback(crate::feedback::NativeFeedback::Silent)]
        }
        Err(RuleError::Refused { feedback }) => feedback.into_iter().map(Event::Feedback).collect(),
        Err(error) => return Err(error),
    };
    // Getup's combatupdate2 refreshes the new actor independently of the old weapon.
    if state.combat.as_ref().and_then(Combat::active_body) != Some(body) {
        refresh_active_item_context(state, content)?;
    }
    Ok(events)
}

pub fn player_shoot_action(
    state: &mut Session,
    content: &Content,
    target: i16,
) -> Result<Vec<Event>, RuleError> {
    player_shoot_targets_action(state, content, &[target], false)
}

pub(crate) fn player_shoot_targets_action(
    state: &mut Session,
    content: &Content,
    targets: &[i16],
    paid: bool,
) -> Result<Vec<Event>, RuleError> {
    player_shoot_action_at(state, content, targets, paid, None)
}

pub(crate) fn player_shoot_at_action(
    state: &mut Session,
    content: &Content,
    center: (i16, i16),
) -> Result<Vec<Event>, RuleError> {
    let body = state
        .combat
        .as_ref()
        .and_then(Combat::active_body)
        .ok_or("active warrior missing")?;
    player_shoot_action_at(state, content, &[body], false, Some(center))
}

pub(super) fn player_shoot_action_at(
    state: &mut Session,
    content: &Content,
    targets: &[i16],
    paid: bool,
    center: Option<(i16, i16)>,
) -> Result<Vec<Event>, RuleError> {
    let mut prepared = crate::combat::shooting::prepare(state, content)?;
    if targets.is_empty()
        || targets.len()
            > if AttackPacket(&prepared.packet).divides_shots_between_targets() {
                prepared.shots.max(1) as usize
            } else {
                1
            }
    {
        return Err(RuleError::Invalid("invalid firearm target count".into()));
    }
    if let Some(location) = center {
        crate::combat::shooting::admit_location(state, content, location, &prepared)?;
    } else {
        for &target in targets {
            crate::combat::shooting::admit(state, content, target, &prepared, paid)?;
        }
    }
    let c = state.combat.as_mut().unwrap();
    c.inactive_rounds = 0;
    state.target_quantity = None;
    let body = c.active_body().unwrap();
    let warrior = &mut state.warriors[body as usize];
    let weapon_id = warrior.record().equipped_weapon()?;
    let weapon = &content.items[weapon_id.unsigned_abs() as usize];
    let ammunition = &content.items[warrior
        .record()
        .inventory_ammunition(prepared.slot)?
        .unsigned_abs() as usize];
    c.weapon_work.prepare(weapon, ammunition)?;
    state
        .native_work
        .ammunition_work
        .prepare_pair(weapon, ammunition)?;
    state.native_work.ammunition_work.item =
        Some(crate::ammunition::definition(content, weapon_id)?);
    state.native_work.loaded_item_field32 = Some(ItemDefinition(weapon).retained_field32()?);
    state.native_work.attack_count = prepared.shots;
    c.attack_packet = Some(prepared.packet.clone());
    c.attack_identity = Some(spell_identity(ItemDefinition(weapon).spell()?));
    if !paid {
        let rounds = prepared.rounds.wrapping_mul(targets.len() as i16);
        let slot = prepared.slot;
        if warrior.record().inventory_quantity(slot)? < rounds {
            return Err(refusal(crate::feedback::CombatRefusal::NotEnoughAmmo));
        }
        let loaded = warrior
            .record()
            .inventory_quantity(slot)?
            .wrapping_sub(rounds);
        warrior.record_mut().set_inventory_quantity(slot, loaded);
        let weight = warrior
            .record()
            .carried_weight()?
            .wrapping_sub(rounds.wrapping_mul(prepared.weight));
        warrior.record_mut().set_carried_weight(weight);
    }
    let actions = warrior.record().action_points()?.wrapping_sub(2);
    let movement = warrior.record().movement_points()?.wrapping_sub(3).max(0);
    warrior.record_mut().set_action_points(actions);
    warrior.record_mut().set_movement_points(movement);
    if AttackPacket(&prepared.packet).divides_shots_between_targets() {
        prepared.shots = 1;
    }
    let mut events = vec![];
    let mission = crate::Session::land_in(
        content,
        state.mission.as_deref(),
        state.scenario_level,
        state.exploration.as_ref(),
    )
    .unwrap();
    for &selected in targets {
        if !alive(c, &state.warriors, selected) {
            continue;
        }
        let mut mask = [0_i16; 110];
        if !AttackPacket(&prepared.packet).requires_area_mask() {
            mask[selected as usize] = 1;
        } else {
            let center = center.unwrap_or_else(|| position(c, selected));
            let target = crate::combat::spells::SpellTarget {
                first: center.0 + c.map.shift.0 + 5,
                second: center.1 + c.map.shift.1 + 5,
                kind: 3,
            };
            let resolved = crate::combat::spells::target_mask(
                c,
                &state.warriors,
                &mission.areas,
                &target,
                &crate::combat::spells::SpellContext {
                    packet: &prepared.packet,
                    id: ItemDefinition(weapon).spell()?,
                    count: prepared.shots,
                    caster: body,
                    initiative: c.cursor as i16,
                    collision: false,
                },
                &c.spell_fields,
                &mut state.random,
            )?;
            mask = resolved.mask;
            c.spell_fields = resolved.fields;
        }
        events.push(Event::Sound {
            resource_id: AttackPacket(&prepared.packet).launch_sound()?,
        });
        // Native duration draw precedes every impact, even when the duration
        // itself is not rendered by this caller.
        let (minimum, maximum) = AttackPacket(&prepared.packet).duration_bounds();
        state.random.range(minimum, maximum);
        for (target, flag) in mask.iter().enumerate() {
            if *flag <= 0 {
                continue;
            }
            let target = target as i16;
            let result = if target < 10 {
                let attacker = state.warriors[body as usize].clone();
                firearm::resolve_friendly_after_duration(
                    &attacker,
                    &mut state.warriors[target as usize],
                    &prepared,
                    &mut state.random,
                    c.difficulty,
                )?
            } else {
                firearm::resolve_after_duration(
                    &mut state.warriors[body as usize],
                    &mut c.enemies.records[(target - 10) as usize],
                    &prepared,
                    &mut state.random,
                    c.difficulty,
                )?
            };
            if result.killed {
                lifecycle::kill_body(c, &mut state.warriors, target)?;
                // killbody12e506 retains every direct defeated body before returning.
                if let Some(health) = state.native_work.health_context.as_mut() {
                    health.last_body = target;
                }
            }
            events.extend(
                result
                    .sounds
                    .into_iter()
                    .map(|resource_id| Event::Sound { resource_id }),
            );
            // showresults158ee0 reads the defeated NPC's cue for every weapon.
            if result.killed && target >= 10 {
                let sound = c.enemies.records[(target - 10) as usize]
                    .record()
                    .death_sound()?;
                if sound != 0 {
                    events.push(Event::Sound { resource_id: sound });
                }
            }
            if result.damage != 0
                && !result.killed
                && AttackPacket(&prepared.packet).plays_impact_when_wounded()
            {
                events.push(Event::Sound {
                    resource_id: AttackPacket(&prepared.packet).impact_sound()?,
                });
            }
            events.push(Event::AttackResolved {
                attacker: body,
                target,
                damage: result.damage,
                defeated: result.killed,
            });
        }
    }
    Ok(events)
}
