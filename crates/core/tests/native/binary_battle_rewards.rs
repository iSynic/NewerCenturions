use crate::prelude::*;

use crate::support::field_bytes as bytes;
use centurions_core::combat::rewards;

fn patched(mut input: Vec<u8>, patches: &Value) -> Vec<u8> {
    for patch in patches.as_array().unwrap() {
        input[patch[0].as_u64().unwrap() as usize] = patch[1].as_u64().unwrap() as u8;
    }
    input
}
fn party(input: &[u8]) -> Vec<Warrior> {
    crate::support::warriors(input)
}
use crate::support::record_bytes as flatten;

#[test]
fn complete_original_reward_table_cash_drops_and_ordinary_close() {
    let fixture: Value = crate::support::fixture("battle-rewards.json");

    let content = crate::support::content();
    let baseline = bytes(&fixture, "warriors");
    let cases = fixture.array("cases");

    for case in cases {
        let name = case.text("name");
        let before = patched(baseline.clone(), &case["before_changes"]);
        let enemy_bytes = bytes(case, "enemies");
        let enemies: Vec<_> = crate::support::monsters(&enemy_bytes);
        let mut random = RandomState {
            seed: case.n::<u32>("seed_before"),
            draws: 0,
        };
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
        crate::support::native_expect! {
        scalar result.share => &case["share"], "binary_battle_rewards";
        scalar result.total => &case["total"], "binary_battle_rewards";
        scalar result.currency => &case["currency_after"], "binary_battle_rewards";
        }
        assert_eq!(
            crate::support::record_bytes(&result.enemies),
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
        crate::support::native_expect! {
        scalar random.seed => &case["seed_after"], "binary_battle_rewards";
        scalar random.draws => &case["rng_draws"], "binary_battle_rewards";
        }
        let closed = flatten(&rewards::close_without_promotion(&result.warriors).unwrap());
        assert_eq!(
            closed,
            patched(prep, &case["close_changes"]),
            "{name} Close"
        );
        assert_eq!(digest(&closed), case.text("after_sha256"), "{name}");
    }
}
