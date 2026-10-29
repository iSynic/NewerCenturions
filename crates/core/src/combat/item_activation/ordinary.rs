//! Combatchoice's prepared-item return, separate from Items' encounter return.
use super::{Caller, activate};
use crate::combat::{lifecycle, medical, player_psi, psi_selection::Selection, runtime, spells};
use crate::{
    Content, Event, MedicalContinuation, MedicalOrigin, MedicalTarget, Phase, RuleError, Session,
};
use centurions_content::records::{AttackPacket, ItemDefinition};
mod groups;

pub(crate) fn begin(
    state: &mut Session,
    content: &Content,
    owner: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    let activation = activate(state, content, owner, slot, Caller::Ordinary)?;
    // The inventory owner was charged by Items; the returning caller prepares
    // and pays tactical costs from the independently selected cast actor.
    let actor = state.selected_warrior;
    if activation.spell == 0 {
        return Err(RuleError::RefusedAfterEffects { feedback: vec![] });
    }
    // loadspell2 leaves the column in WORD4435ca; the 1192bc..1192ca tail runs
    // showspellinfo, which clears WORD44719a at 15939a, when it is not negative.
    if activation.spell % 100 >= 1 {
        state.native_work.fire_item = 0;
    }
    // combatupdate2(selected)119028 precedes prepattack and its mode-byte writer.
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
            .get(state.selected_warrior)
            .ok_or("prepared item selected warrior missing")?,
    )?;
    // Items' power is replaced by the weapon WORD after prepattack1190b8.
    let power = ItemDefinition(&definition).attack_count()?;
    let identity = [
        activation.spell / 1000 - 1,
        activation.spell % 1000 / 100 - 1,
        activation.spell % 100 - 1,
    ];
    let combat = state
        .combat
        .as_mut()
        .ok_or("prepared item combat missing")?;
    combat.weapon_work.prepare(&definition, &ammunition)?;
    combat.attack_packet = packet.clone();
    combat.attack_identity = identity;
    state.native_work.ammunition_work.weapon = Some(definition);
    state.native_work.ammunition_work.ammo = Some(ammunition);
    state.native_work.attack_packet = packet.clone();
    state.native_work.attack_identity = identity;
    state.native_work.attack_count = power;
    state.target_quantity = None;
    // Combatchoice11910e refreshes current ITEM after preparing WEAPON/AMMO.
    runtime::refresh_active_item_context(state, content)?;
    let attack = AttackPacket(&packet);
    if attack.targeting_mode() == 1 && attack.effect()? == 57 {
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
        let health = &mut state.native_work.health_context;
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
    // combatchoice 0x1192d0..0x11936a: a range-0 item below mode 8, or any mode-5
    // item, writes one triple from the party positions 0x44709a at WORD4435d2 x 4
    // (x at +0, y at +2, both scaled; extension words 0x0400/0x0402), the
    // caster's own cell, and goes straight to cast.
    let point = |state: &Session| state.combat.as_ref().unwrap().squad.positions[actor];
    if attack.targeting_mode() == 7 && player_psi::range(&packet, power)? == 0 {
        events.extend(global(state, content, point(state))?);
    } else if attack.targeting_mode() > 8 {
        events.extend(groups::resolve(state, content)?);
    } else if attack.targeting_mode() == 5
        || attack.targeting_mode() < 8 && player_psi::range(&packet, power)? == 0
    {
        events.extend(execute_at(state, content, point(state), 0, false)?);
    }
    Ok(events)
}

