//! Target scanning against current live or retained arrays; visibility stays frozen.
use super::work;
use crate::{Content, RuleError, Session, combat};

fn eligible(state: &Session, side: i16, body: i16) -> Result<bool, RuleError> {
    if (0..state.warriors.len() as i16).contains(&body) {
        let warrior = state.warriors[body as usize].record();
        Ok(warrior.active_flag() != 0 && warrior.faction()? != side)
    } else if (10..110).contains(&body) {
        let record = work::record(state, (body - 10) as usize)?.record();
        Ok(record.stamina()? > 0 && record.faction()? != side)
    } else {
        Ok(false)
    }
}
fn point(state: &Session, body: i16) -> Result<(i16, i16), RuleError> {
    if (0..state.warriors.len() as i16).contains(&body) {
        if let Some(combat) = &state.combat {
            return Ok(combat.squad.positions[body as usize]);
        }
        state
            .native_work
            .tactical_work
            .positions
            .get(body as usize)
            .copied()
            .ok_or_else(|| RuleError::from("target warrior outside the squad"))
    } else {
        work::position(state, (body - 10) as usize)
    }
}
fn visible(
    state: &Session,
    content: &Content,
    from: (i16, i16),
    to: (i16, i16),
) -> Result<bool, RuleError> {
    let mission = work::mission(state, content)?;
    if let Some(combat) = &state.combat {
        return combat::lifecycle::can_see(combat, mission, from, to, 0);
    }
    super::firearm_selection::visible(work::map(state)?, mission, from, to)
}
pub(super) fn select(
    state: &mut Session,
    content: &Content,
    cursor: &mut i16,
) -> Result<(), RuleError> {
    let selected = work::selected(state)?;
    let side = work::record(state, selected)?.record().faction()?;
    let mut target = work::record(state, selected)?.record().target_body()?;
    if !eligible(state, side, target)? {
        let count = if let Some(combat) = &state.combat {
            combat.enemies.records.len() as i16
        } else {
            state.native_work.npc_backing.active_count
        };
        // No writer of the count WORD 0x43874a stores a negative value (clr.w, combatsetup's
        // placed count at 0x11d7b0 and addq.w increments), and validation caps it at 100.
        if !(0..=100).contains(&count) {
            return Err("NPC count WORD 0x43874a outside 0..=100".into());
        }
        // The original redraws until a body qualifies. With no eligible body in
        // the drawn range it never ends; otherwise every candidate recurs within
        // the generator's period, so the search always finishes.
        let range = count.wrapping_add(10);
        let mut any = false;
        for body in 0..range {
            any |= eligible(state, side, body)?;
        }
        if !any {
            return Err(work::missing(
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
        work::record_mut(state, selected)?
            .record_mut()
            .set_target_body(target);
        if visible(
            state,
            content,
            work::position(state, selected)?,
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
                && work::record(state, (candidate - 10) as usize)?
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
            work::record_mut(state, selected)?
                .record_mut()
                .set_target_body(-1);
            return Ok(());
        }
    }
}
