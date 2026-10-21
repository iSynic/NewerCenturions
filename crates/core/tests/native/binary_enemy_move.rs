use crate::{native_common, prelude::*, support};

use centurions_core::combat::enemy::move_enemy;

#[test]
fn enemy_movement_matches_original_routine() {
    for name in ["enemy-move.json", "enemy-flee-move.json"] {
        let fixture: serde_json::Value = support::fixture(name);
        let mission = support::terrain_mission(&fixture);
        for (index, case) in fixture.array("cases").iter().enumerate() {
            let mut combat = native_common::enemy_movement_case(case);
            let mut random = RandomState::new(case.n::<u32>("initial_seed"));
            let result = move_enemy(&mut combat, &mission, 0, &mut random).unwrap();
            support::native_expect! {
                bytes combat.enemies.records[0].bytes => &case["after"], "enemy case {index}";
                bytes support::word_bytes(&combat
                            .map.tiles) => &case["tiles"], "map case {index}";
            }
            assert_eq!(
                combat.enemies.positions[0],
                (case.n::<i16>("x"), case.n::<i16>("y")),
                "position case {index}"
            );
            support::native_expect! {
                rng random => (&case["seed"], &case["draws"]), "binary_enemy_move";
                scalar result.moved => &case["moved"], "binary_enemy_move";
                scalar (result.sounds) => &case["sounds"], "binary_enemy_move";
            }
        }
    }
}
