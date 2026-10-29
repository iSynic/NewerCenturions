//! Neighbor views after the native combat flag has been cleared.
use super::scan as kernel;
use crate::{RuleError, Session};

pub(crate) fn scan(
    state: &mut Session,
    content: &crate::Content,
    mode: usize,
    size: &mut i8,
) -> Result<Vec<i16>, RuleError> {
    let actor = state
        .native_work
        .initiative_work
        .current(state.native_work.selected_enemy)?;
    if !state.native_work.victory_work.reaction_admitted() {
        return Ok(vec![]);
    }
    let mut work = std::mem::take(&mut state.native_work.neighbor_work);
    let mut mask = state.native_work.party_mask;
    let result = (|| {
        let tactical = &state.native_work.tactical_work;
        let map = &tactical.map;
        let direction = state
            .exploration
            .as_ref()
            .ok_or_else(|| RuleError::from("reaction scan outside a mission"))?
            .last_move;
        kernel::run(
            &kernel::Scene {
                warriors: &state.warriors,
                map,
                aliases: crate::combat::map_alias::session_map_aliases!(state, content content),
                table: Some(&state.native_work.warrior_table),
                actor,
                selected_warrior: state.selected_warrior,
                selected_enemy: state.native_work.selected_enemy,
                warrior_position: &|index| {
                    tactical
                        .positions
                        .get(index)
                        .copied()
                        .ok_or_else(|| RuleError::from("selected warrior outside the squad"))
                },
                enemy: &|index| {
                    state
                        .native_work
                        .npc_backing
                        .slot(index)
                        .map(|slot| &slot.record)
                },
                enemy_position: &|index| {
                    let backing = &state.native_work.npc_backing;
                    let point = backing.slot(index)?.position()?;
                    Ok((
                        point.0.wrapping_sub(backing.origin.0) as i16,
                        point.1.wrapping_sub(backing.origin.1) as i16,
                    ))
                },
            },
            &mut work,
            &mut mask,
            mode,
            (direction[0], direction[1]),
            size,
        )
    })();
    state.native_work.neighbor_work = work;
    state.native_work.party_mask = mask;
    result
}
