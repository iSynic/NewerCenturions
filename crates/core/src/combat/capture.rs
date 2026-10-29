//! Mechanical disable/capture: updatecontrols16043e, entercapturemode136928,
//! combatchoice11c15a and attack100af0..1010e2.
use crate::combat::runtime;
use crate::feedback::NativeFeedback;
use crate::{Event, Phase, RandomState, RuleError, Session};
use centurions_content::combat::MonsterRecord;
use centurions_content::{
    Content, Warrior,
    records::{CharacterStatisticsMut, Condition, Statistic},
};

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
        || member.is_none_or(|w| w.stamina().unwrap_or(0) <= 0 || w.record().active_flag() == 0)
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
    let warrior = state.warriors[actor as usize].record();
    if state.phase != Phase::Combat
        || state
            .exploration
            .as_ref()
            .is_none_or(|p| p.presentation.is_some())
        || state.psi_selection.is_some()
        || state.medical_continuation.is_some()
        || combat.firearm_selection.is_some()
        || runtime::controlled_warrior(state).is_some()
        || warrior.special_skill(0).unwrap_or(0) <= 0
        || warrior.action_points().unwrap_or(0) <= 1
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
            enemy.record().faction().ok() != warrior.faction().ok()
                && enemy.stamina().unwrap_or(0) > 0
                && (enemy.record().mechanical_state() == 1
                    || warrior.special_skill(1).unwrap_or(0) != 0
                        && enemy.record().capture_flag() != 0)
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
    let prior_mechanical = combat.enemies.records[enemy].record().mechanical_state();
    let effects = resolve(
        &mut state.warriors[actor],
        &mut combat.enemies.records[enemy],
        &mut state.random,
    )?;
    if prior_mechanical != 255 && combat.enemies.records[enemy].record().mechanical_state() == 255 {
        combat.victory_work.add_total(-1);
    }
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
    if warrior.record().attacks_used()? >= warrior.record().attack_limit()? {
        return Ok(vec![Event::Feedback(NativeFeedback::Warning { index: 45 })]);
    }
    let attacks = warrior.record().attacks_used()?.wrapping_add(1);
    warrior.record_mut().set_attacks_used(attacks);
    let mut events = vec![flash(0x40250d, -1, 0)];
    if enemy.record().mechanical_state() == 0 {
        events.push(Event::Feedback(NativeFeedback::Warning { index: 16 }));
        return Ok(events);
    }
    let capture = enemy.record().mechanical_state() == 255;
    let skill_index = usize::from(capture);
    let level = enemy.record().level()?;
    let skill = warrior
        .record()
        .special_skill(skill_index)?
        .wrapping_add(warrior.record().special_skill_bonus(skill_index)?);
    let chance = crate::rules::pin(
        skill.wrapping_sub(
            level
                .wrapping_sub(1)
                .wrapping_mul(if capture { 2 } else { 1 }),
        ),
        if capture { 3 } else { 5 },
        if capture { 90 } else { 95 },
    );
    let ap = warrior.record().action_points()?.wrapping_sub(2);
    warrior.record_mut().set_action_points(ap);
    if random.bound(100) > chance
        || capture && enemy.record().faction()? == warrior.record().faction()?
    {
        events.push(flash(0x404088, 60, 6000));
        return Ok(events);
    }
    let reward = enemy
        .record()
        .maximum_stamina()?
        .wrapping_mul(if capture { 3 } else { 2 })
        .wrapping_add(level.wrapping_mul(if capture { 15 } else { 10 }));
    warrior
        .record_mut()
        .add_remaining_experience(-i32::from(reward));
    let points = warrior
        .record()
        .upgrade_points()
        .wrapping_add(i32::from(reward));
    warrior.record_mut().set_upgrade_points(points);
    for resource_id in if capture { [-209, 208] } else { [-206, 205] } {
        events.push(Event::Sound { resource_id });
    }
    enemy
        .record_mut()
        .set_mechanical_state(if capture { 1 } else { 255 });
    if !capture {
        enemy.record_mut().set_condition(Condition::Helpless, -1);
    }
    if !capture || warrior.record().faction()? == 0 {
        let bound = crate::rules::pin(
            (if capture { 5000_i16 } else { 10000_i16 })
                .wrapping_sub(level.wrapping_mul(if capture { 50 } else { 75 })),
            if capture { 50 } else { 75 },
            if capture { 1950 } else { 2925 },
        );
        if random.bound(bound) <= level && skill < 95 {
            let improved = warrior.record().special_skill(skill_index)?.wrapping_add(1);
            warrior
                .record_mut()
                .set_special_skill(skill_index, improved);
            events.push(Event::Feedback(NativeFeedback::Warning {
                index: if capture { 143 } else { 142 },
            }));
        }
        CharacterStatisticsMut(&mut warrior.bytes).add(
            if capture {
                Statistic::VehiclesCaptured
            } else {
                Statistic::VehiclesDisabled
            },
            1,
        );
        if capture {
            enemy.record_mut().set_condition(Condition::Helpless, 0);
            enemy.record_mut().set_faction(0);
        }
    } else {
        enemy.record_mut().set_faction(warrior.record().faction()?);
    }
    Ok(events)
}
