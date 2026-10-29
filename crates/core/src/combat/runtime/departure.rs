//! Departure in the original combat execution order.
use super::reaction_calls::{self, ReactionContext, ReactionResult};
use super::{neighbors, neighbors_at, refresh_body_item};
use crate::combat::{
    Combat, enemy,
    neighbor_work::{self, NativeNeighbors},
};
use crate::{Event, RandomState, RuleError};
use centurions_content::Warrior;

pub fn warrior_departure(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    destination: (i16, i16),
    items: &[Vec<u8>],
    attacks: &[u8],
) -> Result<Vec<Event>, RuleError> {
    warrior_departure_with_buffs(c, w, random, body, destination, items, attacks, &[0; 20])
}
#[allow(clippy::too_many_arguments)]
pub fn warrior_departure_with_buffs(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    destination: (i16, i16),
    items: &[Vec<u8>],
    attacks: &[u8],
    buffs: &[u8],
) -> Result<Vec<Event>, RuleError> {
    warrior_departure_inner(
        c,
        w,
        random,
        body,
        destination,
        items,
        attacks,
        buffs,
        &mut Default::default(),
        false,
        None,
    )
    .map(|result| result.events)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn warrior_departure_inner(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    destination: (i16, i16),
    items: &[Vec<u8>],
    attacks: &[u8],
    buffs: &[u8],
    buffers: &mut crate::ammunition::WorkBuffers,
    display_writers: bool,
    mut native: Option<NativeNeighbors<'_>>,
) -> Result<ReactionResult, RuleError> {
    let previous = if let Some(context) = native.as_mut() {
        neighbor_work::begin(c);
        let position = super::position(c, body);
        let direction = (
            destination.0.wrapping_sub(position.0) as i8,
            destination.1.wrapping_sub(position.1) as i8,
        );
        neighbor_work::scan(c, w, 2, direction, context)?
    } else {
        if (body < 10 && w[body as usize].record().reaction_suppression()? != 0)
            || (body >= 10
                && c.enemies.records[(body - 10) as usize]
                    .record()
                    .reaction_suppression()
                    != 0)
        {
            return Ok(ReactionResult::default());
        }
        let mut previous = neighbors(c, w, body)?;
        let projected = neighbors_at(c, w, body, destination)?;
        previous.retain(|other| !projected.contains(other));
        previous.sort_unstable();
        previous
    };
    let mut context = ReactionContext {
        items: Some(items),
        attacks: Some(attacks),
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
    let mut changed = false;
    let result = reaction_calls::guards(
        c,
        w,
        random,
        body,
        true,
        &previous,
        &mut changed,
        &mut context,
    )?;
    if changed && display_writers {
        refresh_body_item(c, w, items, context.actor(c, body), context.buffers)?;
    }
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn move_actor_with_reactions(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    body: i16,
    items: &[Vec<u8>],
    buffs: &[u8],
    buffers: &mut crate::ammunition::WorkBuffers,
    display_writers: bool,
    mut native: Option<NativeNeighbors<'_>>,
    events: &mut Vec<Event>,
) -> Result<enemy::EnemyMove, RuleError> {
    if c.enemies.records[(body - 10) as usize]
        .record()
        .fleeing_duration()
        != 0
        && let Some(context) = native.as_mut()
    {
        neighbor_work::begin(c);
        neighbor_work::scan(c, w, 1, (0, 0), context)?;
    }
    let aliases = native.as_ref().map_or(
        crate::combat::map_alias::MapAliases::of_mission(mission),
        |native| native.aliases,
    );
    let moved = crate::combat::footprint::move_enemy_before_place(
        c,
        (mission, aliases),
        (body - 10) as usize,
        random,
        &mut |projected, rng, destination| {
            let reaction = warrior_departure_inner(
                projected,
                w,
                rng,
                body,
                destination,
                items,
                &mission
                    .combat
                    .as_ref()
                    .ok_or_else(|| RuleError::Invalid("combat tables missing".into()))?
                    .attacks,
                buffs,
                buffers,
                display_writers,
                native.as_mut().map(|context| NativeNeighbors {
                    selected_warrior: context.selected_warrior,
                    party_mask: &mut *context.party_mask,
                    aliases: context.aliases,
                    table: context.table,
                }),
            )?;
            // movemonster13e2b6 ignores checkforenemy(2)'s return value.
            events.extend(reaction.events);
            Ok(())
        },
    )?;
    *c = moved.combat;
    Ok(enemy::EnemyMove {
        moved: moved.moved,
        sounds: moved.sounds,
    })
}
