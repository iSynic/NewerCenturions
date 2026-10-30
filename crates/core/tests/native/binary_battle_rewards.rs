use crate::{native_common, prelude::*, support};

use centurions_core::combat::rewards;

use native_common::patched;
use support::record_bytes as flatten;
use support::warriors as party;

#[test]
fn complete_original_reward_table_cash_drops_and_ordinary_close() {
    let fixture: Value = support::fixture("battle-rewards.json");

    let content = support::content();
    let baseline = fixture.bytes("warriors");
    let cases = fixture.array("cases");

    for case in cases {
        let name = case.text("name");
        let before = patched(baseline.clone(), &case["before_changes"]);
        let enemy_bytes = case.bytes("enemies");
        let enemies: Vec<_> = support::monsters(&enemy_bytes);
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = rewards::prepare(
            &party(&before),
            &enemies,
            &content,
            case.n::<i16>("difficulty"),
            case.n::<i16>("npc_denominator"),
            case.n::<i32>("currency_before"),
            &mut random,
        );
        let result = result.unwrap_or_else(|e| panic!("{name}: {e:?}"));
        let prep = patched(before, &case["prep_changes"]);
        assert_eq!(flatten(&result.warriors), prep, "{name} preparation");
        assert_eq!(digest(&prep), case.text("prep_sha256"), "{name}");
        support::native_expect! {
            scalar result.share => &case["share"], "binary_battle_rewards";
            scalar result.total => &case["total"], "binary_battle_rewards";
            scalar result.currency => &case["currency_after"], "binary_battle_rewards";
        }
        assert_eq!(
            support::record_bytes(&result.monsters),
            patched(enemy_bytes, &case["enemy_changes"]),
            "{name} enemies"
        );
        let mut loot = vec![0; 22000];
        for (i, item) in result.items.iter().enumerate() {
            loot[i * 22..i * 22 + 22].copy_from_slice(&item.0);
        }
        assert_eq!(
            loot,
            patched(vec![0; 22000], &case["crate_changes"]),
            "{name} crate"
        );
        assert_eq!(digest(&loot), case.text("crate_sha256"), "{name}");
        support::assert_rng(&random, &case["seed_after"], &case["rng_draws"]);
        let closed = flatten(&rewards::close_without_promotion(&result.warriors).unwrap());
        assert_eq!(
            closed,
            patched(prep, &case["close_changes"]),
            "{name} Close"
        );
        assert_eq!(digest(&closed), case.text("after_sha256"), "{name}");
    }
}
