use crate::RuleError;
use crate::combat::Combat;
use centurions_content::{
    Warrior,
    records::{CharacterStatisticsMut, Statistic},
};

/// Record-only compatibility callers omit the literal victory work.
/// Session movement reads VictoryWork instead.
pub(crate) fn hostiles_remain(combat: &Combat) -> bool {
    combat.enemies.records.iter().any(|enemy| {
        enemy.stamina().unwrap_or(0) > 0
            && enemy.record().mechanical_state() != 255
            && enemy.record().faction().unwrap_or(1) != 0
    })
}

pub fn kill_body(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    body: i16,
) -> Result<(), RuleError> {
    kill_body_with_processing(combat, warriors, body, None)
}

pub(crate) fn kill_body_with_processing(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    body: i16,
    processing: Option<super::death_scripts::DeathProcessing>,
) -> Result<(), RuleError> {
    combat.pending_last_defeated = Some(body);
    combat.neighbor_work.departure = 0;
    let (position, under) = if (0..6).contains(&body) {
        let warrior = &mut warriors[body as usize];
        if warrior.record().faction()? != 0 {
            combat.victory_work.add_defeated(1);
        }
        if warrior.record().faction()? == 0 && warrior.record().automatic_mode()? < 1 {
            combat.unavailable = combat.unavailable.wrapping_add(1);
        }
        warrior.record_mut().set_active_flag(0);
        let statistic = if warrior.record().stamina()? < -9 {
            Statistic::Deaths
        } else {
            warrior.record_mut().set_bleeding_flag(1);
            Statistic::Knockouts
        };
        CharacterStatisticsMut(&mut warrior.bytes).add(statistic, 1);
        warrior.record_mut().set_defeated_marker(-1);
        warrior.record_mut().set_automatic_mode(0);
        warrior.record_mut().clear_temporary_conditions()?;
        (
            combat.squad.positions[body as usize],
            combat.squad.under_tiles[body as usize],
        )
    } else if body >= 10 {
        let cleanup = super::death_scripts::cleanup_enemy_before_story(combat, warriors, body)?;
        let direct = if cleanup.entry.is_some() {
            if let Some(processing) = processing {
                processing.immediate(combat.ready)
            } else {
                combat.direct_deaths && !combat.death_queue.processing
            }
        } else {
            false
        };
        // Native direct killbody clears targets only after newland returns.
        // The record-only entry point retains its established eager contract.
        if !direct || processing.is_none() {
            super::death_scripts::clear_body_references(combat, warriors, body)?;
        }
        if let Some(entry) = cleanup.entry {
            if direct {
                if combat.direct_death.is_some() {
                    return Err(RuleError::Unsupported {
                        action: "combat.death_story",
                        dependency: "multiple immediate deaths before the caller resumes",
                    });
                }
                combat.direct_death = Some((body, entry));
            } else {
                combat.death_queue.enqueue(entry)?;
            }
        }
        if !direct || processing.is_none() {
            finish_if_party_unavailable(combat, warriors);
        }
        return Ok(());
    } else {
        return Err(RuleError::Invalid("invalid defeated body".into()));
    };
    let tile = centurions_content::mission::OpeningMission::index(
        i32::from(position.0 + combat.map.shift.0 + 5),
        i32::from(position.1 + combat.map.shift.1 + 5),
    )?;
    combat.map.tiles[tile] = under;
    for entry in combat.order.iter_mut().skip(1) {
        if *entry == body {
            *entry = -1;
        }
    }
    // The Undo origin belongs to the active warrior's turn and ends with it.
    if combat.active_body().is_none() {
        combat.turn_origin = None;
    }
    for warrior in &mut *warriors {
        if warrior.record().target_body()? == body {
            warrior.record_mut().set_target_body(-1);
        }
    }
    for monster in &mut combat.enemies.records {
        if monster.record().target_body()? == body {
            monster.record_mut().set_target_body(-1);
        }
    }
    finish_if_party_unavailable(combat, warriors);
    Ok(())
}

pub(crate) fn finish_if_party_unavailable(combat: &mut Combat, warriors: &[Warrior]) {
    if !warriors.iter().any(|warrior| {
        let record = warrior.record();
        record.defeated_marker().unwrap_or(-1) != -1
            && record.stamina().unwrap_or(0) > 0
            && record.faction().unwrap_or(1) == 0
            && record.automatic_mode().unwrap_or(-1) >= 0
    }) {
        combat.victory_work.finish();
    }
}

pub fn can_see(
    combat: &Combat,
    mission: &centurions_content::mission::OpeningMission,
    from: (i16, i16),
    to: (i16, i16),
    delay: i16,
) -> Result<bool, RuleError> {
    let start = (from.0.wrapping_mul(32), from.1.wrapping_mul(32));
    let end = (to.0.wrapping_mul(32), to.1.wrapping_mul(32));
    let divisor = f32::from(delay.wrapping_add(128));
    if divisor == 0.0 {
        return Err(RuleError::Invalid("zero visibility divisor".into()));
    }
    let step = (
        f32::from(end.0.wrapping_sub(start.0)) / divisor,
        f32::from(end.1.wrapping_sub(start.1)) / divisor,
    );
    let mut point = (f32::from(start.0), f32::from(start.1));
    let mut integer = start;
    for _ in 0..128 {
        let index = centurions_content::mission::OpeningMission::index(
            i32::from(integer.0.wrapping_add(16) / 32 + combat.map.shift.0 + 5),
            i32::from(integer.1.wrapping_add(16) / 32 + combat.map.shift.1 + 5),
        )?;
        let tile = combat.map.tiles[index];
        if tile > 999 && mission.tile_word(tile - 1000, 12)? != 0 {
            return Ok(false);
        }
        point.0 += step.0;
        point.1 += step.1;
        integer = (point.0.trunc() as i32 as i16, point.1.trunc() as i32 as i16);
    }
    Ok(true)
}

pub fn destroy_enemy(c: &mut Combat, index: usize) -> Result<(), RuleError> {
    // destroymonster 0x12063c indexes the 100 NPC records 0x439d3c and positions
    // 0x446f0a; records past the battle roster are the retained NPC slots.
    let (p, monster) = if index < c.enemies.records.len() {
        let p = *c
            .enemies
            .positions
            .get(index)
            .ok_or_else(|| RuleError::Invalid("destroyed enemy position unavailable".into()))?;
        (p, c.enemies.records[index].clone())
    } else {
        (c.story_position(index)?, c.story_record(index)?.clone())
    };
    for record in &c.enemies.records {
        centurions_content::span(&record.bytes, 0, 288)?;
    }
    // destroymonster calls bodyfield for every supported footprint, including signed sizes.
    let restored = super::footprint::restore(&c.map, &monster, p)?;
    if monster.record().faction()? != 0 && monster.record().mechanical_state() != 255 {
        c.victory_work.add_defeated(1);
    }
    let mut enemy = c.story_record_mut(index)?.record_mut();
    enemy.set_stamina(0);
    enemy.set_faction(0);
    enemy.set_movement_points(0);
    enemy.clear_conditions();
    c.map = restored;
    for entry in c.order.iter_mut().skip(1) {
        if *entry == index as i16 + 10 {
            *entry = -1;
        }
    }
    for m in &mut c.enemies.records {
        if m.record().target_body()? == index as i16 {
            m.record_mut().set_target_body(-1);
        }
    }
    Ok(())
}
