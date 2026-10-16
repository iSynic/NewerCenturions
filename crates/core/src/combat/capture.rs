//! Mechanical disable/capture: updatecontrols16043e, entercapturemode136928,
//! combatchoice11c15a and attack100af0..1010e2.
use super::*;
use crate::feedback::NativeFeedback;

fn silent() -> RuleError {
    RuleError::Refused { feedback: vec![] }
}

pub(crate) fn validate_selection(state: &Session) -> Result<(), RuleError> {
    let Some(combat) = state.combat.as_ref() else {
        return Ok(());
    };
    let Some(actor) = combat.capture_actor else {
        return Ok(());
    };
    let member = usize::try_from(actor)
        .ok()
        .and_then(|i| state.warriors.get(i));
    if state.phase != Phase::Combat
        || combat.active_body() != Some(actor)
        || member.is_none_or(|w| w.stamina().unwrap_or(0) <= 0 || w.bytes[0x9df] == 0)
        || combat.selection_pending
        || combat.firearm_selection.is_some()
        || state.psi_selection.is_some()
        || state.outdoor_psi.is_some()
        || state.medical_continuation.is_some()
        || state
            .exploration
            .as_ref()
            .is_none_or(|p| p.presentation.is_some())
        || state
            .combat_story
            .as_ref()
            .is_some_and(|story| !story.waiting_child)
    {
        return Err(RuleError::Invalid(
            "conflicting mechanical targeting context".into(),
        ));
    }
    Ok(())
}

pub fn available(state: &Session) -> bool {
    let Some(combat) = &state.combat else {
        return false;
    };
    let Some(actor) = combat
        .active_body()
        .filter(|i| (0..state.warriors.len() as i16).contains(i))
    else {
        return false;
    };
    let warrior = &state.warriors[actor as usize].bytes;
    if state.phase != Phase::Combat
        || state
            .exploration
            .as_ref()
            .is_none_or(|p| p.presentation.is_some())
        || state.psi_selection.is_some()
        || state.medical_continuation.is_some()
        || combat.firearm_selection.is_some()
        || runtime::controlled_warrior(state).is_some()
        || word(warrior, 0xf4).unwrap_or(0) <= 0
        || word(warrior, 0x40).unwrap_or(0) <= 1
    {
        return false;
    }
    let origin = combat.squad.positions[actor as usize];
    (-1..=1).any(|dx| {
        (-1..=1).any(|dy| {
            let Ok(index) = centurions_content::mission::OpeningMission::index(
                i32::from(origin.0 + dx) + i32::from(combat.map.shift.0) + 5,
                i32::from(origin.1 + dy) + i32::from(combat.map.shift.1) + 5,
            ) else {
                return false;
            };
            let body = combat.map.tiles[index].wrapping_abs();
            let Some(enemy) = usize::try_from(body - 10)
                .ok()
                .and_then(|i| combat.enemies.records.get(i))
            else {
                return false;
            };
            word(&enemy.bytes, 0xe).ok() != word(warrior, 0x10).ok()
                && enemy.stamina().unwrap_or(0) > 0
                && (enemy.bytes[0xb0] == 1
                    || word(warrior, 0xf6).unwrap_or(0) != 0 && enemy.bytes[0xb1] != 0)
        })
    })
}

pub fn begin(state: &mut Session) -> Result<Vec<Event>, RuleError> {
    if !available(state) {
        return Err(silent());
    }
    let combat = state.combat.as_mut().unwrap();
    combat.capture_actor = combat.active_body();
    Ok(vec![
        flash(0x4024de, -1, 5000),
        Event::Feedback(NativeFeedback::Tutorial {
            first: 113,
            last: 115,
        }),
    ])
}

fn flash(address: u32, wait: i16, sound: i16) -> Event {
    Event::Feedback(NativeFeedback::Flash {
        address,
        x: 30,
        y: 300,
        wait,
        sound,
    })
}

pub fn cancel(state: &mut Session) -> Result<Vec<Event>, RuleError> {
    let combat = state.combat.as_mut().ok_or_else(silent)?;
    if combat.capture_actor.take().is_none() {
        return Err(silent());
    }
    Ok(vec![flash(0x40250d, -1, 0)])
}

