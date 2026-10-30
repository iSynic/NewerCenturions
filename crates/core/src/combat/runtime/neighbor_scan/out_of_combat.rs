//! The neighbour scan after a battle has ended.
use super::scan as kernel;
use crate::{RuleError, Session};

/// Scans the neighbors of the acting body after the battle has ended, on the party's own state.
///
/// Original: checkforenemy
pub(crate) fn scan(
    state: &mut Session,
    content: &crate::Content,
    mode: usize,
    size: &mut i8,
) -> Result<Vec<i16>, RuleError> {
    let actor = state
        .globals
        .battle_order
        .current(state.globals.monster_up)?;
    if !state.globals.kill_counts.reaction_admitted() {
        return Ok(vec![]);
    }
    let mut neighbors = std::mem::take(&mut state.globals.neighbors);
    let mut mask = state.globals.party_track;
    let result = (|| {
        let tactical = &state.globals.battlefield;
        let map = &tactical.field;
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
                table: Some(&state.globals.warrior_table),
                actor,
                char_select_new: state.char_select_new,
                monster_up: state.globals.monster_up,
                warrior_position: &|index| {
                    tactical
                        .pos
                        .get(index)
                        .copied()
                        .ok_or_else(|| RuleError::from("selected warrior outside the squad"))
                },
                monster: &|index| state.globals.monsters.slot(index).map(|slot| &slot.record),
                enemy_position: &|index| {
                    let backing = &state.globals.monsters;
                    let point = backing.slot(index)?.position()?;
                    Ok((
                        point.0.wrapping_sub(backing.origin.0) as i16,
                        point.1.wrapping_sub(backing.origin.1) as i16,
                    ))
                },
            },
            &mut neighbors,
            &mut mask,
            mode,
            (direction[0], direction[1]),
            size,
        )
    })();
    state.globals.neighbors = neighbors;
    state.globals.party_track = mask;
    result
}
