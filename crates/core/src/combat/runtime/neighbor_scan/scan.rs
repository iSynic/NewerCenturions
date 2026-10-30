//! checkforenemy's scan of a body's neighbours, during or after a battle.
use super::{Neighbors, unsupported};
use crate::combat::map_alias::MapAliases;
use crate::{RuleError, combat::CombatField};
use centurions_content::{Warrior, combat::MonsterRecord, records::WarriorRecord};

/// Everything one scan reads: the squad, the map, the acting body and how to find positions and
/// monster records.
pub(super) struct Scene<'a> {
    pub warriors: &'a [Warrior],
    pub map: &'a CombatField,
    pub aliases: MapAliases<'a>,
    pub table: Option<&'a crate::state::WarriorTable>,
    pub actor: i16,
    pub char_select_new: usize,
    pub monster_up: usize,
    pub warrior_position: &'a dyn Fn(usize) -> Result<(i16, i16), RuleError>,
    pub monster: &'a dyn Fn(usize) -> Result<&'a MonsterRecord, RuleError>,
    pub enemy_position: &'a dyn Fn(usize) -> Result<(i16, i16), RuleError>,
}

/// Lists the opposing bodies around the acting body. Mode 0 looks where the body stands, mode 1 is
/// the rescan on entry, mode 2 looks from the cell it steps to and lists the bodies it has stopped
/// touching. A body of size 1 also looks one extra row, size 2 one extra column and size 3 both, at
/// the lower coordinates. Departures are not collected for a body whose reactions are suppressed.
///
/// Original: checkforenemy
pub(super) fn run(
    scene: &Scene<'_>,
    neighbors: &mut Neighbors,
    mask: &mut [i16; 6],
    mode: usize,
    direction: (i8, i8),
    size: &mut i8,
) -> Result<Vec<i16>, RuleError> {
    let actor = scene.actor;
    // Body 9 takes the selected warrior's position and the selected monster's
    // faction (see docs/source-notes/combat.md).
    let (position, size, faction) = if actor == 9 {
        let position = (scene.warrior_position)(scene.char_select_new)?;
        let faction = (scene.monster)(scene.monster_up)?.record().faction()?;
        (position, *size, faction)
    } else if actor < 9 {
        let warrior = scene
            .warriors
            .get(scene.char_select_new)
            .ok_or_else(|| RuleError::from("selected warrior outside the squad"))?;
        let position = (scene.warrior_position)(scene.char_select_new)?;
        (position, *size, warrior.record().faction()?)
    } else {
        let monster = (scene.monster)(scene.monster_up)?.record();
        *size = monster.size()?.wrapping_abs() as i8;
        (
            (scene.enemy_position)(scene.monster_up)?,
            *size,
            monster.faction()?,
        )
    };
    let mut position = (position.0 as i8, position.1 as i8);
    if mode == 2 {
        position.0 = position.0.wrapping_add(direction.0);
        position.1 = position.1.wrapping_add(direction.1);
    }
    let minimum_x = if size == 2 || size == 3 { -2 } else { -1 };
    let minimum_y = if size == 1 || size == 3 { -2 } else { -1 };
    neighbors.bodies[..12].fill(0);
    neighbors.presence[mode] = vec![0; 110];
    let mut list = vec![];
    for dy in minimum_y..=1 {
        for dx in minimum_x..=1 {
            let x = i32::from(scene.map.map_shift.0) + 5 + i32::from(position.0) + dx;
            let y = i32::from(scene.map.map_shift.1) + 5 + i32::from(position.1) + dy;
            let index = x.wrapping_mul(90).wrapping_add(y);
            // The scan reads past the map edges without clipping.
            let body = scene
                .aliases
                .read(scene.map, index)?
                .ok_or_else(|| {
                    unsupported(
                        "neighbor scan outside the map and the represented memory beside it",
                    )
                })?
                .wrapping_abs();
            if !(0..110).contains(&body) || neighbors.presence[mode][body as usize] != 0 {
                continue;
            }
            let eligible = if body < 9 {
                // Bodies past the squad read the warrior table, the Upgrade records
                // or the editor backup.
                let held = scene.table.and_then(|table| table.bytes(body as usize));
                let record = match scene.warriors.get(body as usize) {
                    Some(warrior) => warrior.record(),
                    None => WarriorRecord::from_bytes(
                        held.as_deref()
                            .ok_or("native neighbor scan without the warrior table")?,
                    ),
                };
                record.faction()? != faction && record.in_battle() != 0
            } else if body >= 10 {
                let monster = (scene.monster)((body - 10) as usize)?.record();
                monster.faction()? != faction
                    && monster.stamina()? > 0
                    && (actor < 9 || !monster.removed() && body != actor)
            } else {
                return Err(unsupported("neighbor body 9 before the NPC array"));
            };
            if eligible {
                neighbors.write_body(list.len(), body, mask)?;
                list.push(body);
                neighbors.presence[mode][body as usize] = 1;
            }
        }
    }
    if mode != 2 {
        return Ok(list);
    }
    // Departures are collected only for order entries 0..8 and from 10.
    if actor == 9 {
        return Ok(vec![]);
    }
    let suppressed = if actor < 9 {
        scene.warriors[scene.char_select_new]
            .record()
            .reaction_suppression()?
            != 0
    } else {
        (scene.monster)(scene.monster_up)?
            .record()
            .reaction_suppression()
            != 0
    };
    if suppressed {
        return Ok(vec![]);
    }
    let before = &neighbors.presence[1];
    let after = &neighbors.presence[2];
    let departed: Vec<_> = before
        .iter()
        .zip(after)
        .enumerate()
        .filter_map(|(body, (before, after))| (*before != 0 && *after == 0).then_some(body as i16))
        .collect();
    for (index, body) in departed.iter().enumerate() {
        neighbors.write_body(index, *body, mask)?;
    }
    if !departed.is_empty() {
        neighbors.departure = 1;
    }
    Ok(departed)
}
