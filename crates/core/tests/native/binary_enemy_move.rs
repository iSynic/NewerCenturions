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
                bytes combat.roster.records[0].bytes => &case["after"], "enemy case {index}";
                bytes support::word_bytes(&combat
                            .field.tiles) => &case["tiles"], "map case {index}";
            }
            assert_eq!(
                combat.roster.mon_pos[0],
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
    // The fresh seed1 move reads a tutorial rectangle before the tactical map.
    let fixture = support::fixture("enemy-tutorial-edge.json");
    let input: Combat = serde_json::from_value(fixture["input"]["combat"].clone()).unwrap();
    let initial: RandomState = serde_json::from_value(fixture["input"]["random"].clone()).unwrap();
    let expected: Combat = serde_json::from_value(fixture["expected_combat"].clone()).unwrap();
    let expected_random: RandomState =
        serde_json::from_value(fixture["expected_random"].clone()).unwrap();
    let content = content();
    let mission = content.map(1).unwrap();
    let mut combat = input.clone();
    let mut random = initial.clone();
    let movement = move_enemy(&mut combat, mission, 8, &mut random).unwrap();
    assert!(movement.moved);
    assert_eq!(combat, expected);
    assert_eq!(random, expected_random);
    assert_eq!(movement.sounds, fixture.n::<Vec<i16>>("sounds"));
    // Unknown backing and the native control's out-of-map write remain atomic.
    for present in [false, true] {
        let mut mission = mission.clone();
        let tables = std::sync::Arc::make_mut(&mut mission.shared);
        if present {
            tables.tutorial_rectangles.as_mut().unwrap()[15924..15926]
                .copy_from_slice(&1155_i16.to_be_bytes());
        } else {
            tables.tutorial_rectangles = None;
        }
        let mut combat = input.clone();
        let mut random = initial.clone();
        assert!(move_enemy(&mut combat, &mission, 8, &mut random).is_err());
        assert_eq!(combat, input);
        assert_eq!(random, initial);
    }
}
