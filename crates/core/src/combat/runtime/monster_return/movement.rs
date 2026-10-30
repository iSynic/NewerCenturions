//! Moving a monster after a reaction's death story.
use super::turn_state;
use crate::{Content, Event, RuleError, Session, combat};
use serde::{Deserialize, Serialize};

/// A monster's step in progress: the cell it leaves, the cell it heads for, the retries made and
/// the movement it costs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MonsterMoveFrame {
    origin: (i16, i16),
    target: (i16, i16),
    retries: i16,
    cost: i16,
}

impl MonsterMoveFrame {
    /// Refuses a saved step with more than the allowed retries or a negative cost.
    pub(super) fn validate(&self) -> Result<(), RuleError> {
        if !(0..=22).contains(&self.retries) || self.cost < 0 {
            return Err("invalid saved NPC movement locals".into());
        }
        Ok(())
    }

    /// Starts a monster's step towards its target, or away from it when it is fleeing, and records
    /// the direction.
    ///
    /// Original: movemonster
    pub(super) fn begin(state: &mut Session) -> Result<Self, RuleError> {
        let selected = turn_state::selected(state)?;
        let record = turn_state::record(state, selected)?.record();
        let fleeing = record.fleeing_duration() != 0;
        let target = record.target()?;
        let target = target_position(state, target)?;
        let origin = turn_state::position(state, selected)?;
        let sign = if fleeing { -1 } else { 1 };
        let direction = (
            (i8::from(target.0 > origin.0) - i8::from(target.0 < origin.0)) * sign,
            (i8::from(target.1 > origin.1) - i8::from(target.1 < origin.1)) * sign,
        );
        turn_state::set_direction(state, direction)?;
        turn_state::record_mut(state, selected)?
            .record_mut()
            .set_movement_direction(direction.0, direction.1);
        Ok(Self {
            origin,
            target,
            retries: 0,
            cost: 0,
        })
    }

    /// Finds a step the monster can make from its cell: a cell ahead that its footprint fits and
    /// its movement points pay for. A blocked direction is shifted toward the target at random
    /// and tried again. After more than 20 retries the monster drops its target and its movement
    /// and the result is false.
    pub(super) fn prepare(
        &mut self,
        state: &mut Session,
        content: &Content,
    ) -> Result<bool, RuleError> {
        loop {
            let previous = self.retries;
            self.retries = self.retries.wrapping_add(1);
            let selected = turn_state::selected(state)?;
            if previous > 20 {
                let record = turn_state::record_mut(state, selected)?;
                record.record_mut().set_target(-1);
                record.record_mut().set_movement(0);
                return Ok(false);
            }
            let direction = turn_state::direction(state)?;
            let record = turn_state::record(state, selected)?.record();
            let size = record.size()?.wrapping_abs();
            // Sizes above 3 have no footprint table.
            if !(0..=3).contains(&size) {
                return Err(turn_state::missing(
                    "NPC movement footprint above 3 reads movemonster's stack frame",
                ));
            }
            let flying = record.flying_flag() != 0;
            let movement = record.movement();
            let next = (
                self.origin.0.wrapping_add(i16::from(direction.0)),
                self.origin.1.wrapping_add(i16::from(direction.1)),
            );
            let mission = turn_state::mission(state, content)?;
            let mut blocked = direction == (0, 0);
            let current = turn_state::body(state)?;
            for (x, y, occupied) in [
                (0, 0, true),
                (0, -1, size == 1 || size == 3),
                (-1, 0, size >= 2),
                (-1, -1, size == 3),
            ] {
                if !occupied || blocked {
                    continue;
                }
                let aliases = combat::map_alias::session_map_aliases!(state, mission);
                let mut tile =
                    combat::map::movement_tile(turn_state::map(state)?, aliases, next, (x, y))?
                        .wrapping_abs();
                if tile == current {
                    tile = mission.default_tile.wrapping_add(1000);
                }
                if tile < 1000 {
                    blocked = true;
                    break;
                }
                let obstruction = mission.tile_word(tile - 1000, 4)?;
                if obstruction != 0 && ((size == 0 && !flying) || (tile > 1000 && obstruction > 1))
                {
                    blocked = true;
                    break;
                }
                self.cost = self.cost.max(crate::rules::movement_cost(
                    mission.tile_word(tile - 1000, 2)?,
                    direction.0,
                    direction.1,
                ));
                if self.cost > i16::from(movement) {
                    break;
                }
            }
            if !blocked && self.cost <= i16::from(movement) {
                return Ok(true);
            }
            if blocked {
                let (mut dx, mut dy) = direction;
                combat::footprint::shift_direction(
                    self.origin,
                    self.target,
                    &mut dx,
                    &mut dy,
                    &mut state.random,
                );
                turn_state::set_direction(state, (dx, dy))?;
            }
        }
    }

