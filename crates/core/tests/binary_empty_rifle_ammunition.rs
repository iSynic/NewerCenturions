use centurions_content::{Content, InventoryRecord, Warrior, word};
use centurions_core::{
    RandomState,
    ammunition::{self, RefitFlags, WorkBuffers},
    combat::firearm,
    equipment, items,
};
use serde_json::Value;
fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
fn party(snapshot: &Value) -> Vec<Warrior> {
    bytes(snapshot["party"].as_str().unwrap())
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|b| Warrior::read(b).unwrap())
        .collect()
}
fn reserves(snapshot: &Value) -> Vec<i32> {
    bytes(snapshot["ammunition"].as_str().unwrap())
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| i32::from_be_bytes(*b))
        .collect()
}
fn data() -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../evidence/fixtures/empty-rifle-ammunition.json"),
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
fn assert_bytes(actual: &[u8], expected: &[u8], label: &str) {
    assert_eq!(actual.len(), expected.len(), "{label} length");
    if let Some(i) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!("{label} byte{i:#x}:{} != {}", actual[i], expected[i]);
    }
}
fn assert_party(warriors: &[Warrior], snapshot: &Value, label: &str) {
    assert_bytes(
        &warriors
            .iter()
            .flat_map(|w| w.bytes.iter().copied())
            .collect::<Vec<_>>(),
        &bytes(snapshot["party"].as_str().unwrap()),
        label,
    );
}
fn sounds(snapshot: &Value) -> Vec<i16> {
    snapshot["sounds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap() as i16)
        .collect()
}
#[test]
fn actual_empty_rifle_collection_wear_and_compaction_match_four_native_cases() {
    let f = data();
    let content = content();
    let map = f["foundations"][0]["session"]["exploration"]["map"]
        .as_u64()
        .unwrap() as i32;
    let mission = content.map(map).unwrap();
    assert_eq!(f["wear_cases"].as_array().unwrap().len(), 4);
    for (i, case) in f["wear_cases"].as_array().unwrap().iter().enumerate() {
        let mut w = party(&case["before"]);
        let count = w[0].inventory().unwrap().len();
        let original = bytes(case["before"]["crate"].as_str().unwrap())
            .as_chunks::<22>()
            .0
            .iter()
            .find(|r| word(*r, 0).unwrap() == 38)
            .map(|r| InventoryRecord(*r))
            .unwrap();
        assert_eq!(word(&original.0, 2).unwrap(), 0);
        assert_eq!(word(&original.0, 4).unwrap(), 0);
        let collected = equipment::collect_item(&w[0], &original, &content, mission).unwrap();
        assert!(collected.collected);
        assert_eq!(collected.sounds, sounds(&case["picked"]));
        w[0] = collected.warrior;
        assert_party(&w, &case["picked"], &format!("pickup{i}"));
        assert_eq!(collected.item.id(), 0);
        let result = equipment::wear(
            &w[0],
            &content,
            mission,
            count,
            case["combat"].as_u64().unwrap() != 0,
        )
        .unwrap();
        assert!(result.changed);
        assert!(result.warnings.is_empty());
        assert_eq!(
            result.sounds,
            &sounds(&case["worn"])[sounds(&case["picked"]).len()..]
        );
        w[0] = result.warrior;
        assert_party(&w, &case["worn"], &format!("wear{i}"));
        items::compact(&mut w[0]).unwrap();
        assert_party(&w, &case["compacted"], &format!("compact{i}"));
        assert_eq!(word(&w[0].bytes, 0x126).unwrap(), 0);
        assert_eq!(case["after"]["rng_draws"], 0);
        assert_eq!(case["before"]["rng_seed"], case["after"]["rng_seed"]);
    }
}
#[test]
fn selected_203_ammunition_matches_24_native_callers_complete_party_stock_rng_and_audio() {
    let f = data();
    let content = content();
    assert_eq!(f["manual_cases"].as_array().unwrap().len(), 24);
    for (i, case) in f["manual_cases"].as_array().unwrap().iter().enumerate() {
        let before = &case["operation_before"];
        let after = &case["after"];
        let mut warriors = party(before);
        let input = warriors.clone();
        let stock = reserves(before);
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let result = firearm::fit_ammunition(
            &warriors[0],
            case["inventory_slot"].as_u64().unwrap() as usize,
            &content,
            &stock,
            case["combat"].as_u64().unwrap() != 0,
            &mut random,
        )
        .unwrap_or_else(|e| panic!("manual{i}:{e:?}"));
        assert_eq!(warriors, input, "manual{i} immutable input");
        warriors[0] = result.warrior;
        assert_party(&warriors, after, &format!("manual{i}"));
        assert_eq!(
            result.ammunition,
            reserves(after),
            "manual{i} all200 reserves"
        );
        assert_eq!(
            result.loaded_item_field32,
            after["item_context"].as_i64().unwrap() as i16,
            "manual{i} context"
        );
        assert_eq!(
            result.loaded,
            case["reserve203"].as_i64().unwrap() > 0,
            "manual{i} accepted"
        );
        assert_eq!(
            result.sounds,
            &sounds(after)[sounds(before).len()..],
            "manual{i} audio"
        );
        assert_eq!(
            random.seed,
            after["rng_seed"].as_u64().unwrap() as u32,
            "manual{i} seed"
        );
        assert_eq!(
            random.draws,
            after["rng_draws"].as_u64().unwrap(),
            "manual{i} draws"
        );
        for key in [
            "crate",
            "loot",
            "packet",
            "identity",
            "attack_count",
            "mask",
            "target_count",
            "target_descriptors",
            "fields",
            "used",
            "map",
            "party_buffs",
            "target",
        ] {
            assert_eq!(before[key], after[key], "manual{i} native unchanged{key}");
        }
        if result.loaded {
            assert_eq!(
                &after["draws"].as_array().unwrap()[before["draws"].as_array().unwrap().len()..],
                &[serde_json::json!({"bound":2,"result":result.sounds.last().unwrap()-403})],
                "manual{i} native ordered draw"
            );
        } else {
            assert_eq!(after["draws"], before["draws"], "manual{i} cancel draws");
        }
    }
}
#[test]
fn automatic_empty_refit_matches_twelve_native_direct_helper_cases() {
    let f = data();
    let content = content();
    assert_eq!(f["refit_cases"].as_array().unwrap().len(), 12);
    for (i, case) in f["refit_cases"].as_array().unwrap().iter().enumerate() {
        let before = &case["operation_before"];
        let after = &case["after"];
        let native = party(after);
        let stock = case["reserve203"].as_i64().unwrap();
        assert_eq!(
            word(&native[0].bytes, 0x126).unwrap(),
            0,
            "refit{i} native cache"
        );
        assert_eq!(
            word(&native[0].bytes, 0x136).unwrap(),
            if stock == 12 {
                203
            } else if stock == 2 {
                200
            } else {
                0
            },
            "refit{i} native instance ID"
        );
        assert_eq!(
            reserves(after)[3],
            if stock == 12 { 9 } else { stock as i32 },
            "refit{i} native stock"
        );
        let mut warriors = party(before);
        let work = WorkBuffers {
            item: Some(bytes(before["item_work"].as_str().unwrap())),
            weapon: Some(bytes(before["weapon"].as_str().unwrap())),
            ammo: Some(bytes(before["ammo"].as_str().unwrap())),
        };
        let stock = reserves(before);
        let mut random = RandomState {
            seed: before["rng_seed"].as_u64().unwrap() as u32,
            draws: before["rng_draws"].as_u64().unwrap(),
        };
        let original = (warriors.clone(), stock.clone(), work.clone());
        // VerifyEmptyRifleAmmunition invokes refit(0,0,1,1): audible,
        // forced, with warning49 suppressed. It never calls DoReload111.
        let result = ammunition::refit(
            &warriors[0],
            &stock,
            &content,
            RefitFlags {
                in_combat: case["combat"].as_u64().unwrap() != 0,
                force: true,
                silent: false,
                warnings: false,
            },
            &work,
            &mut random,
        )
        .unwrap();
        assert_eq!((warriors.clone(), stock, work), original);
        warriors[0] = result.warrior;
        assert_party(&warriors, after, &format!("refit{i} all six warriors"));
        assert_eq!(result.reserves, reserves(after), "refit{i} all200 reserves");
        assert_eq!(
            result.native_return,
            case["refit_return"].as_i64().unwrap() as i16
        );
        for (actual, key) in [
            (result.work.item, "item_work"),
            (result.work.weapon, "weapon"),
            (result.work.ammo, "ammo"),
        ] {
            assert_eq!(
                actual,
                Some(bytes(after[key].as_str().unwrap())),
                "refit{i} full150 {key}"
            );
        }
        assert_eq!(result.sounds, sounds(after)[sounds(before).len()..]);
        assert!(result.warnings.is_empty());
        let mut independent = RandomState {
            seed: before["rng_seed"].as_u64().unwrap() as u32,
            draws: before["rng_draws"].as_u64().unwrap(),
        };
        for draw in &after["draws"].as_array().unwrap()[before["draws"].as_array().unwrap().len()..]
        {
            assert_eq!(
                independent.bound(draw["bound"].as_i64().unwrap() as i16),
                draw["result"].as_i64().unwrap() as i16
            );
        }
        assert_eq!(random, independent);
        assert_eq!(random.seed, after["rng_seed"].as_u64().unwrap() as u32);
        assert_eq!(random.draws, after["rng_draws"].as_u64().unwrap());
        for key in [
            "crate",
            "loot",
            "packet",
            "identity",
            "attack_count",
            "mask",
            "target_count",
            "target_descriptors",
            "fields",
            "used",
            "map",
            "party_buffs",
            "target",
            "markers",
        ] {
            assert_eq!(before[key], after[key], "refit{i} native retained {key}");
        }
    }
}
