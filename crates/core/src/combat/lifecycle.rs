use crate::RuleError;
use crate::combat::Combat;
use centurions_content::{
    Warrior,
    records::{CharacterStatisticsMut, Statistic},
};

pub fn kill_body(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    body: i16,
) -> Result<(), RuleError> {
    combat.pending_last_defeated = Some(body);
    let (position, under) = if (0..6).contains(&body) {
        let warrior = &mut warriors[body as usize];
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
        let cleanup = super::death_scripts::cleanup_enemy(combat, warriors, body)?;
        if let Some(entry) = cleanup.entry {
            // killbody12e8a4 selects direct newland only outside deferred processing.
            if combat.direct_deaths && !combat.death_queue.processing {
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
    for warrior in warriors {
        if warrior.record().target_body()? == body {
            warrior.record_mut().set_target_body(-1);
        }
    }
    for monster in &mut combat.enemies.records {
        if monster.record().target_body()? == body {
            monster.record_mut().set_target_body(-1);
        }
    }
    Ok(())
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
    let p = *c
        .enemies
        .positions
        .get(index)
        .ok_or_else(|| RuleError::Invalid("destroyed enemy position unavailable".into()))?;
    let monster = c
        .enemies
        .records
        .get(index)
        .ok_or_else(|| RuleError::Invalid("destroyed enemy record unavailable".into()))?;
    for record in &c.enemies.records {
        centurions_content::span(&record.bytes, 0, 288)?;
    }
    // destroymonster calls bodyfield for every supported footprint, including signed sizes.
    let restored = super::footprint::restore(&c.map, monster, p)?;
    let mut enemy = c.enemies.records[index].record_mut();
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
