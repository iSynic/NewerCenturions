//! The warrior's turn controls: automatic, delay, undo, escape and weapon choice.
use crate::combat::{Combat, runtime};
use crate::feedback::GameFeedback;
use crate::{Event, Phase, RuleError, Session};
use centurions_content::{
    Content,
    records::{CharacterInformation, Condition, DevelopmentField, ItemDefinition},
};

/// Refuses the control with the numbered warning.
fn warning(index: i16) -> RuleError {
    RuleError::Refused {
        feedback: vec![GameFeedback::Warning { index }],
    }
}

/// The warrior whose turn it is, when no dialog, selection or story is holding the turn and the
/// warrior is alive and in the battle.
pub fn actor(state: &Session) -> Result<usize, RuleError> {
    if state.phase != Phase::Combat
        || state
            .exploration
            .as_ref()
            .is_none_or(|p| p.presentation.is_some())
        || state.psi_selection.is_some()
        || state.medical_continuation.is_some()
        || state
            .combat_story
            .as_ref()
            .is_some_and(|s| !s.waiting_child)
        || state.combat.as_ref().is_none_or(|c| {
            c.selection_pending || c.capture_actor.is_some() || c.firearm_selection.is_some()
        })
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    state
        .combat
        .as_ref()
        .and_then(Combat::active_body)
        .filter(|b| (0..state.warriors.len() as i16).contains(b))
        .filter(|b| {
            state.warriors[*b as usize]
                .stamina()
                .is_ok_and(|stamina| stamina > 0)
                && state.warriors[*b as usize].record().in_battle() != 0
        })
        .map(|b| b as usize)
        .ok_or(RuleError::Refused { feedback: vec![] })
}

/// Puts the current turn on automatic; standing automation is a separate toggle.
///
/// Original: combatchoice
pub fn automatic(state: &mut Session) -> Result<Vec<Event>, RuleError> {
    let member = actor(state)?;
    let warriors = &mut state.warriors[member];
    if warriors.record().computer_control()? == 0 {
        warriors.record_mut().set_computer_control(1);
    }
    Ok(vec![
        Event::Feedback(GameFeedback::Tutorial {
            first: 120,
            last: 120,
        }),
        Event::Sound { resource_id: 141 },
    ])
}

/// Toggles a warrior's standing automation, separate from the computer-control
/// condition.
///
/// Original: checkfordoauto
pub fn toggle_auto(state: &mut Session, member: usize) -> Result<Vec<Event>, RuleError> {
    actor(state)?;
    if member >= state.warriors.len() {
        return Err("automatic warrior outside squad".into());
    }
    let combat = state.combat.as_mut().unwrap();
    combat.do_auto[member] = !combat.do_auto[member];
    if member == combat.active_body().unwrap_or(-1) as usize {
        let warriors = &mut state.warriors[member];
        if combat.do_auto[member] && warriors.record().computer_control()? == 0 {
            warriors.record_mut().set_computer_control(1);
        } else if !combat.do_auto[member]
            && [
                warriors.record().faction(),
                warriors.record().condition(Condition::Retreating),
                warriors.record().condition(Condition::Confused),
            ]
            .into_iter()
            .all(|value| value.unwrap_or(1) == 0)
            && warriors.record().computer_control()? > 0
        {
            warriors.record_mut().set_computer_control(0);
        }
    }
    let mut events = vec![Event::Sound {
        resource_id: if combat.do_auto[member] { 147 } else { 139 },
    }];
    if combat.do_auto[..state.warriors.len()].iter().all(|v| *v) {
        events.push(Event::Feedback(GameFeedback::Warning { index: 107 }));
    }
    Ok(events)
}

/// Delays the turn: only before moving, spending this turn's base attacks.
///
/// Original: combat
pub fn delay(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let member = actor(state)?;
    let warriors = &mut state.warriors[member];
    if warriors.record().movement()? != warriors.record().movement_max()? {
        return Err(warning(59));
    }
    let remaining = warriors.record().attacks()?.wrapping_sub(
        warriors
            .record()
            .attack_bonus()?
            .wrapping_add(warriors.record().normal_attacks()?),
    );
    warriors.record_mut().set_attacks(remaining);
    let combat = state.combat.as_mut().unwrap();
    let cursor = combat.battle_order_index;
    // The delayed body goes to the end of the battle order.
    combat.battle_order[cursor..].rotate_left(1);
    for (slot, body) in combat.battle_order.iter().enumerate().skip(cursor) {
        if (0..state.warriors.len() as i16).contains(body) {
            state.warriors[*body as usize]
                .record_mut()
                .set_position(slot as i16);
        }
    }
    combat.battle_order_index = cursor.saturating_sub(1);
    runtime::end_turn(state, content)
}

