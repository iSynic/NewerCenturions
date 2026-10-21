use crate::{prelude::*, support};

use centurions_core::story_collection::collect_story_item;
use support::assert_bytes;

#[test]
fn quest_collection_matches_original_complete_warriors_and_ordered_effects() {
    let fixture: serde_json::Value = support::fixture("loot-collection.json");

    let content = support::content();
    for case in fixture.array("cases") {
        let before = Warrior::read(&case.bytes("before")).unwrap();
        let loot = InventoryRecord(case.bytes("loot_before").try_into().unwrap());
        let result =
            collect_story_item(&before, &loot, &content, case.n::<i8>("race_movement")).unwrap();
        let context = format!("member {} variant {}", case["member"], case["variant"]);
        support::native_expect! {
            bytes result.warrior.bytes => &case["after"], &context;
            bytes result.item.0 => &case["loot_after"], &context;
            scalar result.collected => &case["collected"], "binary_story_collection";
        }
        assert_eq!(result.sounds, case.n::<Vec<i16>>("sounds"), "{context}");
        assert_eq!(result.warnings, case.n::<Vec<i16>>("warnings"), "{context}");

        let mut party = case.bytes("party_before");
        let base = case.n::<usize>("member") * 3138;
        party[base..base + 3138].copy_from_slice(&result.warrior.bytes);
        assert_bytes(&party, &case.bytes("party_after"), &context);
    }
}
