//! Manual combat controls recovered from combat114378 and changeweapon1174f0.
use crate::combat::{Combat, runtime};
use crate::feedback::NativeFeedback;
use crate::{Event, Phase, RuleError, Session};
use centurions_content::{
    Content,
    records::{CharacterInformation, Condition, DevelopmentField, ItemDefinition},
};

fn warning(index: i16) -> RuleError {
    RuleError::Refused {
        feedback: vec![NativeFeedback::Warning { index }],
    }
}

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
                && state.warriors[*b as usize].record().active_flag() != 0
        })
        .map(|b| b as usize)
        .ok_or(RuleError::Refused { feedback: vec![] })
}

/// combatchoice1181f2 arms the current turn only; persistent automation is separate.
pub fn automatic(state: &mut Session) -> Result<Vec<Event>, RuleError> {
    let member = actor(state)?;
    let w = &mut state.warriors[member];
    if w.record().automatic_mode()? == 0 {
        w.record_mut().set_automatic_mode(1);
    }
    Ok(vec![
        Event::Feedback(NativeFeedback::Tutorial {
            first: 120,
            last: 120,
        }),
        Event::Sound { resource_id: 141 },
    ])
}

/// checkfordoauto toggles a separate per-warrior byte, not the involuntary status.
pub fn toggle_auto(state: &mut Session, member: usize) -> Result<Vec<Event>, RuleError> {
    actor(state)?;
    if member >= state.warriors.len() {
        return Err("automatic warrior outside squad".into());
    }
    let c = state.combat.as_mut().unwrap();
    c.automatic[member] = !c.automatic[member];
    if member == c.active_body().unwrap_or(-1) as usize {
        let w = &mut state.warriors[member];
        if c.automatic[member] && w.record().automatic_mode()? == 0 {
            w.record_mut().set_automatic_mode(1);
        } else if !c.automatic[member]
            && [
                w.record().faction(),
                w.record().condition(Condition::Retreating),
                w.record().condition(Condition::Confused),
            ]
            .into_iter()
            .all(|value| value.unwrap_or(1) == 0)
            && w.record().automatic_mode()? > 0
        {
            w.record_mut().set_automatic_mode(0);
        }
    }
    let mut events = vec![Event::Sound {
        resource_id: if c.automatic[member] { 147 } else { 139 },
    }];
    if c.automatic[..state.warriors.len()].iter().all(|v| *v) {
        events.push(Event::Feedback(NativeFeedback::Warning { index: 107 }));
    }
    Ok(events)
}

/// combat116384: only before movement; debit this turn's base AP and postpone it.
pub fn delay(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let member = actor(state)?;
    let w = &mut state.warriors[member];
    if w.record().movement_points()? != w.record().movement_per_round()? {
        return Err(warning(59));
    }
    let remaining = w.record().action_points()?.wrapping_sub(
        w.record()
            .action_point_bonus()?
            .wrapping_add(w.record().actions_per_round()?),
    );
    w.record_mut().set_action_points(remaining);
    let c = state.combat.as_mut().unwrap();
    let cursor = c.cursor;
    // WORD447192 is order[109], where combat1164d8 places the delayed body.
    c.order[cursor..].rotate_left(1);
    for (slot, body) in c.order.iter().enumerate().skip(cursor) {
        if (0..state.warriors.len() as i16).contains(body) {
            state.warriors[*body as usize]
                .record_mut()
                .set_defeated_marker(slot as i16);
        }
    }
    c.cursor = cursor.saturating_sub(1);
    runtime::end_turn(state, content)
}

/// combat115d2c: undo movement alone, retaining reactions and RNG already consumed.
pub fn undo(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let member = actor(state)?;
    let c = state.combat.as_ref().unwrap();
    let w = &state.warriors[member];
    if w.record().action_points()?
        != w.record()
            .action_point_bonus()?
            .wrapping_add(w.record().actions_per_round()?)
        || c.turn_origin.is_none()
    {
        return Err(warning(82));
    }
    if [
        w.record().condition(Condition::Helpless),
        w.record().faction(),
        w.record().condition(Condition::Confused),
    ]
    .into_iter()
    .any(|value| value.unwrap_or(1) != 0)
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let origin = c.turn_origin.unwrap();
    let here = c.squad.positions[member];
    let cell = |p: (i16, i16)| {
        centurions_content::mission::OpeningMission::index(
            i32::from(p.0 + c.map.shift.0 + 5),
            i32::from(p.1 + c.map.shift.1 + 5),
        )
    };
    let from = cell(here)?;
    let to = cell(origin)?;
    if origin != here && c.map.tiles[to].wrapping_abs() <= 110 {
        return Err(warning(82));
    }
    let c = state.combat.as_mut().unwrap();
    c.map.tiles[from] = c.squad.under_tiles[member];
    c.squad.under_tiles[member] = c.map.tiles[to];
    c.map.tiles[to] = member as i16;
    c.squad.positions[member] = origin;
    let w = &mut state.warriors[member];
    w.record_mut().set_action_points(0);
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
        runtime::initialize_active_turn(c, &mut state.warriors, mission, &mut state.random)?.0,
    );
    Ok(effects)
}

