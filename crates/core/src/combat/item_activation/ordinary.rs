//! Using an item in battle from the combat menu.
use super::{Caller, activate};
use crate::combat::spells::spell_identity;
use crate::combat::{lifecycle, medical, player_psi, psi_selection::Selection, runtime, spells};
use crate::{
    Content, Event, MedicalContinuation, MedicalOrigin, MedicalTarget, Phase, RuleError, Session,
};
use centurions_content::records::{ItemDefinition, SpellInfoRecord};
mod groups;

/// Uses an item in battle. An item with no spell does nothing. A spell item prepares the attack for
/// the user, then waits for a target; a health item (effect 57) goes on to choosing whom to treat.
/// It casts at once if it is a party-duration item of range 0, a side-wide item (mode above 8), a
/// mode-5 item, or a range-0 item of mode below 8.
pub(crate) fn begin(
    state: &mut Session,
    content: &Content,
    owner: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    let activation = activate(state, content, owner, slot, Caller::Ordinary)?;
    // The item's owner has paid the charge; the selected warrior prepares the
    // attack and pays its battle costs.
    let actor = state.char_select_new;
    if activation.spell == 0 {
        return Err(RuleError::RefusedAfterEffects { feedback: vec![] });
    }
    // Showing the spell clears the item in use.
    if activation.spell % 100 >= 1 {
        state.globals.item_used = 0;
    }
    // The panel refreshes before the attack is prepared.
    runtime::refresh_item_context(state, content, actor as i16)?;
    let definition = crate::ammunition::definition(content, activation.item)?;
    let ammunition = crate::ammunition::definition(content, 0)?;
    let map = state.land(content).ok_or("prepared item map missing")?;
    let packet = medical::preparation::prepared_packet(
        content,
        map,
        activation.item,
        state
            .warriors
            .get(state.char_select_new)
            .ok_or("prepared item selected warrior missing")?,
    )?;
    // In battle the weapon's attack count replaces the item's power.
    let power = ItemDefinition(&definition).attack_count()?;
    let identity = spell_identity(activation.spell);
    let combat = state
        .combat
        .as_mut()
        .ok_or("prepared item combat missing")?;
    combat.weapon_traits.prepare(&definition, &ammunition)?;
    combat.spell_info = packet.clone();
    combat.cast_identity = identity;
    state.globals.loaded_items.weapon = Some(definition);
    state.globals.loaded_items.ammo = Some(ammunition);
    state.globals.spell_info = packet.clone();
    state.globals.cast_identity = identity;
    state.globals.attack_count = power;
    state.target_quantity = None;
    // The item in hand refreshes after the weapon and ammunition are prepared.
    runtime::refresh_active_item_context(state, content)?;
    let attack = SpellInfoRecord(&packet);
    if attack.target_type() == 1 && attack.effect()? == 57 {
        let origin = if (700..=703).contains(&activation.item)
            && activation.spell == 1401
            && power == 1
            && attack.effect()? == 57
            && actor == owner
            && state.combat.as_ref().unwrap().active_body() == Some(actor as i16)
        {
            MedicalOrigin::OwnedNeedle
        } else {
            MedicalOrigin::OwnedItem
        };
        let health = &mut state.globals.health_context;
        health.marks_affected = false;
        health.effect = attack.health_effect();
        state.medical_continuation = Some(MedicalContinuation {
            origin,
            actor,
            item: activation.item,
            spell: activation.spell,
            power,
        });
        return Ok(activation.events);
    }
    state.psi_selection = Some(Selection {
        actor,
        spell: activation.spell,
        power,
        targets: vec![],
        item: Some(activation.item),
    });
    let mut events = activation.events;
    // A range-0 item below mode 8, or any mode-5 item, targets the user's own cell
    // and casts at once.
    let point = |state: &Session| state.combat.as_ref().unwrap().squad.pos[actor];
    if attack.is_party_duration() && player_psi::range(&packet, power)? == 0 {
        events.extend(global(state, content, point(state))?);
    } else if attack.target_type() > 8 {
        events.extend(groups::resolve(state, content)?);
    } else if attack.target_type() == 5
        || attack.target_type() < 8 && player_psi::range(&packet, power)? == 0
    {
        events.extend(execute_at(state, content, point(state), 0, false)?);
    }
    Ok(events)
}

