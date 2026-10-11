use centurions_content::{Content, InventoryRecord, Warrior, word};
use centurions_core::{equipment, items};
use serde_json::Value;

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).unwrap())
        .collect()
}

fn party(snapshot: &Value) -> Vec<Warrior> {
    bytes(snapshot["party"].as_str().unwrap())
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|record| Warrior::read(record).unwrap())
        .collect()
}

fn records(hex: &str) -> Vec<InventoryRecord> {
    bytes(hex)
        .as_chunks::<22>()
        .0
        .iter()
        .map(|record| InventoryRecord::read(record).unwrap())
        .collect()
}

fn fixture() -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../evidence/fixtures/earned-player-armor.json"),
        )
        .unwrap(),
    )
    .unwrap()
}

fn content() -> Content {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
}

fn assert_bytes(actual: &[u8], expected: &[u8], context: &str) {
    assert_eq!(actual.len(), expected.len(), "{context} length");
    if let Some(index) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "{context} byte {index:x}: {} != {}",
            actual[index], expected[index]
        );
    }
}

fn assert_party(warriors: &[Warrior], snapshot: &Value, context: &str) {
    assert_bytes(
        &warriors
            .iter()
            .flat_map(|warrior| warrior.bytes.iter().copied())
            .collect::<Vec<_>>(),
        &bytes(snapshot["party"].as_str().unwrap()),
        context,
    );
}

fn assert_records(records: &[InventoryRecord], hex: &str, context: &str) {
    assert_bytes(
        &records
            .iter()
            .flat_map(|record| record.0.iter().copied())
            .collect::<Vec<_>>(),
        &bytes(hex),
        context,
    );
}

