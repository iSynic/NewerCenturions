//! Reactions in the original combat execution order.
use super::{alive, has_active_warrior, neighbors, refresh_body_item, resolve_attack};
use crate::combat::Combat;
use crate::{Event, RandomState, RuleError};
use centurions_content::Warrior;

pub fn check_neighbors(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
) -> Result<Vec<Event>, RuleError> {
    check_neighbors_with_items(
        c,
        w,
        random,
        body,
        None,
        None,
        &mut false,
        &[0; 20],
        &mut Default::default(),
        false,
    )
}

pub fn check_neighbors_with_definitions(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    items: &[Vec<u8>],
) -> Result<Vec<Event>, RuleError> {
    check_neighbors_with_items(
        c,
        w,
        random,
        body,
        Some(items),
        None,
        &mut false,
        &[0; 20],
        &mut Default::default(),
        false,
    )
}

pub fn check_neighbors_with_context(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    items: &[Vec<u8>],
    attacks: &[u8],
) -> Result<Vec<Event>, RuleError> {
    check_neighbors_with_buffs(c, w, random, body, items, attacks, &[0; 20])
}

pub fn check_neighbors_with_buffs(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    items: &[Vec<u8>],
    attacks: &[u8],
    buffs: &[u8],
) -> Result<Vec<Event>, RuleError> {
    check_neighbors_with_items(
        c,
        w,
        random,
        body,
        Some(items),
        Some(attacks),
        &mut false,
        buffs,
        &mut Default::default(),
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn check_neighbors_with_work(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    items: &[Vec<u8>],
    attacks: &[u8],
    buffs: &[u8],
    buffers: &mut crate::ammunition::WorkBuffers,
) -> Result<Vec<Event>, RuleError> {
    check_neighbors_with_items(
        c,
        w,
        random,
        body,
        Some(items),
        Some(attacks),
        &mut false,
        buffs,
        buffers,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn check_neighbors_with_items(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    items: Option<&[Vec<u8>]>,
    attacks: Option<&[u8]>,
    stopped: &mut bool,
    buffs: &[u8],
    buffers: &mut crate::ammunition::WorkBuffers,
    display_writers: bool,
) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    for _ in 0..110 {
        if *stopped {
            return Ok(events);
        }
        let list = neighbors(c, w, body)?;
        let mut changed = false;
        for &other in &list {
            let guard = if other < 10 {
                w[other as usize].record().guarding()? != 0
                    && w[other as usize].record().turn_skip_duration()? == 0
            } else {
                let m = c.enemies.records[(other - 10) as usize].record();
                m.guarding() != 0
                    && m.turn_skip_duration() == 0
                    && !m.removed()
                    && (body >= 10 || !matches!(w[body as usize].record().profession()?, 4 | 5))
            };
            if guard {
                if display_writers {
                    refresh_body_item(c, w, items.unwrap(), other, buffers)?;
                }
                changed = true;
                if other < 10 {
                    w[other as usize].record_mut().set_guarding(0);
                } else {
                    c.enemies.records[(other - 10) as usize]
                        .record_mut()
                        .set_reaction_ready(false);
                }
                if other >= 10 {
                    c.weapon_work.natural_attack_index = Some(0);
                }
                events.extend(resolve_attack(
                    c, w, random, other, body, 0, body, items, attacks, buffs, buffers,
                )?);
                if other >= 10 && !has_active_warrior(w) {
                    *stopped = true;
                }
                if !alive(c, w, body) {
                    return Ok(events);
                }
            }
        }
        if changed && display_writers {
            refresh_body_item(c, w, items.unwrap(), body, buffers)?;
        }
        if list.is_empty() || body < 10 {
            return Ok(events);
        }
        let target = list[(random.bound(list.len() as i16) - 1) as usize];
        let e = (body - 10) as usize;
        c.enemies.records[e].record_mut().set_target_body(target);
        let count = c.enemies.records[e].record().actions_per_round() / 2;
        let mut killed = false;
        c.weapon_work.natural_attack_index = Some(0);
        for index in 0..count.max(0) as usize {
            let actor = &mut c.enemies.records[e];
            let actions = actor.record().action_points().wrapping_sub(2);
            actor.record_mut().set_action_points(actions);
            if actions < 0 {
                if actions == -1 {
                    actor.record_mut().set_reaction_ready(true);
                }
                return Ok(events);
            }
            let movement = actor.record().movement_points().wrapping_sub(3);
            actor.record_mut().set_movement_points(movement);
            if actions < 2 {
                actor.record_mut().set_reaction_ready(actions >= 1);
            }
            events.extend(resolve_attack(
                c, w, random, body, target, index, body, items, attacks, buffs, buffers,
            )?);
            if !has_active_warrior(w) {
                *stopped = true;
            }
            if !alive(c, w, target) {
                killed = true;
                break;
            }
            c.weapon_work.natural_attack_index = Some(index as i16 + 1);
        }
        if !killed {
            return Ok(events);
        }
    }
    Err(RuleError::Invalid(
        "neighbor attack exceeded body capacity".into(),
    ))
}