/// Checks that a prepared item use is still sound: it is the active warrior's, in battle, with
/// targets that are standing and not more than the item's power or seven.
pub(crate) fn validate(pending: &Selection, state: &Session) -> Result<(), RuleError> {
    let combat = state
        .combat
        .as_ref()
        .ok_or("prepared item combat missing")?;
    if state.phase != Phase::Combat
        || combat
            .active_body()
            .is_none_or(|body| !(0..6).contains(&body))
        || state.char_select_new != pending.actor
        || pending.actor >= state.warriors.len()
        || pending.item.is_none_or(|id| id <= 0)
        || pending.spell <= 1100
        || state.medical_continuation.is_some()
        || combat.firearm_selection.is_some()
        || pending.targets.len() > 7
        || pending.targets.len() > pending.power.max(0) as usize
        || combat.cast_identity != spell_identity(pending.spell)
        || combat.spell_info.len() != 58
        || combat.spell_info[1] != 0 && !pending.targets.is_empty()
        || pending.targets.iter().any(|body| {
            *body < 0
                || *body >= state.warriors.len() as i16 && *body < 10
                || *body >= 10 + combat.roster.records.len() as i16
        })
    {
        return Err("invalid prepared item selection".into());
    }
    for body in &pending.targets {
        let active = if *body < 10 {
            state.warriors[*body as usize].record().in_battle() != 0
        } else {
            combat.roster.records[(*body - 10) as usize].stamina()? > 0
        };
        if !active {
            return Err("inactive body in prepared item selection".into());
        }
    }
    Ok(())
}

/// Checks a chosen cell for the item: it must lie within the item's range and, for a ranged item,
/// in sight. Returns the body that a single-target item strikes, or the user for an area item.
pub(crate) fn admit(
    state: &Session,
    content: &Content,
    pending: &Selection,
    point: (i16, i16),
) -> Result<i16, RuleError> {
    validate(pending, state)?;
    let combat = state.combat.as_ref().unwrap();
    let packet = &combat.spell_info;
    let from = combat.squad.pos[pending.actor];
    if !player_psi::in_range(from, point, player_psi::range(packet, pending.power)?) {
        return refusal(crate::feedback::CombatRefusal::OutOfRange);
    }
    let attack = SpellInfoRecord(packet);
    let map = state.land(content).ok_or("prepared item map missing")?;
    if attack.range_base().wrapping_add(attack.range_per_power()) > 0
        && !lifecycle::can_see(combat, map, from, point, 0)?
    {
        return refusal(crate::feedback::CombatRefusal::Blocked);
    }
    if attack.target_type() < 2 {
        player_psi::selected_body(combat, &state.warriors, point)
    } else {
        Ok(pending.actor as i16)
    }
}

/// Fires the prepared item at the clicked cell, turned by `rotation` quarter turns.
pub(crate) fn fire_at(
    state: &mut Session,
    content: &Content,
    point: (i16, i16),
    rotation: u8,
) -> Result<Vec<Event>, RuleError> {
    execute_at(state, content, point, rotation, true)
}

/// Fires the prepared item at a cell: a party-duration item takes effect on the party, any other is
/// cast at the cell or the body in it. The user's action is then finished. A click plays the
/// targeting sound.
fn execute_at(
    state: &mut Session,
    content: &Content,
    point: (i16, i16),
    rotation: u8,
    clicked: bool,
) -> Result<Vec<Event>, RuleError> {
    let pending = state
        .psi_selection
        .clone()
        .ok_or("prepared item selection missing")?;
    if rotation > 3 {
        return Err("item target rotation outside quarter turns".into());
    }
    let packet = state
        .combat
        .as_ref()
        .map(|c| &c.spell_info)
        .ok_or("prepared item packet missing")?;
    if SpellInfoRecord(packet).target_type() == 0 && player_psi::range(packet, pending.power)? != 0
    {
        return Err("multi-body item requires selected targets".into());
    }
    let body = admit(state, content, &pending, point)?;
    let mut events = if clicked {
        vec![Event::Sound { resource_id: 145 }]
    } else {
        vec![]
    };
    if SpellInfoRecord(packet).is_party_duration() {
        events.extend(global(state, content, point)?);
        return Ok(events);
    }
    let map = state.land(content).ok_or("prepared item map missing")?;
    let body = (SpellInfoRecord(&state.combat.as_ref().unwrap().spell_info).target_type() < 2)
        .then_some(body);
    state.target_picks_mut().enter(0, 1);
    events.extend(player_psi::cast_item(
        state,
        (map, &content.items),
        &pending,
        point,
        rotation,
        body,
        true,
    )?);
    state.target_picks_mut().leave();
    state.clear_targets();
    state.psi_selection = None;
    events.extend(runtime::finish_player_action(
        state,
        content,
        pending.actor as i16,
    )?);
    Ok(events)
}

/// Fires the prepared item at each body chosen for it in turn, then finishes the user's action.
/// Choosing no body is refused.
pub(crate) fn fire_targets(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    let pending = state
        .psi_selection
        .clone()
        .ok_or("prepared item selection missing")?;
    validate(&pending, state)?;
    if pending.targets.is_empty() {
        return refusal(crate::feedback::CombatRefusal::NeedTarget);
    }
    let map = state.land(content).ok_or("prepared item map missing")?;
    let mut events = vec![];
    let total = pending.targets.len();
    for (index, body) in pending.targets.iter().copied().enumerate() {
        state.target_picks_mut().enter(index, total);
        let combat = state.combat.as_ref().unwrap();
        let point = combat.body_position(body);
        events.extend(player_psi::cast_item(
            state,
            (map, &content.items),
            &pending,
            point,
            0,
            Some(body),
            index == 0,
        )?);
        if state.target_picks_mut().loop_ended(index) {
            break;
        }
    }
    state.target_picks_mut().leave();
    state.clear_targets();
    state.psi_selection = None;
    events.extend(runtime::finish_player_action(
        state,
        content,
        pending.actor as i16,
    )?);
    Ok(events)
}

