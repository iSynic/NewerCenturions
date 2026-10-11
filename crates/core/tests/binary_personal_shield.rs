use centurions_content::{Content, InventoryRecord, Warrior, word};
use centurions_core::combat::firearm;
use centurions_core::{RandomState, equipment, items};
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/personal-shield.json"
    ))
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

fn bytes(value: &Value) -> Vec<u8> {
    value
        .as_str()
        .unwrap()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn party(snapshot: &Value) -> Vec<Warrior> {
    bytes(&snapshot["party"])
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|record| Warrior::read(record).unwrap())
        .collect()
}

fn stock(snapshot: &Value) -> Vec<i32> {
    bytes(&snapshot["ammunition"])
        .as_chunks::<4>()
        .0
        .iter()
        .map(|record| i32::from_be_bytes(*record))
        .collect()
}

fn random(snapshot: &Value) -> RandomState {
    RandomState {
        seed: snapshot["rng_seed"].as_u64().unwrap() as u32,
        draws: snapshot["rng_draws"].as_u64().unwrap(),
    }
}

fn numbers(value: &Value) -> Vec<i16> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_i64().unwrap() as i16)
        .collect()
}

fn added(before: &Value, after: &Value, field: &str) -> Vec<i16> {
    numbers(&after[field])[before[field].as_array().unwrap().len()..].to_vec()
}

fn assert_bytes(actual: &[u8], expected: &[u8], label: &str) {
    assert!(
        actual == expected,
        "{label}: lengths {} / {}; differences {:?}",
        actual.len(),
        expected.len(),
        actual
            .iter()
            .zip(expected)
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .take(16)
            .collect::<Vec<_>>()
    );
}

fn assert_party(actual: &[Warrior], snapshot: &Value) {
    let actual: Vec<_> = actual
        .iter()
        .flat_map(|record| record.bytes.iter().copied())
        .collect();
    assert_bytes(&actual, &bytes(&snapshot["party"]), "complete party");
}

fn slot(warrior: &Warrior, id: i16) -> usize {
    warrior
        .inventory()
        .unwrap()
        .iter()
        .position(|item| item.id() == id)
        .unwrap()
}

fn assert_receipts(before: &Value, after: &Value) {
    let mut expected = random(before);
    for draw in &after["draws"].as_array().unwrap()[before["draws"].as_array().unwrap().len()..] {
        assert_eq!(
            expected.bound(draw["bound"].as_i64().unwrap() as i16),
            draw["result"].as_i64().unwrap() as i16
        );
    }
    assert_eq!(expected, random(after));
}

fn assert_isolation(before: &Value, after: &Value) {
    for field in [
        "packet",
        "target",
        "target_count",
        "target_descriptors",
        "mask",
        "fields",
        "used",
        "map",
        "party_buffs",
        "order",
        "cursor",
        "monster_index",
        "round",
        "death_count",
        "enemy_phase",
        "party_positions",
        "monster_positions",
        "memory",
        "selected",
        "powers",
        "unavailable",
        "combat",
    ] {
        assert!(before[field] == after[field], "native {field} changed");
    }
}

fn assert_equipment(
    result: equipment::EquipmentResult,
    warriors: &mut [Warrior],
    before: &Value,
    after: &Value,
    changed: bool,
) {
    assert_eq!(result.changed, changed);
    assert_eq!(result.sounds, added(before, after, "sounds"));
    assert_eq!(result.warnings, added(before, after, "warnings"));
    let mut markers = bytes(&before["markers"]);
    if changed {
        markers[6..8].copy_from_slice(&1_i16.to_be_bytes());
    }
    assert_bytes(
        &markers,
        &bytes(&after["markers"]),
        "equipment changed marker",
    );
    warriors[3] = result.warrior;
    assert_party(warriors, after);
    assert_eq!(random(before), random(after));
    assert_eq!(stock(before), stock(after));
    assert_isolation(before, after);
}

fn collect(
    warriors: &mut [Warrior],
    crate_items: &mut Vec<InventoryRecord>,
    id: i16,
    content: &Content,
    before: &Value,
    after: &Value,
) {
    let source = crate_items.iter().position(|item| item.id() == id).unwrap();
    let result = equipment::collect_item(
        &warriors[3],
        &crate_items[source],
        content,
        content.map(3).unwrap(),
    )
    .unwrap();
    assert!(result.collected);
    assert_eq!(result.sounds, added(before, after, "sounds"));
    assert_eq!(result.warnings, added(before, after, "warnings"));
    warriors[3] = result.warrior;
    crate_items[source] = result.item;
    let length = crate_items.len();
    crate_items.retain(|item| item.id() != 0);
    crate_items.resize(length, InventoryRecord([0; 22]));
    assert_party(warriors, after);
    let actual: Vec<_> = crate_items.iter().flat_map(|record| record.0).collect();
    assert_bytes(&actual, &bytes(&after["crate"]), "complete closed crate");
    assert_eq!(random(before), random(after));
    assert_eq!(stock(before), stock(after));
    assert_isolation(before, after);
}

