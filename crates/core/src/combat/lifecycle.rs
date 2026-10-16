use super::*;

pub fn kill_body(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    body: i16,
) -> Result<(), RuleError> {
    combat.pending_last_defeated = Some(body);
    let (position, under) = if (0..6).contains(&body) {
        let w = &mut warriors[body as usize].bytes;
        if word(w, 0x10)? == 0 && word(w, 0x90)? < 1 {
            combat.unavailable = combat.unavailable.wrapping_add(1);
        }
        w[0x9df] = 0;
        let (offset, bleeding) = if word(w, 0x50)? < -9 {
            (0xa5e, false)
        } else {
            (0xa62, true)
        };
        if bleeding {
            w[0x9de] = 1;
        }
        let value = i32::from_be_bytes(w[offset..offset + 4].try_into().unwrap()).wrapping_add(1);
        w[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
        put_word(w, 0x20, -1);
        put_word(w, 0x90, 0);
        for offset in (0x5e..0xae).step_by(2) {
            if word(w, offset)? > 0 {
                put_word(w, offset, 0);
            }
        }
        (
            combat.squad.positions[body as usize],
            combat.squad.under_tiles[body as usize],
        )
    } else if body >= 10 {
        let cleanup = super::death_scripts::cleanup_enemy(combat, warriors, body)?;
        if let Some(entry) = cleanup.entry {
            combat.death_queue.enqueue(entry)?;
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
    for warrior in warriors {
        if word(&warrior.bytes, 0x18)? == body {
            put_word(&mut warrior.bytes, 0x18, -1);
        }
    }
    for monster in &mut combat.enemies.records {
        if word(&monster.bytes, 0x10c)? == body {
            put_word(&mut monster.bytes, 0x10c, -1);
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
    let m = &mut c.enemies.records[index].bytes;
    put_word(m, 0x110, 0);
    put_word(m, 0xe, 0);
    m[0x106] = 0;
    m[0xb2..0xda].fill(0);
    c.map = restored;
    for entry in c.order.iter_mut().skip(1) {
        if *entry == index as i16 + 10 {
            *entry = -1;
        }
    }
    for m in &mut c.enemies.records {
        if word(&m.bytes, 0x10c)? == index as i16 {
            put_word(&mut m.bytes, 0x10c, -1);
        }
    }
    Ok(())
}
