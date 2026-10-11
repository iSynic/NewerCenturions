use centurions_core::{story_move_away, story_registration};

#[test]
fn original_registration_and_caller_match_stable_synthetic_words() {
    let original: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/registration.json")).unwrap();
    let caller: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/registration-caller.json"
    ))
    .unwrap();
    for fixture in [&original, &caller] {
        for case in fixture["cases"].as_array().unwrap() {
            let accepted = story_registration::registration_word_accepted(
                case["synthetic_word"].as_u64().unwrap() as u32,
            );
            assert_eq!(accepted, case["accepted"].as_bool().unwrap());
            assert_eq!(case["rng_draws"], 0);
        }
    }
    for case in caller["cases"].as_array().unwrap() {
        let accepted = case["accepted"].as_bool().unwrap();
        let undone = story_move_away::move_away(5, 50, 0, 1, 61, [0; 6]);
        assert_eq!(case["x_after"], if accepted { 5 } else { undone.x });
        assert_eq!(case["y_after"], if accepted { 50 } else { undone.y });
        assert_eq!(case["messages"], i32::from(!accepted));
        assert_eq!(case["move_calls"], i32::from(!accepted));
        assert_eq!(case["script_before"], case["script_after"]);
    }
}
