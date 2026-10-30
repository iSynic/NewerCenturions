//! Medical items during combat.
use super::{end_turn, finish_player_action};
use crate::combat::Combat;
use crate::combat::item_activation::ordinary;
use crate::combat::medical;
use crate::combat::medical::FollowUp;
use crate::combat::player_psi::selection::Selection;
use crate::{Event, Phase, RuleError, Session};
use centurions_content::Content;

/// The warrior whose turn a medical action takes; refused unless combat has an unobstructed warrior
/// turn.
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

/// Reloads the item the active body strikes with.
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

/// Reloads the item a body strikes with: a monster's weapon, or a warrior's second weapon in firing
/// mode and the melee weapon otherwise. A warrior in firing mode with no second weapon leaves
/// firing mode.
pub(crate) fn refresh_item_context(
    state: &mut Session,
    content: &Content,
    body: i16,
) -> Result<(), RuleError> {
    if body >= 10 {
        let combat = state.combat.as_ref().unwrap();
        let item = combat.roster.records[(body - 10) as usize]
            .record()
            .weapon()?;
        if item != 0 {
            state.load_item(content, item)?;
        }
        return Ok(());
    }
    if !(0..6).contains(&body) {
        return Err(RuleError::Invalid(
            "item refresh body outside represented actors".into(),
        ));
    }
    let warrior = &state.warriors[body as usize];
    let item = if warrior.record().toggle() {
        warrior.record().equipped_weapon()?
    } else {
        warrior.record().melee_weapon()?
    };
    let clear_firing = warrior.record().toggle() && item == 0;
    // The second weapon loads by its id, without looking it up in the inventory.
    state.load_item(content, item)?;
    if clear_firing {
        // A missing second weapon clears firing mode.
        state.warriors[body as usize].record_mut().set_toggle(false);
    }
    Ok(())
}

/// Stops the chosen warrior's bleeding as the turn's action, only while the previous action left
/// the bandage ready. The turn ends afterwards.
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
    state.globals.party_track.fill(0);
    let mut events = vec![Event::Sound { resource_id: 10105 }];
    if let Some(member) = warrior {
        state.globals.party_track[member] = 1;
        state.warriors[member].record_mut().set_bleeding(0);
        events.push(Event::Sound { resource_id: 141 });
    }
    events.extend(end_turn(state, content)?);
    refresh_active_item_context(state, content)?;
    Ok(events)
}

/// Begins using a medical item from an inventory slot on a warrior's turn.
pub fn begin_medical_item(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    medical_turn(state)?;
    ordinary::begin(state, content, warrior, slot)
}

/// Begins the active warrior's learned heal at the chosen power. The warrior's remembered psi menu
/// choice becomes that power and the heal waits for its target.
pub fn begin_learned_heal(
    state: &mut Session,
    content: &Content,
    power: i16,
) -> Result<Vec<Event>, RuleError> {
    let actor = medical_turn(state)?;
    crate::combat::player_psi::select_combat_caster(state);
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
    let mut result = medical::begin_learned_heal(state, content, power)?;
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
    state.char_select_new = actor;
    // The warrior's remembered spell power is forgotten.
    state.globals.old_power_level[actor] = None;
    state.psi_menu_choices[actor] = Some(crate::PsiMenuChoice {
        row: 0,
        column: 0,
        power,
    });
    state.globals.attack_count = result.count;
    state.globals.party_track.fill(0);
    let mut health = crate::story::health::reselected_context(actor, &state.globals.health_context);
    health.affected[actor] = 1;
    health.effect = centurions_content::records::SpellInfoRecord(&result.packet).health_effect();
    state.globals.health_context = health;
    let combat = state.combat.as_mut().unwrap();
    combat.spell_info = result.packet;
    combat.cast_identity = result.identity;
    combat.weapon_traits = result.weapon_traits;
    state.set_in_spell(1);
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

/// Chooses the target of the pending medical item or heal and carries it out.
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
        state.psi_selection = Some(Selection {
            actor: pending.actor,
            spell: pending.spell,
            power: pending.power,
            targets: vec![],
            item: Some(pending.item),
        });
        return ordinary::fire_at(state, content, (x, y), 0);
    }
    let mission = state
        .land(content)
        .ok_or_else(|| RuleError::Invalid("medical map missing".into()))?;
    let resolution = medical::resolve(
        state.combat.as_mut().unwrap(),
        &mut state.warriors,
        mission,
        &mut state.random,
        &pending,
        &selection,
    )?;
    let packet = &state.combat.as_ref().unwrap().spell_info;
    let health = &mut state.globals.health_context;
    health.effect = centurions_content::records::SpellInfoRecord(packet).health_effect();
    if pending.origin == crate::MedicalOrigin::LearnedHeal
        && matches!(selection, crate::MedicalTarget::Portrait { .. })
    {
        let memory = &mut state.globals.define_spells;
        centurions_content::records::PsiSelectionMemoryMut(memory)
            .remember_portrait_heal(pending.actor, pending.power);
    }
    state.medical_continuation = None;
    state.globals.party_track.fill(0);
    let mut events = resolution.events;
    let health = &mut state.globals.health_context;
    health.kill_party = state.combat.as_ref().unwrap().kill_party;
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
        FollowUp::FinishPlayerAction => {
            events.extend(finish_player_action(state, content, pending.actor as i16)?);
        }
        FollowUp::EndTurn => events.extend(end_turn(state, content)?),
        FollowUp::None => {}
    }
    refresh_active_item_context(state, content)?;
    Ok(events)
}
