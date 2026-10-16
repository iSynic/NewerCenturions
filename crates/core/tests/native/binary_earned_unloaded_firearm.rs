use crate::prelude::*;

use centurions_core::items;

use crate::support::buffer as bytes;

fn party(f: &Value, snapshot: &Value) -> Vec<Warrior> {
    crate::support::warriors(&bytes(f, &snapshot["party"]))
}

fn records(f: &Value, snapshot: &Value, key: &str) -> Vec<InventoryRecord> {
    crate::support::inventory(&bytes(f, &snapshot[key]))
}

fn sounds(case: &Value, before: &Value, after: &Value) -> Vec<i16> {
    case.array("sounds")[before.n::<usize>("sound_count")..after.n::<usize>("sound_count")]
        .iter()
        .map(|sound| sound.as_i64().unwrap() as i16)
        .collect()
}

fn assert_party(actual: &[Warrior], f: &Value, native: &Value) {
    assert_eq!(
        crate::support::record_bytes(actual),
        bytes(f, &native["party"])
    );
}

#[test]
fn earned_unloaded_collection_matches_all_five_native_party_and_crate_cases() {
    let content = crate::support::content();
    let f: Value = crate::support::fixture("earned-unloaded-firearm.json");

    for case in f.array("cases") {
        let before = party(&f, &case["before"]);
        let source = records(&f, &case["before"], "crate")[9].clone();
        let result =
            equipment::collect_item(&before[4], &source, &content, content.map(3).unwrap())
                .unwrap();
        let mut actual = before.clone();
        actual[4] = result.warrior;
        assert_party(&actual, &f, &case["picked"]);
        assert_eq!(result.collected, case["pickup_return"] == 1);
        assert_eq!(
            result.sounds,
            sounds(case, &case["opened"], &case["picked"])
        );
        assert_eq!(
            serde_json::to_value(result.warnings).unwrap(),
            case["warnings"]
        );
        assert_eq!(result.item, records(&f, &case["picked"], "loot")[9]);
        let mut survivors = records(&f, &case["before"], "crate");
        if result.collected {
            survivors[9] = result.item;
        }
        survivors.retain(|record| record.id() != 0);
        survivors.resize(1000, InventoryRecord([0; 22]));
        assert_eq!(survivors, records(&f, &case["closed"], "crate"));
    }
}

#[test]
fn full_outdoor_wear_and_compaction_preserve_empty_ammunition_and_native_level_denial() {
    let content = crate::support::content();
    let f: Value = crate::support::fixture("earned-unloaded-firearm.json");
    for case in f
        .array("cases")
        .iter()
        .filter(|case| case.get("wear_before").is_some())
    {
        let before = party(&f, &case["wear_before"]);
        let result =
            equipment::wear(&before[4], &content, content.map(3).unwrap(), 4, false).unwrap();
        assert_eq!(result.changed, case["wear_return"] == 1);
        assert_eq!(
            result.sounds,
            sounds(case, &case["wear_before"], &case["worn"])
        );
        assert!(result.warnings.is_empty());
        let mut actual = before.clone();
        actual[4] = result.warrior;
        assert_party(&actual, &f, &case["worn"]);
        if result.changed {
            // wear165344 copies the new equipped record before common accuracy/range.
            // This scratch replaces the earlier old1 removal copy and is not a save field.
            let mut work = actual[4].bytes.clone();
            work[4..6].copy_from_slice(&32_i16.to_be_bytes());
            work[0x130..0x132].fill(0);
            assert_eq!(bytes(&f, &case["worn"]["remove_warrior_work"]), work);
            items::compact(&mut actual[4]).unwrap();
            assert_party(&actual, &f, &case["compacted"]);
        } else {
            // Native wear reports sound6000; public EquipItem retains its separate diagnostic.
            assert_eq!(actual, before);
        }
    }
}
