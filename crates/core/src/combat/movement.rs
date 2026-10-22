//! Movement for combat.
use crate::combat::{interaction, occupied_cell, runtime};
use crate::{Event, RuleError, Session};
use centurions_content::Content;

pub fn move_warrior(
    state: &mut Session,
    content: &Content,
    dx: i8,
    dy: i8,
) -> Result<Vec<Event>, RuleError> {
    move_warrior_inner(state, content, dx, dy, false)
}

pub(crate) fn move_controlled(
    state: &mut Session,
    content: &Content,
    dx: i8,
    dy: i8,
) -> Result<Vec<Event>, RuleError> {
    move_warrior_inner(state, content, dx, dy, true)
}

pub(super) fn move_warrior_inner(
    state: &mut Session,
    content: &Content,
    dx: i8,
    dy: i8,
    automatic: bool,
) -> Result<Vec<Event>, RuleError> {
    if !(-1..=1).contains(&dx) || !(-1..=1).contains(&dy) || (dx == 0 && dy == 0) {
        return Err(RuleError::Invalid("invalid tactical direction".into()));
    }
    let combat = state
        .combat
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
    let body = combat
        .active_body()
        .ok_or_else(|| RuleError::Invalid("no active body".into()))?;
    if !(0..6).contains(&body) {
        return Err(RuleError::Invalid(
            "movement requires an active warrior".into(),
        ));
    }
    let body = body as usize;
    let mission = crate::Session::land_in(
        content,
        state.mission.as_deref(),
        state.scenario_level,
        state.exploration.as_ref(),
    )
    .ok_or_else(|| RuleError::Invalid("mission missing".into()))?;
    let position = combat.squad.positions[body];
    let destination = (
        position.0.wrapping_add(i16::from(dx)),
        position.1.wrapping_add(i16::from(dy)),
    );
    let origin = (combat.map.shift.0 + 5, combat.map.shift.1 + 5);
    let current_index = centurions_content::mission::OpeningMission::index(
        i32::from(position.0 + origin.0),
        i32::from(position.1 + origin.1),
    )?;
    let next_index = centurions_content::mission::OpeningMission::index(
        i32::from(destination.0 + origin.0),
        i32::from(destination.1 + origin.1),
    )?;
    let tile = combat.map.tiles[next_index].wrapping_abs();
    if tile <= 110 {
        if interaction::friendly_neighbor_parts(combat, &state.warriors, tile)? {
            let exchange = interaction::can_exchange_parts(combat, &state.warriors, tile)?;
            let mut effects = runtime::check_neighbors_with_work(
                combat,
                &mut state.warriors,
                &mut state.random,
                body as i16,
                &content.items,
                &mission.combat.as_ref().unwrap().attacks,
                &state.party_buffs,
                &mut state.native_work.ammunition_work,
            )?;
            if state.warriors[body].record().active_flag() == 0 {
                effects.extend(runtime::end_turn(state, content)?);
                return Ok(effects);
            }
            state.warriors[body].record_mut().face_horizontal_step(dx);
            if automatic {
                effects.extend(if exchange {
                    interaction::exchange(state, tile)?
                } else {
                    runtime::player_attack_friend(state, content, tile)?
                });
                return Ok(effects);
            }
            effects.extend([
                Event::Sound { resource_id: 20005 },
                if exchange {
                    Event::ExchangeOffered { body: tile }
                } else {
                    Event::FriendAttackOffered { body: tile }
                },
            ]);
            return Ok(effects);
        }
        if tile < 110
            && !state.warriors[body].record().firing_mode()
            && runtime::neighbors(combat, &state.warriors, body as i16)?.contains(&tile)
        {
            return runtime::player_attack(state, content, tile);
        }
        return occupied_cell::reload_at_occupied_cell(state, content, mission, body, tile, dx);
    }
    if tile < 1000 {
        return Err(RuleError::Unsupported {
            action: "combat.move",
            dependency: "non-terrain tactical marker",
        });
    }
    if !automatic && state.warriors[body].record().automatic_mode()? != 0 {
        return Err(RuleError::Unsupported {
            action: "combat.move",
            dependency: "involuntary movement",
        });
    }
    let definitions = &content.items;
    let attacks = mission
        .combat
        .as_ref()
        .map_or(&[][..], |tables| &tables.attacks);
    let mut effects = runtime::check_neighbors_with_work(
        combat,
        &mut state.warriors,
        &mut state.random,
        body as i16,
        definitions,
        attacks,
        &state.party_buffs,
        &mut state.native_work.ammunition_work,
    )?;
    state.warriors[body].record_mut().face_horizontal_step(dx);
    if mission.movement_blocker(tile - 1000)? != 0 {
        effects.push(Event::Sound { resource_id: -129 });
        effects.push(Event::Sound { resource_id: -130 });
        return Ok(effects);
    }
    let cost = crate::rules::movement_cost(mission.tactical_movement_property(tile)?, dx, dy);
    let remaining = state.warriors[body]
        .record()
        .movement_points()?
        .wrapping_sub(cost);
    if remaining < 0 {
        effects.push(Event::Sound { resource_id: -130 });
        return Ok(effects);
    }
    effects.extend(runtime::warrior_departure_with_work(
        combat,
        &mut state.warriors,
        &mut state.random,
        body as i16,
        destination,
        definitions,
        attacks,
        &state.party_buffs,
        &mut state.native_work.ammunition_work,
    )?);
    if state.warriors[body].record().active_flag() == 0 {
        effects.extend(runtime::end_turn(state, content)?);
        return Ok(effects);
    }
    let remaining = state.warriors[body]
        .record()
        .movement_points()?
        .wrapping_sub(cost);
    state.warriors[body]
        .record_mut()
        .set_movement_points(remaining);
    combat.map.tiles[current_index] = combat.squad.under_tiles[body];
    combat.squad.under_tiles[body] = combat.map.tiles[next_index];
    combat.map.tiles[next_index] = body as i16;
    combat.squad.positions[body] = destination;
    effects.extend([
        Event::TacticalMoved {
            body: body as i16,
            x: destination.0,
            y: destination.1,
        },
        Event::Sound {
            resource_id: mission.movement_sound(tile - 1000)?,
        },
    ]);
    effects.extend(runtime::collide_fields_with_mask(
        combat,
        &mut state.warriors,
        mission,
        &mut state.native_work.attack_count,
        &mut state.random,
        body as i16,
        &mut state.native_work.party_mask,
    )?);
    if state.warriors[body].stamina()? < 1 {
        state.warriors[body].record_mut().set_action_points(0);
        state.warriors[body].record_mut().set_movement_points(0);
    } else {
        effects.extend(runtime::check_neighbors_with_work(
            combat,
            &mut state.warriors,
            &mut state.random,
            body as i16,
            definitions,
            attacks,
            &state.party_buffs,
            &mut state.native_work.ammunition_work,
        )?);
        if state.warriors[body].record().active_flag() == 0 {
            effects.extend(runtime::end_turn(state, content)?);
        }
    }
    Ok(effects)
}
