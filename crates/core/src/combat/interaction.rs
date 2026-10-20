use super::records::{BodySlot, CombatBodyId, CombatPosition};
use crate::combat::Combat;
use crate::{Command, Event, Phase, RuleError, Session};
use centurions_content::Warrior;

pub fn is_friendly_neighbor(state: &Session, target: i16) -> Result<bool, RuleError> {
    let Some(combat) = &state.combat else {
        return Ok(false);
    };
    friendly_neighbor_parts(combat, &state.warriors, target)
}

pub(crate) fn friendly_neighbor_parts(
    combat: &Combat,
    warriors: &[Warrior],
    target: i16,
) -> Result<bool, RuleError> {
    let Some(actor) = combat
        .active_body()
        .filter(|body| (0..warriors.len() as i16).contains(body))
    else {
        return Ok(false);
    };
    let (position, faction, living) = match CombatBodyId::from_native(target)
        .slot(warriors.len(), combat.enemies.records.len())
    {
        Some(BodySlot::Warrior(index)) => (
            combat.squad.positions[index],
            warriors[index].record().faction()?,
            warriors[index].record().active_flag() != 0,
        ),
        Some(BodySlot::Enemy(index)) => {
            let enemy = combat.enemies.records[index].record();
            (
                combat.enemies.positions[index],
                enemy.faction()?,
                enemy.stamina()? > 0 && !enemy.removed(),
            )
        }
        None => return Ok(false),
    };
    let origin = combat.squad.positions[actor as usize];
    Ok(actor != target
        && living
        && faction == warriors[actor as usize].record().faction()?
        && (origin.0 - position.0)
            .abs()
            .max((origin.1 - position.1).abs())
            == 1)
}

pub fn can_exchange(state: &Session, target: i16) -> Result<bool, RuleError> {
    let Some(combat) = &state.combat else {
        return Ok(false);
    };
    can_exchange_parts(combat, &state.warriors, target)
}

pub(crate) fn can_exchange_parts(
    combat: &Combat,
    warriors: &[Warrior],
    target: i16,
) -> Result<bool, RuleError> {
    let Some(actor) = combat
        .active_body()
        .filter(|body| (0..warriors.len() as i16).contains(body))
    else {
        return Ok(false);
    };
    if actor == target || warriors[actor as usize].record().movement_points()? <= 4 {
        return Ok(false);
    }
    let (position, faction, living, size) = match CombatBodyId::from_native(target)
        .slot(warriors.len(), combat.enemies.records.len())
    {
        Some(BodySlot::Warrior(index)) => (
            combat.squad.positions[index],
            warriors[index].record().faction()?,
            warriors[index].record().active_flag() != 0,
            0,
        ),
        Some(BodySlot::Enemy(index)) => {
            let enemy = combat.enemies.records[index].record();
            (
                combat.enemies.positions[index],
                enemy.faction()?,
                enemy.stamina()? > 0 && !enemy.removed(),
                enemy.size()?,
            )
        }
        None => return Ok(false),
    };
    let from = combat.squad.positions[actor as usize];
    Ok(living
        && size == 0
        && faction == warriors[actor as usize].record().faction()?
        && (i32::from(from.0) - i32::from(position.0))
            .abs()
            .max((i32::from(from.1) - i32::from(position.1)).abs())
            == 1)
}

/// Confirmed combatchoice exchange block, 0x11c564..0x11c854.
pub fn exchange(state: &mut Session, target: i16) -> Result<Vec<Event>, RuleError> {
    if state.phase != Phase::Combat
        || !can_exchange(state, target)?
        || state
            .exploration
            .as_ref()
            .is_none_or(|position| position.presentation.is_some())
    {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::NativeFeedback::Silent],
        });
    }
    let combat = state.combat.as_mut().unwrap();
    let actor = combat.active_body().unwrap() as usize;
    let from = combat.squad.positions[actor];
    let to;
    if target < 10 {
        to = combat.squad.positions[target as usize];
        combat.squad.positions.swap(actor, target as usize);
        combat.squad.under_tiles.swap(actor, target as usize);
    } else {
        let index = (target - 10) as usize;
        to = combat.enemies.positions[index];
        combat.enemies.positions[index] = from;
        combat.squad.positions[actor] = to;
        let terrain = combat.enemies.records[index].record().underlying_tile()?;
        combat.enemies.records[index]
            .record_mut()
            .set_underlying_tile(combat.squad.under_tiles[actor]);
        combat.squad.under_tiles[actor] = terrain;
    }
    let index = |position: (i16, i16)| {
        centurions_content::mission::OpeningMission::index(
            i32::from(position.0) + i32::from(combat.map.shift.0) + 5,
            i32::from(position.1) + i32::from(combat.map.shift.1) + 5,
        )
    };
    let first = index(from)?;
    let second = index(to)?;
    combat.map.tiles[first] = target;
    combat.map.tiles[second] = actor as i16;
    let movement = state.warriors[actor]
        .record()
        .movement_points()?
        .wrapping_sub(5);
    state.warriors[actor]
        .record_mut()
        .set_movement_points(movement);
    Ok(vec![
        Event::Sound { resource_id: 654 },
        Event::TacticalMoved {
            body: actor as i16,
            x: to.0,
            y: to.1,
        },
        Event::TacticalMoved {
            body: target,
            x: from.0,
            y: from.1,
        },
    ])
}

/// combatchoice movement uses the next cell; firing mode retains the clicked target.
pub fn click_command(state: &Session, x: i16, y: i16) -> Result<Option<Command>, RuleError> {
    Ok(resolve_click(state, CombatPosition::from_native((x, y)))?.into_command())
}

/// Read-only gesture resolution. In particular, selecting a target does not
/// attempt an attack here, and capture takes precedence over firearm mode.
enum ClickIntent {
    NoAction,
    Capture(CombatPosition),
    FireAt(CombatBodyId),
    Step { dx: i8, dy: i8 },
}

impl ClickIntent {
    fn into_command(self) -> Option<Command> {
        match self {
            Self::NoAction => None,
            Self::Capture(position) => Some(Command::SelectCaptureTarget {
                x: position.x,
                y: position.y,
            }),
            Self::FireAt(body) => Some(Command::ShootEnemy {
                body: body.native(),
            }),
            Self::Step { dx, dy } => Some(Command::Move { dx, dy }),
        }
    }
}

fn resolve_click(state: &Session, target: CombatPosition) -> Result<ClickIntent, RuleError> {
    let combat = state
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
    let actor = combat
        .active_body()
        .filter(|body| (0..state.warriors.len() as i16).contains(body))
        .ok_or_else(|| RuleError::Invalid("active warrior missing".into()))?
        as usize;
    let index = target.map_index(combat.map.shift)?;
    if combat.capture_actor.is_some() {
        return Ok(ClickIntent::Capture(target));
    }
    let body = CombatBodyId::from_native(combat.map.tiles[index].wrapping_abs());
    if state.warriors[actor].record().firing_mode()
        && (body.native() != actor as i16)
        && body.is_present(state.warriors.len(), combat.enemies.records.len())
    {
        return Ok(ClickIntent::FireAt(body));
    }
    let origin = CombatPosition::from_native(combat.squad.positions[actor]);
    let (dx, dy) = origin.step_toward(target);
    Ok(if dx == 0 && dy == 0 {
        ClickIntent::NoAction
    } else {
        ClickIntent::Step { dx, dy }
    })
}