pub(crate) fn validate(pending: &Selection, state: &Session) -> Result<(), RuleError> {
    let combat = state
        .combat
        .as_ref()
        .ok_or("prepared item combat missing")?;
    if state.phase != Phase::Combat
        || combat
            .active_body()
            .is_none_or(|body| !(0..6).contains(&body))
        || state.selected_warrior != pending.actor
        || pending.actor >= state.warriors.len()
        || pending.item.is_none_or(|id| id <= 0)
        || pending.spell <= 1100
        || state.medical_continuation.is_some()
        || combat.firearm_selection.is_some()
        || pending.targets.len() > 7
        || pending.targets.len() > pending.power.max(0) as usize
        || combat.attack_identity
            != [
                pending.spell / 1000 - 1,
                pending.spell % 1000 / 100 - 1,
                pending.spell % 100 - 1,
            ]
        || combat.attack_packet.len() != 58
        || combat.attack_packet[1] != 0 && !pending.targets.is_empty()
        || pending.targets.iter().any(|body| {
            *body < 0
                || *body >= state.warriors.len() as i16 && *body < 10
                || *body >= 10 + combat.enemies.records.len() as i16
        })
    {
        return Err("invalid prepared item selection".into());
    }
    for body in &pending.targets {
        let active = if *body < 10 {
            state.warriors[*body as usize].record().active_flag() != 0
        } else {
            combat.enemies.records[(*body - 10) as usize].stamina()? > 0
        };
        if !active {
            return Err("inactive body in prepared item selection".into());
        }
    }
    Ok(())
}

pub(crate) fn admit(
    state: &Session,
    content: &Content,
    pending: &Selection,
    point: (i16, i16),
) -> Result<i16, RuleError> {
    validate(pending, state)?;
    let combat = state.combat.as_ref().unwrap();
    let packet = &combat.attack_packet;
    let from = combat.squad.positions[pending.actor];
    if !player_psi::in_range(from, point, player_psi::range(packet, pending.power)?) {
        return refusal(crate::feedback::CombatRefusal::OutOfRange);
    }
    let attack = AttackPacket(packet);
    let map = state.land(content).ok_or("prepared item map missing")?;
    if attack.base_range().wrapping_add(attack.range_per_power()) > 0
        && !lifecycle::can_see(combat, map, from, point, 0)?
    {
        return refusal(crate::feedback::CombatRefusal::Blocked);
    }
    if attack.targeting_mode() < 2 {
        player_psi::selected_body(combat, &state.warriors, point)
    } else {
        Ok(pending.actor as i16)
    }
}

pub(crate) fn fire_at(
    state: &mut Session,
    content: &Content,
    point: (i16, i16),
    rotation: u8,
) -> Result<Vec<Event>, RuleError> {
    execute_at(state, content, point, rotation, true)
}

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
        .map(|c| &c.attack_packet)
        .ok_or("prepared item packet missing")?;
    if AttackPacket(packet).targeting_mode() == 0 && player_psi::range(packet, pending.power)? != 0
    {
        return Err("multi-body item requires selected targets".into());
    }
    let body = admit(state, content, &pending, point)?;
    let mut events = if clicked {
        vec![Event::Sound { resource_id: 145 }]
    } else {
        vec![]
    };
    if AttackPacket(packet).targeting_mode() == 7 {
        events.extend(global(state, content, point)?);
        return Ok(events);
    }
    let map = state.land(content).ok_or("prepared item map missing")?;
    let body = (AttackPacket(&state.combat.as_ref().unwrap().attack_packet).targeting_mode() < 2)
        .then_some(body);
    state.target_work_mut().enter(0, 1);
    events.extend(player_psi::cast_item(
        state,
        (map, &content.items),
        &pending,
        point,
        rotation,
        body,
        true,
    )?);
    state.target_work_mut().leave();
    state.clear_targets();
    state.psi_selection = None;
    events.extend(runtime::finish_player_action(
        state,
        content,
        pending.actor as i16,
    )?);
    Ok(events)
}

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
        state.target_work_mut().enter(index, total);
        let combat = state.combat.as_ref().unwrap();
        let point = if body < 10 {
            combat.squad.positions[body as usize]
        } else {
            combat.enemies.positions[(body - 10) as usize]
        };
        events.extend(player_psi::cast_item(
            state,
            (map, &content.items),
            &pending,
            point,
            0,
            Some(body),
            index == 0,
        )?);
        if state.target_work_mut().loop_ended(index) {
            break;
        }
    }
    state.target_work_mut().leave();
    state.clear_targets();
    state.psi_selection = None;
    events.extend(runtime::finish_player_action(
        state,
        content,
        pending.actor as i16,
    )?);
    Ok(events)
}

