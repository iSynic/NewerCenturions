use crate::{prelude::*, support};

use centurions_core::combat::runtime::victory_records;

#[test]
fn victory_rewards_match_original_branch() {
    let fixture: serde_json::Value = support::fixture("victory.json");
    for case in fixture.array("cases") {
        let mut warriors: Vec<Warrior> = support::warriors(&case["warriors"]);
        let mut enemies = vec![
            MonsterRecord {
                bytes: case.bytes("enemy")
            };
            case.n::<usize>("body")
        ];
        let mut random = RandomState::new(12);
        victory_records(&mut warriors, &mut enemies, &mut random).unwrap();
        support::native_expect! {
            bytes &support::record_bytes(&warriors) => &case.bytes("after_warriors"), "native victory records";
            bytes enemies[0].bytes => &case["after_enemy"], "binary_victory";
            rng random => (&case["seed"], &case["draws"]), "binary_victory";
        }
    }
}

#[test]
fn army_rewards_and_sorted_loot_match_original_full_records_and_randomness() {
    let content = support::content();
    let fixture: serde_json::Value = support::fixture("army-victory.json");
    for case in fixture.array("cases") {
        let mut warriors: Vec<_> = support::warriors(&case["warriors"]);
        let mut enemies: Vec<_> = support::monsters(&case["enemy"]);
        let mut random = RandomState::new(case.n::<u32>("initial_seed"));
        let (_, loot) = centurions_core::combat::runtime::victory_loot(
            &mut warriors,
            &mut enemies,
            &mut random,
            Some(&content),
        )
        .unwrap();
        support::native_expect! {
            bytes &support::record_bytes(&warriors) => &case.bytes("after_warriors"), "native victory records";
            bytes &support::record_bytes(&enemies) => &case.bytes("after_enemy"), "native victory records";
        };
        let expected: Vec<_> = case
            .bytes("loot")
            .as_chunks::<22>()
            .0
            .iter()
            .filter(|b| b[..2] != [0, 0])
            .map(|b| b.to_vec())
            .collect();
        assert_eq!(
            loot.iter().map(|i| i.0.to_vec()).collect::<Vec<_>>(),
            expected,
            "loot seed {}",
            case["initial_seed"]
        );
        support::assert_rng(&random, &case["seed"], &case["draws"]);
    }
}
