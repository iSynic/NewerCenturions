use crate::prelude::*;

use centurions_core::typed_encounter::{
    PasswordSelection, QuestionPlan, TypedContinuation, attempt_count, continuation,
    finish_attempt, password_label, question_labels, question_plan, select_password,
};

fn fixture() -> serde_json::Value {
    crate::support::fixture("typed-encounter.json")
}

#[test]
fn password_prefix_and_case_selection_match_original_comparison() {
    let fixture = fixture();
    for case in fixture.array("password_cases") {
        let record = case.bytes("record");
        let input = case.bytes("input");
        assert_eq!(
            select_password(&record, &input).unwrap(),
            PasswordSelection::Selected(case.n::<i8>("selector")),
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
}

#[test]
fn typed_branch_and_last_attempt_match_complete_original_script() {
    let fixture = fixture();
    for case in fixture.array("dispatch_cases") {
        let record = case.bytes("record");
        let before = case.bytes("before");
        let after = case.bytes("after");
        let plan = continuation(
            &record,
            case.n::<i8>("selected"),
            case.n::<i16>("remaining"),
        )
        .unwrap();
        let TypedContinuation::Branch { selector, body } = plan else {
            panic!("Expected authored branch");
        };
        crate::support::expect(i64::from(selector), &case["effective_selector"]);
        assert_eq!(&after[8..40], body);
        assert_eq!(&after[..8], &before[..8]);
        assert_eq!(&after[40..], &before[40..]);
    }
    for case in fixture.array("repeat_cases") {
        assert_eq!(
            finish_attempt(case.n::<i16>("before")),
            (case.n::<i16>("after"), case.n::<bool>("repeat"))
        );
    }
}

#[test]
fn question_condition_and_continuation_match_original_blocks() {
    let fixture = fixture();
    for case in fixture.array("question_cases") {
        let extra = case.bytes("extra");
        let plan = question_plan(&extra, case.n::<bool>("answer")).unwrap();
        let (yes, no) = question_labels(&extra).unwrap();
        assert_eq!((yes, no), (185, 186));
        match plan {
            QuestionPlan::Continue => {}
            QuestionPlan::MoveAway => {}
            QuestionPlan::ReplaceScript(id) => {
                crate::support::expect(i64::from(id), &case["loaded"])
            }
            QuestionPlan::MenuBranch(branch) | QuestionPlan::TypedBranch(branch) => {
                let record = case.bytes("branch_record");
                let expected =
                    centurions_core::story::dispatch_menu_branch(&record, branch as i8 + 1)
                        .unwrap();
                assert_eq!(&case.bytes("after")[8..40], expected);
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
}