pub(crate) fn cancel(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let pending = state
        .psi_selection
        .as_ref()
        .ok_or("prepared item selection missing")?;
    validate(pending, state)?;
    let actor = pending.actor;
    let movement = state.warriors[actor]
        .record()
        .movement_points()?
        .wrapping_sub(3)
        .max(0);
    state.warriors[actor]
        .record_mut()
        .set_movement_points(movement);
    state.set_ready(0);
    state.psi_selection = None;
    runtime::refresh_active_item_context(state, content)?;
    Ok(vec![
        Event::Sound { resource_id: 6000 },
        Event::Sound { resource_id: 20005 },
    ])
}

fn global(
    state: &mut Session,
    content: &Content,
    point: (i16, i16),
) -> Result<Vec<Event>, RuleError> {
    // cast10d4fe clears the inactive-round counter before duration work.
    state
        .combat
        .as_mut()
        .ok_or("prepared item combat missing")?
        .inactive_rounds = 0;
    let pending = state
        .psi_selection
        .take()
        .ok_or("prepared item selection missing")?;
    let packet = state.combat.as_ref().unwrap().attack_packet.clone();
    // cast10d4fe clamps after target selection, not in the waiting frame.
    let power = pending.power.max(1);
    state.native_work.attack_count = power;
    let actor = &mut state.warriors[pending.actor];
    let movement = actor.record().movement_points()?.wrapping_sub(3).max(0);
    let actions = actor.record().action_points()?.wrapping_sub(2);
    actor.record_mut().set_movement_points(movement);
    actor.record_mut().set_action_points(actions);
    let mut events = vec![Event::Sound {
        resource_id: AttackPacket(&packet).launch_sound()?,
    }];
    let jitter = spells::flight_jitter(&packet, &mut state.random);
    // combatchoice's single triple is the chosen cell, or the caster's own for a
    // range-0 item that fires at once.
    let combat = state.combat.as_ref().unwrap();
    let target = spells::SpellTarget {
        first: point.0.wrapping_add(combat.map.shift.0).wrapping_add(5),
        second: point.1.wrapping_add(combat.map.shift.1).wrapping_add(5),
        kind: 3,
    };
    let flight = spells::flight(combat, pending.actor as i16, &target, &packet, 1, jitter);
    let duration = spells::detached::duration(&packet, power, &mut state.random);
    match AttackPacket(&packet).effect()?.wrapping_abs() {
        50 => {
            events.push(Event::Sound { resource_id: 601 });
            super::super::party_buffs::extend_from_count(&mut state.party_buffs, power)?;
        }
        110 => super::super::party_buffs::extend_scenario_tail(state, duration)?,
        effect @ 0..=9 => {
            super::super::party_buffs::extend(&mut state.party_buffs, effect, duration)?
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
    state.set_ready(0);
    crate::event::Flight::attach(&mut events, &[(0, flight)]);
    events.extend(runtime::finish_player_action(
        state,
        content,
        pending.actor as i16,
    )?);
    Ok(events)
}

fn refusal<T>(kind: crate::feedback::CombatRefusal) -> Result<T, RuleError> {
    Err(RuleError::Refused {
        feedback: vec![crate::feedback::NativeFeedback::Combat { kind }],
    })
}

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
    let attack = AttackPacket(&state.combat.as_ref().unwrap().attack_packet);
    if attack.targeting_mode() != 1 || attack.effect()? != 57 {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::NativeFeedback::Sound { resource_id: 143 }],
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