pub fn select(
    state: &mut Session,
    content: &Content,
    x: i16,
    y: i16,
) -> Result<Vec<Event>, RuleError> {
    let combat = state.combat.as_ref().ok_or_else(silent)?;
    let actor = combat.capture_actor.ok_or_else(silent)? as usize;
    let origin = combat.squad.positions[actor];
    let point = (
        origin.0.wrapping_add(x.wrapping_sub(origin.0).signum()),
        origin.1.wrapping_add(y.wrapping_sub(origin.1).signum()),
    );
    let index = centurions_content::mission::OpeningMission::index(
        i32::from(point.0) + i32::from(combat.map.shift.0) + 5,
        i32::from(point.1) + i32::from(combat.map.shift.1) + 5,
    )?;
    let body = combat.map.tiles[index].wrapping_abs();
    // The original passes the next cell to attack, never walks in capture mode.
    let Some(enemy) = usize::try_from(body - 10)
        .ok()
        .filter(|i| *i < combat.enemies.records.len())
    else {
        return Err(silent());
    };
    let combat = state.combat.as_mut().unwrap();
    let effects = resolve(
        &mut state.warriors[actor],
        &mut combat.enemies.records[enemy],
        &mut state.random,
    )?;
    // The attack-limit warning returns before exitcapturemode in the original.
    if !effects
        .iter()
        .any(|e| matches!(e, Event::Feedback(NativeFeedback::Warning { index: 45 })))
    {
        combat.capture_actor = None;
    }
    if combat.capture_actor.is_some() {
        return Ok(effects);
    }
    let mut events = effects;
    events.extend(runtime::finish_player_action(state, content, actor as i16)?);
    Ok(events)
}

/// Complete native record mutations; presentation services are returned in order.
pub fn resolve(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    random: &mut RandomState,
) -> Result<Vec<Event>, RuleError> {
    let w = &mut warrior.bytes;
    let m = &mut enemy.bytes;
    if word(w, 0xacc)? >= word(w, 0xaca)? {
        return Ok(vec![Event::Feedback(NativeFeedback::Warning { index: 45 })]);
    }
    let attacks = word(w, 0xacc)?.wrapping_add(1);
    put_word(w, 0xacc, attacks);
    let mut events = vec![flash(0x40250d, -1, 0)];
    if m[0xb0] == 0 {
        events.push(Event::Feedback(NativeFeedback::Warning { index: 16 }));
        return Ok(events);
    }
    let capture = m[0xb0] == 255;
    let offset = if capture { 0xf6 } else { 0xf4 };
    let bonus = if capture { 0xb64 } else { 0xb62 };
    let level = word(m, 0)?;
    let skill = word(w, offset)?.wrapping_add(word(w, bonus)?);
    let chance = crate::rules::pin(
        skill.wrapping_sub(
            level
                .wrapping_sub(1)
                .wrapping_mul(if capture { 2 } else { 1 }),
        ),
        if capture { 3 } else { 5 },
        if capture { 90 } else { 95 },
    );
    let ap = word(w, 0x40)?.wrapping_sub(2);
    put_word(w, 0x40, ap);
    if random.bound(100) > chance || capture && word(m, 0xe)? == word(w, 0x10)? {
        events.push(flash(0x404088, 60, 6000));
        return Ok(events);
    }
    let reward = word(m, 0x112)?
        .wrapping_mul(if capture { 3 } else { 2 })
        .wrapping_add(level.wrapping_mul(if capture { 15 } else { 10 }));
    for (offset, delta) in [(0x9ce, -i32::from(reward)), (0x9d2, i32::from(reward))] {
        let current = i32::from_be_bytes(w[offset..offset + 4].try_into().unwrap());
        w[offset..offset + 4].copy_from_slice(&current.wrapping_add(delta).to_be_bytes());
    }
    for resource_id in if capture { [-209, 208] } else { [-206, 205] } {
        events.push(Event::Sound { resource_id });
    }
    m[0xb0] = if capture { 1 } else { 255 };
    if !capture {
        m[0xb3] = 255;
    }
    if !capture || word(w, 0x10)? == 0 {
        let bound = crate::rules::pin(
            (if capture { 5000_i16 } else { 10000_i16 })
                .wrapping_sub(level.wrapping_mul(if capture { 50 } else { 75 })),
            if capture { 50 } else { 75 },
            if capture { 1950 } else { 2925 },
        );
        if random.bound(bound) <= level && skill < 95 {
            let improved = word(w, offset)?.wrapping_add(1);
            put_word(w, offset, improved);
            events.push(Event::Feedback(NativeFeedback::Warning {
                index: if capture { 143 } else { 142 },
            }));
        }
        let stat = if capture { 0xa6e } else { 0xa6a };
        let current = i32::from_be_bytes(w[stat..stat + 4].try_into().unwrap());
        w[stat..stat + 4].copy_from_slice(&current.wrapping_add(1).to_be_bytes());
        if capture {
            m[0xb3] = 0;
            put_word(m, 0xe, 0);
        }
    } else {
        put_word(m, 0xe, word(w, 0x10)?);
    }
    Ok(events)
}
