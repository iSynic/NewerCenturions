use centurions_core::{story_move_away, typed_encounter};
use serde_json::Value;

fn bytes(value: &Value, name: &str) -> Vec<u8> {
    let text = value[name].as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn original_move_away_matches_signed_position_and_flags() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/move-away.json")).unwrap();
    assert_eq!(fixture["draws"], 0);
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 55);
    for case in cases {
        let caller = case["caller"].as_bool().unwrap();
        let answer = case["answer"].as_bool().unwrap();
        assert_eq!(bytes(case, "working_before"), bytes(case, "working_after"));
        assert_eq!(bytes(case, "working_before"), bytes(case, "header_after"));
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
            assert_eq!(case["extra_loads"], 1);
            assert_eq!(case["question_arguments"], serde_json::json!([239, 238]));
            assert_eq!(case["cursor_after"], 2);
            assert_eq!(case["return"], 1);
        }
        if caller && answer {
            assert_eq!(case["x"], case["x_after"]);
            assert_eq!(case["y"], case["y_after"]);
            assert_eq!(case["flags_before"], case["flags_after"]);
            assert_eq!(case["movement_calls"], 0);
            continue;
        }
        let flags: [i8; 6] = bytes(case, "flags_before")
            .into_iter()
            .map(|v| v as i8)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let result = story_move_away::move_away(
            case["x"].as_i64().unwrap() as i32,
            case["y"].as_i64().unwrap() as i32,
            case["dx"].as_i64().unwrap() as i8,
            case["dy"].as_i64().unwrap() as i8,
            case["floor"].as_i64().unwrap() as i16,
            flags,
        );
        assert_eq!(result.x, case["x_after"].as_i64().unwrap() as i32);
        assert_eq!(result.y, case["y_after"].as_i64().unwrap() as i32);
        assert_eq!(result.floor, case["floor_after"].as_i64().unwrap() as i16);
        assert_eq!(result.mark, case["mark_after"].as_i64().unwrap() as i8);
        assert_eq!(
            result.flags.map(|v| v as u8).to_vec(),
            bytes(case, "flags_after")
        );
        assert_eq!(
            result.saved_delta.map(|v| v as u8).to_vec(),
            bytes(case, "saved_delta")
        );
        assert_eq!(result.update_controls, case["controls"] == 1);
        assert_eq!(case["center"], 1);
        assert_eq!(case["movement_calls"], 1);
    }
}
