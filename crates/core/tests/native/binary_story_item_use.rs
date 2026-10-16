use crate::prelude::*;

use centurions_core::story_item_use::use_scenario_item;

use crate::support::warriors;

fn content(id: usize, definition: Vec<u8>) -> Content {
    let mut items = vec![vec![0; 150]; 833];
    items[id] = definition;
    Content {
        items,
        ..crate::support::fixture_content("item-use fixture")
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
            None,
        )
        .unwrap();
        let actual = crate::support::record_bytes(&result.warriors);
        crate::support::native_expect! {
        bytes actual => &case["after"], &format!("member {} variant {}", case["member"], case["variant"]);
        scalar result.return_code => &case["return_code"], "binary_story_item_use";
        }
        assert_eq!(
            result.script,
            (case["script"] != -1).then(|| case.n::<i16>("script"))
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
        }
        assert_eq!(case["effects"], Value::Array(effects));
    }
    let case = fixture
        .array("cases")
        .iter()
        .find(|case| case["member"] == 5 && case["variant"] == 0)
        .unwrap();
    let installed = crate::support::content();
    for worn in [false, true] {
        let mut state = crate::support::mission_session(&installed);
        state.warriors = warriors(&case["before"]);
        let mut selected_slot = case.n("slot");
        if worn {
            apply(
                Command::EquipItem {
                    warrior: 5,
                    slot: case.n("slot"),
                },
                &mut state,
                &installed,
            )
            .unwrap();
            selected_slot = state.warriors[5]
                .inventory()
                .unwrap()
                .iter()
                .position(|item| item.id() == 832 && item.0[9] != 0)
                .unwrap();
            state = crate::support::checkpoint(&state, &installed, "worn-script-item");
        }
        apply(
            Command::UseItem {
                warrior: 5,
                slot: selected_slot,
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
    }
    // Source-derived dropitem boundary beyond its fixed 29-slot shift.
    for item in [827_i16, 832] {
        let mut extended = crate::support::mission_session(&installed);
        extended.warriors = warriors(&case["before"]);
        let bytes = &mut extended.warriors[5].bytes;
        let mut selected = bytes[0x132 + case.n::<usize>("slot") * 22..][..22].to_vec();
        selected[..2].copy_from_slice(&item.to_be_bytes());
        let old_count = word(bytes, 0x1a).unwrap() as usize;
        for slot in old_count..31 {
            bytes[0x132 + slot * 22..0x132 + (slot + 1) * 22].copy_from_slice(&selected);
        }
        bytes[0x1a..0x1c].copy_from_slice(&31_i16.to_be_bytes());
        let prefix = bytes[0x132..0x132 + 30 * 22].to_vec();
        let mut resumed = crate::support::checkpoint(&extended, &installed, "extended-script-item");
        replay_command(
            Command::UseItem {
                warrior: 5,
                slot: 30,
            },
            &mut extended,
            &mut resumed,
            &installed,
        );
        assert_eq!(extended.warriors[5].inventory().unwrap().len(), 30);
        assert_eq!(&extended.warriors[5].bytes[0x132..0x132 + 30 * 22], prefix);
    }
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
