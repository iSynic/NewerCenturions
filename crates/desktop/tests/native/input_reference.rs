//! Independent native input, viewport and attack-estimate comparisons.
use super::*;

pub(super) fn compare_attack_estimates() -> centurions_content::Content {
    let receipt: serde_json::Value =
        crate::original_content::fixture("thunder-attack-estimates.json");
    let content = crate::original_content::load();
    let native = centurions_content::game_text::GameText::load(
        &content,
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../analysis/extracted"),
    )
    .unwrap();
    let mut estimate: Session = serde_json::from_value(receipt["input"].clone()).unwrap();
    for case in receipt["cases"].as_array().unwrap() {
        estimate.warriors[4].bytes[0x9dd] = u8::from(case["firing"].as_bool().unwrap());
        let plan = centurions_core::item_display::attack_estimate(
            &estimate,
            &content,
            10,
            &native,
            &Default::default(),
        )
        .unwrap();
        let numbers: Vec<_> = plan
            .effects
            .iter()
            .filter_map(|effect| match effect {
                centurions_core::item_display::output::PanelEffect::NumberToString {
                    value,
                    ..
                } => Some(*value),
                _ => None,
            })
            .collect();
        assert_eq!(serde_json::json!(numbers), case["numbers"]);
        let retained = plan.loaded_items.as_ref().unwrap();
        for (bytes, field) in [
            (&retained.item, "item_after"),
            (&retained.ammo, "ammunition_after"),
            (&retained.weapon, "weapon_after"),
        ] {
            let expected = case[field].as_str().unwrap();
            let actual = bytes.as_deref().unwrap_or(&[0; 150]);
            assert_eq!(
                actual
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>(),
                expected
            );
        }
    }
    content
}

pub(super) fn compare_pointer_hold(input: &mut MovementInput) {
    let native: serde_json::Value = crate::original_content::fixture("thunder-mouse-hold.json");
    for case in native["cases"].as_array().unwrap() {
        input.pointer_step((1, 0), true, true, 5.);
        assert_eq!(
            input
                .pointer_step((1, 0), false, case["held"].as_bool().unwrap(), 6.)
                .is_some(),
            case["repeat"].as_bool().unwrap()
        );
    }
}

type ViewportCase = ((i32, i32), (i32, i32));

pub(super) fn compare_outdoor_centers() -> Vec<ViewportCase> {
    let fixture: serde_json::Value = crate::original_content::fixture("outdoor-viewport.json");
    fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|case| {
            let x = case["position"][0].as_i64().unwrap() as i32;
            let y = case["position"][1].as_i64().unwrap() as i32;
            let expected = (
                case["origin"][0].as_i64().unwrap() as i32,
                case["origin"][1].as_i64().unwrap() as i32,
            );
            assert_eq!(crate::layout::outdoor_origin(x, y), expected);
            ((x, y), expected)
        })
        .collect()
}
