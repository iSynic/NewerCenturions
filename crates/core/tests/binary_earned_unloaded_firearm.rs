use centurions_content::{Content, InventoryRecord, Warrior, digest, word};
use centurions_core::{Session, equipment, items};
use serde_json::Value;

fn fixture() -> (Content, Value) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let fixture = serde_json::from_slice(
        &std::fs::read(root.join("evidence/fixtures/earned-unloaded-firearm.json")).unwrap(),
    )
    .unwrap();
    (content, fixture)
}

fn bytes(f: &Value, value: &Value) -> Vec<u8> {
    let text = value.as_str().unwrap_or_else(|| {
        f["shared_rule_buffers"][value["shared_buffer_sha256"].as_str().unwrap()]
            .as_str()
            .unwrap()
    });
    assert_eq!(text.len() % 2, 0);
    let result: Vec<_> = text
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect();
    if value.is_object() {
        assert_eq!(result.len() as u64, value["byte_length"].as_u64().unwrap());
        assert_eq!(
            digest(&result),
            value["shared_buffer_sha256"].as_str().unwrap()
        );
    }
    result
}

fn party(f: &Value, snapshot: &Value) -> Vec<Warrior> {
    bytes(f, &snapshot["party"])
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|record| Warrior::read(record).unwrap())
        .collect()
}

fn records(f: &Value, snapshot: &Value, key: &str) -> Vec<InventoryRecord> {
    bytes(f, &snapshot[key])
        .as_chunks::<22>()
        .0
        .iter()
        .map(|record| InventoryRecord(*record))
        .collect()
}

fn sounds(case: &Value, before: &Value, after: &Value) -> Vec<i16> {
    case["sounds"].as_array().unwrap()[before["sound_count"].as_u64().unwrap() as usize
        ..after["sound_count"].as_u64().unwrap() as usize]
        .iter()
        .map(|sound| sound.as_i64().unwrap() as i16)
        .collect()
}

fn assert_party(actual: &[Warrior], f: &Value, native: &Value) {
    let actual: Vec<_> = actual
        .iter()
        .flat_map(|warrior| warrior.bytes.clone())
        .collect();
    assert_eq!(actual, bytes(f, &native["party"]));
}

fn assert_snapshot(f: &Value, native: &Value) {
    assert_eq!(native.as_object().unwrap().len(), 99);
    for value in native.as_object().unwrap().values() {
        if value.is_object() {
            bytes(f, value);
        }
    }
    let party = bytes(f, &native["party"]);
    assert_eq!(party.len(), 18_828);
    assert_eq!(
        bytes(f, &native["member_record"]),
        party[4 * 3138..5 * 3138]
    );
    assert_eq!(
        bytes(f, &native["targets"])[44..102],
        bytes(f, &native["packet"])
    );
    assert_eq!(
        bytes(f, &native["targets"])[102..],
        bytes(f, &native["attack_cache"])[..558]
    );
    for key in ["item_work", "weapon_work", "ammo_work"] {
        assert_eq!(bytes(f, &native[key]).len(), 150);
    }
    assert_eq!(
        word(&bytes(f, &native["item_work"]), 32).unwrap() as i64,
        native["item_context"].as_i64().unwrap()
    );
}

fn assert_retained(before: &Value, after: &Value, changed: &[&str]) {
    assert_eq!(
        before.as_object().unwrap().keys().collect::<Vec<_>>(),
        after.as_object().unwrap().keys().collect::<Vec<_>>()
    );
    for (key, value) in before.as_object().unwrap() {
        if !changed.contains(&key.as_str()) {
            assert_eq!(value, &after[key], "retained native {key}");
        }
    }
}

