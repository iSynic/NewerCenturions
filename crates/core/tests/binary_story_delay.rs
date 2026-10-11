#[test]
fn original_delay_dispatch_preserves_rules_and_advances_to_next_script_slot() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/story-delay.json")).unwrap();
    let cases = fixture["caller_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 7);
    for case in cases {
        assert_eq!(case["parameter"], case["delay_argument"]);
        assert_eq!(
            case["next_slot"].as_i64().unwrap(),
            case["slot"].as_i64().unwrap() + 1
        );
        assert_eq!(case["rng_draws"], 0);
        assert_eq!(case["rules_unchanged"], true);
    }
    let original = cases.last().unwrap();
    assert_eq!(original["slot"], 6);
    assert_eq!(original["parameter"], 75);
    assert_eq!(original["next_opcode"], 39);
    assert_eq!(original["next_parameter"], 89);
}

#[test]
fn native_tick_delay_uses_zero_default_strict_unsigned_elapsed_and_wrapping_ticks() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/story-delay.json")).unwrap();
    let expected = [
        (1, 7, 0, 3, true),
        (75, 7, 0, 77, true),
        (0, 7, 0, 9, true),
        (0, 0, 0, 2, true),
        (75, 7, 1, 77, true),
        (-32768, 7, 2, 2, true),
        (-1, 7, 0, 8, false),
    ];
    let cases = fixture["loop_cases"].as_array().unwrap();
    assert_eq!(cases.len(), expected.len());
    for (case, (parameter, default, variant, reads, returned)) in cases.iter().zip(expected) {
        assert_eq!(case["parameter"], parameter);
        assert_eq!(case["default_ticks"], default);
        assert_eq!(case["variant"], variant);
        assert_eq!(case["tick_reads"], reads);
        assert_eq!(case["returned"], returned);
        assert_eq!(case["rng_draws"], 0);
        if parameter > 0 {
            assert_eq!(
                centurions_core::story_delay::elapsed_ticks(parameter).unwrap(),
                reads - 1
            );
        } else {
            assert!(centurions_core::story_delay::elapsed_ticks(parameter).is_err());
        }
    }
}
