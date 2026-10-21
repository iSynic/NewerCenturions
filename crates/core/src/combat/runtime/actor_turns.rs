//! Actor turns in the original combat execution order.
use super::{
    EnemyWork, alive, end_turn, enemy_action_with_work_and_buffs, finish_player_action, hostile,
    neighbors, player_shoot_action, position, refresh_active_item_context,
};
use crate::{Event, Phase, RuleError, Session};
use centurions_content::{Content, Warrior};

pub fn enemy_turn(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let mission = content
        .map(
            state
                .exploration
                .as_ref()
                .map_or(0, |position| position.map),
        )
        .ok_or_else(|| RuleError::Invalid("mission missing".into()))?;
    let mut party_hp = state
        .warriors
        .iter()
        .map(Warrior::stamina)
        .collect::<Result<Vec<_>, _>>()?;
    let c = state
        .combat
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
    let actor = c
        .active_body()
        .ok_or_else(|| RuleError::Invalid("active enemy missing".into()))?;
    let e = usize::try_from(actor - 10)
        .map_err(|_| RuleError::Invalid("active body is not an enemy".into()))?;
    let mut events = if c.enemies.records[e].record().faction()? == 0
        && c.enemies.records[e].record().primary_weapon()? == 1
    {
        let context = state.loaded_item_field32.ok_or(RuleError::Unsupported {
            action: "combat.scientist",
            dependency: "unreconstructable legacy work-item context",
        })?;
        crate::combat::ally_turns::scientist_turn_with_work(
            c,
            &mut state.warriors,
            mission,
            &mut state.random,
            &content.items,
            context,
            &state.party_buffs,
            &mut state.ammunition_work,
            &mut state.attack_count,
        )?
    } else {
        let mut work = EnemyWork {
            item_context: state.loaded_item_field32,
            attack_count: state.attack_count,
            packet: c.attack_packet.clone(),
            identity: c.attack_identity,
            spell_attempt: 0,
            spell_disabled: state.enemy_spell_disabled,
            buffers: state.ammunition_work.clone(),
            display_writers: true,
        };
        let events = enemy_action_with_work_and_buffs(
            c,
            &mut state.warriors,
            mission,
            &mut state.random,
            &content.items,
            &mut work,
            &state.party_buffs,
        )?;
        state.loaded_item_field32 = work.item_context;
        state.attack_count = work.attack_count;
        state.ammunition_work = work.buffers;
        events
    };
    if let Some(health) = state.health_context.as_mut() {
        for event in &events {
            match event {
                Event::SpellResolved {
                    target,
                    damage,
                    defeated,
                    ..
                } => {
                    if let Some(hp) = usize::try_from(*target)
                        .ok()
                        .and_then(|body| party_hp.get_mut(body))
                        && *damage > 0
                        && *hp >= 0
                    {
                        health.affected[*target as usize] = -1;
                        *hp = hp.wrapping_sub(*damage);
                    }
                    if *defeated {
                        health.last_body = *target;
                    }
                }
                Event::AttackResolved {
                    target,
                    damage,
                    defeated,
                    ..
                } => {
                    if let Some(hp) = usize::try_from(*target)
                        .ok()
                        .and_then(|body| party_hp.get_mut(body))
                        && *damage > 0
                    {
                        *hp = hp.wrapping_sub(*damage);
                    }
                    if *defeated {
                        health.last_body = *target;
                    }
                }
                _ => {}
            }
        }
    }
    events.extend(finish_player_action(state, content, actor)?);
    refresh_active_item_context(state, content)?;
    Ok(events)
}

pub fn controlled_warrior(state: &Session) -> Option<usize> {
    let body = state.combat.as_ref()?.active_body()?;
    let member = usize::try_from(body).ok()?;
    let warrior = state.warriors.get(member)?;
    (state.phase == Phase::Combat
        && warrior.record().active_flag() != 0
        && warrior.stamina().ok()? > 0
        && (state.combat.as_ref()?.automatic[member]
            || warrior
                .record()
                .automatic_mode()
                .is_ok_and(|value| value != 0)
            || warrior.record().faction().is_ok_and(|value| value != 0)
            || warrior
                .record()
                .fleeing_duration()
                .is_ok_and(|value| value != 0)))
    .then_some(member)
}

/// Automatic combatchoice movement; each command completes a move, an attack, or the turn.
pub fn controlled_turn(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let member = controlled_warrior(state).ok_or("no controlled warrior turn")?;
    if state
        .exploration
        .as_ref()
        .is_none_or(|p| p.presentation.is_some())
        || state
            .combat_story
            .as_ref()
            .is_some_and(|s| !s.waiting_child)
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let combat = state.combat.as_ref().unwrap();
    let edge_point = combat.squad.positions[member];
    let absolute = (
        edge_point.0 + combat.map.shift.0 + 5,
        edge_point.1 + combat.map.shift.1 + 5,
    );
    if !(1..=88).contains(&absolute.0) || !(1..=88).contains(&absolute.1) {
        return crate::combat::retreat::leave_edge(state, content, member);
    }
    if state.warriors[member].record().movement_points()? < 1
        || state.warriors[member].record().action_points()? < 2
    {
        state.warriors[member].record_mut().set_guarding(1);
        return end_turn(state, content);
    }
    let neighbors = neighbors(combat, &state.warriors, member as i16)?;
    let mut target = neighbors
        .first()
        .copied()
        .unwrap_or(state.warriors[member].record().target_body()?);
    if !alive(combat, &state.warriors, target)
        || !hostile(combat, &state.warriors, member as i16, target)?
    {
        let eligible = (0..110_i16).any(|body| {
            alive(combat, &state.warriors, body)
                && hostile(combat, &state.warriors, member as i16, body).unwrap_or(false)
        });
        if !eligible {
            return end_turn(state, content);
        }
        let mut selected = None;
        for _ in 0..10_000 {
            let candidate = state.random.range(0, 109);
            if alive(combat, &state.warriors, candidate)
                && hostile(combat, &state.warriors, member as i16, candidate)?
            {
                selected = Some(candidate);
                break;
            }
        }
        target = selected.ok_or("automatic target selection budget exceeded")?;
    }
    let to = position(combat, target);
    let from = position(combat, member as i16);
    let fleeing = state.warriors[member].record().fleeing_duration()? != 0;
    let mut direction = (
        (to.0 - from.0).signum() as i8,
        (to.1 - from.1).signum() as i8,
    );
    if fleeing {
        direction = (-direction.0, -direction.1);
    }
    state.warriors[member].record_mut().set_target_body(target);
    state.warriors[member].record_mut().set_automatic_mode(1);
    if !fleeing && state.warriors[member].record().firing_mode() {
        match player_shoot_action(state, content, target) {
            Ok(mut events) => {
                events.extend(finish_player_action(state, content, member as i16)?);
                return Ok(events);
            }
            Err(RuleError::Refused { .. }) => {}
            Err(error) => return Err(error),
        }
    }
    for _ in 0..21 {
        if direction != (0, 0) {
            let before = state.combat.as_ref().unwrap().squad.positions[member];
            let events = crate::combat::move_controlled(state, content, direction.0, direction.1)?;
            if state.combat.as_ref().is_none_or(|c| {
                c.active_body() != Some(member as i16) || c.squad.positions[member] != before
            }) || events
                .iter()
                .any(|e| matches!(e, Event::AttackResolved { .. }))
            {
                return Ok(events);
            }
        }
        direction = (
            state.random.bound(3) as i8 - 2,
            state.random.bound(3) as i8 - 2,
        );
    }
    state.warriors[member].record_mut().set_guarding(1);
    end_turn(state, content)
}
