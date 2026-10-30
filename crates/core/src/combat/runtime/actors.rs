//! Combat actors.
use crate::RuleError;
use crate::combat::Combat;
use centurions_content::Warrior;

/// A refusal that shows the player the combat message for why the action is not allowed.
pub(super) fn refusal(kind: crate::feedback::CombatRefusal) -> RuleError {
    RuleError::Refused {
        feedback: vec![crate::feedback::GameFeedback::Combat { kind }],
    }
}

pub(super) use crate::combat::spells::spell_identity;

/// How a firing attempt ended: it failed, shot a physical missile or cast a spell.
#[derive(PartialEq, Eq)]
pub(super) enum FireOutcome {
    Failed,
    Physical,
    Spell,
}

/// Two combatants are enemies when they stand on different sides.
pub(super) fn hostile(
    combat: &Combat,
    warriors: &[Warrior],
    a: i16,
    b: i16,
) -> Result<bool, RuleError> {
    Ok(combat.body_faction(warriors, a)? != combat.body_faction(warriors, b)?)
}
/// Whether any warrior of the party's side is still in the battle.
pub(super) fn has_active_warrior(warriors: &[Warrior]) -> bool {
    warriors.iter().any(|warrior| {
        warrior.record().in_battle() != 0 && warrior.record().faction().unwrap_or(1) == 0
    })
}
/// The living enemies in the cells around a combatant, within reach of its footprint.
pub fn neighbors(combat: &Combat, warriors: &[Warrior], body: i16) -> Result<Vec<i16>, RuleError> {
    let position = combat.body_position(body);
    neighbors_at(combat, warriors, body, position)
}
/// The living enemies in the cells around position `p` as seen by `body`. A larger monster looks
/// one cell farther back along each extra footprint dimension.
///
/// Original: checkforenemy
pub(super) fn neighbors_at(
    combat: &Combat,
    warriors: &[Warrior],
    body: i16,
    p: (i16, i16),
) -> Result<Vec<i16>, RuleError> {
    let mut result = vec![];
    let (minimum_x, minimum_y) = if body >= 10 {
        crate::combat::footprint::neighbor_scan(&combat.roster.records[(body - 10) as usize])?
    } else {
        (-1, -1)
    };
    for dy in minimum_y..=1 {
        for dx in minimum_x..=1 {
            let absolute = (
                p.0 + dx + combat.field.map_shift.0 + 5,
                p.1 + dy + combat.field.map_shift.1 + 5,
            );
            if !(0..90).contains(&absolute.0) || !(0..90).contains(&absolute.1) {
                continue;
            }
            let cell = centurions_content::mission::OpeningMission::index(
                i32::from(absolute.0),
                i32::from(absolute.1),
            )?;
            let other = combat.field.tiles[cell].wrapping_abs();
            if other < 110
                && other != body
                && combat.body_alive(warriors, other)
                && hostile(combat, warriors, body, other)?
                && !result.contains(&other)
            {
                result.push(other);
            }
        }
    }
    Ok(result)
}
