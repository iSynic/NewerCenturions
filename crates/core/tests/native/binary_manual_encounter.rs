use crate::prelude::*;

use centurions_core::story_manual_encounter::select;

#[test]
fn manual_selection_matches_original_order_edges_and_randomness() {
    let fixture: serde_json::Value = crate::support::fixture("manual-encounter.json");
    for case in fixture.array("cases") {
        let mut regions = case.bytes("before");
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let selected = select(
            &mut regions,
            (case.n::<i32>("x"), case.n::<i32>("y")),
            &mut random,
        )
        .unwrap();
        crate::support::native_expect! {
        scalar selected => &case["script"], "binary_manual_encounter";
        bytes regions => &case["after"], "{case}";
        scalar random.seed => &case["seed_after"], "binary_manual_encounter";
        scalar random.draws => &case["draws"], "binary_manual_encounter";
        }

        assert_eq!(case["last_move_after"], serde_json::json!([0, 0]));
        assert_eq!(
            case["newland_args"],
            serde_json::json!([0, 0, 1, selected, 0])
        );
    }
    for case in fixture.array("direct_cases") {
        let cancelled = case.n::<i64>("ui_response") == 0;
        assert_eq!(case["result"], if cancelled { 1 } else { -1 });
        assert_eq!(case["save_calls"], i64::from(cancelled));
        assert_eq!(case["move_calls"], i64::from(cancelled));
        assert_eq!(case["typed_id"], if case["script"] == 0 { 0 } else { 8 });
    }
}
