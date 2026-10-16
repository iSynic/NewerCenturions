use crate::prelude::*;

use centurions_core::{story_move_away, typed_encounter};

#[test]
fn original_move_away_matches_signed_position_and_flags() {
    let fixture: Value = crate::support::fixture("move-away.json");

    let cases = fixture.array("cases");

    for case in cases {
        let caller = case.n::<bool>("caller");
        let answer = case.n::<bool>("answer");
        assert_eq!(case.bytes("working_before"), case.bytes("working_after"));
        assert_eq!(case.bytes("working_before"), case.bytes("header_after"));
        if caller {
            let extra = [0, 1, 0, 0, 0, 0, 0, 239, 0, 238];
            assert_eq!(
                typed_encounter::question_plan(&extra, answer).unwrap(),
                if answer {
                    typed_encounter::QuestionPlan::Continue
                } else {
                    typed_encounter::QuestionPlan::MoveAway
                }
            );

            assert_eq!(case["question_arguments"], serde_json::json!([239, 238]));
        }
        if caller && answer {
            continue;
        }
        let flags: [i8; 6] = case
            .bytes("flags_before")
            .into_iter()
            .map(|v| v as i8)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let result = story_move_away::move_away(
            case.n::<i32>("x"),
            case.n::<i32>("y"),
            case.n::<i8>("dx"),
            case.n::<i8>("dy"),
            case.n::<i16>("floor"),
            flags,
        );
        crate::support::native_expect! {
        scalar result.x => &case["x_after"], "binary_move_away";
        scalar result.y => &case["y_after"], "binary_move_away";
        scalar result.floor => &case["floor_after"], "binary_move_away";
        scalar result.mark => &case["mark_after"], "binary_move_away";
        }
        assert_eq!(
            result.flags.map(|v| v as u8).to_vec(),
            case.bytes("flags_after")
        );
        assert_eq!(
            result.saved_delta.map(|v| v as u8).to_vec(),
            case.bytes("saved_delta")
        );
        assert_eq!(result.update_controls, case["controls"] == 1);
    }
}
