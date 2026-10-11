use centurions_core::story_remove_allies::remove_class;

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn removals_match_original_record_copying_and_menu_normalization() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/remove-allies.json"
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let result = remove_class(
            &bytes(case["before"].as_str().unwrap()),
            case["count_before"].as_u64().unwrap() as usize,
            case["class"].as_i64().unwrap() as i16,
        )
        .unwrap();
        assert_eq!(result.count, case["count_after"].as_u64().unwrap() as usize);
        assert_eq!(
            result.records,
            bytes(case["after"].as_str().unwrap()),
            "{case}"
        );
    }
}

#[test]
fn invalid_count_and_short_records_are_diagnostic() {
    assert!(remove_class(&[0; 20 * 288], 21, 121).is_err());
    assert!(remove_class(&[0; 288], 1, 121).is_err());
}