    /// Moves the monster into the cell ahead: its footprint leaves the old cell and is placed in
    /// the new one, the movement cost is spent, and the movement sound (when it starts with its
    /// full movement) and the tile sound are played.
    ///
    /// Original: movemonster
    pub(super) fn place(
        &self,
        state: &mut Session,
        content: &Content,
    ) -> Result<Vec<Event>, RuleError> {
        let selected = turn_state::selected(state)?;
        let direction = turn_state::direction(state)?;
        let point = (
            self.origin.0.wrapping_add(i16::from(direction.0)),
            self.origin.1.wrapping_add(i16::from(direction.1)),
        );
        let current = turn_state::position(state, selected)?;
        let record = turn_state::record(state, selected)?;
        let map = combat::footprint::restore(turn_state::map(state)?, record, current)?;
        turn_state::set_map(state, map)?;
        turn_state::set_position(state, selected, point)?;
        let record = turn_state::record_mut(state, selected)?;
        let mut events = vec![];
        if i16::from(record.record().movement()) == record.record().movement_max()?
            && record.record().movement_sound()? != 0
        {
            events.push(Event::Sound {
                resource_id: record.record().movement_sound()?,
            });
        }
        let remaining = record.record().movement().wrapping_sub(self.cost as i8);
        record.record_mut().set_movement(remaining);
        let placed = combat::footprint::place(
            turn_state::map(state)?,
            turn_state::record(state, selected)?,
            point,
            selected as i16 + 10,
        )?;
        turn_state::set_map(state, placed.map)?;
        *turn_state::record_mut(state, selected)? = placed.monster;
        let tile = turn_state::record(state, selected)?
            .record()
            .underlying_tile()?;
        events.push(Event::Sound {
            resource_id: turn_state::mission(state, content)?.tile_word(tile - 1000, 0)?,
        });
        events.push(Event::TacticalMoved {
            body: selected as i16 + 10,
            x: point.0,
            y: point.1,
        });
        Ok(events)
    }

    /// Checks the cell the monster has moved to against the map edge (absolute cells 2 to 87). A
    /// monster whose survivor marker is -1 turns around; any other loses its reaction and movement,
    /// is set to surrender at 101 percent and steps the battle order index back by one.
    pub(super) fn edge(&self, state: &mut Session) -> Result<(), RuleError> {
        let direction = turn_state::direction(state)?;
        let shift = turn_state::map(state)?.map_shift;
        let absolute = (
            self.origin
                .0
                .wrapping_add(shift.0)
                .wrapping_add(5)
                .wrapping_add(i16::from(direction.0)),
            self.origin
                .1
                .wrapping_add(shift.1)
                .wrapping_add(5)
                .wrapping_add(i16::from(direction.1)),
        );
        if !(2..=87).contains(&absolute.0) || !(2..=87).contains(&absolute.1) {
            let selected = turn_state::selected(state)?;
            let record = turn_state::record_mut(state, selected)?;
            if record.record().survivor_marker()? == -1 {
                turn_state::set_direction(
                    state,
                    (direction.0.wrapping_neg(), direction.1.wrapping_neg()),
                )?;
            } else {
                record.record_mut().set_reaction_ready(false);
                record.record_mut().set_movement(0);
                record.record_mut().set_surrender_percent(101);
                if let Some(combat) = &mut state.combat {
                    combat.battle_order_index =
                        usize::from((combat.battle_order_index as u16).wrapping_sub(1));
                } else {
                    let order = &mut state.globals.battle_order;
                    order.index = usize::from((order.index as u16).wrapping_sub(1));
                }
            }
        }
        Ok(())
    }
}

/// The target's position, from the party or monster positions.
///
/// Original: movemonster
fn target_position(state: &Session, target: i16) -> Result<(i16, i16), RuleError> {
    let missing =
        || turn_state::missing("NPC movement target outside the represented position arrays");
    let slot = match combat::footprint::target_slot(target).ok_or_else(missing)? {
        combat::footprint::TargetSlot::Monster(slot) => return turn_state::position(state, slot),
        combat::footprint::TargetSlot::Party(slot) => slot,
    };
    if slot >= state.warriors.len() {
        return Err(missing());
    }
    match &state.combat {
        Some(combat) => combat.squad.pos.get(slot),
        None => state.globals.battlefield.pos.get(slot),
    }
    .copied()
    .ok_or_else(missing)
}
