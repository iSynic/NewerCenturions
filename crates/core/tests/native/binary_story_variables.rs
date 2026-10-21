use crate::{prelude::*, support};

use centurions_core::story_variables::{
    StoryContinuation, party_item_condition, update_variable, variable_predicate,
};

fn continuation(case: &serde_json::Value) -> StoryContinuation {
    if case["local_action_index"] == 99 {
        return StoryContinuation::EndScript;
    }
    let target = case.n::<i16>("target");
    match case.text("continuation") {
        "continue" => StoryContinuation::Continue,
        "jump" => StoryContinuation::ReplaceScript(target),
        "encounter" => StoryContinuation::Encounter(target),
        "typed" => StoryContinuation::TypedEncounter(target),
        value => panic!("unknown original continuation {value}"),
    }
}

#[test]
fn party_item_conditions_match_original_complete_warriors_and_continuations() {
    let fixture: serde_json::Value = support::fixture("item-condition.json");
    for case in fixture.array("cases") {
        let before = case.bytes("before");
        let warriors = support::warriors(&before);
        let extra = case.bytes("extra");
        assert_eq!(
            party_item_condition(&warriors, &extra).unwrap(),
            continuation(case),
            "{extra:?}"
        );
        assert_eq!(support::record_bytes(&warriors), case.bytes("after"));
    }
}

#[test]
fn variable_updates_match_original_complete_bytes_and_continuations() {
    let fixture = support::fixture("story-variables.json");

    for case in fixture.array("update_cases") {
        let before = case.bytes("before");
        let extra = case.bytes("extra");
        let result = update_variable(&before, &extra).unwrap();
        assert_eq!(result.flags, case.bytes("after"), "{extra:?}");
        assert_eq!(result.continuation, continuation(case), "{extra:?}");
        assert_eq!(before, case.bytes("before"));
        assert_eq!(
            case["local_action_index"],
            if matches!(result.continuation, StoryContinuation::ReplaceScript(_)) {
                -1
            } else {
                3
            }
        );
    }
}

#[test]
fn variable_predicates_match_original_signed_comparisons_and_targets() {
    let fixture = support::fixture("story-variables.json");

    for case in fixture.array("predicate_cases") {
        let before = case.bytes("before");
        let extra = case.bytes("extra");
        let result = variable_predicate(&before, &extra).unwrap();
        assert_eq!(
            result.matched,
            case.n::<u64>("comparison") != 0,
            "{extra:?}"
        );
        assert_eq!(result.continuation, continuation(case), "{extra:?}");
        assert_eq!(before, case.bytes("after"));
        assert_eq!(
            case["local_action_index"],
            if matches!(result.continuation, StoryContinuation::ReplaceScript(_)) {
                -1
            } else {
                3
            }
        );
    }
}
