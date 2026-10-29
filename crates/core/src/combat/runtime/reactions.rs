//! Reactions in the original combat execution order.
use super::reaction_calls::{self, AttackCall, ReactionContext, ReactionResult};
use super::{has_active_warrior, neighbors, refresh_body_item};
use crate::combat::{
    Combat,
    neighbor_work::{self, NativeNeighbors},
};
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
        None,
    )
    .map(|result| result.events)
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
        None,
    )
    .map(|result| result.events)
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
        None,
    )
    .map(|result| result.events)
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
    native: Option<NativeNeighbors<'_>>,
) -> Result<ReactionResult, RuleError> {
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
        native,
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
    mut native: Option<NativeNeighbors<'_>>,
) -> Result<ReactionResult, RuleError> {
    let mut result = ReactionResult::default();
    let mut context = ReactionContext {
        items,
        attacks,
        buffs,
        buffers,
        display_writers,
        native: native.as_mut().map(|native| NativeNeighbors {
            selected_warrior: native.selected_warrior,
            party_mask: &mut *native.party_mask,
            aliases: native.aliases,
            table: native.table,
        }),
    };
    if context.native.is_some() {
        neighbor_work::begin(c);
    }
    // bVar2 belongs to the invocation, not to an individual rescan.
    let mut changed = false;
    for _ in 0..110 {
        if *stopped {
            return Ok(result);
        }
        let list = if let Some(native) = context.native.as_mut() {
            neighbor_work::scan(c, w, 0, (0, 0), native)?
        } else {
            neighbors(c, w, body)?
        };
        let guards =
            reaction_calls::guards(c, w, random, body, false, &list, &mut changed, &mut context)?;
        result.events.extend(guards.events);
        if guards.interrupted {
            if !has_active_warrior(w) {
                *stopped = true;
            }
            result.interrupted = true;
            return Ok(result);
        }
        let actor = context.actor(c, body);
        if changed && context.display_writers {
            refresh_body_item(c, w, context.items.unwrap(), actor, context.buffers)?;
        }
        if list.is_empty() || actor < 10 {
            return Ok(result);
        }
        let target_index = (random.bound(list.len() as i16) - 1) as usize;
        let target = context.body(c, &list, target_index)?;
        let enemy = if context.native.is_some() {
            c.selected_enemy
        } else {
            (body - 10) as usize
        };
        neighbor_work::require_roster_body(c, w, enemy as i16 + 10)?;
        c.enemies.records[enemy]
            .record_mut()
            .set_target_body(target);
        c.weapon_work.natural_attack_index = 0;
        loop {
            let enemy = if context.native.is_some() {
                c.selected_enemy
            } else {
                (body - 10) as usize
            };
            neighbor_work::require_roster_body(c, w, enemy as i16 + 10)?;
            let index = c.weapon_work.natural_attack_index;
            if index >= i16::from(c.enemies.records[enemy].record().actions_per_round() / 2) {
                return Ok(result);
            }
            let actor = &mut c.enemies.records[enemy];
            let actions = actor.record().action_points().wrapping_sub(2);
            actor.record_mut().set_action_points(actions);
            if actions < 0 {
                if actions == -1 {
                    actor.record_mut().set_reaction_ready(true);
                }
                return Ok(result);
            }
            let movement = actor.record().movement_points().wrapping_sub(3);
            actor.record_mut().set_movement_points(movement);
            if actions < 2 {
                actor.record_mut().set_reaction_ready(actions >= 1);
            }
            let target = actor.record().target_body()?;
            neighbor_work::require_roster_body(c, w, target)?;
            let index = usize::try_from(index)
                .map_err(|_| RuleError::Invalid("negative natural-attack index".into()))?;
            let outcome = reaction_calls::attack(
                c,
                w,
                random,
                AttackCall {
                    attacker: enemy as i16 + 10,
                    recipient: target,
                    index,
                    retained_actor: context.actor(c, body),
                },
                &mut context,
            )?;
            result.events.extend(outcome.events);
            if !has_active_warrior(w) {
                *stopped = true;
            }
            if outcome.returned {
                break;
            }
            let index = c.weapon_work.natural_attack_index;
            c.weapon_work.natural_attack_index = index.wrapping_add(1);
        }
    }
    Err(RuleError::Invalid(
        "neighbor attack exceeded body capacity".into(),
    ))
}
