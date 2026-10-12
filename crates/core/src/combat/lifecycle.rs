use super::*;

pub fn kill_body(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    body: i16,
) -> Result<(), RuleError> {
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

#[cfg(test)]
mod source_tests {
    use super::*;

    fn combat(size: i16) -> Combat {
        let mut bytes = vec![0; 288];
        for (offset, value) in [
            (0x10, size),
            (0xe, 1),
            (0x110, 9),
            (0x118, 1118),
            (0x11a, 1119),
            (0x11c, 1120),
            (0x11e, 1121),
            (0x10c, 0),
        ] {
            put_word(&mut bytes, offset, value);
        }
        bytes[0xb2..0xda].fill(3);
        bytes[0x106] = 7;
        let mut order = vec![-1; 110];
        order[0] = 10;
        order[1] = 10;
        order[2] = 10;
        Combat {
            battle: 2,
            map: TacticalMap {
                tiles: vec![77; 8100],
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: vec![],
                under_tiles: vec![],
            },
            enemies: EnemySetup {
                records: vec![MonsterRecord { bytes }],
                positions: vec![(10, 10)],
            },
            order,
            cursor: 1,
            round: 0,
            round_scripts: Default::default(),
            death_queue: Default::default(),
            spell_fields: vec![0; 1200],
            field_used: vec![0; 128],
            attack_packet: None,
            attack_identity: None,
            weapon_work: Default::default(),
            unavailable: 0,
        }
    }

    #[test]
    fn source_destroy_restores_all_signed_footprints_and_keeps_original_cleanup() {
        for size in [-32768_i16, -32767, -4, -3, -2, -1, 0, 1, 2, 3, 4, 32767] {
            let mut state = combat(size);
            destroy_enemy(&mut state, 0).unwrap();
            for (x, y, tile, included) in [
                (14, 14, 1118, size.wrapping_abs() == 3),
                (14, 15, 1119, size.wrapping_abs() > 1),
                (15, 14, 1120, size.wrapping_abs() == 3 || size == 1),
                (15, 15, 1121, true),
            ] {
                assert_eq!(
                    state.map.tiles[x * 90 + y],
                    if included { tile } else { 77 },
                    "size {size}"
                );
            }
            let bytes = &state.enemies.records[0].bytes;
            assert_eq!(word(bytes, 0xe).unwrap(), 0);
            assert_eq!(word(bytes, 0x110).unwrap(), 0);
            assert_eq!(word(bytes, 0x10c).unwrap(), -1);
            assert_eq!(bytes[0x106], 0);
            assert!(bytes[0xb2..0xda].iter().all(|byte| *byte == 0));
            assert_eq!(&state.order[..3], &[10, -1, -1]);
        }
    }

    #[test]
    fn invalid_record_or_position_fails_before_cleanup() {
        let mut state = combat(4);
        state.enemies.records[0].bytes.pop();
        let before = state.clone();
        assert!(destroy_enemy(&mut state, 0).is_err());
        assert_eq!(state, before);
        assert!(destroy_enemy(&mut state, 1).is_err());
        assert_eq!(state, before);
        state = combat(3);
        state.enemies.positions[0] = (-5, -5);
        let before = state.clone();
        assert!(destroy_enemy(&mut state, 0).is_err());
        assert_eq!(state, before);
    }

    #[test]
    fn bodyfield_warrior_branch_restores_anchor_and_rejects_missing_tables() {
        let mut state = combat(0);
        state.squad.positions = vec![(3, 4)];
        state.squad.under_tiles = vec![1234];
        let restored = super::super::footprint::restore_body(&state, 0).unwrap();
        assert_eq!(restored.tiles[8 * 90 + 9], 1234);
        assert_eq!(state.map.tiles[8 * 90 + 9], 77);
        assert!(super::super::footprint::restore_body(&state, -1).is_err());
        assert!(super::super::footprint::restore_body(&state, 1).is_err());
        assert_eq!(
            super::super::footprint::restore_body(&state, 10)
                .unwrap()
                .tiles[15 * 90 + 15],
            1121
        );
    }
}
