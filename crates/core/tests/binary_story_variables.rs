use centurions_content::Warrior;
use centurions_core::story_variables::{
    StoryContinuation, party_item_condition, update_variable, variable_predicate,
};

fn bytes(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/story-variables.json"
    ))
    .unwrap()
}

fn continuation(case: &serde_json::Value) -> StoryContinuation {
    if case["local_action_index"] == 99 {
        return StoryContinuation::EndScript;
    }
    let target = case["target"].as_i64().unwrap() as i16;
    match case["continuation"].as_str().unwrap() {
        "continue" => StoryContinuation::Continue,
        "jump" => StoryContinuation::ReplaceScript(target),
        "encounter" => StoryContinuation::Encounter(target),
        "typed" => StoryContinuation::TypedEncounter(target),
        value => panic!("unknown original continuation {value}"),
    }
}

#[test]
fn party_item_conditions_match_original_complete_warriors_and_continuations() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/item-condition.json"
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let before = bytes(case["before"].as_str().unwrap());
        let warriors = before
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|record| Warrior::read(record).unwrap())
            .collect::<Vec<_>>();
        let extra = bytes(case["extra"].as_str().unwrap());
        assert_eq!(
            party_item_condition(&warriors, &extra).unwrap(),
            continuation(case),
            "{extra:?}"
        );
        assert_eq!(
            warriors
                .iter()
                .flat_map(|warrior| warrior.bytes.iter().copied())
                .collect::<Vec<_>>(),
            bytes(case["after"].as_str().unwrap())
        );
    }
}

#[test]
fn variable_updates_match_original_complete_bytes_and_continuations() {
    let fixture = fixture();
    assert_eq!(fixture["rng_draws"], 0);
    for case in fixture["update_cases"].as_array().unwrap() {
        let before = bytes(case["before"].as_str().unwrap());
        let extra = bytes(case["extra"].as_str().unwrap());
        let result = update_variable(&before, &extra).unwrap();
        assert_eq!(
            result.flags,
            bytes(case["after"].as_str().unwrap()),
            "{extra:?}"
        );
        assert_eq!(result.continuation, continuation(case), "{extra:?}");
        assert_eq!(before, bytes(case["before"].as_str().unwrap()));
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
    let fixture = fixture();
    assert_eq!(fixture["rng_draws"], 0);
    for case in fixture["predicate_cases"].as_array().unwrap() {
        let before = bytes(case["before"].as_str().unwrap());
        let extra = bytes(case["extra"].as_str().unwrap());
        let result = variable_predicate(&before, &extra).unwrap();
        assert_eq!(
            result.matched,
            case["comparison"].as_u64().unwrap() != 0,
            "{extra:?}"
        );
        assert_eq!(result.continuation, continuation(case), "{extra:?}");
        assert_eq!(before, bytes(case["after"].as_str().unwrap()));
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
