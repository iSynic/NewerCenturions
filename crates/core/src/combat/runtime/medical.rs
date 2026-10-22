//! Medical in the original combat execution order.
use super::{end_turn, finish_player_action};
use crate::combat::Combat;
use crate::{Event, Phase, RuleError, Session};
use centurions_content::Content;

pub(super) fn medical_turn(state: &Session) -> Result<usize, RuleError> {
    if state.phase != Phase::Combat
        || state
            .combat_story
            .as_ref()
            .is_some_and(|story| !story.waiting_child)
        || state
            .exploration
            .as_ref()
            .is_none_or(|p| p.presentation.is_some())
    {
        return Err(RuleError::Invalid(
            "medical aid requires an unobstructed combat turn".into(),
        ));
    }
    state
        .combat
        .as_ref()
        .and_then(Combat::active_body)
        .filter(|body| (0..6).contains(body))
        .map(|body| body as usize)
        .ok_or_else(|| RuleError::Invalid("medical aid requires an active warrior".into()))
}

pub(crate) fn refresh_active_item_context(
    state: &mut Session,
    content: &Content,
) -> Result<(), RuleError> {
    state.target_quantity = None;
    let Some(body) = state.combat.as_ref().and_then(Combat::active_body) else {
        return Ok(());
    };
    if body >= 10 {
        let combat = state.combat.as_ref().unwrap();
        let item = combat.enemies.records[(body - 10) as usize]
            .record()
            .selected_item()?;
        if item != 0 {
            state.load_item_work(content, item)?;
        }
        return Ok(());
    }
    if !(0..6).contains(&body) {
        return Err(RuleError::Invalid(
            "item refresh body outside represented actors".into(),
        ));
    }
    let warrior = &state.warriors[body as usize];
    let item = if warrior.record().firing_mode() {
        warrior.record().equipped_weapon()?
    } else {
        warrior.record().melee_weapon()?
    };
    if warrior.record().firing_mode()
        && !warrior
            .inventory()?
            .iter()
            .any(|record| record.id() == item && record.equipment_flag() == 1)
    {
        return Err(RuleError::Unsupported {
            action: "combat.medical_aid",
            dependency: "missing selected secondary during native item refresh",
        });
    }
    state.load_item_work(content, item)?;
    Ok(())
}

pub fn stop_bleeding(
    state: &mut Session,
    content: &Content,
    warrior: Option<usize>,
) -> Result<Vec<Event>, RuleError> {
    medical_turn(state)?;
    match state.bandage_ready {
        None => {
            return Err(RuleError::Unsupported {
                action: "combat.stop_bleeding",
                dependency: "unreconstructable original interaction marker",
            });
        }
        Some(0) => {
            return Err(RuleError::Invalid(
                "Stop Bleeding is unavailable after the previous action".into(),
            ));
        }
        Some(_) => {}
    }
    if warrior.is_some_and(|member| member >= state.warriors.len()) {
        return Err(RuleError::Invalid("bandaging target outside squad".into()));
    }
    state.native_work.party_mask.fill(0);
    let mut events = vec![Event::Sound { resource_id: 10105 }];
    if let Some(member) = warrior {
        state.native_work.party_mask[member] = 1;
        state.warriors[member].record_mut().set_bleeding_flag(0);
        events.push(Event::Sound { resource_id: 141 });
    }
    events.extend(end_turn(state, content)?);
    refresh_active_item_context(state, content)?;
    Ok(events)
}

pub fn begin_medical_item(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    medical_turn(state)?;
    let actor = state
        .warriors
        .get_mut(warrior)
        .ok_or_else(|| RuleError::Invalid("medical warrior outside squad".into()))?;
    let selected = actor
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or_else(|| RuleError::Invalid("medical inventory slot missing".into()))?;
    crate::items::compact(actor)?;
    if actor.inventory()?.get(slot) != Some(&selected) {
        return Err(RuleError::Unsupported {
            action: "combat.medical_aid",
            dependency: "inventory selection moved during item pane initialization",
        });
    }
    crate::equipment::calculate_weight(actor, content)?;
    for member in &mut state.warriors {
        member.record_mut().set_turn_skip_duration(0);
    }
    let result = crate::combat::medical::begin(
        state,
        content,
        crate::combat::medical::BeginRequest { warrior, slot },
    )?;
    state.warriors[warrior] = result.warrior;
    state.selected_warrior = warrior;
    state.native_work.loaded_item_field32 = Some(result.loaded_item_field32);
    state.native_work.attack_count = result.count;
    state.native_work.ammunition_work.weapon =
        Some(crate::ammunition::definition(content, selected.id())?);
    state.native_work.ammunition_work.ammo = Some(crate::ammunition::definition(content, 0)?);
    state.native_work.party_mask.fill(0);
    let mut health = crate::story_health::context(
        &state.warriors,
        warrior,
        state.native_work.health_context.as_ref(),
    );
    health.marks_affected = false;
    health.effect = centurions_content::records::AttackPacket(&result.packet).health_effect();
    state.native_work.health_context = Some(health);
    let combat = state.combat.as_mut().unwrap();
    combat.attack_packet = Some(result.packet);
    combat.attack_identity = Some(result.identity);
    combat.weapon_work = result.weapon_work;
    state.medical_continuation = Some(crate::MedicalContinuation {
        origin: crate::MedicalOrigin::OwnedNeedle,
        actor: warrior,
        item: selected.id(),
        spell: 1401,
        power: result.count,
    });
    refresh_active_item_context(state, content)?;
    Ok(result.events)
}

