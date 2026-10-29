use crate::{prelude::*, support};

use centurions_core::combat::entry_party::initiative;

use support::patch_bytes as changes;
#[test]
fn inactive_party_initiative_matches_original_records_order_and_random_consumption() {
    for name in ["inactive-initiative.json", "partial-initiative.json"] {
        let fixture: serde_json::Value = support::fixture(name);
        let original = fixture.bytes("warriors");
        let enemies: Vec<_> = fixture.n::<Vec<i16>>("enemy_speeds");
        let cases = fixture.array("cases");

        for (index, case) in cases.iter().enumerate() {
            let count = case["party_count"].as_u64().unwrap_or(6) as usize;
            let mut before = original[..count * WARRIOR_BYTES].to_vec();
            changes(&mut before, &case["before_changes"]);
            assert_eq!(digest(&before), case["before_sha256"], "case{index} input");
            let warriors: Vec<_> = support::warriors(&before);
            let mut random = RandomState::new(case.n::<u32>("seed_before"));
            let outcome = initiative(
                &warriors,
                &enemies,
                case.n::<i16>("mode"),
                case.n::<i8>("unavailable"),
                &mut random,
            )
            .unwrap();
            let actual: Vec<_> = support::record_bytes(&outcome.warriors);
            let mut expected = before;
            changes(&mut expected, &case["after_changes"]);
            assert_eq!(actual, expected, "case{index} complete warrior records");
            assert_eq!(digest(&actual), case["after_sha256"], "case{index} output");
            let order: Vec<_> = support::words(&case.bytes("order"));
            assert_eq!(outcome.order, order, "case{index} order");
            support::assert_rng(&random, &case["seed_after"], &case["draws"]);
        }
    }
}

#[test]
fn unavailable_counter_capacity_diagnostics_preserve_party_and_randomness() {
    let fixture: serde_json::Value = support::fixture("inactive-initiative.json");
    let original = fixture.bytes("warriors");
    let warriors: Vec<_> = support::warriors(&original);
    for (counter, mode) in [(-128, 0), (127, 0), (1, 1), (127, 1)] {
        let mut random = RandomState::new(12);
        let before = random.clone();
        let party = warriors.clone();
        assert!(initiative(&party, &[8; 9], mode, counter, &mut random).is_err());
        assert_eq!(party, warriors);
        assert_eq!(random, before);
    }
}
