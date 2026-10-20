//! Manual combat controls recovered from combat114378 and changeweapon1174f0.
use crate::combat::{Combat, put_word, runtime};
use crate::feedback::NativeFeedback;
use crate::{Event, Phase, RuleError, Session};
use centurions_content::{Content, word};

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
    let w = &mut state.warriors[member].bytes;
    if word(w, 0x90)? == 0 {
        put_word(w, 0x90, 1);
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
        let w = &mut state.warriors[member].bytes;
        if c.automatic[member] && word(w, 0x90)? == 0 {
            put_word(w, 0x90, 1);
        } else if !c.automatic[member]
            && [0x10, 0x5e, 0x98]
                .iter()
                .all(|offset| word(w, *offset).unwrap_or(1) == 0)
            && word(w, 0x90)? > 0
        {
            put_word(w, 0x90, 0);
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
    let w = &mut state.warriors[member].bytes;
    if word(w, 0x3c)? != word(w, 0x3e)? {
        return Err(warning(59));
    }
    let remaining = word(w, 0x40)?.wrapping_sub(word(w, 0x1e)?.wrapping_add(word(w, 0x12)?));
    put_word(w, 0x40, remaining);
    let c = state.combat.as_mut().unwrap();
    let cursor = c.cursor;
    // WORD447192 is order[109], where combat1164d8 places the delayed body.
    c.order[cursor..].rotate_left(1);
    for (slot, body) in c.order.iter().enumerate().skip(cursor) {
        if (0..state.warriors.len() as i16).contains(body) {
            put_word(&mut state.warriors[*body as usize].bytes, 0x20, slot as i16);
        }
    }
    c.cursor = cursor.saturating_sub(1);
    runtime::end_turn(state, content)
}

/// combat115d2c: undo movement alone, retaining reactions and RNG already consumed.
pub fn undo(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let member = actor(state)?;
    let c = state.combat.as_ref().unwrap();
    let w = &state.warriors[member].bytes;
    if word(w, 0x40)? != word(w, 0x1e)?.wrapping_add(word(w, 0x12)?) || c.turn_origin.is_none() {
        return Err(warning(82));
    }
    if [0x60, 0x10, 0x98]
        .iter()
        .any(|offset| word(w, *offset).unwrap_or(1) != 0)
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
    let w = &mut state.warriors[member].bytes;
    put_word(w, 0x40, 0);
    let mission = content
        .map(state.exploration.as_ref().unwrap().map)
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
    let w = &state.warriors[member].bytes;
    let origin = c.squad.positions[member];
    let faction = word(w, 0x10)?;
    let near = |p: (i16, i16)| {
        let dx = f64::from(p.0.wrapping_sub(origin.0));
        let dy = f64::from(p.1.wrapping_sub(origin.1));
        (dx * dx + dy * dy).sqrt() < 10.
    };
    for (i, other) in state.warriors.iter().enumerate() {
        if other.bytes[0x9df] != 0
            && other.stamina()? > 0
            && word(&other.bytes, 0x10)? != faction
            && near(c.squad.positions[i])
        {
            return Err(warning(81));
        }
    }
    for (i, other) in c.enemies.records.iter().enumerate() {
        if other.bytes[0xb0] != 255
            && other.stamina()? > 0
            && word(&other.bytes, 0xe)? != faction
            && near(c.enemies.positions[i])
        {
            return Err(warning(81));
        }
    }
    if [0x60, 0x10, 0x98, 0x62, 0x6a]
        .iter()
        .any(|offset| word(w, *offset).unwrap_or(1) != 0)
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
                .filter(move |(_, item)| (item.0[10] as i8).unsigned_abs() == kind)
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
    let kind = inventory[slot].0[10] as i8;
    let old = inventory
        .iter()
        .position(|item| item.0[9] != 0 && item.0[10] as i8 == kind);
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
    if state.inventory_record.is_some() {
        state.inventory_record = Some(selected.clone());
    }
    let mut worn = crate::equipment::wear(
        &state.warriors[member],
        content,
        content
            .map(state.exploration.as_ref().unwrap().map)
            .ok_or("weapon map missing")?,
        slot,
        false,
    )?;
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
    let w = &mut state.warriors[member].bytes;
    let ap = word(w, 0x40)?.wrapping_sub(1);
    put_word(w, 0x40, ap);
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
    let w = &state
        .warriors
        .get(member)
        .ok_or("warrior outside squad")?
        .bytes;
    let was_enabled = w[0x9dd] != 0;
    let mut effects = vec![Event::Sound { resource_id: 141 }];
    if enabled && word(w, 0xe4)? == 0 {
        effects.push(Event::Sound { resource_id: 6000 });
        return Ok(effects);
    }
    let melee = word(w, 0xca)?;
    let natural = word(w, 0x38)?.wrapping_mul(8).wrapping_add(30);
    if !enabled {
        let attack = if melee == 0 {
            natural
        } else {
            state.load_item_work(content, melee)?;
            word(
                state
                    .ammunition_work
                    .item
                    .as_ref()
                    .ok_or("melee item work missing")?,
                44,
            )?
        };
        put_word(&mut state.warriors[member].bytes, 0x128, attack);
    }
    state.warriors[member].bytes[0x9dd] = u8::from(enabled);
    if was_enabled == enabled {
        effects.push(Event::Sound { resource_id: 6000 });
    }
    Ok(effects)
}
