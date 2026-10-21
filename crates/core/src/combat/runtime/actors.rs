//! Actors in the original combat execution order.
use crate::RuleError;
use crate::combat::Combat;
use centurions_content::Warrior;

pub(super) fn refusal(kind: crate::feedback::CombatRefusal) -> RuleError {
    RuleError::Refused {
        feedback: vec![crate::feedback::NativeFeedback::Combat { kind }],
    }
}

pub(super) fn spell_identity(id: i16) -> [i16; 3] {
    [id / 1000 - 1, id % 1000 / 100 - 1, id % 100 - 1]
}

#[derive(PartialEq, Eq)]
pub(super) enum FireOutcome {
    Failed,
    Physical,
    Spell,
}

pub(super) fn position(c: &Combat, body: i16) -> (i16, i16) {
    if body < 10 {
        c.squad.positions[body as usize]
    } else {
        c.enemies.positions[(body - 10) as usize]
    }
}
pub(super) fn hostile(c: &Combat, w: &[Warrior], a: i16, b: i16) -> Result<bool, RuleError> {
    let side = |body: i16| {
        if body < 10 {
            w[body as usize].record().faction()
        } else {
            c.enemies.records[(body - 10) as usize].record().faction()
        }
    };
    Ok(side(a)? != side(b)?)
}
pub(super) fn alive(c: &Combat, w: &[Warrior], body: i16) -> bool {
    if (0..6).contains(&body) {
        w.get(body as usize)
            .is_some_and(|warrior| warrior.record().active_flag() != 0)
    } else if body >= 10 {
        c.enemies
            .records
            .get((body - 10) as usize)
            .is_some_and(|m| m.stamina().unwrap_or(0) > 0 && !m.record().removed())
    } else {
        false
    }
}
pub(super) fn has_active_warrior(warriors: &[Warrior]) -> bool {
    warriors.iter().any(|warrior| {
        warrior.record().active_flag() != 0 && warrior.record().faction().unwrap_or(1) == 0
    })
}
pub fn neighbors(c: &Combat, w: &[Warrior], body: i16) -> Result<Vec<i16>, RuleError> {
    let p = position(c, body);
    neighbors_at(c, w, body, p)
}
pub(super) fn neighbors_at(
    c: &Combat,
    w: &[Warrior],
    body: i16,
    p: (i16, i16),
) -> Result<Vec<i16>, RuleError> {
    let mut result = vec![];
    let (minimum_x, minimum_y) = if body >= 10 {
        crate::combat::footprint::neighbor_scan(&c.enemies.records[(body - 10) as usize])?
    } else {
        (-1, -1)
    };
    for dy in minimum_y..=1 {
        for dx in minimum_x..=1 {
            let absolute = (p.0 + dx + c.map.shift.0 + 5, p.1 + dy + c.map.shift.1 + 5);
            if !(0..90).contains(&absolute.0) || !(0..90).contains(&absolute.1) {
                continue;
            }
            let cell = centurions_content::mission::OpeningMission::index(
                i32::from(absolute.0),
                i32::from(absolute.1),
            )?;
            let other = c.map.tiles[cell].wrapping_abs();
            if other < 110
                && other != body
                && alive(c, w, other)
                && hostile(c, w, body, other)?
                && !result.contains(&other)
            {
                result.push(other);
            }
        }
    }
    Ok(result)
}
