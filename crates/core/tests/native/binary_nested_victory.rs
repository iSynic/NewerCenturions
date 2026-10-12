use crate::prelude::*;

use centurions_core::combat::runtime::victory_loot;

#[test]
fn nested_battle_rewards_consume_count_once_and_preserve_backing_records() {
    let fixture: serde_json::Value = crate::support::fixture("nested-victory.json");
    for case in fixture.array("cases") {
        let mut enemies: Vec<_> = crate::support::monsters(&case["records_before"]);
        let mut warriors: Vec<_> =
            crate::support::warriors(&bytes(&case["calls"][0]["party_before"]));
        let mut count = case.n::<usize>("count");
        let mut random = RandomState::new(case.n::<u32>("setup_seed_after"));
        for call in case.array("calls") {
            let before_draws = random.draws;
            crate::support::native_expect! {
            scalar random.seed => &call["seed_before"], "binary_nested_victory";
            bytes warriors
                                .iter()
                                .flat_map(|warrior| &warrior.bytes)
                                .copied()
                                .collect::<Vec<_>>() => &call["party_before"], "binary_nested_victory";
            }
            let (share, loot) =
                victory_loot(&mut warriors, &mut enemies[..count], &mut random, None).unwrap();
            crate::support::expect(share, &call["share"]);
            assert!(loot.is_empty());
            assert!(call.bytes("loot_after").iter().all(|value| *value == 0));
            crate::support::native_expect! {
            bytes enemies
                                .iter()
                                .flat_map(|enemy| &enemy.bytes)
                                .copied()
                                .collect::<Vec<_>>() => &call["records_after"], "binary_nested_victory";
            bytes warriors
                                .iter()
                                .flat_map(|warrior| &warrior.bytes)
                                .copied()
                                .collect::<Vec<_>>() => &call["party_after"], "binary_nested_victory";
            scalar random.seed => &call["seed_after"], "binary_nested_victory";
            scalar random.draws - before_draws => &call["rng_draws"], "binary_nested_victory";
            }
            count = call.n::<usize>("enemy_count_after");
        }
        assert_eq!(count, 0);
        assert_eq!(enemies[0].bytes[0..2], 7_i16.to_be_bytes());
    }
}
