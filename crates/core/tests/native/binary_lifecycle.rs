use crate::prelude::*;

use centurions_core::combat::lifecycle::kill_body;

#[test]
fn defeat_cleanup_matches_original_routine() {
    let fixture: serde_json::Value = crate::support::fixture("lifecycle.json");
    for case in fixture.array("cases") {
        let mut warriors: Vec<Warrior> = crate::support::warriors(&case["warriors"]);
        let mut tiles = vec![1155; 8100];
        for i in 0..6 {
            tiles[(25 + i) * 90 + 25] = i as i16;
        }
        tiles[35 * 90 + 35] = 10;
        let mut order = vec![-1; 110];
        for i in 0..6 {
            order[i + 1] = i as i16;
        }
        order[7] = 10;
        let mut combat = Combat {
            battle: 2,
            map: TacticalMap {
                tiles,
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: (0..6).map(|i| (20 + i, 20)).collect(),
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: vec![MonsterRecord {
                    bytes: case.bytes("enemy"),
                }],
                positions: vec![(30, 30)],
            },
            order,
            ..crate::support::battle()
        };
        kill_body(&mut combat, &mut warriors, case.n::<i16>("body")).unwrap();
        crate::support::native_expect! {
        bytes crate::support::record_bytes(&warriors) => &case["after_warriors"], format!("warriors body {}",case["body"]);
        bytes combat.enemies.records[0].bytes => &case["after_enemy"], "binary_lifecycle";
        bytes crate::support::word_bytes(&combat.order) => &case["order"], "binary_lifecycle";
        bytes crate::support::word_bytes(&combat
                        .map.tiles) => &case["map"], "binary_lifecycle";
        }
    }
}