fn assert_native_evidence(f: &Value, case: &Value, content: &Content) {
    for key in [
        "before",
        "opened",
        "picked",
        "closed",
        "wear_before",
        "worn",
        "compacted",
        "after",
    ] {
        if let Some(snapshot) = case.get(key) {
            assert_snapshot(f, snapshot);
        }
    }
    assert_retained(
        &case["before"],
        &case["after"],
        &[
            "member_record",
            "party",
            "crate",
            "loot",
            "item_work",
            "item_context",
            "selected_record",
            "sound_count",
            "native_stack",
            "remove_warrior_work",
            "race_profile_work",
            "mos_profile_work",
            "scratch_selection",
            "inventory_reserve_scan",
        ],
    );
    let map = content.map(3).unwrap();
    assert_eq!(
        bytes(f, &case["before"]["attack_cache"]),
        map.combat.as_ref().unwrap().attacks
    );
    assert_eq!(bytes(f, &case["before"]["item_cache0"]), map.item_cache);
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let scenario = std::fs::read(
        root.join("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"),
    )
    .unwrap();
    assert_eq!(
        bytes(f, &case["before"]["scenario_cache"]),
        scenario[..30000]
    );
    let mut definition = content.items[165].clone();
    let class = word(&definition, 26).unwrap().wrapping_abs();
    definition[26..28].copy_from_slice(&class.to_be_bytes());
    assert_eq!(bytes(f, &case["after"]["item_work"]), definition);
    assert!(case["rng"].as_array().unwrap().is_empty());
    for phase in case["native_phases"].as_array().unwrap() {
        assert_snapshot(f, &phase["before"]);
        assert_snapshot(f, &phase["after"]);
        assert_eq!(
            phase["after"]["native_stack"].as_str().unwrap(),
            format!("{:x}", phase["entry_sp_value"].as_u64().unwrap() + 4)
        );
        if phase["address"] == "157586" {
            assert_eq!(phase["arguments"], serde_json::json!([165, 9, 4]));
            let receiver = &party(f, &phase["before"])[4];
            assert!(equipment::can_use(receiver, &content.items[165], map).unwrap());
            assert_eq!(phase["d0_return_word"], 1);
            assert_eq!(
                bytes(f, &phase["after"]["race_profile_work"]),
                map.races[8 * 408..9 * 408]
            );
            assert_eq!(
                bytes(f, &phase["after"]["mos_profile_work"]),
                map.profiles[3 * 576..4 * 576]
            );
        }
    }
    for instruction in case["instruction_events"].as_array().unwrap() {
        assert_snapshot(f, &instruction["before_instruction"]);
        assert_snapshot(f, &instruction["after_instruction"]);
    }
}

#[test]
fn earned_unloaded_collection_matches_all_five_native_party_and_crate_cases() {
    let (content, f) = fixture();
    assert_eq!(f["cases"].as_array().unwrap().len(), 5);
    for case in f["cases"].as_array().unwrap() {
        assert_native_evidence(&f, case, &content);
        let before = party(&f, &case["before"]);
        let source = records(&f, &case["before"], "crate")[9].clone();
        assert_eq!(
            source.0,
            [
                0, 165, 0, 0, 0, 0, 0, 1, 0, 0, 15, 0, 0, 4, 0, 6, 0, 0, 0, 0, 0, 0
            ]
        );
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
        assert_eq!(case["before"]["seed"], case["after"]["seed"]);
        assert_eq!(case["before"]["draws"], case["after"]["draws"]);
    }
}

#[test]
fn full_outdoor_wear_and_compaction_preserve_empty_ammunition_and_native_level_denial() {
    let (content, f) = fixture();
    let mut checked = 0;
    for case in f["cases"]
        .as_array()
        .unwrap()
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
            let inventory = actual[4].inventory().unwrap();
            assert_eq!(
                inventory
                    .iter()
                    .map(InventoryRecord::id)
                    .collect::<Vec<_>>(),
                [165, 403, 460, 1, 700]
            );
            assert_eq!(word(&inventory[0].0, 2).unwrap(), 0);
            assert_eq!(word(&inventory[0].0, 4).unwrap(), 0);
            assert_eq!(inventory[0].0[15], 6);
            assert_eq!(word(&actual[4].bytes, 4).unwrap(), 38);
            assert_eq!(word(&actual[4].bytes, 0x126).unwrap(), 0);
            assert_eq!(word(&actual[4].bytes, 0x130).unwrap(), 22);
            assert!(matches!(
                equipment::remove(&actual[4], &content, content.map(3).unwrap(), 0),
                Err(centurions_core::RuleError::Unsupported { .. })
            ));
        } else {
            // Native wear reports sound6000; public EquipItem retains its separate diagnostic.
            assert_eq!(actual, before);
            assert_eq!(word(&actual[4].bytes, 0xe4).unwrap(), 1);
        }
        checked += 1;
    }
    assert_eq!(checked, 3);
    let foundation: Session = serde_json::from_value(f["foundation"]["session"].clone()).unwrap();
    assert_eq!(foundation.random.draws, 75_603);
}