/// combat115f42: nearest hostile must be at least ten cells away, before confirmation.
pub fn escape_check(state: &Session) -> Result<usize, RuleError> {
    let member = actor(state)?;
    let c = state.combat.as_ref().unwrap();
    let w = &state.warriors[member];
    let origin = c.squad.positions[member];
    let faction = w.record().faction()?;
    let near = |p: (i16, i16)| {
        let dx = f64::from(p.0.wrapping_sub(origin.0));
        let dy = f64::from(p.1.wrapping_sub(origin.1));
        (dx * dx + dy * dy).sqrt() < 10.
    };
    for (i, other) in state.warriors.iter().enumerate() {
        if other.record().active_flag() != 0
            && other.stamina()? > 0
            && other.record().faction()? != faction
            && near(c.squad.positions[i])
        {
            return Err(warning(81));
        }
    }
    for (i, other) in c.enemies.records.iter().enumerate() {
        if other.record().mechanical_state() != 255
            && other.stamina()? > 0
            && other.record().faction()? != faction
            && near(c.enemies.positions[i])
        {
            return Err(warning(81));
        }
    }
    if [
        w.record().condition(Condition::Helpless),
        w.record().faction(),
        w.record().condition(Condition::Confused),
        w.record().condition(Condition::Entangled),
        w.record().condition(Condition::MovingSlowly),
    ]
    .into_iter()
    .any(|value| value.unwrap_or(1) != 0)
    {
        return Err(warning(83));
    }
    Ok(member)
}

pub fn escape(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let member = escape_check(state)?;
    super::retreat::leave(state, content, member, false)
}

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

/// changeweapon11791c..117a04 replaces the equipped category and debits one AP.
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
        .position(|item| item.equipment_flag() != 0 && item.category_code() as i8 == kind);
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
    state.load_item_work(content, selected.id())?;
    state.native_work.inventory_record = selected.clone();
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
            .map(|index| Event::Feedback(NativeFeedback::Warning { index })),
    );
    state.warriors[member] = worn.warrior;
    let w = &mut state.warriors[member];
    let ap = w.record().action_points()?.wrapping_sub(1);
    w.record_mut().set_action_points(ap);
    crate::items::compact(&mut state.warriors[member])?;
    events.extend(runtime::finish_player_action(
        state,
        content,
        member as i16,
    )?);
    Ok(events)
}

/// combatchoice117f02..118040: W/fist mode preserves the original attack selector.
pub fn firearm_mode(
    state: &mut Session,
    content: &Content,
    member: usize,
    enabled: bool,
) -> Result<Vec<Event>, RuleError> {
    if state.phase == Phase::Combat && actor(state)? != member {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let w = &state.warriors.get(member).ok_or("warrior outside squad")?;
    let was_enabled = w.record().firing_mode();
    let mut effects = vec![Event::Sound { resource_id: 141 }];
    if enabled && w.record().equipped_weapon()? == 0 {
        effects.push(Event::Sound { resource_id: 6000 });
        return Ok(effects);
    }
    let melee = w.record().melee_weapon()?;
    let natural = CharacterInformation(&w.bytes)
        .gender()?
        .wrapping_mul(8)
        .wrapping_add(30);
    if !enabled {
        // combatchoice 0x117fb8 stores the loaded item's word 12 (0x453648), where
        // toweap stores its impact sound.
        let attack = if melee == 0 {
            natural
        } else {
            state.load_item_work(content, melee)?;
            ItemDefinition(
                state
                    .native_work
                    .ammunition_work
                    .item
                    .as_ref()
                    .ok_or("melee item work missing")?,
            )
            .pickup_sound()?
        };
        DevelopmentField::MeleeSound.write(&mut state.warriors[member].bytes, attack);
    }
    state.warriors[member].record_mut().set_firing_mode(enabled);
    if was_enabled == enabled {
        effects.push(Event::Sound { resource_id: 6000 });
    }
    Ok(effects)
}
