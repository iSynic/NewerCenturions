//! The literal checkforenemy neighborhood scan over live or retained record views.
use super::{NeighborWork, unsupported};
use crate::{RuleError, combat::TacticalMap};
use centurions_content::{Warrior, combat::MonsterRecord, records::WarriorRecord};

pub(super) struct Scene<'a> {
    pub warriors: &'a [Warrior],
    pub map: &'a TacticalMap,
    pub aliases: crate::combat::map_alias::MapAliases<'a>,
    pub table: Option<&'a crate::state::WarriorTable>,
    pub actor: i16,
    pub selected_warrior: usize,
    pub selected_enemy: usize,
    pub warrior_position: &'a dyn Fn(usize) -> Result<(i16, i16), RuleError>,
    pub enemy: &'a dyn Fn(usize) -> Result<&'a MonsterRecord, RuleError>,
    pub enemy_position: &'a dyn Fn(usize) -> Result<(i16, i16), RuleError>,
}

pub(super) fn run(
    scene: &Scene<'_>,
    work: &mut NeighborWork,
    mask: &mut [i16; 6],
    mode: usize,
    direction: (i8, i8),
    size: &mut i8,
) -> Result<Vec<i16>, RuleError> {
    let actor = scene.actor;
    // checkforenemy 0x110e96 takes an order entry 9 through the party position branch
    // (< 10) and the NPC faction branch (>= 9): the selected warrior's position, the
    // selected NPC's faction WORD 0x439d4a.
    let (position, size, faction) = if actor == 9 {
        let position = (scene.warrior_position)(scene.selected_warrior)?;
        let faction = (scene.enemy)(scene.selected_enemy)?.record().faction()?;
        (position, *size, faction)
    } else if actor < 9 {
        let warrior = scene
            .warriors
            .get(scene.selected_warrior)
            .ok_or_else(|| RuleError::from("selected warrior outside the squad"))?;
        let position = (scene.warrior_position)(scene.selected_warrior)?;
        (position, *size, warrior.record().faction()?)
    } else {
        let enemy = (scene.enemy)(scene.selected_enemy)?.record();
        *size = enemy.size()?.wrapping_abs() as i8;
        (
            (scene.enemy_position)(scene.selected_enemy)?,
            *size,
            enemy.faction()?,
        )
    };
    let mut position = (position.0 as i8, position.1 as i8);
    if mode == 2 {
        position.0 = position.0.wrapping_add(direction.0);
        position.1 = position.1.wrapping_add(direction.1);
    }
    let minimum_x = if size == 2 || size == 3 { -2 } else { -1 };
    let minimum_y = if size == 1 || size == 3 { -2 } else { -1 };
    work.bodies[..12].fill(0);
    work.presence[mode] = vec![0; 110];
    let mut list = vec![];
    for dy in minimum_y..=1 {
        for dx in minimum_x..=1 {
            let x = i32::from(scene.map.shift.0) + 5 + i32::from(position.0) + dx;
            let y = i32::from(scene.map.shift.1) + 5 + i32::from(position.1) + dy;
            let index = x.wrapping_mul(90).wrapping_add(y);
            // checkforenemy reads past the map edges without clipping.
            let body = scene
                .aliases
                .read(scene.map, index)?
                .ok_or_else(|| {
                    unsupported(
                        "neighbor scan outside the map and the represented memory beside it",
                    )
                })?
                .wrapping_abs();
            if !(0..110).contains(&body) || work.presence[mode][body as usize] != 0 {
                continue;
            }
            let eligible = if body < 9 {
                // Bodies past the squad read the table, the Upgrade records or the
                // editor backup 0x44ecb0.
                let held = scene.table.and_then(|table| table.bytes(body as usize));
                let record = match scene.warriors.get(body as usize) {
                    Some(warrior) => warrior.record(),
                    None => WarriorRecord::from_bytes(
                        held.as_deref()
                            .ok_or("native neighbor scan without the warrior table")?,
                    ),
                };
                record.faction()? != faction && record.active_flag() != 0
            } else if body >= 10 {
                let enemy = (scene.enemy)((body - 10) as usize)?.record();
                enemy.faction()? != faction
                    && enemy.stamina()? > 0
                    && (actor < 9 || !enemy.removed() && body != actor)
            } else {
                return Err(unsupported("neighbor body 9 before the NPC array"));
            };
            if eligible {
                work.write_body(list.len(), body, mask)?;
                list.push(body);
                work.presence[mode][body as usize] = 1;
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
        scene.warriors[scene.selected_warrior]
            .record()
            .reaction_suppression()?
            != 0
    } else {
        (scene.enemy)(scene.selected_enemy)?
            .record()
            .reaction_suppression()
            != 0
    };
    if suppressed {
        return Ok(vec![]);
    }
    let before = &work.presence[1];
    let after = &work.presence[2];
    let departed: Vec<_> = before
        .iter()
        .zip(after)
        .enumerate()
        .filter_map(|(body, (before, after))| (*before != 0 && *after == 0).then_some(body as i16))
        .collect();
    for (index, body) in departed.iter().enumerate() {
        work.write_body(index, *body, mask)?;
    }
    if !departed.is_empty() {
        work.departure = 1;
    }
    Ok(departed)
}
