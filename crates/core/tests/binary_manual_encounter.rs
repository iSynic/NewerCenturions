use centurions_core::{RandomState, story_manual_encounter::select};

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).unwrap())
        .collect()
}

#[test]
fn manual_selection_matches_original_order_edges_and_randomness() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/manual-encounter.json"
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let mut regions = bytes(case["before"].as_str().unwrap());
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let selected = select(
            &mut regions,
            (
                case["x"].as_i64().unwrap() as i32,
                case["y"].as_i64().unwrap() as i32,
            ),
            &mut random,
        )
        .unwrap();
        assert_eq!(selected, case["script"].as_i64().unwrap() as i16, "{case}");
        assert_eq!(regions, bytes(case["after"].as_str().unwrap()), "{case}");
        assert_eq!(random.seed, case["seed_after"].as_u64().unwrap() as u32);
        assert_eq!(random.draws, case["draws"].as_u64().unwrap());
        assert_eq!(case["saved_regions"], case["after"]);
        assert_eq!(case["last_move_after"], serde_json::json!([0, 0]));
        assert_eq!(
            case["newland_args"],
            serde_json::json!([0, 0, 1, selected, 0])
        );
        assert_eq!(case["manual_during_newland"], 1);
        assert_eq!(case["manual_after"], 0);
    }
    for case in fixture["direct_cases"].as_array().unwrap() {
        assert_eq!(case["mode"], 1);
        assert_eq!(case["load_script"], case["script"]);
        assert_eq!(case["draws"], 0);
        let cancelled = case["ui_response"].as_i64().unwrap() == 0;
        assert_eq!(case["result"], if cancelled { 1 } else { -1 });
        assert_eq!(case["save_calls"], i64::from(cancelled));
        assert_eq!(case["move_calls"], i64::from(cancelled));
        assert_eq!(case["typed_id"], if case["script"] == 0 { 0 } else { 8 });
        assert_eq!(case["record_after"], case["expected_loaded_record"]);
    }
}

#[test]
fn malformed_records_leave_state_and_randomness_unchanged() {
    for length in [0, 643, 645] {
        let mut regions = vec![17; length];
        let before = regions.clone();
        let mut random = RandomState::new(12);
        let initial = random.clone();
        assert!(select(&mut regions, (87, 87), &mut random).is_err());
        assert_eq!(regions, before);
        assert_eq!(random, initial);
    }
}
