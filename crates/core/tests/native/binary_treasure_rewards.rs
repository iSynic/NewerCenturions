use crate::prelude::*;

use crate::support::field_bytes as bytes;
use centurions_core::story_treasure;

#[test]
fn original_treasure_eight_preserves_participation_and_awards_experience() {
    let fixture: Value = crate::support::fixture("treasure-rewards.json");

    let content = crate::support::content();
    let cases = fixture.array("cases");

    for case in cases {
        let before = bytes(case, "before");
        let warriors: Vec<_> = crate::support::warriors(&before);
        let mut random = RandomState {
            seed: case.n::<u32>("seed_before"),
            draws: 0,
        };
        let result = story_treasure::prepare(
            &warriors,
            &bytes(case, "table_before"),
            &content,
            &mut random,
        )
        .unwrap();
        assert_eq!(
            crate::support::record_bytes(&result.warriors),
            bytes(case, "after")
        );
        crate::support::expect(result.share, &case["share"]);
        let mut crate_bytes = vec![0; 22000];
        for (slot, item) in result.items.iter().enumerate() {
            crate_bytes[slot * 22..(slot + 1) * 22].copy_from_slice(&item.0);
        }
        assert_eq!(crate_bytes, bytes(case, "crate_after"));

        crate::support::native_expect! {
        scalar random.seed => &case["seed_after"], "binary_treasure_rewards";
        scalar random.draws => &case["rng_draws"], "binary_treasure_rewards";
        }
    }
}
