use crate::prelude::*;

use centurions_core::combat::enemy::move_enemy;

#[test]
fn enemy_movement_matches_original_routine() {
    for name in ["enemy-move.json", "enemy-flee-move.json"] {
        let fixture: serde_json::Value = crate::support::fixture(name);
        let mission = crate::support::terrain_mission(&fixture);
        for (index, case) in fixture.array("cases").iter().enumerate() {
            let mut combat = crate::native_common::enemy_movement_case(case);
            let mut random = RandomState::new(case.n::<u32>("initial_seed"));
            let result = move_enemy(&mut combat, &mission, 0, &mut random).unwrap();
            crate::support::native_expect! {
            bytes combat.enemies.records[0].bytes => &case["after"], "enemy case {index}";
            bytes crate::support::word_bytes(&combat
                            .map.tiles) => &case["tiles"], "map case {index}";
            }
            assert_eq!(
                combat.enemies.positions[0],
                (case.n::<i16>("x"), case.n::<i16>("y")),
                "position case {index}"
            );
            crate::support::native_expect! {
            scalar random.seed => &case["seed"], "binary_enemy_move";
            scalar random.draws => &case["draws"], "binary_enemy_move";
            scalar result.moved => &case["moved"], "binary_enemy_move";
            scalar (result.sounds) => &case["sounds"], "binary_enemy_move";
            }
        }
    }
}
