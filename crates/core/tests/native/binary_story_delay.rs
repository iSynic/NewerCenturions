use crate::{prelude::*, support};

#[test]
fn native_tick_delay_uses_zero_default_strict_unsigned_elapsed_and_wrapping_ticks() {
    let fixture: serde_json::Value = support::fixture("story-delay.json");
    let expected = [
        (1, 7, 0, 3, true),
        (75, 7, 0, 77, true),
        (0, 7, 0, 9, true),
        (0, 0, 0, 2, true),
        (75, 7, 1, 77, true),
        (-32768, 7, 2, 2, true),
        (-1, 7, 0, 8, false),
    ];
    let cases = fixture.array("loop_cases");
    assert_eq!(cases.len(), expected.len());
    for (case, (parameter, default, variant, reads, returned)) in cases.iter().zip(expected) {
        assert_eq!(case["parameter"], parameter);
        assert_eq!(case["default_ticks"], default);
        assert_eq!(case["variant"], variant);
        assert_eq!(case["tick_reads"], reads);
        assert_eq!(case["returned"], returned);

        if returned {
            let duration = if parameter == 0 { default } else { parameter };
            assert_eq!(
                centurions_core::story_delay::elapsed_ticks_with_default(parameter, default)
                    .unwrap(),
                (i32::from(duration) as u32) + 1
            );
        } else {
            assert!(centurions_core::story_delay::elapsed_ticks(parameter).is_err());
        }
    }
}
