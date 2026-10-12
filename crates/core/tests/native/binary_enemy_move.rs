use crate::prelude::*;

use centurions_core::combat::enemy::move_enemy;

#[test]
fn enemy_movement_matches_original_routine() {
    let fixture: serde_json::Value = crate::support::fixture("enemy-move.json");
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
        let layout = case.n::<i64>("layout");
        let mut tiles = vec![1155; 8100];
        if layout == 1 || layout == 3 {
            tiles[((25 + dx) * 90 + 25 + dy) as usize] = if layout == 3 { 1000 } else { 1061 };
        }
        if layout == 2 {
            for x in 24..=26 {
                for y in 24..=26 {
                    tiles[x * 90 + y] = 1061;
                }
            }
        }
        tiles[25 * 90 + 25] = 10;
        tiles[((25 + dx * 4) * 90 + 25 + dy * 4) as usize] = 0;
        let mut combat = Combat {
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
