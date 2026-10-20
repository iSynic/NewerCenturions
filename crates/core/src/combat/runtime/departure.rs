//! Departure in the original combat execution order.
use super::{alive, neighbors, neighbors_at, refresh_body_item, resolve_attack};
use crate::combat::{Combat, enemy, put_word};
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
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn warrior_departure_with_work(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    body: i16,
    destination: (i16, i16),
    items: &[Vec<u8>],
    attacks: &[u8],
    buffs: &[u8],
    buffers: &mut crate::ammunition::WorkBuffers,
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
        buffers,
        true,
    )
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
) -> Result<Vec<Event>, RuleError> {
    if (body < 10 && w[body as usize].record().reaction_suppression()? != 0)
        || (body >= 10
            && c.enemies.records[(body - 10) as usize]
                .record()
                .reaction_suppression()
                != 0)
    {
        return Ok(vec![]);
    }
    let mut previous = neighbors(c, w, body)?;
    let projected = neighbors_at(c, w, body, destination)?;
    previous.retain(|other| !projected.contains(other));
    previous.sort_unstable();
    let mut events = vec![];
    let mut changed = false;
    for other in previous {
        if other < 10 {
            if w[other as usize].record().turn_skip_duration()? != 0 {
                continue;
            }
            put_word(&mut w[other as usize].bytes, 0x16, 0);
        } else {
            let m = &mut c.enemies.records[(other - 10) as usize].bytes;
            if m[0xb3] != 0
                || m[0xb0] == 255
                || (body < 10 && matches!(w[body as usize].record().profession()?, 4 | 5))
            {
                continue;
            }
            m[0x104] = 0;
        }
        if display_writers {
            refresh_body_item(c, w, items, other, buffers)?;
        }
        changed = true;
        events.extend(resolve_attack(
            c,
            w,
            random,
            other,
            body,
            0,
            body,
            Some(items),
            Some(attacks),
            buffs,
            buffers,
        )?);
        if !alive(c, w, body) {
            changed = false;
            break;
        }
    }
    if changed && display_writers {
        refresh_body_item(c, w, items, body, buffers)?;
    }
    Ok(events)
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
    events: &mut Vec<Event>,
) -> Result<enemy::EnemyMove, RuleError> {
    let moved = crate::combat::footprint::move_enemy_before_place(
        c,
        mission,
        (body - 10) as usize,
        random,
        &mut |projected, rng, destination| {
            events.extend(warrior_departure_inner(
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
            )?);
            Ok(())
        },
    )?;
    *c = moved.combat;
    Ok(enemy::EnemyMove {
        moved: moved.moved,
        sounds: moved.sounds,
    })
}
