use centurions_core::combat::direction_offset;
#[test]
fn direction_offsets_match_bounded_numeric_instruction_cases() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/direction.json")).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let direction = case["direction"].as_i64().unwrap() as i16;
        assert_eq!(
            direction_offset(direction, 1),
            (
                case["x"].as_i64().unwrap() as i16,
                case["y"].as_i64().unwrap() as i16
            ),
            "direction {direction}"
        );
    }
}
