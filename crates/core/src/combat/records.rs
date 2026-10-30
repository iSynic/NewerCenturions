//! Body numbers and map coordinates shared by the combat rules.
use crate::RuleError;

/// A body in the battle: warriors 0 to 5, monsters from 10.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct CombatBodyId(i16);

/// Whether a body is a warrior or a monster, and its index in that list.
pub(crate) enum BodySlot {
    Warrior(usize),
    Enemy(usize),
}

impl CombatBodyId {
    /// Wraps a body number.
    pub(crate) fn new(body: i16) -> Self {
        Self(body)
    }

    /// The body number.
    pub(crate) fn number(self) -> i16 {
        self.0
    }

    /// Whether the body is a warrior in the squad or a monster in the battle.
    pub(crate) fn is_present(self, warriors: usize, enemies: usize) -> bool {
        (0..warriors as i16).contains(&self.0) || (10..10 + enemies as i16).contains(&self.0)
    }

    /// Bodies 0..5 are the squad and monsters begin at 10. Whether a body must be
    /// alive is each caller's rule.
    pub(crate) fn slot(self, warriors: usize, enemies: usize) -> Option<BodySlot> {
        if (0..warriors as i16).contains(&self.0) {
            Some(BodySlot::Warrior(self.0 as usize))
        } else if self.0 >= 10 && ((self.0 - 10) as usize) < enemies {
            Some(BodySlot::Enemy((self.0 - 10) as usize))
        } else {
            None
        }
    }
}

/// A place on the tactical map.
#[derive(Clone, Copy)]
pub(crate) struct CombatPosition {
    pub(crate) x: i16,
    pub(crate) y: i16,
}

impl CombatPosition {
    /// Wraps a map position.
    pub(crate) fn new((x, y): (i16, i16)) -> Self {
        Self { x, y }
    }

    /// The step, at most one in each direction, from here toward the target.
    pub(crate) fn step_toward(self, target: Self) -> (i8, i8) {
        (
            target.x.wrapping_sub(self.x).signum() as i8,
            target.y.wrapping_sub(self.y).signum() as i8,
        )
    }

    /// The cell of the battle map this position falls on.
    pub(crate) fn map_index(self, shift: (i16, i16)) -> Result<usize, RuleError> {
        grid_index((self.x, self.y), shift)
    }
}

/// Map grid point of a tactical position: `position + shift + 5`, widened to i32.
pub(crate) fn grid_point(position: (i16, i16), shift: (i16, i16)) -> (i32, i32) {
    (
        i32::from(position.0) + i32::from(shift.0) + 5,
        i32::from(position.1) + i32::from(shift.1) + 5,
    )
}

/// Linear map index of a tactical position's grid point.
pub(crate) fn grid_index(position: (i16, i16), shift: (i16, i16)) -> Result<usize, RuleError> {
    let (x, y) = grid_point(position, shift);
    Ok(centurions_content::mission::OpeningMission::index(x, y)?)
}

/// Inverse of `grid_point`, truncated to signed words.
pub(crate) fn grid_position(point: (i32, i32), shift: (i16, i16)) -> (i16, i16) {
    (
        point.0.wrapping_sub(i32::from(shift.0) + 5) as i16,
        point.1.wrapping_sub(i32::from(shift.1) + 5) as i16,
    )
}

/// Grid point in signed words, added with wrapping.
pub(crate) fn wrapping_grid_point(position: (i16, i16), shift: (i16, i16)) -> (i16, i16) {
    (
        position.0.wrapping_add(shift.0).wrapping_add(5),
        position.1.wrapping_add(shift.1).wrapping_add(5),
    )
}

/// Body lookups. Bodies below 10 index the squad and the rest the monsters at
/// `body - 10`; each method keeps its callers' bounds behaviour.
impl super::Combat {
    /// Body position by direct indexing; panics outside the squad or enemy table.
    pub(crate) fn body_position(&self, body: i16) -> (i16, i16) {
        if body < 10 {
            self.squad.pos[body as usize]
        } else {
            self.roster.mon_pos[(body - 10) as usize]
        }
    }

    /// Body position, or `None` outside the squad or enemy table.
    pub(crate) fn get_body_position(&self, body: usize) -> Option<(i16, i16)> {
        if body < 10 {
            self.squad.pos.get(body).copied()
        } else {
            self.roster.mon_pos.get(body - 10).copied()
        }
    }

    /// Body faction by direct indexing; panics outside the warrior or enemy table.
    pub(crate) fn body_faction(
        &self,
        warriors: &[centurions_content::Warrior],
        body: i16,
    ) -> Result<i16, RuleError> {
        if body < 10 {
            Ok(warriors[body as usize].record().faction()?)
        } else {
            Ok(self.roster.records[(body - 10) as usize]
                .record()
                .faction()?)
        }
    }

    /// Whether a warrior body 0..6 is active or an enemy body is living and present;
    /// any other body, or a missing record, is not alive.
    pub(crate) fn body_alive(&self, warriors: &[centurions_content::Warrior], body: i16) -> bool {
        if (0..6).contains(&body) {
            warriors
                .get(body as usize)
                .is_some_and(|warrior| warrior.record().in_battle() != 0)
        } else {
            self.enemy_body_alive(body)
        }
    }

    /// Whether a body at or above 10 names a living, present enemy record.
    pub(crate) fn enemy_body_alive(&self, body: i16) -> bool {
        body >= 10
            && self
                .roster
                .records
                .get((body - 10) as usize)
                .is_some_and(|m| m.stamina().unwrap_or(0) > 0 && !m.record().removed())
    }

    /// Body pixel by direct indexing; panics outside the squad or enemy table.
    pub(crate) fn body_pixel(&self, body: i16) -> Result<(i16, i16), RuleError> {
        Ok(if body < 10 {
            super::spells::body_pixel(self.squad.pos[body as usize], None)
        } else {
            let index = (body - 10) as usize;
            let size = self.roster.records[index].record().size()?.wrapping_abs();
            super::spells::body_pixel(self.roster.mon_pos[index], Some(size))
        })
    }

    /// Body pixel, or `None` for a missing body, record or unreadable size.
    pub(crate) fn get_body_pixel(&self, body: i16) -> Option<(i16, i16)> {
        if body < 10 {
            Some(super::spells::body_pixel(
                *self.squad.pos.get(usize::try_from(body).ok()?)?,
                None,
            ))
        } else {
            let index = usize::try_from(body - 10).ok()?;
            let size = self.roster.records.get(index)?.record().size().ok()?;
            Some(super::spells::body_pixel(
                *self.roster.mon_pos.get(index)?,
                Some(size.wrapping_abs()),
            ))
        }
    }
}

/// Spends a warrior's attack action: movement points less 3, floored at 0, then
/// action points less 2, written in that order.
pub(crate) fn spend_attack_action(
    warrior: &mut centurions_content::Warrior,
) -> Result<(), RuleError> {
    let movement = warrior.record().movement()?.wrapping_sub(3).max(0);
    let actions = warrior.record().action_points()?.wrapping_sub(2);
    warrior.record_mut().set_movement(movement);
    warrior.record_mut().set_action_points(actions);
    Ok(())
}