/// Gives up the prepared item. The user loses 3 movement and the selection is cleared.
pub(crate) fn cancel(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let pending = state
        .psi_selection
        .as_ref()
        .ok_or("prepared item selection missing")?;
    validate(pending, state)?;
    let actor = pending.actor;
    let movement = state.warriors[actor]
        .record()
        .movement()?
        .wrapping_sub(3)
        .max(0);
    state.warriors[actor].record_mut().set_movement(movement);
    state.set_in_spell(0);
    state.psi_selection = None;
    runtime::refresh_active_item_context(state, content)?;
    Ok(vec![
        Event::Sound { resource_id: 6000 },
        Event::Sound { resource_id: 20005 },
    ])
}

/// An item that lasts on the party: the user spends an attack action and the item's launch sound
/// plays; the effect is added to the party's conditions for the duration the item rolls (or its
/// power for effect 50).
fn global(
    state: &mut Session,
    content: &Content,
    point: (i16, i16),
) -> Result<Vec<Event>, RuleError> {
    // Casting resets the inactive-round count before any duration.
    state
        .combat
        .as_mut()
        .ok_or("prepared item combat missing")?
        .blank_round = 0;
    let pending = state
        .psi_selection
        .take()
        .ok_or("prepared item selection missing")?;
    let packet = state.combat.as_ref().unwrap().spell_info.clone();
    // The power is at least 1, applied once targets are chosen.
    let power = pending.power.max(1);
    state.globals.attack_count = power;
    let actor = &mut state.warriors[pending.actor];
    crate::combat::records::spend_attack_action(actor)?;
    let mut events = vec![Event::Sound {
        resource_id: SpellInfoRecord(&packet).launch_sound()?,
    }];
    let jitter = spells::flight_jitter(&packet, &mut state.random);
    // The single target is the chosen cell, or the user's own for a range-0 item.
    let combat = state.combat.as_ref().unwrap();
    let (x, y) = crate::combat::records::wrapping_grid_point(point, combat.field.map_shift);
    let target = spells::SpellTarget::cell(x, y);
    let flight = spells::flight(combat, pending.actor as i16, &target, &packet, 1, jitter);
    let duration = spells::out_of_combat::duration(&packet, power, &mut state.random);
    match SpellInfoRecord(&packet).effect()?.wrapping_abs() {
        50 => {
            events.push(Event::Sound { resource_id: 601 });
            super::super::party_buffs::extend_from_count(&mut state.party_condition, power)?;
        }
        110 => super::super::party_buffs::extend_scenario_tail(state, duration)?,
        effect @ 0..=9 => {
            super::super::party_buffs::extend(&mut state.party_condition, effect, duration)?
        }
        effect => {
            if !super::super::party_buffs::extend_indexed(state, effect, duration)? {
                return Err(RuleError::Unsupported {
                    action: "combat.item_activation",
                    dependency: "indexed global item duration outside retained owners",
                });
            }
        }
    }
    state.clear_targets();
    state.set_in_spell(0);
    crate::event::Flight::attach(&mut events, &[(0, flight)]);
    events.extend(runtime::finish_player_action(
        state,
        content,
        pending.actor as i16,
    )?);
    Ok(events)
}

/// Refuses a targeting choice with the combat message for it.
fn refusal<T>(kind: crate::feedback::CombatRefusal) -> Result<T, RuleError> {
    Err(RuleError::Refused {
        feedback: vec![crate::feedback::GameFeedback::Combat { kind }],
    })
}

/// Picks a warrior's portrait as the target of a prepared healing item; any other item refuses with
/// a sound.
pub(crate) fn portrait(
    state: &mut Session,
    content: &Content,
    warrior: usize,
) -> Result<Vec<Event>, RuleError> {
    let pending = state
        .psi_selection
        .as_ref()
        .ok_or("prepared item selection missing")?;
    validate(pending, state)?;
    let attack = SpellInfoRecord(&state.combat.as_ref().unwrap().spell_info);
    if attack.target_type() != 1 || attack.effect()? != 57 {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::GameFeedback::Sound { resource_id: 143 }],
        });
    }
    let pending = state.psi_selection.take().unwrap();
    state.medical_continuation = Some(MedicalContinuation {
        origin: MedicalOrigin::OwnedItem,
        actor: pending.actor,
        item: pending.item.unwrap(),
        spell: pending.spell,
        power: pending.power,
    });
    runtime::select_medical_target(state, content, MedicalTarget::Portrait { warrior })
}
