use centurions_content::{Content, InventoryRecord, Warrior};
use centurions_core::story_collection::collect_story_item;
use std::path::Path;

fn bytes(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn assert_bytes(actual: &[u8], expected: &[u8], context: &str) {
    assert_eq!(actual.len(), expected.len(), "{context} length");
    if let Some(index) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "{context} differs at byte {index}: {} != {}",
            actual[index], expected[index]
        );
    }
}

fn sounds(case: &serde_json::Value, key: &str) -> Vec<i16> {
    case[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_i64().unwrap() as i16)
        .collect()
}

#[test]
fn quest_collection_matches_original_complete_warriors_and_ordered_effects() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/loot-collection.json"
    ))
    .unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let before = Warrior::read(&bytes(case["before"].as_str().unwrap())).unwrap();
        let loot = InventoryRecord(
            bytes(case["loot_before"].as_str().unwrap())
                .try_into()
                .unwrap(),
        );
        let result = collect_story_item(
            &before,
            &loot,
            &content,
            case["race_movement"].as_i64().unwrap() as i8,
        )
        .unwrap();
        let context = format!("member {} variant {}", case["member"], case["variant"]);
        assert_bytes(
            &result.warrior.bytes,
            &bytes(case["after"].as_str().unwrap()),
            &context,
        );
        assert_bytes(
            &result.item.0,
            &bytes(case["loot_after"].as_str().unwrap()),
            &context,
        );
        assert_eq!(
            result.collected,
            case["collected"].as_bool().unwrap(),
            "{context}"
        );
        assert_eq!(result.sounds, sounds(case, "sounds"), "{context}");
        assert_eq!(result.warnings, sounds(case, "warnings"), "{context}");
        assert_eq!(case["rng_draws"], 0);
        let mut party = bytes(case["party_before"].as_str().unwrap());
        let base = case["member"].as_u64().unwrap() as usize * 3138;
        party[base..base + 3138].copy_from_slice(&result.warrior.bytes);
        assert_bytes(
            &party,
            &bytes(case["party_after"].as_str().unwrap()),
            &context,
        );
    }
}
