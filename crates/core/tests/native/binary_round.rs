use crate::prelude::*;

#[test]
fn round_transition_matches_original_reduce_and_heal() {
    verify(include_str!("../../../../evidence/fixtures/round.json"));
}

#[test]
fn bleeding_round_matches_original_records_and_death_randomness() {
    verify(include_str!(
        "../../../../evidence/fixtures/bleeding-round.json"
    ));
}

fn verify(input: &str) {
    let fixture: serde_json::Value = serde_json::from_str(input).unwrap();
    for case in fixture.array("cases") {
        let mut warriors: Vec<_> = crate::support::warriors(&case["warriors"]);
        let mut enemies = vec![MonsterRecord {
            bytes: case.bytes("enemy"),
        }];
        let mut random = RandomState::new(1);
        let mut combat = centurions_core::combat::Combat {
            battle: 2,
            map: centurions_core::combat::TacticalMap {
                tiles: vec![1155; 8100],
                shift: (0, 0),
                edge_word: 0,
            },
            squad: centurions_core::combat::SquadPlacement {
                positions: vec![(20, 20); 6],
                under_tiles: vec![1155; 6],
            },
            enemies: centurions_core::combat::EnemySetup {
                records: enemies.clone(),
                positions: vec![(20, 20)],
            },
            cursor: 0,
            round: 1,
            ..crate::support::battle()
        };
        let events =
            centurions_core::combat::runtime::reduce_round(&mut combat, &mut warriors, &mut random)
                .unwrap();
        if let Some(sounds) = case.get("bleeding_sounds") {
            assert_eq!(
                events,
                sounds
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|id| centurions_core::Event::Sound {
                        resource_id: id.as_i64().unwrap() as i16
                    })
                    .collect::<Vec<_>>()
            );
        }
        enemies = combat.enemies.records;
        initialize_turn(
            case.n::<i16>("body"),
            &mut warriors,
            &mut enemies,
            &[12, 14, 12, 14, 10, 12, 10, 10, 14, 16, 0, 12],
            &mut random,
        )
        .unwrap();
        crate::support::native_expect! {
        bytes crate::support::record_bytes(&warriors) => &case["after_warriors"], format!("body {}",case["body"]);
        bytes enemies[0].bytes => &case["after_enemy"], "binary_round";
        scalar random.seed => &case["seed"], "binary_round";
        scalar random.draws => &case["draws"], "binary_round";
        }
    }
}
