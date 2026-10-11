use centurions_content::word;
use centurions_core::typed_encounter::{
    PasswordSelection, QuestionPlan, TypedContinuation, attempt_count, continuation,
    finish_attempt, password_label, question_labels, question_plan, select_password,
};

fn bytes(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).unwrap())
        .collect()
}

fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/typed-encounter.json"
    ))
    .unwrap()
}

#[test]
fn password_prefix_and_case_selection_match_original_comparison() {
    let fixture = fixture();
    for case in fixture["password_cases"].as_array().unwrap() {
        let record = bytes(case["record"].as_str().unwrap());
        let input = bytes(case["input"].as_str().unwrap());
        assert_eq!(
            select_password(&record, &input).unwrap(),
            PasswordSelection::Selected(case["selector"].as_i64().unwrap() as i8),
            "{}",
            String::from_utf8_lossy(&input)
        );
        assert_eq!(
            attempt_count(&record).unwrap(),
            i16::from(record[152] as i8)
        );
        assert!(!password_label(&record).unwrap().is_empty());
        assert_eq!(
            select_password(&record, b"").unwrap(),
            PasswordSelection::Retry
        );
        assert!(select_password(&record, &[0x80]).is_err());
    }
    assert_eq!(fixture["control_cases"][0]["retry"], true);
    assert_eq!(fixture["control_cases"][0]["prompts"], 1);
    assert_eq!(fixture["control_cases"][1]["prompts"], 0);
    assert_eq!(fixture["rng_draws"], 0);
}

#[test]
fn typed_branch_and_last_attempt_match_complete_original_script() {
    let fixture = fixture();
    for case in fixture["dispatch_cases"].as_array().unwrap() {
        let record = bytes(case["record"].as_str().unwrap());
        let before = bytes(case["before"].as_str().unwrap());
        let after = bytes(case["after"].as_str().unwrap());
        let plan = continuation(
            &record,
            case["selected"].as_i64().unwrap() as i8,
            case["remaining"].as_i64().unwrap() as i16,
        )
        .unwrap();
        let TypedContinuation::Branch { selector, body } = plan else {
            panic!("Expected authored branch");
        };
        assert_eq!(
            i64::from(selector),
            case["effective_selector"].as_i64().unwrap()
        );
        assert_eq!(&after[8..40], body);
        assert_eq!(&after[..8], &before[..8]);
        assert_eq!(&after[40..], &before[40..]);
    }
    for case in fixture["repeat_cases"].as_array().unwrap() {
        assert_eq!(
            finish_attempt(case["before"].as_i64().unwrap() as i16),
            (
                case["after"].as_i64().unwrap() as i16,
                case["repeat"].as_bool().unwrap()
            )
        );
    }
}

#[test]
fn question_condition_and_continuation_match_original_blocks() {
    let fixture = fixture();
    for case in fixture["question_cases"].as_array().unwrap() {
        let extra = bytes(case["extra"].as_str().unwrap());
        let plan = question_plan(&extra, case["answer"].as_bool().unwrap()).unwrap();
        let (yes, no) = question_labels(&extra).unwrap();
        assert_eq!((yes, no), (185, 186));
        match plan {
            QuestionPlan::Continue => {
                assert_eq!(case["stop"], "14673c");
                assert_eq!(case["cursor"], 0);
                assert_eq!(case["after"], case["before"]);
            }
            QuestionPlan::MoveAway => {
                assert_eq!(case["moved"], 1);
                assert_eq!(case["stop"], "14694c");
            }
            QuestionPlan::ReplaceScript(id) => {
                assert_eq!(i64::from(id), case["loaded"].as_i64().unwrap())
            }
            QuestionPlan::MenuBranch(branch) | QuestionPlan::TypedBranch(branch) => {
                assert_eq!(case["stop"], "14032e");
                assert_eq!(case["cursor"], 8);
                let record = bytes(case["branch_record"].as_str().unwrap());
                let expected =
                    centurions_core::story::dispatch_menu_branch(&record, branch as i8 + 1)
                        .unwrap();
                assert_eq!(&bytes(case["after"].as_str().unwrap())[8..40], expected);
            }
            QuestionPlan::EndScript => assert_eq!(case["cursor"], 9),
        }
        assert_eq!(word(&extra, 8).unwrap(), 186);
    }
}

#[test]
fn typed_cancel_and_header_persistence_preserve_original_boundaries() {
    let fixture = fixture();
    let record = bytes(fixture["password_cases"][0]["record"].as_str().unwrap());
    assert_eq!(
        continuation(&record, 0, 2).unwrap(),
        TypedContinuation::Cancel
    );
    assert_eq!(
        continuation(&record, -1, 2).unwrap(),
        TypedContinuation::Abort
    );
    assert!(continuation(&record, -4, 2).is_err());
    let cancel = &fixture["cancel_case"];
    assert_eq!(cancel["original"], cancel["restored"]);
    assert_eq!(cancel["original"], cancel["persisted"]);
    assert_eq!(cancel["menu_active"], 0);
    assert_eq!(cancel["save_mode"], 2);
    assert_eq!(cancel["moved"], 1);
    for case in fixture["save_cases"].as_array().unwrap() {
        assert_eq!(case["record"], case["written"]);
        assert_eq!(bytes(case["written"].as_str().unwrap()).len(), 156);
        assert_eq!(
            case["offset"].as_i64().unwrap(),
            case["id"].as_i64().unwrap() * 516
        );
    }
}
