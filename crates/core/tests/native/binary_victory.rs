use crate::prelude::*;

use centurions_core::combat::runtime::victory_records;

#[test]
fn victory_rewards_match_original_branch() {
    let fixture: serde_json::Value = crate::support::fixture("victory.json");
    for case in fixture.array("cases") {
        let mut warriors: Vec<Warrior> = crate::support::warriors(&case["warriors"]);
        let mut enemies = vec![
            MonsterRecord {
                bytes: case.bytes("enemy")
            };
            case.n::<usize>("body")
        ];
        let mut random = RandomState::new(12);
        victory_records(&mut warriors, &mut enemies, &mut random).unwrap();
        crate::support::assert_bytes(
            &crate::support::record_bytes(&warriors),
            &case.bytes("after_warriors"),
            "native victory records",
        );
        crate::support::native_expect! {
        bytes enemies[0].bytes => &case["after_enemy"], "binary_victory";
        scalar random.seed => &case["seed"], "binary_victory";
        scalar random.draws => &case["draws"], "binary_victory";
        }
    }
}

#[test]
fn army_rewards_and_sorted_loot_match_original_full_records_and_randomness() {
    let content = crate::support::content();
    let fixture: serde_json::Value = crate::support::fixture("army-victory.json");
    for case in fixture.array("cases") {
        let mut warriors: Vec<_> = crate::support::warriors(&case["warriors"]);
        let mut enemies: Vec<_> = crate::support::monsters(&case["enemy"]);
        let mut random = RandomState::new(case.n::<u32>("initial_seed"));
        let (_, loot) = centurions_core::combat::runtime::victory_loot(
            &mut warriors,
            &mut enemies,
            &mut random,
            Some(&content),
        )
        .unwrap();
        crate::support::assert_bytes(
            &crate::support::record_bytes(&warriors),
            &case.bytes("after_warriors"),
            "native victory records",
        );
        crate::support::assert_bytes(
            &crate::support::record_bytes(&enemies),
            &case.bytes("after_enemy"),
            "native victory records",
        );
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
        crate::support::native_expect! {
        scalar random.seed => &case["seed"], "binary_victory";
        scalar random.draws => &case["draws"], "binary_victory";
        }
    }
}
