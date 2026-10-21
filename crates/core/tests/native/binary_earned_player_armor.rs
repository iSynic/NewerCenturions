use crate::{native_common, prelude::*, support};

use centurions_core::items;
use native_common::snapshot_party as party;

use support::inventory as records;

use support::assert_bytes;

use support::compare_party_named as assert_party;

fn assert_records(records: &[InventoryRecord], hex: &str, context: &str) {
    assert_bytes(&support::record_bytes(records), &bytes(hex), context);
}

#[test]
fn earned_armor_can_use_matches_all_24_native_profile_gates() {
    let fixture = support::fixture("earned-player-armor.json");
    let content = content();
    let mission = content.map(0).unwrap();
    let cases = fixture.array("gate_cases");

    for case in cases {
        let warriors = party(&case["before"]);
        let member = case.n::<usize>("member");
        let id = case.n::<usize>("item_id");
        assert_eq!(
            equipment::can_use(&warriors[member], &content.items[id], mission).unwrap(),
            case.n::<i64>("allowed") != 0
        );
        support::native_expect! {
            bytes &content.items[id] => &case["after"].bytes("item_work"), "native gate loaded item";
        };
    }
}

#[test]
fn earned_armor_collection_matches_12_native_party_crate_and_loot_cases() {
    let fixture = support::fixture("earned-player-armor.json");
    let content = content();
    let mission = content.map(0).unwrap();
    let cases = fixture.array("transfer_cases");

    for (index, case) in cases.iter().enumerate() {
        let member = case.n::<usize>("member");
        let slot = case.n::<usize>("crate_slot");
        let mut warriors = party(&case["before"]);
        let crate_items = records(case["before"].text("crate"));
        let mut loot = crate_items.clone();
        assert_records(&loot, case["opened"].text("loot"), "native crate copy");
        let result =
            equipment::collect_item(&warriors[member], &loot[slot], &content, mission).unwrap();
        assert_eq!(result.collected, case.n::<i64>("collected") != 0);
        if result.collected {
            warriors[member] = result.warrior;
            loot[slot] = result.item;
        }
        assert_party(
            &warriors,
            &case["picked"],
            &format!("transfer {index} party"),
        );
        assert_records(
            &crate_items,
            case["picked"].text("crate"),
            "native retained crate",
        );
        assert_records(&loot, case["picked"].text("loot"), "native selected loot");
        let mut compacted = loot
            .iter()
            .filter(|item| item.id() != 0)
            .cloned()
            .collect::<Vec<_>>();
        compacted.resize(loot.len(), InventoryRecord([0; 22]));
        assert_records(
            &compacted,
            case["after"].text("crate"),
            "native closed crate",
        );
        assert_party(&warriors, &case["after"], "native closed party");
        assert_eq!(result.sounds, case["picked"].n::<Vec<i16>>("sounds"));
        assert_eq!(result.warnings, case["picked"].n::<Vec<i16>>("warnings"));
    }
}

#[test]
fn earned_armor_wear_remove_and_compaction_match_all_16_native_compositions() {
    let fixture = support::fixture("earned-player-armor.json");
    let content = content();
    let mission = content.map(0).unwrap();
    let cases = fixture.array("equipment_cases");

    for (index, case) in cases.iter().enumerate() {
        let member = case.n::<usize>("member");
        let slot = case.n::<usize>("crate_slot");
        let in_combat = case.n::<bool>("combat");
        let mut warriors = party(&case["before"]);
        let mut crate_items = records(case["before"].text("crate"));
        let collected =
            equipment::collect_item(&warriors[member], &crate_items[slot], &content, mission)
                .unwrap();
        assert!(collected.collected);
        warriors[member] = collected.warrior;
        crate_items[slot] = collected.item;
        crate_items.retain(|item| item.id() != 0);
        crate_items.resize(1000, InventoryRecord([0; 22]));
        assert_records(
            &crate_items,
            case["picked"].text("crate"),
            "composed crate collection",
        );
        assert_party(&warriors, &case["picked"], "composed armor pickup");
        let mut audio = collected.sounds;
        let inventory_slot = case.n::<usize>("inventory_slot");
        let worn = equipment::wear(
            &warriors[member],
            &content,
            mission,
            inventory_slot,
            in_combat,
        )
        .unwrap();
        assert!(worn.changed);
        assert!(worn.warnings.is_empty());
        warriors[member] = worn.warrior;
        audio.extend(worn.sounds);
        assert_party(&warriors, &case["worn"], &format!("wear {index}"));
        assert_eq!(audio, case["worn"].n::<Vec<i16>>("sounds"));
        items::compact(&mut warriors[member]).unwrap();
        assert_party(&warriors, &case["compacted"], "first compaction");
        let slot = case.n::<usize>("compacted_slot");
        let removed = equipment::remove(&warriors[member], &content, mission, slot).unwrap();
        assert!(removed.changed);
        assert!(removed.warnings.is_empty());
        warriors[member] = removed.warrior;
        audio.extend(removed.sounds);
        assert_party(&warriors, &case["removed"], "complete remove");
        assert_eq!(audio, case["removed"].n::<Vec<i16>>("sounds"));
        let reworn =
            equipment::wear(&warriors[member], &content, mission, slot, in_combat).unwrap();
        assert!(reworn.changed);
        assert!(reworn.warnings.is_empty());
        warriors[member] = reworn.warrior;
        audio.extend(reworn.sounds);
        assert_party(&warriors, &case["reworn"], "complete re-wear");
        assert_eq!(audio, case["reworn"].n::<Vec<i16>>("sounds"));
        items::compact(&mut warriors[member]).unwrap();
        assert_party(&warriors, &case["after"], "final compaction");
        assert_eq!(audio, case["after"].n::<Vec<i16>>("sounds"));
    }
}
