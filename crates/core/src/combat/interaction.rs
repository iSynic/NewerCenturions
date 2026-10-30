use super::records::{BodySlot, CombatBodyId, CombatPosition};
use crate::combat::Combat;
use crate::{Command, Event, Phase, RuleError, Session};
use centurions_content::Warrior;

/// Whether the target, other than the active warrior, is a living warrior or monster on the active
/// warrior's side standing in a neighbouring cell.
pub fn is_friendly_neighbor(state: &Session, target: i16) -> Result<bool, RuleError> {
    let Some(combat) = &state.combat else {
        return Ok(false);
    };
    friendly_neighbor_parts(combat, &state.warriors, target)
}

/// The same test against a battle and its warriors.
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
    let (position, faction, living) =
        match CombatBodyId::new(target).slot(warriors.len(), combat.roster.records.len()) {
            Some(BodySlot::Warrior(index)) => (
                combat.squad.pos[index],
                warriors[index].record().faction()?,
                warriors[index].record().in_battle() != 0,
            ),
            Some(BodySlot::Enemy(index)) => {
                let monster = combat.roster.records[index].record();
                (
                    combat.roster.mon_pos[index],
                    monster.faction()?,
                    monster.stamina()? > 0 && !monster.removed(),
                )
            }
            None => return Ok(false),
        };
    let origin = combat.squad.pos[actor as usize];
    Ok(actor != target
        && living
        && faction == warriors[actor as usize].record().faction()?
        && (origin.0 - position.0)
            .abs()
            .max((origin.1 - position.1).abs())
            == 1)
}

/// Whether the active warrior can swap places with the target: more than four movement left, and a
/// living, small, friendly body in a neighbouring cell.
pub fn can_exchange(state: &Session, target: i16) -> Result<bool, RuleError> {
    let Some(combat) = &state.combat else {
        return Ok(false);
    };
    can_exchange_parts(combat, &state.warriors, target)
}

/// The same test against a battle and its warriors.
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
    if actor == target || warriors[actor as usize].record().movement()? <= 4 {
        return Ok(false);
    }
    let (position, faction, living, size) =
        match CombatBodyId::new(target).slot(warriors.len(), combat.roster.records.len()) {
            Some(BodySlot::Warrior(index)) => (
                combat.squad.pos[index],
                warriors[index].record().faction()?,
                warriors[index].record().in_battle() != 0,
                0,
            ),
            Some(BodySlot::Enemy(index)) => {
                let monster = combat.roster.records[index].record();
                (
                    combat.roster.mon_pos[index],
                    monster.faction()?,
                    monster.stamina()? > 0 && !monster.removed(),
                    monster.size()?,
                )
            }
            None => return Ok(false),
        };
    let from = combat.squad.pos[actor as usize];
    Ok(living
        && size == 0
        && faction == warriors[actor as usize].record().faction()?
        && (i32::from(from.0) - i32::from(position.0))
            .abs()
            .max((i32::from(from.1) - i32::from(position.1)).abs())
            == 1)
}

/// Two warriors swap places.
///
/// Original: combatchoice
pub fn exchange(state: &mut Session, target: i16) -> Result<Vec<Event>, RuleError> {
    if state.phase != Phase::Combat
        || !can_exchange(state, target)?
        || state
            .exploration
            .as_ref()
            .is_none_or(|position| position.presentation.is_some())
    {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::GameFeedback::Silent],
        });
    }
    let combat = state.combat.as_mut().unwrap();
    combat.in_spell = 0;
    let actor = combat.active_body().unwrap() as usize;
    let from = combat.squad.pos[actor];
    let to;
    if target < 10 {
        to = combat.squad.pos[target as usize];
        combat.squad.pos.swap(actor, target as usize);
        combat.squad.char_under.swap(actor, target as usize);
    } else {
        let index = (target - 10) as usize;
        to = combat.roster.mon_pos[index];
        combat.roster.mon_pos[index] = from;
        combat.squad.pos[actor] = to;
        let terrain = combat.roster.records[index].record().underlying_tile()?;
        combat.roster.records[index]
            .record_mut()
            .set_underlying_tile(combat.squad.char_under[actor]);
        combat.squad.char_under[actor] = terrain;
    }
    let index =
        |position: (i16, i16)| crate::combat::records::grid_index(position, combat.field.map_shift);
    let first = index(from)?;
    let second = index(to)?;
    combat.field.tiles[first] = target;
    combat.field.tiles[second] = actor as i16;
    let movement = state.warriors[actor].record().movement()?.wrapping_sub(5);
    state.warriors[actor].record_mut().set_movement(movement);
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

/// A click on the field: a step toward the next cell, or in firing mode the
/// clicked target.
///
/// Original: combatchoice
pub fn click_command(state: &Session, x: i16, y: i16) -> Result<Option<Command>, RuleError> {
    Ok(resolve_click(state, CombatPosition::new((x, y)))?.into_command())
}

/// What a click means. Choosing a target does not attack, and capture mode
/// comes before firing mode.
enum ClickIntent {
    NoAction,
    Capture(CombatPosition),
    FireAt(CombatBodyId),
    Step { dx: i8, dy: i8 },
}

impl ClickIntent {
    /// The command the click stands for, if any.
    fn into_command(self) -> Option<Command> {
        match self {
            Self::NoAction => None,
            Self::Capture(position) => Some(Command::SelectCaptureTarget {
                x: position.x,
                y: position.y,
            }),
            Self::FireAt(body) => Some(Command::ShootEnemy {
                body: body.number(),
            }),
            Self::Step { dx, dy } => Some(Command::Move { dx, dy }),
        }
    }
}

/// Decides what a click on the battle map means: a capture target in capture mode, a shot at a body
/// in firing mode, otherwise a step toward the cell.
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
    let index = target.map_index(combat.field.map_shift)?;
    if combat.capture_actor.is_some() {
        return Ok(ClickIntent::Capture(target));
    }
    let body = CombatBodyId::new(combat.field.tiles[index].wrapping_abs());
    if state.warriors[actor].record().toggle()
        && (body.number() != actor as i16)
        && body.is_present(state.warriors.len(), combat.roster.records.len())
    {
        return Ok(ClickIntent::FireAt(body));
    }
    let origin = CombatPosition::new(combat.squad.pos[actor]);
    let (dx, dy) = origin.step_toward(target);
    Ok(if dx == 0 && dy == 0 {
        ClickIntent::NoAction
    } else {
        ClickIntent::Step { dx, dy }
    })
}