#[test]
fn twelve_profile_gates_and_two_manual_equipment_compositions_match_native_records() {
    let f = fixture();
    let content = content();
    let mission = content.map(3).unwrap();
    assert_eq!(f["gate_cases"].as_array().unwrap().len(), 12);
    for case in f["gate_cases"].as_array().unwrap() {
        let warriors = party(&case["before"]);
        assert_eq!(
            equipment::can_use(
                &warriors[case["member"].as_u64().unwrap() as usize],
                &content.items[case["item_id"].as_u64().unwrap() as usize],
                mission
            )
            .unwrap(),
            case["allowed"] == 1
        );
        assert_party(&warriors, &case["after"]);
        assert_eq!(random(&case["before"]), random(&case["after"]));
    }
    let mut checked = 0;
    for case in f["equipment_cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["kind"] == "manual")
    {
        checked += 1;
        let mut warriors = party(&case["before"]);
        let mut crate_items: Vec<_> = bytes(&case["before"]["crate"])
            .as_chunks::<22>()
            .0
            .iter()
            .map(|record| InventoryRecord(*record))
            .collect();
        collect(
            &mut warriors,
            &mut crate_items,
            600,
            &content,
            &case["before"],
            &case["shield_picked"],
        );
        let result = equipment::wear(
            &warriors[3],
            &content,
            mission,
            slot(&warriors[3], 600),
            false,
        )
        .unwrap();
        assert_equipment(
            result,
            &mut warriors,
            &case["shield_picked"],
            &case["empty_shield_worn"],
            false,
        );
        collect(
            &mut warriors,
            &mut crate_items,
            637,
            &content,
            &case["empty_shield_worn"],
            &case["battery_picked"],
        );
        let result = equipment::wear(
            &warriors[3],
            &content,
            mission,
            slot(&warriors[3], 637),
            false,
        )
        .unwrap();
        assert_equipment(
            result,
            &mut warriors,
            &case["battery_picked"],
            &case["battery_worn"],
            true,
        );
        let mut actual = random(&case["battery_worn"]);
        let result = firearm::fit_ammunition(
            &warriors[3],
            slot(&warriors[3], 637),
            &content,
            &stock(&case["battery_worn"]),
            false,
            &mut actual,
        )
        .unwrap();
        assert!(result.loaded);
        assert_eq!(result.loaded_item_field32, 24);
        assert_eq!(
            result.sounds,
            added(&case["battery_worn"], &case["refitted"], "sounds")
        );
        warriors[3] = result.warrior;
        assert_party(&warriors, &case["refitted"]);
        assert_eq!(result.ammunition, stock(&case["refitted"]));
        assert_eq!(actual, random(&case["refitted"]));
        assert_receipts(&case["battery_worn"], &case["refitted"]);
        assert_isolation(&case["battery_worn"], &case["refitted"]);
        let result = equipment::wear(
            &warriors[3],
            &content,
            mission,
            slot(&warriors[3], 600),
            false,
        )
        .unwrap();
        assert_equipment(
            result,
            &mut warriors,
            &case["refitted"],
            &case["worn"],
            true,
        );
        items::compact(&mut warriors[3]).unwrap();
        assert_party(&warriors, &case["compacted"]);
        assert_eq!(
            word(&warriors[3].bytes, 0x126).unwrap(),
            if case["source"] == 12 { 201 } else { 200 }
        );
        let result =
            equipment::remove(&warriors[3], &content, mission, slot(&warriors[3], 637)).unwrap();
        assert_equipment(
            result,
            &mut warriors,
            &case["compacted"],
            &case["battery_remove_rejected"],
            false,
        );
        let result =
            equipment::remove(&warriors[3], &content, mission, slot(&warriors[3], 600)).unwrap();
        assert_equipment(
            result,
            &mut warriors,
            &case["battery_remove_rejected"],
            &case["shield_removed"],
            true,
        );
        let result =
            equipment::remove(&warriors[3], &content, mission, slot(&warriors[3], 637)).unwrap();
        assert_equipment(
            result,
            &mut warriors,
            &case["shield_removed"],
            &case["battery_removed"],
            true,
        );
        let result = equipment::wear(
            &warriors[3],
            &content,
            mission,
            slot(&warriors[3], 637),
            false,
        )
        .unwrap();
        assert_equipment(
            result,
            &mut warriors,
            &case["battery_removed"],
            &case["battery_reworn"],
            true,
        );
        let result = equipment::wear(
            &warriors[3],
            &content,
            mission,
            slot(&warriors[3], 600),
            false,
        )
        .unwrap();
        assert_equipment(
            result,
            &mut warriors,
            &case["battery_reworn"],
            &case["after"],
            true,
        );
        items::compact(&mut warriors[3]).unwrap();
        assert_party(&warriors, &case["after"]);
    }
    assert_eq!(checked, 2);
}

#[test]
fn five_native_manual_controls_match_full_records_stock_rng_audio_and_charge_gate() {
    let f = fixture();
    let content = content();
    let mission = content.map(3).unwrap();
    assert_eq!(f["manual_controls"].as_array().unwrap().len(), 5);
    for case in f["manual_controls"].as_array().unwrap() {
        let mut warriors = party(&case["before"]);
        let mut actual = random(&case["before"]);
        let result = firearm::fit_ammunition(
            &warriors[3],
            slot(&warriors[3], 637),
            &content,
            &stock(&case["before"]),
            false,
            &mut actual,
        )
        .unwrap();
        assert_eq!(result.loaded, case["reserve307"] != 0);
        assert_eq!(result.loaded_item_field32, 24);
        assert_eq!(
            result.sounds,
            added(&case["before"], &case["charged"], "sounds")
        );
        warriors[3] = result.warrior;
        assert_party(&warriors, &case["charged"]);
        assert_eq!(result.ammunition, stock(&case["charged"]));
        assert_eq!(actual, random(&case["charged"]));
        assert_receipts(&case["before"], &case["charged"]);
        assert_isolation(&case["before"], &case["charged"]);
        let before_wear;
        if case["repeat"] == true {
            let repeat = firearm::fit_ammunition(
                &warriors[3],
                slot(&warriors[3], 637),
                &content,
                &stock(&case["charged"]),
                false,
                &mut actual,
            )
            .unwrap();
            assert!(!repeat.loaded);
            assert_eq!(repeat.sounds, vec![404]);
            assert_eq!(repeat.warrior.bytes, warriors[3].bytes);
            assert_eq!(repeat.ammunition, stock(&case["charged"]));
            assert_eq!(actual, random(&case["after"]));
            assert_receipts(&case["charged"], &case["after"]);
            before_wear = case["charged"].clone();
            let result = equipment::wear(
                &warriors[3],
                &content,
                mission,
                slot(&warriors[3], 600),
                false,
            )
            .unwrap();
            assert!(result.changed);
            assert_eq!(result.sounds, vec![231]);
            assert!(result.warnings.is_empty());
            warriors[3] = result.warrior;
            assert_party(&warriors, &case["after"]);
        } else {
            before_wear = case["charged"].clone();
            let result = equipment::wear(
                &warriors[3],
                &content,
                mission,
                slot(&warriors[3], 600),
                false,
            )
            .unwrap();
            assert_equipment(
                result,
                &mut warriors,
                &before_wear,
                &case["after"],
                case["shield_wear_return"] == 1,
            );
        }
        assert_isolation(&before_wear, &case["after"]);
        assert_eq!(stock(&before_wear), stock(&case["after"]));
    }
}

#[test]
fn unverified_battery_instances_refit_outputs_and_contexts_are_atomic_diagnostics() {
    let f = fixture();
    let content = content();
    let case = f["equipment_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["kind"] == "manual")
        .unwrap();
    let before = &case["battery_worn"];
    let warrior = party(before)[3].clone();
    let selected = slot(&warrior, 637);
    let mut stocks = stock(before);
    let initial = random(before);
    for offset in [
        0x132 + selected * 22 + 10,
        0x132 + selected * 22 + 13,
        0x132 + selected * 22 + 14,
    ] {
        let mut changed = warrior.clone();
        changed.bytes[offset] = changed.bytes[offset].wrapping_add(1);
        let preserved = changed.bytes.clone();
        let mut actual = initial.clone();
        assert!(
            firearm::fit_ammunition(&changed, selected, &content, &stocks, false, &mut actual)
                .is_err()
        );
        assert_eq!(actual, initial);
        assert_eq!(changed.bytes, preserved);
    }
    let mut actual = initial.clone();
    assert!(
        firearm::fit_ammunition(&warrior, selected, &content, &stocks, true, &mut actual).is_err()
    );
    assert_eq!(actual, initial);
    stocks[107] = -1;
    assert!(
        firearm::fit_ammunition(&warrior, selected, &content, &stocks, false, &mut actual).is_err()
    );
    assert_eq!(actual, initial);
    for case in f["equipment_cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["kind"] == "automatic")
    {
        let warrior = party(&case["refitted"])[3].clone();
        let selected = slot(&warrior, 637);
        let mut actual = random(&case["refitted"]);
        let initial = actual.clone();
        assert!(
            firearm::fit_ammunition(
                &warrior,
                selected,
                &content,
                &stock(&case["refitted"]),
                false,
                &mut actual
            )
            .is_err()
        );
        assert_eq!(actual, initial);
    }
    let warrior = party(&case["refitted"])[3].clone();
    let selected = slot(&warrior, 637);
    let mut stocks = stock(&case["refitted"]);
    stocks[107] = 1;
    let mut actual = initial.clone();
    assert!(
        firearm::fit_ammunition(&warrior, selected, &content, &stocks, false, &mut actual).is_err()
    );
    assert_eq!(actual, initial);
    let item = warrior.inventory().unwrap()[selected].clone();
    assert!(equipment::collect_item(&warrior, &item, &content, content.map(3).unwrap()).is_err());
    assert_eq!(warrior, party(&case["refitted"])[3]);
}
