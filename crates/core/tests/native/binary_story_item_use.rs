use crate::prelude::*;

use centurions_core::story_item_use::use_scenario_item;

use crate::support::warriors;

fn content(id: usize, definition: Vec<u8>) -> Content {
    let mut items = vec![vec![0; 150]; 833];
    items[id] = definition;
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
    let fixture = crate::support::fixture("item-use.json");
    let movement = fixture.n::<Vec<i8>>("race_movements");
    for case in fixture.array("cases") {
        let before = warriors(&case["before"]);
        let content = content(832, case.bytes("definition"));
        let result = use_scenario_item(
            &before,
            case.n::<usize>("member"),
            case.n::<usize>("slot"),
            &content,
            &movement,
            centurions_core::story_item_use::ScriptItemContext::Exploration,
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
    let case = fixture
        .array("cases")
        .iter()
        .find(|case| case["member"] == 5 && case["variant"] == 0)
        .unwrap();
    let installed = crate::support::content();
    let mut state = crate::support::mission_session(&installed);
    state.warriors = warriors(&case["before"]);
    apply(
        Command::UseItem {
            warrior: 5,
            slot: case.n("slot"),
        },
        &mut state,
        &installed,
    )
    .unwrap();
    assert_eq!(state.selected_warrior, 5);
    assert_bytes(
        &crate::support::record_bytes(&state.warriors),
        &case.bytes("after"),
        "original item caller before script216",
    );
    assert!(matches!(
        state.exploration.as_ref().unwrap().presentation,
        Some(Presentation::Dialogue { id: 851, .. })
    ));
    crate::support::checkpoint(&state, &installed, "selected-script-item");
    let fixture = crate::support::fixture("inert-item-use.json");
    let installed = crate::support::content();
    for case in fixture.array("cases") {
        let mut state = crate::support::mission_session(&installed);
        state.warriors = warriors(&case["before"]);
        let before = state.clone();
        let error = apply(
            Command::UseItem {
                warrior: 0,
                slot: case.n("slot"),
            },
            &mut state,
            &installed,
        )
        .unwrap_err();
        assert!(
            matches!(error, centurions_core::RuleError::Refused { feedback }
            if feedback == case.n::<Vec<i16>>("warnings").into_iter()
                .map(|index| centurions_core::feedback::NativeFeedback::Warning { index }).collect::<Vec<_>>())
        );
        crate::support::same_session(&state, &before);
        assert_bytes(
            &crate::support::record_bytes(&state.warriors),
            &case.bytes("after"),
            "native inert item",
        );
        assert_eq!(case.n::<u32>("seed_before"), case.n::<u32>("seed_after"));
        assert_eq!(case.n::<u64>("draws"), 0);
    }
}
