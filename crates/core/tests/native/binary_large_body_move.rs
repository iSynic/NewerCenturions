use crate::{prelude::*, support};

use centurions_core::combat::footprint::move_enemy;

#[test]
fn large_body_movement_matches_original_routine() {
    let fixture: serde_json::Value = support::fixture("large-body-move.json");
    let mission = support::terrain_mission(&fixture);
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let dx = case.n::<i16>("dx");
        let dy = case.n::<i16>("dy");
        let tiles = support::words(&case.bytes("before_tiles"));
        let mut combat = support::battle();
        combat.battle = 2;
        combat.field.tiles = tiles;
        combat.squad.pos = vec![(20 + dx * 4, 20 + dy * 4); 6];
        combat.roster.records = vec![MonsterRecord {
            bytes: case.bytes("input"),
        }];
        combat.roster.mon_pos = vec![(20, 20)];
        combat.battle_order = vec![-1, 10];
        let mut random = RandomState::new(case.n::<u32>("initial_seed"));
        let result = move_enemy(&combat, &mission, 0, &mut random).unwrap();
        let combat = &result.combat;
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
            rng random => (&case["seed"], &case["draws"]), "binary_large_body_move";
            scalar result.moved => &case["moved"], "binary_large_body_move";
            scalar (result.sounds) => &case["sounds"], "binary_large_body_move";
        }
    }
}