/// Undoes movement only; reactions and random draws already taken stand.
///
/// Original: combat
pub fn undo(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let member = actor(state)?;
    let combat = state.combat.as_ref().unwrap();
    let warriors = &state.warriors[member];
    if warriors.record().attacks()?
        != warriors
            .record()
            .attack_bonus()?
            .wrapping_add(warriors.record().normal_attacks()?)
        || combat.turn_origin.is_none()
    {
        return Err(warning(82));
    }
    if [
        warriors.record().condition(Condition::Helpless),
        warriors.record().faction(),
        warriors.record().condition(Condition::Confused),
    ]
    .into_iter()
    .any(|value| value.unwrap_or(1) != 0)
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let origin = combat.turn_origin.unwrap();
    let here = combat.squad.pos[member];
    let cell = |p: (i16, i16)| {
        centurions_content::mission::OpeningMission::index(
            i32::from(p.0 + combat.field.map_shift.0 + 5),
            i32::from(p.1 + combat.field.map_shift.1 + 5),
        )
    };
    let from = cell(here)?;
    let to = cell(origin)?;
    if origin != here && combat.field.tiles[to].wrapping_abs() <= 110 {
        return Err(warning(82));
    }
    let combat = state.combat.as_mut().unwrap();
    combat.field.tiles[from] = combat.squad.char_under[member];
    combat.squad.char_under[member] = combat.field.tiles[to];
    combat.field.tiles[to] = member as i16;
    combat.squad.pos[member] = origin;
    let warriors = &mut state.warriors[member];
    warriors.record_mut().set_attacks(0);
    let mission = crate::Session::land_in(
        content,
        state.mission.as_deref(),
        state.scenario_level,
        state.exploration.as_ref(),
    )
    .ok_or("undo map missing")?;
    let mut effects = vec![
        Event::Sound { resource_id: 664 },
        Event::TacticalMoved {
            body: member as i16,
            x: origin.0,
            y: origin.1,
        },
    ];
    effects.extend(
        runtime::initialize_active_turn(combat, &mut state.warriors, mission, &mut state.random)?.0,
    );
    Ok(effects)
}

/// A warrior may escape only when the nearest hostile is ten or more cells away.
///
/// Original: combat
pub fn escape_check(state: &Session) -> Result<usize, RuleError> {
    let member = actor(state)?;
    let combat = state.combat.as_ref().unwrap();
    let warriors = &state.warriors[member];
    let origin = combat.squad.pos[member];
    let faction = warriors.record().faction()?;
    let near = |p: (i16, i16)| {
        let dx = f64::from(p.0.wrapping_sub(origin.0));
        let dy = f64::from(p.1.wrapping_sub(origin.1));
        (dx * dx + dy * dy).sqrt() < 10.
    };
    for (i, other) in state.warriors.iter().enumerate() {
        if other.record().in_battle() != 0
            && other.stamina()? > 0
            && other.record().faction()? != faction
            && near(combat.squad.pos[i])
        {
            return Err(warning(81));
        }
    }
    for (i, other) in combat.roster.records.iter().enumerate() {
        if other.record().mechanical_state() != 255
            && other.stamina()? > 0
            && other.record().faction()? != faction
            && near(combat.roster.mon_pos[i])
        {
            return Err(warning(81));
        }
    }
    if [
        warriors.record().condition(Condition::Helpless),
        warriors.record().faction(),
        warriors.record().condition(Condition::Confused),
        warriors.record().condition(Condition::Entangled),
        warriors.record().condition(Condition::MovingSlowly),
    ]
    .into_iter()
    .any(|value| value.unwrap_or(1) != 0)
    {
        return Err(warning(83));
    }
    Ok(member)
}

