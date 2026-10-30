//! Capture mode: a warrior disables a mechanical enemy instead of destroying it.
use crate::combat::runtime;
use crate::feedback::GameFeedback;
use crate::{Event, Phase, RandomState, RuleError, Session};
use centurions_content::combat::MonsterRecord;
use centurions_content::{
    Content, Warrior,
    records::{CharacterStatisticsMut, Condition, Statistic},
};

/// A refusal that shows the player nothing.
fn silent() -> RuleError {
    RuleError::Refused { feedback: vec![] }
}

/// Checks that a capture in progress still has its warrior's turn, a live warrior in the battle,
/// and no other selection, dialog or story open.
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
        || member.is_none_or(|w| w.stamina().unwrap_or(0) <= 0 || w.record().in_battle() == 0)
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

/// Whether the warrior whose turn it is can try to disable or capture: the first special skill
/// above zero, attacks above one, and an adjacent living opposing monster that is a working machine
/// (mechanical state 1) or, with the second skill, one with the capture flag set.
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
        || warrior.spec(0).unwrap_or(0) <= 0
        || warrior.attacks().unwrap_or(0) <= 1
    {
        return false;
    }
    let origin = combat.squad.pos[actor as usize];
    (-1..=1).any(|dx| {
        (-1..=1).any(|dy| {
            let Ok(index) = crate::combat::records::grid_index(
                (origin.0 + dx, origin.1 + dy),
                combat.field.map_shift,
            ) else {
                return false;
            };
            let body = combat.field.tiles[index].wrapping_abs();
            let Some(monster) = usize::try_from(body - 10)
                .ok()
                .and_then(|i| combat.roster.records.get(i))
            else {
                return false;
            };
            monster.record().faction().ok() != warrior.faction().ok()
                && monster.stamina().unwrap_or(0) > 0
                && (monster.record().mechanical_state() == 1
                    || warrior.spec(1).unwrap_or(0) != 0 && monster.record().capture_flag() != 0)
        })
    })
}

/// Enters capture mode for the active warrior and prompts for a target.
pub fn begin(state: &mut Session) -> Result<Vec<Event>, RuleError> {
    if !available(state) {
        return Err(silent());
    }
    let combat = state.combat.as_mut().unwrap();
    combat.capture_actor = combat.active_body();
    Ok(vec![
        flash(0x4024de, -1, 5000),
        Event::Feedback(GameFeedback::Tutorial {
            first: 113,
            last: 115,
        }),
    ])
}

/// The event that flashes the text at `address` at a fixed spot on screen, with the given wait and
/// sound.
fn flash(address: u32, wait: i16, sound: i16) -> Event {
    Event::Feedback(GameFeedback::Flash {
        address,
        x: 30,
        y: 300,
        wait,
        sound,
    })
}

/// Leaves capture mode without attacking.
pub fn cancel(state: &mut Session) -> Result<Vec<Event>, RuleError> {
    let combat = state.combat.as_mut().ok_or_else(silent)?;
    if combat.capture_actor.take().is_none() {
        return Err(silent());
    }
    Ok(vec![flash(0x40250d, -1, 0)])
}

/// Attacks the monster in the cell next to the warrior toward the chosen point. Capture mode
/// attacks and never walks.
pub fn select(
    state: &mut Session,
    content: &Content,
    x: i16,
    y: i16,
) -> Result<Vec<Event>, RuleError> {
    let combat = state.combat.as_ref().ok_or_else(silent)?;
    let actor = combat.capture_actor.ok_or_else(silent)? as usize;
    let origin = combat.squad.pos[actor];
    let point = (
        origin.0.wrapping_add(x.wrapping_sub(origin.0).signum()),
        origin.1.wrapping_add(y.wrapping_sub(origin.1).signum()),
    );
    let index = crate::combat::records::grid_index(point, combat.field.map_shift)?;
    let body = combat.field.tiles[index].wrapping_abs();
    // Capture mode attacks the next cell; it never walks.
    let Some(monster) = usize::try_from(body - 10)
        .ok()
        .filter(|i| *i < combat.roster.records.len())
    else {
        return Err(silent());
    };
    let combat = state.combat.as_mut().unwrap();
    let prior_mechanical = combat.roster.records[monster].record().mechanical_state();
    let effects = resolve(
        &mut state.warriors[actor],
        &mut combat.roster.records[monster],
        &mut state.random,
    )?;
    if prior_mechanical != 255 && combat.roster.records[monster].record().mechanical_state() == 255
    {
        combat.kill_counts.add_num_enemy(-1);
    }
    // The attack-limit warning leaves capture mode on.
    if !effects
        .iter()
        .any(|e| matches!(e, Event::Feedback(GameFeedback::Warning { index: 45 })))
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

/// Resolves one disable or capture attempt on a machine. It counts one use against the attack limit
/// (only warning 45 when none are left) and takes two from the warrior's attacks. The roll uses the
/// first special skill to disable a working machine or the second to capture a disabled one.
/// Success moves a reward from the warrior's remaining experience into upgrade points and may raise
/// the skill. Sounds and messages come back in order.
///
/// Original: attack
pub fn resolve(
    warrior: &mut Warrior,
    monster: &mut MonsterRecord,
    random: &mut RandomState,
) -> Result<Vec<Event>, RuleError> {
    if warrior.record().attacks_used()? >= warrior.record().attack_limit()? {
        return Ok(vec![Event::Feedback(GameFeedback::Warning { index: 45 })]);
    }
    let attacks = warrior.record().attacks_used()?.wrapping_add(1);
    warrior.record_mut().set_attacks_used(attacks);
    let mut events = vec![flash(0x40250d, -1, 0)];
    if monster.record().mechanical_state() == 0 {
        events.push(Event::Feedback(GameFeedback::Warning { index: 16 }));
        return Ok(events);
    }
    let capture = monster.record().mechanical_state() == 255;
    let skill_index = usize::from(capture);
    let level = monster.record().hit_dice()?;
    let skill = warrior
        .record()
        .spec(skill_index)?
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
    let ap = warrior.record().attacks()?.wrapping_sub(2);
    warrior.record_mut().set_attacks(ap);
    if random.bound(100) > chance
        || capture && monster.record().faction()? == warrior.record().faction()?
    {
        events.push(flash(0x404088, 60, 6000));
        return Ok(events);
    }
    let reward = monster
        .record()
        .stamina_max()?
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
    monster
        .record_mut()
        .set_mechanical_state(if capture { 1 } else { 255 });
    if !capture {
        monster.record_mut().set_condition(Condition::Helpless, -1);
    }
    if !capture || warrior.record().faction()? == 0 {
        let bound = crate::rules::pin(
            (if capture { 5000_i16 } else { 10000_i16 })
                .wrapping_sub(level.wrapping_mul(if capture { 50 } else { 75 })),
            if capture { 50 } else { 75 },
            if capture { 1950 } else { 2925 },
        );
        if random.bound(bound) <= level && skill < 95 {
            let improved = warrior.record().spec(skill_index)?.wrapping_add(1);
            warrior.record_mut().set_spec(skill_index, improved);
            events.push(Event::Feedback(GameFeedback::Warning {
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
            monster.record_mut().set_condition(Condition::Helpless, 0);
            monster.record_mut().set_faction(0);
        }
    } else {
        monster
            .record_mut()
            .set_faction(warrior.record().faction()?);
    }
    Ok(events)
}
