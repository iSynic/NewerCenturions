use crate::prelude::*;

use centurions_core::combat::*;

use crate::support::positions;
fn origin(c: &serde_json::Value) -> (i16, i16) {
    let b = c.bytes("origin");
    (
        i32::from_be_bytes(b[..4].try_into().unwrap()) as i16,
        i32::from_be_bytes(b[4..].try_into().unwrap()) as i16,
    )
}

#[test]
fn composed_shot_matches_live_original_records_rng_and_next_turn() {
    let f: serde_json::Value = crate::support::fixture("live-firearm.json");

    let content = crate::support::content();
    let before = &f["before"];
    let after = &f["after"];
    let c = &before["combat"];
    let hex = |key: &str| c[key].as_str().unwrap();
    let o = origin(c);
    let mut state = crate::support::mission_session(&content);
    state.phase = Phase::Combat;
    state.warriors = before
        .array("warriors")
        .iter()
        .map(|v| Warrior::read(&decode(v.as_str().unwrap())).unwrap())
        .collect();
    state.random = RandomState::new(before.n::<u32>("random_seed"));
    state.combat = Some(Combat {
        battle: 2,
        map: TacticalMap {
            tiles: words(hex("map")),
            shift: (o.0 - 5, o.1 - 5),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: positions(hex("squad_positions")),
            under_tiles: words(hex("under_tiles")),
        },
        enemies: EnemySetup {
            records: crate::support::monsters(&decode(hex("enemies"))),
            positions: positions(hex("enemy_positions")),
        },
        order: words(hex("order")),
        cursor: words(hex("cursor_enemy_index"))[0] as usize,
        ..crate::support::battle()
    });
    let events = apply(Command::ShootEnemy { body: 12 }, &mut state, &content).unwrap();
    assert!(events.iter().any(|e| matches!(
        e,
        centurions_core::Event::AttackResolved {
            attacker: 3,
            target: 12,
            damage: 2,
            defeated: false
        }
    )));
    crate::support::expect(state.random.seed, &after["random_seed"]);
    for (i, warrior) in state.warriors.iter().enumerate() {
        assert_eq!(
            warrior.bytes,
            decode(after["warriors"][i].as_str().unwrap()),
            "warrior {i}"
        );
    }
    let expected = &after["combat"];
    let c = state.combat.unwrap();
    assert_eq!(
        c.cursor,
        words(expected.text("cursor_enemy_index"))[0] as usize
    );
    crate::support::native_expect! {
    words &(c.order) => &expected["order"], "live_firearm";
    words &(c.map.tiles) => &expected["map"], "live_firearm";
    bytes c.enemies
                .records
                .iter()
                .flat_map(|m| &m.bytes)
                .copied()
                .collect::<Vec<_>>() => &expected["enemies"], "live_firearm";
    words &(c.squad.under_tiles) => &expected["under_tiles"], "live_firearm";
    }
    let after_origin = origin(expected);
    for (actual, key) in [&c.squad.positions, &c.enemies.positions]
        .into_iter()
        .zip(["squad_positions", "enemy_positions"])
    {
        let absolute: Vec<_> = actual.iter().map(|p| (p.0 + o.0, p.1 + o.1)).collect();
        let expected: Vec<_> = positions(expected[key].as_str().unwrap())
            .iter()
            .map(|p| (p.0 + after_origin.0, p.1 + after_origin.1))
            .collect();
        assert_eq!(absolute, expected);
    }
}

#[test]
fn victory_point_fields_match_live_original_five_wasp_rewards() {
    let f: serde_json::Value = crate::support::fixture("live-firearm.json");
    let v = &f["victory"];
    let mut warriors: Vec<_> = v
        .array("warriors")
        .iter()
        .map(|w| Warrior::read(&decode(w.as_str().unwrap())).unwrap())
        .collect();
    let mut enemies: Vec<_> = crate::support::monsters(&v["enemies"]);
    let share =
        runtime::victory_records(&mut warriors, &mut enemies, &mut RandomState::new(12)).unwrap();
    crate::support::expect(share, &v["share"]);
    for (i, w) in warriors.iter().enumerate() {
        let expected = decode(v["after_warriors"][i].as_str().unwrap());
        assert_eq!(
            w.bytes[0x9ce..0x9d6],
            expected[0x9ce..0x9d6],
            "warrior {i} point fields"
        );
    }
}
