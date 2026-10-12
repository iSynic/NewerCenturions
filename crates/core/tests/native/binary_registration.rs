use crate::prelude::*;

use centurions_core::{story_move_away, story_registration};

#[test]
fn original_registration_and_caller_match_stable_synthetic_words() {
    let original: serde_json::Value = crate::support::fixture("registration.json");
    let caller: serde_json::Value = crate::support::fixture("registration-caller.json");
    for fixture in [&original, &caller] {
        for case in fixture.array("cases") {
            let accepted =
                story_registration::registration_word_accepted(case.n::<u32>("synthetic_word"));
            crate::support::expect(accepted, &case["accepted"]);
        }
    }
    for case in caller.array("cases") {
        let accepted = case.n::<bool>("accepted");
        let undone = story_move_away::move_away(5, 50, 0, 1, 61, [0; 6]);
        assert_eq!(case["x_after"], if accepted { 5 } else { undone.x });
        assert_eq!(case["y_after"], if accepted { 50 } else { undone.y });
        assert_eq!(case["messages"], i32::from(!accepted));
        assert_eq!(case["move_calls"], i32::from(!accepted));
    }
}