pub fn begin_learned_heal(
    state: &mut Session,
    content: &Content,
    power: i16,
) -> Result<Vec<Event>, RuleError> {
    let actor = medical_turn(state)?;
    if state.psi_menu_choices[actor]
        .as_ref()
        .is_some_and(|choice| {
            choice.row > 6 || choice.column > 6 || !(1..=7).contains(&choice.power)
        })
    {
        return Err(RuleError::Unsupported {
            action: "combat.learned_portrait_heal",
            dependency: "other remembered psi menu selection",
        });
    }
    let mut result = crate::combat::medical::begin_learned_heal(state, content, power)?;
    if state.psi_menu_choices[actor]
        .as_ref()
        .is_some_and(|choice| choice.row != 0)
    {
        result.events[2] = Event::Sound { resource_id: 10129 };
    }
    let previous_power = state.psi_menu_choices[actor]
        .as_ref()
        .map_or(1, |choice| choice.power);
    result
        .events
        .retain(|event| !matches!(event, Event::Sound { resource_id: 137 }));
    if power != previous_power {
        result.events.push(Event::Sound { resource_id: 137 });
    }
    state.warriors[actor] = result.warrior;
    state.selected_warrior = actor;
    state.psi_menu_choices[actor] = Some(crate::PsiMenuChoice {
        row: 0,
        column: 0,
        power,
    });
    state.native_work.attack_count = result.count;
    state.native_work.party_mask.fill(0);
    let mut health = crate::story_health::context(
        &state.warriors,
        actor,
        state.native_work.health_context.as_ref(),
    );
    health.affected[actor] = 1;
    health.effect = centurions_content::records::AttackPacket(&result.packet).health_effect();
    state.native_work.health_context = Some(health);
    let combat = state.combat.as_mut().unwrap();
    combat.attack_packet = Some(result.packet);
    combat.attack_identity = Some(result.identity);
    combat.weapon_work = result.weapon_work;
    state.medical_continuation = Some(crate::MedicalContinuation {
        origin: crate::MedicalOrigin::LearnedHeal,
        actor,
        item: 0,
        spell: 1101,
        power,
    });
    refresh_active_item_context(state, content)?;
    Ok(result.events)
}

pub fn select_medical_target(
    state: &mut Session,
    content: &Content,
    selection: crate::MedicalTarget,
) -> Result<Vec<Event>, RuleError> {
    medical_turn(state)?;
    let pending = state
        .medical_continuation
        .clone()
        .ok_or_else(|| RuleError::Invalid("no medical target selection is pending".into()))?;
    let mission = state
        .land(content)
        .ok_or_else(|| RuleError::Invalid("medical map missing".into()))?;
    let resolution = crate::combat::medical::resolve(
        state.combat.as_mut().unwrap(),
        &mut state.warriors,
        mission,
        &mut state.random,
        &pending,
        &selection,
    )?;
    let packet = state
        .combat
        .as_ref()
        .unwrap()
        .attack_packet
        .as_ref()
        .unwrap();
    if let Some(health) = state.native_work.health_context.as_mut() {
        health.effect = centurions_content::records::AttackPacket(packet).health_effect();
    }
    if pending.origin == crate::MedicalOrigin::LearnedHeal
        && matches!(selection, crate::MedicalTarget::Portrait { .. })
    {
        if state.native_work.psi_memory.is_none() {
            state.native_work.psi_memory = Some(crate::initial_psi_memory(&state.warriors)?);
        }
        let memory = state.native_work.psi_memory.as_mut().unwrap();
        centurions_content::records::PsiSelectionMemoryMut(memory)
            .remember_portrait_heal(pending.actor, pending.power);
    }
    state.medical_continuation = None;
    state.native_work.party_mask.fill(0);
    let mut events = resolution.events;
    if let Some(health) = state.native_work.health_context.as_mut() {
        health.unavailable = state.combat.as_ref().unwrap().unavailable;
        if health.marks_affected {
            for event in &events {
                if let Event::SpellResolved { target, damage, .. } = event
                    && *damage < 0
                    && (0..state.warriors.len() as i16).contains(target)
                    && state.warriors[*target as usize].record().stamina()? > -10
                {
                    health.affected[*target as usize] = -1;
                }
            }
        }
    }
    match resolution.follow_up {
        crate::combat::medical::FollowUp::FinishPlayerAction => {
            events.extend(finish_player_action(state, content, pending.actor as i16)?);
        }
        crate::combat::medical::FollowUp::EndTurn => events.extend(end_turn(state, content)?),
        crate::combat::medical::FollowUp::None => {}
    }
    refresh_active_item_context(state, content)?;
    Ok(events)
}
