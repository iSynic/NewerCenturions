//! Reactions when a body leaves a cell.
use super::reaction_calls::{self, ReactionContext, ReactionResult};
use super::{neighbors, neighbors_at, refresh_body_item};
use crate::combat::map_alias::MapAliases;
use crate::combat::{
    Combat, enemy,
    runtime::neighbor_scan::{self, NeighborScan},
};
use crate::{Event, RandomState, RuleError};
use centurions_content::Warrior;

/// Lets the neighbors of a warrior react as it leaves for `destination`.
pub fn warrior_departure(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    destination: (i16, i16),
    items: &[Vec<u8>],
    attacks: &[u8],
) -> Result<Vec<Event>, RuleError> {
    warrior_departure_with_buffs(
        combat,
        warriors,
        random,
        body,
        destination,
        items,
        attacks,
        &[0; 20],
    )
}
/// Lets the neighbors of a warrior react as it leaves for `destination`, with the party's buff
/// table.
#[allow(clippy::too_many_arguments)]
pub fn warrior_departure_with_buffs(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    destination: (i16, i16),
    items: &[Vec<u8>],
    attacks: &[u8],
    buffs: &[u8],
) -> Result<Vec<Event>, RuleError> {
    warrior_departure_inner(
        combat,
        warriors,
        random,
        body,
        destination,
        &mut ReactionContext {
            items: Some(items),
            attacks: Some(attacks),
            buffs,
            buffers: &mut Default::default(),
            display_writers: false,
            scan: None,
        },
    )
    .map(|result| result.events)
}

/// Lets the guards around a body react as it leaves for `destination`. Only neighbors the body
/// stops touching can react; a body with reactions suppressed provokes none.
///
/// Original: checkforenemy
pub(super) fn warrior_departure_inner(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    destination: (i16, i16),
    context: &mut ReactionContext<'_>,
) -> Result<ReactionResult, RuleError> {
    let previous = if let Some(scan) = context.scan.as_mut() {
        neighbor_scan::begin(combat);
        let position = combat.body_position(body);
        let direction = (
            destination.0.wrapping_sub(position.0) as i8,
            destination.1.wrapping_sub(position.1) as i8,
        );
        neighbor_scan::scan(combat, warriors, 2, direction, scan)?
    } else {
        if (body < 10 && warriors[body as usize].record().reaction_suppression()? != 0)
            || (body >= 10
                && combat.roster.records[(body - 10) as usize]
                    .record()
                    .reaction_suppression()
                    != 0)
        {
            return Ok(ReactionResult::default());
        }
        let mut previous = neighbors(combat, warriors, body)?;
        let projected = neighbors_at(combat, warriors, body, destination)?;
        previous.retain(|other| !projected.contains(other));
        previous.sort_unstable();
        previous
    };
    let mut changed = false;
    let result = reaction_calls::guards(
        combat,
        warriors,
        random,
        body,
        true,
        &previous,
        &mut changed,
        context,
    )?;
    if changed && context.display_writers {
        refresh_body_item(
            combat,
            warriors,
            context.items.unwrap(),
            context.actor(combat, body),
            context.buffers,
        )?;
    }
    Ok(result)
}

/// Moves a monster one step. The guards it leaves behind react as it departs, and a reaction never
/// stops the move. A fleeing monster first rescans its surroundings.
///
/// Original: movemonster
#[allow(clippy::too_many_arguments)]
pub(crate) fn move_actor_with_reactions(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    body: i16,
    items: &[Vec<u8>],
    buffs: &[u8],
    buffers: &mut crate::ammunition::LoadedItems,
    display_writers: bool,
    mut scan: Option<NeighborScan<'_>>,
    events: &mut Vec<Event>,
) -> Result<enemy::EnemyMove, RuleError> {
    if combat.roster.records[(body - 10) as usize]
        .record()
        .fleeing_duration()
        != 0
        && let Some(context) = scan.as_mut()
    {
        neighbor_scan::begin(combat);
        neighbor_scan::scan(combat, warriors, 1, (0, 0), context)?;
    }
    let aliases = scan
        .as_ref()
        .map_or(MapAliases::of_mission(mission), |scan| scan.aliases);
    let moved = crate::combat::footprint::move_enemy_before_place(
        combat,
        (mission, aliases),
        (body - 10) as usize,
        random,
        &mut |projected, rng, destination| {
            let reaction = warrior_departure_inner(
                projected,
                warriors,
                rng,
                body,
                destination,
                &mut ReactionContext {
                    items: Some(items),
                    attacks: Some(
                        &mission
                            .combat
                            .as_ref()
                            .ok_or_else(|| RuleError::Invalid("combat tables missing".into()))?
                            .attacks,
                    ),
                    buffs,
                    buffers,
                    display_writers,
                    scan: scan.as_mut().map(|context| NeighborScan {
                        char_select_new: context.char_select_new,
                        party_track: &mut *context.party_track,
                        aliases: context.aliases,
                        table: context.table,
                    }),
                },
            )?;
            // The departure reaction never stops the move.
            events.extend(reaction.events);
            Ok(())
        },
    )?;
    *combat = moved.combat;
    Ok(enemy::EnemyMove {
        moved: moved.moved,
        sounds: moved.sounds,
    })
}
