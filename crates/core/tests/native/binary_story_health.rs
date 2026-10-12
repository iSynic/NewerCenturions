use crate::prelude::*;

use crate::native_common::patch_bytes as changes;
use centurions_core::story_health::heal_outdoors;
use centurions_core::story_health::party_health;

use crate::native_common::health_event as event;

#[test]
fn outdoor_heal_and_party_callers_match_original_records_effect_order_and_rng() {
    let fixture: serde_json::Value = crate::support::fixture("story-health.json");
    let original = fixture.bytes("warriors");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let mut before = original.clone();
        changes(&mut before, &case["before_changes"]);
        assert_eq!(digest(&before), case["before_sha256"], "input {index}");
        let warriors: Vec<_> = before
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|raw| Warrior {
                name: "fixture".into(),
                original_sha256: String::new(),
                bytes: raw.to_vec(),
            })
            .collect();
        let affected = case.bytes("affected_before");
        let context = HealthContext {
            unavailable: case.n::<i8>("unavailable_before"),
            selected_warrior: case.n::<usize>("member"),
            effect: 77,
            last_body: -7,
            marks_affected: case.n::<bool>("marks_affected"),
            affected: std::array::from_fn(|member| word(&affected, member * 2).unwrap()),
        };
        let mask_bytes = case.bytes("mask");
        let mask = std::array::from_fn(|member| word(&mask_bytes, member * 2).unwrap());
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let outcome = if case["kind"] == "heal" {
            heal_outdoors(
                &warriors,
                context.selected_warrior,
                case.n::<i16>("amount"),
                case.n::<bool>("animate"),
                &context,
                &mut random,
            )
        } else {
            party_health(
                &warriors,
                &case.bytes("extra"),
                case["kind"] == "all",
                &mask,
                &context,
                &mut random,
            )
        }
        .unwrap();
        let actual: Vec<_> = crate::support::record_bytes(&outcome.warriors);
        let mut expected = before.clone();
        changes(&mut expected, &case["after_changes"]);
        assert_eq!(actual, expected, "full warrior state {index}");
        assert_eq!(digest(&actual), case["after_sha256"], "output {index}");
        crate::support::native_expect! {
        scalar outcome.context.unavailable => &case["unavailable_after"], "binary_story_health";
        scalar outcome.context.effect => &case["effect_after"], "binary_story_health";
        scalar outcome.context.last_body => &case["last_body_after"], "binary_story_health";
        }
        let actual_affected: Vec<_> = crate::support::word_bytes(&outcome.context.affected);
        assert_eq!(
            actual_affected,
            case.bytes("affected_after"),
            "affected {index}"
        );
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
        scalar random.seed => &case["seed_after"], "binary_story_health";
        scalar random.draws => &case["draws"], "binary_story_health";
        }
    }
}
