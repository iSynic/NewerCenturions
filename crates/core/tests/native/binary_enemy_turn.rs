use crate::{native_common, prelude::*, support};

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
    let mission = support::terrain_mission(&fixture);

    let content = support::content();
    let mut mission = mission;
    std::sync::Arc::make_mut(&mut mission.shared).combat = content.map(0).unwrap().combat.clone();
    std::sync::Arc::make_mut(&mut mission.shared).areas = content.map(0).unwrap().areas.clone();
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let dx = case.n::<i16>("dx");
        let dy = case.n::<i16>("dy");
        let mut combat = native_common::enemy_movement_case(case);
        let mut random = RandomState::new(case.n::<u32>("initial_seed"));
        let bytes = case.bytes("warriors");
        let mut warriors: Vec<centurions_content::Warrior> = support::warriors(&bytes);
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
        support::native_expect! {
            bytes &support::record_bytes(&warriors) => &case.bytes("after_warriors"), &format!("enemy turn case{index}");
            bytes combat.enemies.records[0].bytes => &case["after"], "enemy case {index}";
            bytes support::word_bytes(&combat
                        .map.tiles) => &case["tiles"], "map case {index}";
        }
        assert_eq!(
            combat.enemies.positions[0],
            (case.n::<i16>("x"), case.n::<i16>("y")),
            "position case {index}"
        );
        support::assert_rng(&random, &case["seed"], &case["draws"]);
    }
}
