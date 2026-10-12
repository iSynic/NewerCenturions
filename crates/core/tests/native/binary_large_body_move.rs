use crate::prelude::*;

use centurions_core::combat::footprint::move_enemy;

#[test]
fn large_body_movement_matches_original_routine() {
    let fixture: serde_json::Value = crate::support::fixture("large-body-move.json");
    let properties = fixture.bytes("properties");
    let mission = OpeningMission {
        tile_properties: properties[64..12064].to_vec(),
        shared: std::sync::Arc::new(centurions_content::mission::StoryTables {
            ..crate::support::stories()
        }),
        ..crate::support::mission()
    };
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let dx = case.n::<i16>("dx");
        let dy = case.n::<i16>("dy");
        let tiles = crate::support::words(&case.bytes("before_tiles"));
        let combat = Combat {
            battle: 2,
            map: TacticalMap {
                tiles,
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: vec![(20 + dx * 4, 20 + dy * 4); 6],
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: vec![MonsterRecord {
                    bytes: case.bytes("input"),
                }],
                positions: vec![(20, 20)],
            },
            order: vec![-1, 10],
            ..crate::support::battle()
        };
        let mut random = RandomState::new(case.n::<u32>("initial_seed"));
        let result = move_enemy(&combat, &mission, 0, &mut random).unwrap();
        let combat = &result.combat;
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
        scalar random.seed => &case["seed"], "binary_large_body_move";
        scalar random.draws => &case["draws"], "binary_large_body_move";
        scalar result.moved => &case["moved"], "binary_large_body_move";
        scalar (result.sounds) => &case["sounds"], "binary_large_body_move";
        }
    }
}
