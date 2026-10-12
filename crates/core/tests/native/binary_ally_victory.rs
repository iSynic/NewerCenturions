use crate::prelude::*;

use centurions_core::combat::allies::synchronize;
use centurions_core::combat::runtime::victory_loot;

#[test]
fn survivor_cleanup_and_rewards_match_original_composite_execution() {
    let content = crate::support::content();
    let fixture: serde_json::Value = crate::support::fixture("ally-victory.json");
    for case in fixture.array("cases") {
        let mut warriors: Vec<_> = crate::support::warriors(&case["warriors_before"]);
        let count = case.n::<usize>("count");
        let mut enemies: Vec<_> = case
            .bytes("enemies_before")
            .as_chunks::<288>()
            .0
            .iter()
            .take(count)
            .map(|b| MonsterRecord { bytes: b.to_vec() })
            .collect();
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = synchronize(
            &case.bytes("menu_before"),
            case.n::<usize>("ally_count"),
            &mut enemies,
            &mut random,
        )
        .unwrap();
        crate::support::native_expect! {
        bytes result.menu => &case["menu_after"], "binary_ally_victory";
        scalar result.count => &case["ally_count_after"], "binary_ally_victory";
        }
        let (share, loot) =
            victory_loot(&mut warriors, &mut enemies, &mut random, Some(&content)).unwrap();
        crate::support::native_expect! {
        scalar share => &case["share"], "binary_ally_victory";
        bytes crate::support::record_bytes(&warriors) => &case["warriors_after"], "binary_ally_victory";
        }
        assert_eq!(
            crate::support::record_bytes(&enemies),
            case.bytes("enemies_after")[..count * 288]
        );
        assert_eq!(
            loot.iter().flat_map(|i| i.0).collect::<Vec<_>>(),
            case.bytes("loot_after")[..loot.len() * 22]
        );
        crate::support::native_expect! {
        scalar random.seed => &case["seed_after"], "binary_ally_victory";
        scalar random.draws => &case["rng_draws"], "binary_ally_victory";
        }
    }
}
