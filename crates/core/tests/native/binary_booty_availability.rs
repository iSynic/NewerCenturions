use crate::prelude::*;

use crate::support::field_bytes as bytes;
use centurions_core::combat::rewards;

use crate::support::record_bytes as flatten;

#[test]
fn original_booty_recounts_unavailable_warriors_before_normalizing_and_preserves_count_at_close() {
    let fixture: Value = crate::support::fixture("booty-availability.json");

    let content = crate::support::content();
    let cases = fixture.array("cases");

    for (index, case) in cases.iter().enumerate() {
        let before = bytes(case, "before");
        let warriors: Vec<_> = crate::support::warriors(&before);
        let enemies: Vec<_> = crate::support::monsters(&bytes(case, "enemies"));
        let mut random = RandomState {
            seed: case.n::<u32>("seed_before"),
            draws: 0,
        };
        let result = rewards::prepare(&warriors, &enemies, &content, 0, 0, 37, &mut random)
            .unwrap_or_else(|error| panic!("case {index}: {error:?}"));
        crate::support::expect(result.unavailable, &case["unavailable_prepared"]);
        let prepared = flatten(&result.warriors);
        assert_eq!(
            prepared,
            bytes(case, "prepared"),
            "case {index}: preparation"
        );
        assert_eq!(digest(&prepared), case.text("prep_sha256"));
        crate::support::native_expect! {
        scalar result.share => &case["share"], "binary_booty_availability";
        scalar result.total => &case["total"], "binary_booty_availability";
        scalar result.currency => &case["currency_after"], "binary_booty_availability";
        }
        assert!(
            result.items.is_empty(),
            "case {index}: controlled empty loot"
        );
        let native_enemy = bytes(case, "enemies");
        let mut expected_enemy = native_enemy;
        for change in case.array("enemy_changes") {
            expected_enemy[change[0].as_u64().unwrap() as usize] =
                change[1].as_u64().unwrap() as u8;
        }
        assert_eq!(
            crate::support::record_bytes(&result.enemies),
            expected_enemy,
            "case {index}: enemy normalization"
        );
        crate::support::native_expect! {
        scalar random.seed => &case["seed_after"], "binary_booty_availability";
        scalar random.draws => &case["rng_draws"], "binary_booty_availability";
        }
        let closed = flatten(&rewards::close_without_promotion(&result.warriors).unwrap());
        assert_eq!(closed, bytes(case, "after"), "case {index}: Close");
        assert_eq!(digest(&closed), case.text("after_sha256"));
        crate::support::expect(result.unavailable, &case["unavailable_after"]);
    }
}
