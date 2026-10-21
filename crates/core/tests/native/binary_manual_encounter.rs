use crate::{prelude::*, support};

use centurions_core::story_manual_encounter::select;

#[test]
fn manual_selection_matches_original_order_edges_and_randomness() {
    let fixture: serde_json::Value = support::fixture("manual-encounter.json");
    for case in fixture.array("cases") {
        let mut regions = case.bytes("before");
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let selected = select(
            &mut regions,
            (case.n::<i32>("x"), case.n::<i32>("y")),
            &mut random,
        )
        .unwrap();
        support::native_expect! {
            scalar selected => &case["script"], "binary_manual_encounter";
            bytes regions => &case["after"], "{case}";
            rng random => (&case["seed_after"], &case["draws"]), "binary_manual_encounter";
        }
    }
}
