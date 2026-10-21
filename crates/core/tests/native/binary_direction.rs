use crate::{prelude::*, support};

use centurions_core::combat::direction_offset;
#[test]
fn direction_offsets_match_bounded_numeric_instruction_cases() {
    let fixture: serde_json::Value = support::fixture("direction.json");
    for case in fixture.array("cases") {
        let direction = case.n::<i16>("direction");
        assert_eq!(
            direction_offset(direction, 1),
            (case.n::<i16>("x"), case.n::<i16>("y")),
            "direction {direction}"
        );
    }
}
