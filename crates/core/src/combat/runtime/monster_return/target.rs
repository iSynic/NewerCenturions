//! Choosing a monster's target; what it can see stays as it was.
use super::turn_state;
use crate::{Content, RuleError, Session, combat};

/// Whether a body can be a monster's target: a warrior in the battle or a monster with stamina, on
/// the other side.
fn eligible(state: &Session, side: i16, body: i16) -> Result<bool, RuleError> {
    if (0..state.warriors.len() as i16).contains(&body) {
        let warrior = state.warriors[body as usize].record();
        Ok(warrior.in_battle() != 0 && warrior.faction()? != side)
    } else if (10..110).contains(&body) {
        let record = turn_state::record(state, (body - 10) as usize)?.record();
        Ok(record.stamina()? > 0 && record.faction()? != side)
    } else {
        Ok(false)
    }
}
/// The cell a body stands in.
fn point(state: &Session, body: i16) -> Result<(i16, i16), RuleError> {
    if (0..state.warriors.len() as i16).contains(&body) {
        if let Some(combat) = &state.combat {
            return Ok(combat.squad.pos[body as usize]);
        }
        state
            .globals
            .battlefield
            .pos
            .get(body as usize)
            .copied()
            .ok_or_else(|| RuleError::from("target warrior outside the squad"))
    } else {
        turn_state::position(state, (body - 10) as usize)
    }
}
/// Whether the first cell sees the second, in the battle or after it.
fn visible(
    state: &Session,
    content: &Content,
    from: (i16, i16),
    to: (i16, i16),
) -> Result<bool, RuleError> {
    let mission = turn_state::mission(state, content)?;
    if let Some(combat) = &state.combat {
        return combat::lifecycle::can_see(combat, mission, from, to, 0);
    }
    super::firearm_selection::visible(turn_state::map(state)?, mission, from, to)
}
/// Chooses the acting monster's target. A target that is no longer eligible is replaced by a random
/// eligible body; a target the monster cannot see is replaced by the next eligible body in order.
/// When none is in sight the monster is left with no target.
///
/// Original: combat
pub(super) fn select(
    state: &mut Session,
    content: &Content,
    cursor: &mut i16,
) -> Result<(), RuleError> {
    let selected = turn_state::selected(state)?;
    let side = turn_state::record(state, selected)?.record().faction()?;
    let mut target = turn_state::record(state, selected)?.record().target()?;
    if !eligible(state, side, target)? {
        let count = if let Some(combat) = &state.combat {
            combat.roster.records.len() as i16
        } else {
            state.globals.monsters.num_mon
        };
        // num_mon is never negative, and validation caps it at 100.
        if !(0..=100).contains(&count) {
            return Err("NPC count WORD 0x43874a outside 0..=100".into());
        }
        // The game redraws until a body qualifies. With no eligible body it never
        // ends; otherwise every body recurs within the generator's period, so the
        // search finishes.
        let range = count.wrapping_add(10);
        let mut any = false;
        for body in 0..range {
            any |= eligible(state, side, body)?;
        }
        if !any {
            return Err(turn_state::missing(
                "NPC target scan has no eligible body; the original search never ends",
            ));
        }
        target = loop {
            let candidate = state.random.bound(range) - 1;
            if eligible(state, side, candidate)? {
                break candidate;
            }
        };
    }
    loop {
        turn_state::record_mut(state, selected)?
            .record_mut()
            .set_target(target);
        if visible(
            state,
            content,
            turn_state::position(state, selected)?,
            point(state, target)?,
        )? || *cursor > 109
        {
            return Ok(());
        }
        let mut next = None;
        while *cursor < 110 {
            let candidate = *cursor;
            *cursor = cursor.wrapping_add(1);
            if !eligible(state, side, candidate)? {
                continue;
            }
            if candidate >= 10
                && turn_state::record(state, (candidate - 10) as usize)?
                    .record()
                    .excluded_from_target_scan()
            {
                continue;
            }
            next = Some(candidate);
            break;
        }
        if let Some(next) = next {
            target = next;
        } else {
            turn_state::record_mut(state, selected)?
                .record_mut()
                .set_target(-1);
            return Ok(());
        }
    }
}
