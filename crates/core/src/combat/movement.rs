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
    crate::Session::land_in(
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
    centurions_content::mission::OpeningMission::index(
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
            return runtime::begin_move(
                state,
                content,
                body,
                (dx, dy),
                runtime::MovePlan::Friend {
                    tile,
                    exchange,
                    automatic,
                },
            );
        }
        if tile < 110
            && !state.warriors[body].record().firing_mode()
            && runtime::neighbors(combat, &state.warriors, body as i16)?.contains(&tile)
        {
            return runtime::player_attack(state, content, tile);
        }
        return occupied_cell::reload_at_occupied_cell(state, content, body, tile, dx, dy);
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
    runtime::begin_move(
        state,
        content,
        body,
        (dx, dy),
        runtime::MovePlan::Ground { tile },
    )
}
