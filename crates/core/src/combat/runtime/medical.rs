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
    refresh_item_context(state, content, body)
}

pub(crate) fn refresh_item_context(
    state: &mut Session,
    content: &Content,
    body: i16,
) -> Result<(), RuleError> {
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
    let clear_firing = warrior.record().firing_mode() && item == 0;
    // combatupdate2 11f662 loads the secondary WORD without an inventory lookup.
    state.load_item_work(content, item)?;
    if clear_firing {
        // combatupdate2's missing-secondary branch11fb7a clears the mode byte.
        state.warriors[body as usize]
            .record_mut()
            .set_firing_mode(false);
    }
    Ok(())
}

pub fn stop_bleeding(
    state: &mut Session,
    content: &Content,
    warrior: Option<usize>,
) -> Result<Vec<Event>, RuleError> {
    medical_turn(state)?;
    if state.bandage_ready == 0 {
        return Err(RuleError::Invalid(
            "Stop Bleeding is unavailable after the previous action".into(),
        ));
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
    crate::combat::item_activation::ordinary::begin(state, content, warrior, slot)
}

pub fn begin_learned_heal(
    state: &mut Session,
    content: &Content,
    power: i16,
) -> Result<Vec<Event>, RuleError> {
    let actor = medical_turn(state)?;
    crate::combat::player_psi::select_native_caster(state);
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
    result.events.retain(|event| event.sound() != Some(137));
    if power != previous_power {
        result.events.push(Event::Sound { resource_id: 137 });
    }
    state.warriors[actor] = result.warrior;
    state.selected_warrior = actor;
    // castspell 0x10d328 stores the cast power in WORD 0x437a7a + 2 * member.
    state.native_work.psi_powers[actor] = None;
    state.psi_menu_choices[actor] = Some(crate::PsiMenuChoice {
        row: 0,
        column: 0,
        power,
    });
    state.native_work.attack_count = result.count;
    state.native_work.party_mask.fill(0);
    let mut health =
        crate::story_health::retained_context(actor, &state.native_work.health_context);
    health.affected[actor] = 1;
    health.effect = centurions_content::records::AttackPacket(&result.packet).health_effect();
    state.native_work.health_context = health;
    let combat = state.combat.as_mut().unwrap();
    combat.attack_packet = result.packet;
    combat.attack_identity = result.identity;
    combat.weapon_work = result.weapon_work;
    state.set_ready(1);
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
    if pending.origin == crate::MedicalOrigin::OwnedItem
        && let crate::MedicalTarget::LivingPoint { x, y } = selection
    {
        state.medical_continuation = None;
        state.psi_selection = Some(crate::combat::psi_selection::Selection {
            actor: pending.actor,
            spell: pending.spell,
            power: pending.power,
            targets: vec![],
            item: Some(pending.item),
        });
        return crate::combat::item_activation::ordinary::fire_at(state, content, (x, y), 0);
    }
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
    let packet = &state.combat.as_ref().unwrap().attack_packet;
    let health = &mut state.native_work.health_context;
    health.effect = centurions_content::records::AttackPacket(packet).health_effect();
    if pending.origin == crate::MedicalOrigin::LearnedHeal
        && matches!(selection, crate::MedicalTarget::Portrait { .. })
    {
        let memory = &mut state.native_work.psi_memory;
        centurions_content::records::PsiSelectionMemoryMut(memory)
            .remember_portrait_heal(pending.actor, pending.power);
    }
    state.medical_continuation = None;
    state.native_work.party_mask.fill(0);
    let mut events = resolution.events;
    let health = &mut state.native_work.health_context;
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
