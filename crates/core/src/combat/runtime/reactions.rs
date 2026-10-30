//! Neighbors reacting to a body beside them.
use super::reaction_calls::{self, AttackCall, ReactionContext, ReactionResult};
use super::{has_active_warrior, neighbors, refresh_body_item};
use crate::combat::{
    Combat,
    runtime::neighbor_scan::{self, NeighborScan},
};
use crate::{Event, RandomState, RuleError};
use centurions_content::Warrior;

/// Lets the neighbors of `body` react to it without any item tables.
pub fn check_neighbors(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
) -> Result<Vec<Event>, RuleError> {
    check_neighbors_with_items(
        combat,
        warriors,
        random,
        body,
        &mut ReactionContext {
            items: None,
            attacks: None,
            buffs: &[0; 20],
            buffers: &mut Default::default(),
            display_writers: false,
            scan: None,
        },
        &mut false,
    )
    .map(|result| result.events)
}

/// Lets the neighbors of `body` react, with the item definitions.
pub fn check_neighbors_with_definitions(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    items: &[Vec<u8>],
) -> Result<Vec<Event>, RuleError> {
    check_neighbors_with_items(
        combat,
        warriors,
        random,
        body,
        &mut ReactionContext {
            items: Some(items),
            attacks: None,
            buffs: &[0; 20],
            buffers: &mut Default::default(),
            display_writers: false,
            scan: None,
        },
        &mut false,
    )
    .map(|result| result.events)
}

/// Lets the neighbors of `body` react, with the item and attack tables.
pub fn check_neighbors_with_context(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    items: &[Vec<u8>],
    attacks: &[u8],
) -> Result<Vec<Event>, RuleError> {
    check_neighbors_with_buffs(combat, warriors, random, body, items, attacks, &[0; 20])
}

/// Lets the neighbors of `body` react, with the item and attack tables and the buff table.
pub fn check_neighbors_with_buffs(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    items: &[Vec<u8>],
    attacks: &[u8],
    buffs: &[u8],
) -> Result<Vec<Event>, RuleError> {
    check_neighbors_with_items(
        combat,
        warriors,
        random,
        body,
        &mut ReactionContext {
            items: Some(items),
            attacks: Some(attacks),
            buffs,
            buffers: &mut Default::default(),
            display_writers: false,
            scan: None,
        },
        &mut false,
    )
    .map(|result| result.events)
}

/// Lets the neighbors of `body` react with the loaded item buffers and, when given, the shared
/// neighbor scan; the form used by live turns.
#[allow(clippy::too_many_arguments)]
pub(crate) fn check_neighbors_with_loaded(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    items: &[Vec<u8>],
    attacks: &[u8],
    buffs: &[u8],
    buffers: &mut crate::ammunition::LoadedItems,
    scan: Option<NeighborScan<'_>>,
) -> Result<ReactionResult, RuleError> {
    check_neighbors_with_items(
        combat,
        warriors,
        random,
        body,
        &mut ReactionContext {
            items: Some(items),
            attacks: Some(attacks),
            buffs,
            buffers,
            display_writers: true,
            scan,
        },
        &mut false,
    )
}

/// Lets the neighbors of `body` react to it. Guarding neighbors strike first; a kill by a guard
/// interrupts `body` and ends the check. If `body` is a monster with neighbors, it then picks one
/// at random as its target and strikes it with each of its natural attacks while it has actions
/// left, each costing two action points and three movement. A natural attack that kills starts the
/// check over with a fresh scan, repeated at most 110 times.
///
/// Original: checkforenemy
pub(super) fn check_neighbors_with_items(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    context: &mut ReactionContext<'_>,
    stopped: &mut bool,
) -> Result<ReactionResult, RuleError> {
    let mut result = ReactionResult::default();
    if context.scan.is_some() {
        neighbor_scan::begin(combat);
    }
    // The change flag covers the whole check, not each rescan.
    let mut changed = false;
    for _ in 0..110 {
        if *stopped {
            return Ok(result);
        }
        let list = if let Some(scan) = context.scan.as_mut() {
            neighbor_scan::scan(combat, warriors, 0, (0, 0), scan)?
        } else {
            neighbors(combat, warriors, body)?
        };
        let guards = reaction_calls::guards(
            combat,
            warriors,
            random,
            body,
            false,
            &list,
            &mut changed,
            context,
        )?;
        result.events.extend(guards.events);
        if guards.interrupted {
            if !has_active_warrior(warriors) {
                *stopped = true;
            }
            result.interrupted = true;
            return Ok(result);
        }
        let actor = context.actor(combat, body);
        if changed && context.display_writers {
            refresh_body_item(
                combat,
                warriors,
                context.items.unwrap(),
                actor,
                context.buffers,
            )?;
        }
        if list.is_empty() || actor < 10 {
            return Ok(result);
        }
        let target_index = (random.bound(list.len() as i16) - 1) as usize;
        let target = context.body(combat, &list, target_index)?;
        let monster = if context.scan.is_some() {
            combat.monster_up
        } else {
            (body - 10) as usize
        };
        neighbor_scan::require_roster_body(combat, warriors, monster as i16 + 10)?;
        combat.roster.records[monster]
            .record_mut()
            .set_target(target);
        combat.weapon_traits.natural_attack_index = 0;
        loop {
            let monster = if context.scan.is_some() {
                combat.monster_up
            } else {
                (body - 10) as usize
            };
            neighbor_scan::require_roster_body(combat, warriors, monster as i16 + 10)?;
            let index = combat.weapon_traits.natural_attack_index;
            if index >= i16::from(combat.roster.records[monster].record().attack_count() / 2) {
                return Ok(result);
            }
            let actor = &mut combat.roster.records[monster];
            let actions = actor.record().action_points().wrapping_sub(2);
            actor.record_mut().set_action_points(actions);
            if actions < 0 {
                if actions == -1 {
                    actor.record_mut().set_reaction_ready(true);
                }
                return Ok(result);
            }
            let movement = actor.record().movement().wrapping_sub(3);
            actor.record_mut().set_movement(movement);
            if actions < 2 {
                actor.record_mut().set_reaction_ready(actions >= 1);
            }
            let target = actor.record().target()?;
            neighbor_scan::require_roster_body(combat, warriors, target)?;
            let index = usize::try_from(index)
                .map_err(|_| RuleError::Invalid("negative natural-attack index".into()))?;
            let outcome = reaction_calls::attack(
                combat,
                warriors,
                random,
                AttackCall {
                    attacker: monster as i16 + 10,
                    recipient: target,
                    index,
                    turn_actor: context.actor(combat, body),
                },
                context,
            )?;
            result.events.extend(outcome.events);
            if !has_active_warrior(warriors) {
                *stopped = true;
            }
            if outcome.returned {
                break;
            }
            let index = combat.weapon_traits.natural_attack_index;
            combat.weapon_traits.natural_attack_index = index.wrapping_add(1);
        }
    }
    Err(RuleError::Invalid(
        "neighbor attack exceeded body capacity".into(),
    ))
}
