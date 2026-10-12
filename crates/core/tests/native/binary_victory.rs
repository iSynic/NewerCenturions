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
        let expected = case.bytes("after_warriors");
        for (i, (a, b)) in warriors
            .iter()
            .flat_map(|w| w.bytes.iter())
            .zip(&expected)
            .enumerate()
        {
            assert_eq!(
                a,
                b,
                "warrior {} byte {:#x} group {}",
                i / 3138,
                i % 3138,
                case["body"]
            );
        }
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
        let expected = case.bytes("after_warriors");
        for (i, (a, b)) in warriors
            .iter()
            .flat_map(|w| w.bytes.iter())
            .zip(&expected)
            .enumerate()
        {
            assert_eq!(
                a,
                b,
                "warrior {} offset {:#x} seed {}",
                i / 3138,
                i % 3138,
                case["initial_seed"]
            );
        }
        let expected = case.bytes("after_enemy");
        for (i, (a, b)) in enemies
            .iter()
            .flat_map(|m| m.bytes.iter())
            .zip(&expected)
            .enumerate()
        {
            assert_eq!(
                a,
                b,
                "enemy {} offset {:#x} seed {}",
                i / 288,
                i % 288,
                case["initial_seed"]
            );
        }
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
