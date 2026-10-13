use crate::prelude::*;

use centurions_core::story_item_use::use_scenario_item;

fn fixture() -> Value {
    crate::support::fixture("item-use.json")
}
use crate::support::warriors;

fn content(definition: Vec<u8>) -> Content {
    let mut items = vec![vec![0; 150]; 833];
    items[832] = definition;
    Content {
        tutorials: vec![],
        identity: String::from("item-use fixture"),
        warriors: vec![],
        items,
        resources: vec![],
        original_services: None,
        scenario: None,
    }
}
use crate::support::assert_bytes;

#[test]
fn scenario_item_use_matches_original_complete_party_ordered_costs_and_returns() {
    let fixture = fixture();
    let movement = fixture
        .array("race_movements")
        .iter()
        .map(|value| value.as_i64().unwrap() as i8)
        .collect::<Vec<_>>();
    for case in fixture.array("cases") {
        let before = warriors(&case["before"]);
        let content = content(case.bytes("definition"));
        let result = use_scenario_item(
            &before,
            case.n::<usize>("member"),
            case.n::<usize>("slot"),
            &content,
            &movement,
        )
        .unwrap();
        let actual = crate::support::record_bytes(&result.warriors);
        crate::support::native_expect! {
        bytes actual => &case["after"], &format!("member {} variant {}", case["member"], case["variant"]);
        scalar result.return_code => &case["return_code"], "binary_story_item_use";
        }
        assert_eq!(
            result.script,
            if case["script"] == -1 {
                None
            } else {
                Some(case.n::<i16>("script"))
            }
        );
        crate::support::native_expect! {
        scalar (result.sounds) => &case["sounds"], "binary_story_item_use";
        scalar (result.warnings) => &case["warnings"], "binary_story_item_use";
        }
        let mut effects = result
            .warnings
            .iter()
            .map(|warning| serde_json::json!(["warning", warning]))
            .chain(
                result
                    .sounds
                    .iter()
                    .map(|sound| serde_json::json!(["sound", sound])),
            )
            .collect::<Vec<_>>();
        if let Some(script) = result.script {
            effects.push(serde_json::json!(["script", script]));
            let args = case.bytes("script_args");
            assert_eq!(&args[..8], &[0; 8]);
            assert_eq!(word(&args, 8).unwrap(), 1);
            assert_eq!(word(&args, 10).unwrap(), script);
            assert_eq!(word(&args, 12).unwrap(), 0);
        }
        assert_eq!(case["effects"], Value::Array(effects));

        let original = crate::support::record_bytes(&before);
        assert_bytes(&original, &case.bytes("before"), "caller inputs");
    }
}
