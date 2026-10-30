use crate::combat::Combat;
use crate::{RandomState, RuleError};
use centurions_content::mission::OpeningMission;

/// Whether a monster moved this turn and the sounds its steps made.
#[derive(Debug, PartialEq, Eq)]
pub struct EnemyMove {
    pub moved: bool,
    pub sounds: Vec<i16>,
}

/// Moves one monster a step and reports the result.
///
/// Original: movemonster
pub fn move_enemy(
    combat: &mut Combat,
    mission: &OpeningMission,
    index: usize,
    random: &mut RandomState,
) -> Result<EnemyMove, RuleError> {
    let moved = super::footprint::move_enemy(combat, mission, index, random)?;
    *combat = moved.combat;
    Ok(EnemyMove {
        moved: moved.moved,
        sounds: moved.sounds,
    })
}