fn sounds(snapshot: &Value) -> Vec<i16> {
    snapshot["sounds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_i64().unwrap() as i16)
        .collect()
}

fn unchanged_context(before: &Value, after: &Value) {
    for key in [
        "weapon",
        "ammo",
        "ammunition",
        "party_buffs",
        "used",
        "packet",
        "target",
        "target_count",
        "target_descriptors",
        "mask",
        "rng_seed",
        "rng_draws",
        "rng_sites",
        "draws",
    ] {
        assert_eq!(before[key], after[key], "native unchanged {key}");
    }
}

#[test]
fn earned_armor_can_use_matches_all_24_native_profile_gates() {
    let fixture = fixture();
    let content = content();
    let mission = content.map(0).unwrap();
    let cases = fixture["gate_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 24);
    for case in cases {
        let warriors = party(&case["before"]);
        let member = case["member"].as_u64().unwrap() as usize;
        let id = case["item_id"].as_u64().unwrap() as usize;
        assert_eq!(
            equipment::can_use(&warriors[member], &content.items[id], mission).unwrap(),
            case["allowed"].as_i64().unwrap() != 0
        );
        assert_party(&warriors, &case["after"], "native gate party");
        for (key, value) in case["before"].as_object().unwrap() {
            if key != "item_work" {
                assert_eq!(value, &case["after"][key], "native gate unchanged {key}");
            }
        }
        assert_bytes(
            &content.items[id],
            &bytes(case["after"]["item_work"].as_str().unwrap()),
            "native gate loaded item",
        );
    }
}

#[test]
fn earned_armor_collection_matches_12_native_party_crate_and_loot_cases() {
    let fixture = fixture();
    let content = content();
    let mission = content.map(0).unwrap();
    let cases = fixture["transfer_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 12);
    for (index, case) in cases.iter().enumerate() {
        let member = case["member"].as_u64().unwrap() as usize;
        let slot = case["crate_slot"].as_u64().unwrap() as usize;
        let mut warriors = party(&case["before"]);
        let crate_items = records(case["before"]["crate"].as_str().unwrap());
        let mut loot = crate_items.clone();
        assert_records(
            &loot,
            case["opened"]["loot"].as_str().unwrap(),
            "native crate copy",
        );
        let result =
            equipment::collect_item(&warriors[member], &loot[slot], &content, mission).unwrap();
        assert_eq!(result.collected, case["collected"].as_i64().unwrap() != 0);
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
            case["picked"]["crate"].as_str().unwrap(),
            "native retained crate",
        );
        assert_records(
            &loot,
            case["picked"]["loot"].as_str().unwrap(),
            "native selected loot",
        );
        let mut compacted = loot
            .iter()
            .filter(|item| item.id() != 0)
            .cloned()
            .collect::<Vec<_>>();
        compacted.resize(loot.len(), InventoryRecord([0; 22]));
        assert_records(
            &compacted,
            case["after"]["crate"].as_str().unwrap(),
            "native closed crate",
        );
        assert_party(&warriors, &case["after"], "native closed party");
        assert_eq!(result.sounds, sounds(&case["picked"]));
        assert_eq!(
            result.warnings,
            case["picked"]["warnings"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i16)
                .collect::<Vec<_>>()
        );
        assert_eq!(result.sounds, [450]);
        assert_eq!(case["after"]["item_context"], 0);
        for phase in ["opened", "picked", "after"] {
            unchanged_context(&case["before"], &case[phase]);
        }
    }
}

#[test]
fn earned_armor_wear_remove_and_compaction_match_all_16_native_compositions() {
    let fixture = fixture();
    let content = content();
    let mission = content.map(0).unwrap();
    let cases = fixture["equipment_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 16);
    for (index, case) in cases.iter().enumerate() {
        let member = case["member"].as_u64().unwrap() as usize;
        let slot = case["crate_slot"].as_u64().unwrap() as usize;
        let in_combat = case["combat"].as_bool().unwrap();
        let mut warriors = party(&case["before"]);
        let mut crate_items = records(case["before"]["crate"].as_str().unwrap());
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
            case["picked"]["crate"].as_str().unwrap(),
            "composed crate collection",
        );
        assert_party(&warriors, &case["picked"], "composed armor pickup");
        let mut audio = collected.sounds;
        let inventory_slot = case["inventory_slot"].as_u64().unwrap() as usize;
        let instance = warriors[member].inventory().unwrap()[inventory_slot].clone();
        let opaque = [instance.0[11], instance.0[12]];
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
        assert_eq!(audio, sounds(&case["worn"]));
        items::compact(&mut warriors[member]).unwrap();
        assert_party(&warriors, &case["compacted"], "first compaction");
        let slot = case["compacted_slot"].as_u64().unwrap() as usize;
        let compacted = warriors[member].inventory().unwrap()[slot].clone();
        assert_eq!([compacted.0[11], compacted.0[12]], opaque);
        let removed = equipment::remove(&warriors[member], &content, mission, slot).unwrap();
        assert!(removed.changed);
        assert!(removed.warnings.is_empty());
        warriors[member] = removed.warrior;
        audio.extend(removed.sounds);
        assert_party(&warriors, &case["removed"], "complete remove");
        assert_eq!(audio, sounds(&case["removed"]));
        let reworn =
            equipment::wear(&warriors[member], &content, mission, slot, in_combat).unwrap();
        assert!(reworn.changed);
        assert!(reworn.warnings.is_empty());
        warriors[member] = reworn.warrior;
        audio.extend(reworn.sounds);
        assert_party(&warriors, &case["reworn"], "complete re-wear");
        assert_eq!(audio, sounds(&case["reworn"]));
        items::compact(&mut warriors[member]).unwrap();
        assert_party(&warriors, &case["after"], "final compaction");
        assert_eq!(audio, sounds(&case["after"]));
        assert_eq!(audio, [450; 4]);
        for phase in ["picked", "worn", "compacted", "removed", "reworn", "after"] {
            unchanged_context(&case["before"], &case[phase]);
            assert_eq!(case[phase]["item_context"], 0);
            assert_eq!(
                case[phase]["rng_seed"],
                fixture["foundation"]["session"]["random"]["seed"]
            );
            assert_eq!(
                case[phase]["rng_draws"],
                fixture["foundation"]["session"]["random"]["draws"]
            );
            assert_records(
                &crate_items,
                case[phase]["crate"].as_str().unwrap(),
                "retained composed crate",
            );
            assert_eq!(case[phase]["loot"], case["picked"]["loot"]);
        }
        assert_eq!(
            word(&warriors[member].bytes, 0x40).unwrap(),
            word(&party(&case["before"])[member].bytes, 0x40).unwrap()
                - if in_combat { 2 } else { 0 }
        );
    }
}
