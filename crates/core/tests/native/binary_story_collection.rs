use crate::prelude::*;

use crate::support::assert_bytes;
use centurions_core::story_collection::collect_story_item;

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
    let fixture: serde_json::Value = crate::support::fixture("loot-collection.json");

    let content = crate::support::content();
    for case in fixture.array("cases") {
        let before = Warrior::read(&case.bytes("before")).unwrap();
        let loot = InventoryRecord(case.bytes("loot_before").try_into().unwrap());
        let result =
            collect_story_item(&before, &loot, &content, case.n::<i8>("race_movement")).unwrap();
        let context = format!("member {} variant {}", case["member"], case["variant"]);
        crate::support::native_expect! {
        bytes result.warrior.bytes => &case["after"], &context;
        bytes result.item.0 => &case["loot_after"], &context;
        scalar result.collected => &case["collected"], "binary_story_collection";
        }
        assert_eq!(result.sounds, sounds(case, "sounds"), "{context}");
        assert_eq!(result.warnings, sounds(case, "warnings"), "{context}");

        let mut party = case.bytes("party_before");
        let base = case.n::<usize>("member") * 3138;
        party[base..base + 3138].copy_from_slice(&result.warrior.bytes);
        assert_bytes(&party, &case.bytes("party_after"), &context);
    }
}
