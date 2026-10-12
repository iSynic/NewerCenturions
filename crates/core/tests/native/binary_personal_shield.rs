use crate::prelude::*;

use centurions_core::items;

fn fixture() -> Value {
    crate::support::fixture("personal-shield.json")
}
use crate::native_common::snapshot_party as party;

use crate::native_common::snapshot_ammunition as stock;

use crate::native_common::snapshot_rng as random;

use crate::native_common::scalar_words as numbers;

fn added(before: &Value, after: &Value, field: &str) -> Vec<i16> {
    numbers(&after[field])[before[field].as_array().unwrap().len()..].to_vec()
}
use crate::support::assert_bytes;
use crate::support::compare_party as assert_party;

fn slot(warrior: &Warrior, id: i16) -> usize {
    warrior
        .inventory()
        .unwrap()
        .iter()
        .position(|item| item.id() == id)
        .unwrap()
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
    let mut markers = before.bytes("markers");
    if changed {
        markers[6..8].copy_from_slice(&1_i16.to_be_bytes());
    }
    assert_bytes(
        &markers,
        &after.bytes("markers"),
        "equipment changed marker",
    );
    warriors[3] = result.warrior;
    assert_party(warriors, after);
    assert_eq!(random(before), random(after));
    assert_eq!(stock(before), stock(after));
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
    assert_bytes(&actual, &after.bytes("crate"), "complete closed crate");
    assert_eq!(random(before), random(after));
    assert_eq!(stock(before), stock(after));
}

#[test]
fn twelve_profile_gates_and_two_manual_equipment_compositions_match_native_records() {
    let f = fixture();
    let content = content();
    let mission = content.map(3).unwrap();

    for case in f.array("gate_cases") {
        let warriors = party(&case["before"]);
        assert_eq!(
            equipment::can_use(
                &warriors[case.n::<usize>("member")],
                &content.items[case.n::<usize>("item_id")],
                mission
            )
            .unwrap(),
            case["allowed"] == 1
        );
        assert_party(&warriors, &case["after"]);
        assert_eq!(random(&case["before"]), random(&case["after"]));
    }
    let mut checked = 0;
    for case in f
        .array("equipment_cases")
        .iter()
        .filter(|case| case["kind"] == "manual")
    {
        checked += 1;
        let mut warriors = party(&case["before"]);
        let mut crate_items: Vec<_> = crate::support::inventory(&case["before"].bytes("crate"));
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

    for case in f.array("manual_controls") {
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

        assert_eq!(stock(&before_wear), stock(&case["after"]));
    }
}
