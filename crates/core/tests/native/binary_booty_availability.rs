use crate::{prelude::*, support};

use centurions_core::combat::rewards;

use support::record_bytes as flatten;

#[test]
fn original_booty_recounts_unavailable_warriors_before_normalizing_and_preserves_count_at_close() {
    let fixture: Value = support::fixture("booty-availability.json");

    let content = support::content();
    let cases = fixture.array("cases");

    for (index, case) in cases.iter().enumerate() {
        let before = case.bytes("before");
        let warriors: Vec<_> = support::warriors(&before);
        let enemies: Vec<_> = support::monsters(&case.bytes("enemies"));
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = rewards::prepare(&warriors, &enemies, &content, 0, 0, 37, &mut random)
            .unwrap_or_else(|error| panic!("case {index}: {error:?}"));
        support::expect(result.kill_party, &case["unavailable_prepared"]);
        let prepared = flatten(&result.warriors);
        assert_eq!(
            prepared,
            case.bytes("prepared"),
            "case {index}: preparation"
        );
        assert_eq!(digest(&prepared), case.text("prep_sha256"));
        support::native_expect! {
            scalar result.share => &case["share"], "binary_booty_availability";
            scalar result.total => &case["total"], "binary_booty_availability";
            scalar result.currency => &case["currency_after"], "binary_booty_availability";
        }
        assert!(
            result.items.is_empty(),
            "case {index}: controlled empty loot"
        );
        let native_enemy = case.bytes("enemies");
        let mut expected_enemy = native_enemy;
        for change in case.array("enemy_changes") {
            expected_enemy[change[0].as_u64().unwrap() as usize] =
                change[1].as_u64().unwrap() as u8;
        }
        assert_eq!(
            support::record_bytes(&result.monsters),
            expected_enemy,
            "case {index}: enemy normalization"
        );
        support::assert_rng(&random, &case["seed_after"], &case["rng_draws"]);
        let closed = flatten(&rewards::close_without_promotion(&result.warriors).unwrap());
        assert_eq!(closed, case.bytes("after"), "case {index}: Close");
        assert_eq!(digest(&closed), case.text("after_sha256"));
        support::expect(result.kill_party, &case["unavailable_after"]);
    }
}
