//! Movemonster locals and its placement suffix over current shared records.
use super::work;
use crate::{Content, Event, RuleError, Session, combat};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct NpcMoveFrame {
    origin: (i16, i16),
    target: (i16, i16),
    retries: i16,
    cost: i16,
}

impl NpcMoveFrame {
    pub(super) fn validate(&self) -> Result<(), RuleError> {
        if !(0..=22).contains(&self.retries) || self.cost < 0 {
            return Err("invalid saved NPC movement locals".into());
        }
        Ok(())
    }

    pub(super) fn begin(state: &mut Session) -> Result<Self, RuleError> {
        let selected = work::selected(state)?;
        let record = work::record(state, selected)?.record();
        let fleeing = record.fleeing_duration() != 0;
        let target = record.target_body()?;
        let target = target_position(state, target)?;
        let origin = work::position(state, selected)?;
        let sign = if fleeing { -1 } else { 1 };
        let direction = (
            (i8::from(target.0 > origin.0) - i8::from(target.0 < origin.0)) * sign,
            (i8::from(target.1 > origin.1) - i8::from(target.1 < origin.1)) * sign,
        );
        work::set_direction(state, direction)?;
        work::record_mut(state, selected)?
            .record_mut()
            .set_movement_direction(direction.0, direction.1);
        Ok(Self {
            origin,
            target,
            retries: 0,
            cost: 0,
        })
    }

    /// Complete destination validation before the optional departure reaction.
    pub(super) fn prepare(
        &mut self,
        state: &mut Session,
        content: &Content,
    ) -> Result<bool, RuleError> {
        loop {
            let previous = self.retries;
            self.retries = self.retries.wrapping_add(1);
            let selected = work::selected(state)?;
            if previous > 20 {
                let record = work::record_mut(state, selected)?;
                record.record_mut().set_target_body(-1);
                record.record_mut().set_movement_points(0);
                return Ok(false);
            }
            let direction = work::direction(state)?;
            let record = work::record(state, selected)?.record();
            let size = record.size()?.wrapping_abs();
            // movemonster13e276 reads local_c[0..=size]; a size above 3 reads its frame.
            if !(0..=3).contains(&size) {
                return Err(work::missing(
                    "NPC movement footprint above 3 reads movemonster's stack frame",
                ));
            }
            let flying = record.flying_flag() != 0;
            let movement = record.movement_points();
            let next = (
                self.origin.0.wrapping_add(i16::from(direction.0)),
                self.origin.1.wrapping_add(i16::from(direction.1)),
            );
            let mission = work::mission(state, content)?;
            let mut blocked = direction == (0, 0);
            let current = work::body(state)?;
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
                    combat::map::movement_tile(work::map(state)?, aliases, next, (x, y))?
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
                work::set_direction(state, (dx, dy))?;
            }
        }
    }

    /// checkforenemy(2)'s result is deliberately not a predicate here.
    pub(super) fn place(
        &self,
        state: &mut Session,
        content: &Content,
    ) -> Result<Vec<Event>, RuleError> {
        let selected = work::selected(state)?;
        let direction = work::direction(state)?;
        let point = (
            self.origin.0.wrapping_add(i16::from(direction.0)),
            self.origin.1.wrapping_add(i16::from(direction.1)),
        );
        let current = work::position(state, selected)?;
        let record = work::record(state, selected)?;
        let map = combat::footprint::restore(work::map(state)?, record, current)?;
        work::set_map(state, map)?;
        work::set_position(state, selected, point)?;
        let record = work::record_mut(state, selected)?;
        let mut events = vec![];
        if i16::from(record.record().movement_points()) == record.record().movement_per_round()?
            && record.record().movement_sound()? != 0
        {
            events.push(Event::Sound {
                resource_id: record.record().movement_sound()?,
            });
        }
        let remaining = record
            .record()
            .movement_points()
            .wrapping_sub(self.cost as i8);
        record.record_mut().set_movement_points(remaining);
        let placed = combat::footprint::place(
            work::map(state)?,
            work::record(state, selected)?,
            point,
            selected as i16 + 10,
        )?;
        work::set_map(state, placed.map)?;
        *work::record_mut(state, selected)? = placed.monster;
        let tile = work::record(state, selected)?.record().underlying_tile()?;
        events.push(Event::Sound {
            resource_id: work::mission(state, content)?.tile_word(tile - 1000, 0)?,
        });
        events.push(Event::TacticalMoved {
            body: selected as i16 + 10,
            x: point.0,
            y: point.1,
        });
        Ok(events)
    }

    /// Native edge handling follows collide, including its possible nonreturn.
    pub(super) fn edge(&self, state: &mut Session) -> Result<(), RuleError> {
        let direction = work::direction(state)?;
        let shift = work::map(state)?.shift;
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
            let selected = work::selected(state)?;
            let record = work::record_mut(state, selected)?;
            if record.record().survivor_marker()? == -1 {
                work::set_direction(
                    state,
                    (direction.0.wrapping_neg(), direction.1.wrapping_neg()),
                )?;
            } else {
                record.record_mut().set_reaction_ready(false);
                record.record_mut().set_movement_points(0);
                record.record_mut().set_fleeing_chance(101);
                if let Some(combat) = &mut state.combat {
                    combat.cursor = usize::from((combat.cursor as u16).wrapping_sub(1));
                } else {
                    let order = &mut state.native_work.initiative_work;
                    order.cursor = usize::from((order.cursor as u16).wrapping_sub(1));
                }
            }
        }
        Ok(())
    }
}

/// The target position through movemonster13e276's party and NPC position arrays.
fn target_position(state: &Session, target: i16) -> Result<(i16, i16), RuleError> {
    let missing = || work::missing("NPC movement target outside the represented position arrays");
    let slot = match combat::footprint::target_slot(target).ok_or_else(missing)? {
        combat::footprint::TargetSlot::Npc(slot) => return work::position(state, slot),
        combat::footprint::TargetSlot::Party(slot) => slot,
    };
    if slot >= state.warriors.len() {
        return Err(missing());
    }
    match &state.combat {
        Some(combat) => combat.squad.positions.get(slot),
        None => state.native_work.tactical_work.positions.get(slot),
    }
    .copied()
    .ok_or_else(missing)
}
