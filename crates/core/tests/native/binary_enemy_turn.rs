use crate::prelude::*;

use centurions_core::combat::runtime::enemy_action;

#[test]
fn enemy_turn_matches_original_decision_loop() {
    verify(false);
}

#[test]
fn chemical_enemy_turn_matches_original_decision_loop() {
    verify(true);
}

fn verify(ranged: bool) {
    let fixture: serde_json::Value = serde_json::from_str(if ranged {
        include_str!("../../../../evidence/fixtures/enemy-ranged-turn.json")
    } else {
        include_str!("../../../../evidence/fixtures/enemy-turn.json")
    })
    .unwrap();
    let properties = fixture.bytes("properties");
    let mission = OpeningMission {
        tile_properties: properties[64..12064].to_vec(),
        shared: std::sync::Arc::new(centurions_content::mission::StoryTables {
            ..crate::support::stories()
        }),
        ..crate::support::mission()
    };

    let content = crate::support::content();
    let mut mission = mission;
    std::sync::Arc::make_mut(&mut mission.shared).combat = content.map(0).unwrap().combat.clone();
    std::sync::Arc::make_mut(&mut mission.shared).areas = content.map(0).unwrap().areas.clone();
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
        let bytes = case.bytes("warriors");
        let mut warriors: Vec<centurions_content::Warrior> = crate::support::warriors(&bytes);
        for i in 1..6 {
            combat.squad.positions[i] = if ranged {
                (20 + dx * 4 + i as i16, 20 + dy * 4)
            } else {
                (40 + i as i16, 40)
            };
            if ranged {
                combat.map.tiles[((25 + dx * 4 + i as i16) * 90 + 25 + dy * 4) as usize] = i as i16;
            }
        }
        let _result = enemy_action(
            &mut combat,
            &mut warriors,
            &mission,
            &mut random,
            &content.items,
        )
        .unwrap();
        crate::support::assert_bytes(
            &crate::support::record_bytes(&warriors),
            &case.bytes("after_warriors"),
            &format!("enemy turn case{index}"),
        );
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
        scalar random.seed => &case["seed"], "binary_enemy_turn";
        scalar random.draws => &case["draws"], "binary_enemy_turn";
        }
    }
}