/// Leaves the battle through the active warrior's turn, once the escape check passes.
///
/// Original: combat
pub fn escape(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let member = escape_check(state)?;
    super::retreat::leave(state, content, member, false)
}

/// The inventory slots the warrior can switch to: the items of category 15, then those of category
/// 2.
pub fn weapons(state: &Session) -> Result<Vec<usize>, RuleError> {
    let member = actor(state)?;
    let inventory = state.warriors[member].inventory()?;
    Ok([15_u8, 2]
        .into_iter()
        .flat_map(|kind| {
            inventory
                .iter()
                .enumerate()
                .filter(move |(_, item)| (item.category_code() as i8).unsigned_abs() == kind)
                .map(|(slot, _)| slot)
        })
        .collect())
}

/// Changes the equipped weapon, at the cost of one attack.
///
/// Original: changeweapon
pub fn select_weapon(
    state: &mut Session,
    content: &Content,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    let member = actor(state)?;
    if !weapons(state)?.contains(&slot) {
        return Err("combat weapon slot invalid".into());
    }
    let inventory = state.warriors[member].inventory()?;
    let kind = inventory[slot].category_code() as i8;
    let old = inventory
        .iter()
        .position(|item| item.equipped() != 0 && item.category_code() as i8 == kind);
    let mut events = vec![Event::Sound { resource_id: 405 }];
    let mut selected = inventory[slot].clone();
    if kind > 0
        && let Some(old) = old
    {
        events.extend(crate::equipment::unequip(state, content, member, old)?);
        if old == slot {
            selected.0[9] = 0;
        }
    }
    // Unequipping compacts the inventory; recover this entry by its complete original record.
    let slot = state.warriors[member]
        .inventory()?
        .iter()
        .position(|item| *item == selected)
        .ok_or("combat weapon moved unexpectedly")?;
    state.load_item(content, selected.id())?;
    state.globals.inventory_record = selected.clone();
    let mut worn = crate::equipment::wear(
        &state.warriors[member],
        content,
        state.land(content).ok_or("weapon map missing")?,
        slot,
        false,
    )?;
    state.apply_equipment_globals(&worn.global_writes)?;
    events.extend(
        worn.sounds
            .drain(..)
            .map(|resource_id| Event::Sound { resource_id }),
    );
    events.extend(
        worn.warnings
            .drain(..)
            .map(|index| Event::Feedback(GameFeedback::Warning { index })),
    );
    state.warriors[member] = worn.warrior;
    let warriors = &mut state.warriors[member];
    let ap = warriors.record().attacks()?.wrapping_sub(1);
    warriors.record_mut().set_attacks(ap);
    crate::items::compact(&mut state.warriors[member])?;
    events.extend(runtime::finish_player_action(
        state,
        content,
        member as i16,
    )?);
    Ok(events)
}

/// Switches between firing mode and fists; the attack selector is kept.
///
/// Original: combatchoice
pub fn firearm_mode(
    state: &mut Session,
    content: &Content,
    member: usize,
    enabled: bool,
) -> Result<Vec<Event>, RuleError> {
    if state.phase == Phase::Combat && actor(state)? != member {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let warriors = &state.warriors.get(member).ok_or("warrior outside squad")?;
    let was_enabled = warriors.record().toggle();
    let mut effects = vec![Event::Sound { resource_id: 141 }];
    if enabled && warriors.record().equipped_weapon()? == 0 {
        effects.push(Event::Sound { resource_id: 6000 });
        return Ok(effects);
    }
    let melee = warriors.record().melee_weapon()?;
    let natural = CharacterInformation(&warriors.bytes)
        .gender()?
        .wrapping_mul(8)
        .wrapping_add(30);
    if !enabled {
        // The loaded item's impact sound becomes the attack.
        let attack = if melee == 0 {
            natural
        } else {
            state.load_item(content, melee)?;
            ItemDefinition(
                state
                    .globals
                    .loaded_items
                    .item
                    .as_ref()
                    .ok_or("melee item work missing")?,
            )
            .sound()?
        };
        DevelopmentField::MeleeSound.write(&mut state.warriors[member].bytes, attack);
    }
    state.warriors[member].record_mut().set_toggle(enabled);
    if was_enabled == enabled {
        effects.push(Event::Sound { resource_id: 6000 });
    }
    Ok(effects)
}
