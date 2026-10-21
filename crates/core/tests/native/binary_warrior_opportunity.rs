use crate::{prelude::*, support};

use centurions_core::combat::move_warrior;

#[test]
fn warrior_walk_matches_native_factions_guard_departure_order_and_cost() {
    let fixture: serde_json::Value = support::fixture("warrior-opportunity.json");

    let mut content = support::content();
    content.scenario.as_mut().unwrap().maps[0].tile_properties =
        fixture.bytes("properties")[64..12064].to_vec();
    for case in fixture.array("cases") {
        if case.n::<bool>("forced_kill_return") {
            continue;
        }
        let context = format!("layout{} seed{}", case["layout"], case["seed_before"]);
        let mut state = Session::new(&content);
        state.random = RandomState::new(case.n::<u32>("seed_before"));
        state.warriors = support::warriors(&case["warriors_before"]);
        let mut order = vec![-1; 110];
        order[1] = 0;
        state.combat = Some(Combat {
            battle: 1,
            map: TacticalMap {
                tiles: case.words("map_before"),
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: (0..6)
                    .map(|member| {
                        if member == 0 {
                            (20, 20)
                        } else {
                            (40 + member, 40)
                        }
                    })
                    .collect(),
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: support::monsters(&case["records_before"]),
                positions: case
                    .array("enemy_positions")
                    .iter()
                    .map(|point| {
                        (
                            point[0].as_i64().unwrap() as i16,
                            point[1].as_i64().unwrap() as i16,
                        )
                    })
                    .collect(),
            },
            order,
            ..support::battle()
        });
        let events = move_warrior(&mut state, &content, case.n::<i8>("dx"), case.n::<i8>("dy"))
            .unwrap_or_else(|error| panic!("{context}: {error}"));
        let combat = current_combat(&state);
        support::native_expect! {
            records &state.warriors => &case["warriors_after"], &format!("{context} party");
            records &combat.enemies.records => &case["records_after"], &format!("{context} monsters");
        }
        assert_eq!(combat.map.tiles, case.words("map_after"), "{context} map");
        assert_eq!(
            combat.squad.positions[0],
            (case.n::<i16>("x"), case.n::<i16>("y")),
            "{context} position"
        );
        support::native_expect! {
            scalar combat.squad.under_tiles[0] => &case["under"], "binary_warrior_opportunity";
            bytes combat.attack_packet.as_ref().unwrap() => &case["packet"], &format!("{context} packet");
            scalar state.random.seed => &case["seed_after"], "binary_warrior_opportunity";
            scalar state.random.draws => &case["rng_draws"], "binary_warrior_opportunity";
        }
        let sounds: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::Sound { resource_id } => Some(i64::from(*resource_id)),
                _ => None,
            })
            .collect();
        support::expect_array(&(sounds), &case["sounds"]);
        let calls: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::AttackResolved {
                    attacker, target, ..
                } => Some((i64::from(*attacker), i64::from(*target))),
                _ => None,
            })
            .collect();
        assert_eq!(
            calls,
            case.array("calls")
                .iter()
                .map(|v| (v[0].as_i64().unwrap(), v[1].as_i64().unwrap()))
                .collect::<Vec<_>>(),
            "{context} reaction order"
        );
        assert_eq!(combat.cursor, 1);
        assert_eq!(combat.order[1], 0);
        assert_eq!(combat.round, 0);
    }
}

#[test]
fn native_nonzero_reaction_return_advances_before_movement_cost_or_map_writes() {
    let fixture: serde_json::Value = support::fixture("warrior-opportunity.json");
    let cases: Vec<_> = fixture
        .array("cases")
        .iter()
        .filter(|case| case.n::<bool>("forced_kill_return"))
        .collect();

    for case in cases {
        assert_eq!(case["phases"], serde_json::json!([0, 1, 2]));

        let mut expected = case.bytes("warriors_before");
        expected[0x9e0] = 0;
        support::native_expect! {
            bytes &case.bytes("warriors_after") => &expected, "aborted walk facing";
        };

        assert_eq!(
            case.array("calls").last().unwrap(),
            &serde_json::json!([10, 0, 0, 2])
        );
    }
}
