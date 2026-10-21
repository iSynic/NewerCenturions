use crate::{prelude::*, support};

use centurions_core::combat::lifecycle::kill_body;

#[test]
fn defeat_cleanup_matches_original_routine() {
    let fixture: serde_json::Value = support::fixture("lifecycle.json");
    for case in fixture.array("cases") {
        let mut warriors: Vec<Warrior> = support::warriors(&case["warriors"]);
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
        let mut combat = support::battle();
        combat.battle = 2;
        combat.map.tiles = tiles;
        combat.squad.positions = (0..6).map(|i| (20 + i, 20)).collect();
        combat.enemies.records = vec![MonsterRecord {
            bytes: case.bytes("enemy"),
        }];
        combat.enemies.positions = vec![(30, 30)];
        combat.order = order;
        kill_body(&mut combat, &mut warriors, case.n::<i16>("body")).unwrap();
        support::native_expect! {
            records &warriors => &case["after_warriors"], format!("warriors body {}",case["body"]);
            bytes combat.enemies.records[0].bytes => &case["after_enemy"], "binary_lifecycle";
            bytes support::word_bytes(&combat.order) => &case["order"], "binary_lifecycle";
            bytes support::word_bytes(&combat
                        .map.tiles) => &case["map"], "binary_lifecycle";
        }
    }
}
