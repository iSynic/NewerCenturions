use crate::native_common::patch_bytes as changes;
use crate::prelude::*;

use crate::native_common::health_event as event;
#[test]
fn outdoor_disease_reduction_matches_original_complete_warriors_revival_counter_and_rng() {
    let fixture: serde_json::Value = crate::support::fixture("outdoor-disease.json");

    crate::binary_outdoor_disease::compare_outdoor_disease_reduction_matches_original_complete_warriors_revival_counter_and_rng(fixture);
}

pub(super) fn compare_outdoor_disease_reduction_matches_original_complete_warriors_revival_counter_and_rng(
    fixture: Value,
) {
    let original = fixture.bytes("warriors");
    let cases = fixture.array("cases");

    for (index, case) in cases.iter().enumerate() {
        let mut before = original.clone();
        changes(&mut before, &case["before_changes"]);
        assert_eq!(digest(&before), case["before_sha256"], "input {index}");
        let warriors: Vec<_> = crate::support::warriors(&before);
        let affected = case.bytes("affected_before");
        let context = HealthContext {
            unavailable: case.n::<i8>("unavailable_before"),
            selected_warrior: 0,
            effect: 77,
            last_body: -7,
            marks_affected: false,
            affected: std::array::from_fn(|member| word(&affected, member * 2).unwrap()),
        };
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let outcome = reduce_outdoors(&warriors, &context, &mut random).unwrap();
        let actual: Vec<_> = crate::support::record_bytes(&outcome.warriors);
        let mut expected = before.clone();
        changes(&mut expected, &case["after_changes"]);
        if let Some(offset) = actual.iter().zip(&expected).position(|(a, b)| a != b) {
            panic!(
                "case {index} full warrior byte {offset}: {} != {}",
                actual[offset], expected[offset]
            );
        }
        assert_eq!(digest(&actual), case["after_sha256"], "output {index}");
        crate::support::native_expect! {
        scalar outcome.context.unavailable => &case["unavailable_after"], "binary_outdoor_disease";
        scalar outcome.context.effect => &case["effect_after"], "binary_outdoor_disease";
        scalar outcome.context.last_body => &case["last_body_after"], "binary_outdoor_disease";
        }
        assert_eq!(outcome.context.affected, context.affected);
        assert_eq!(
            outcome.events.iter().map(event).collect::<Vec<_>>(),
            case.array("events"),
            "events {index}"
        );
        assert_eq!(
            serde_json::to_value(outcome.amounts).unwrap(),
            case["amounts"],
            "amounts {index}"
        );
        crate::support::native_expect! {
        scalar random.seed => &case["seed_after"], "binary_outdoor_disease";
        scalar random.draws => &case["draws"], "binary_outdoor_disease";
        }
    }
}
