use crate::prelude::*;

use centurions_core::story_remove_allies::remove_class;

#[test]
fn removals_match_original_record_copying_and_menu_normalization() {
    let fixture: serde_json::Value = crate::support::fixture("remove-allies.json");
    for case in fixture.array("cases") {
        let result = remove_class(
            &case.bytes("before"),
            case.n::<usize>("count_before"),
            case.n::<i16>("class"),
        )
        .unwrap();
        crate::support::native_expect! {
        scalar result.count => &case["count_after"], "binary_remove_allies";
        bytes result.records => &case["after"], "{case}";
        }
    }
}
