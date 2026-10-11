use centurions_content::{Content, Warrior, word};
use centurions_core::{
    RandomState,
    combat::damage::{DamageFlags, DamagePacket, resolve_damage},
    combat::firearm,
};
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/shield-ranged.json"
    ))
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

fn random(snapshot: &Value) -> RandomState {
    RandomState {
        seed: snapshot["rng_seed"].as_u64().unwrap() as u32,
        draws: snapshot["rng_draws"].as_u64().unwrap(),
    }
}

fn content() -> Content {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
}

fn stock(snapshot: &Value) -> Vec<i32> {
    bytes(&snapshot["ammunition"])
        .as_chunks::<4>()
        .0
        .iter()
        .map(|record| i32::from_be_bytes(*record))
        .collect()
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

#[test]
fn two_native_charged_shots_match_complete_damage_records_rng_and_shield_audio() {
    let f = fixture();
    assert_eq!(f["shot_cases"].as_array().unwrap().len(), 2);
    for case in f["shot_cases"].as_array().unwrap() {
        let before = &case["targeted"];
        let after = &case["fired"];
        let source = case["source"].as_i64().unwrap();
        let mut actual = random(before);
        let draws =
            &after["draws"].as_array().unwrap()[before["draws"].as_array().unwrap().len()..];
        let sites = &after["rng_sites"].as_array().unwrap()
            [before["rng_sites"].as_array().unwrap().len()..];
        assert_eq!(
            sites[..3],
            [
                Value::from("1384bc"),
                Value::from("1502ca"),
                Value::from("1503f0")
            ]
        );
        // The native duration and hit prefix precedes the recovered damage boundary.
        for draw in &draws[..3] {
            assert_eq!(
                actual.bound(draw["bound"].as_i64().unwrap() as i16),
                draw["result"].as_i64().unwrap() as i16
            );
        }
        let entry = after["native_calls"]
            .as_array()
            .unwrap()
            .iter()
            .find(|call| call["address"] == "15075a")
            .unwrap();
        assert_eq!(entry["arguments"], serde_json::json!([3, 1, 1, 1, 0]));
        assert_eq!(actual.draws, entry["draws"].as_u64().unwrap());
        let mut warriors = party(before);
        let packet = bytes(&before["packet"]);
        assert_eq!(packet, bytes(&case["resolutions"][0]["packet"]));
        let result = resolve_damage(
            &mut warriors[3].bytes,
            false,
            &DamagePacket {
                bytes: packet,
                attack_count: 1,
            },
            DamageFlags {
                shield: true,
                armor: true,
                saves: true,
                armor_filter: false,
            },
            &mut actual,
        )
        .unwrap();
        let expected = party(after);
        assert_eq!(
            result.total,
            word(&party(before)[3].bytes, 0x50).unwrap() - word(&expected[3].bytes, 0x50).unwrap()
        );
        assert_eq!(result.total, if source == 12 { 11 } else { 21 });
        assert_eq!(result.components[0], result.total);
        assert!(result.components[1..].iter().all(|amount| *amount == 0));
        assert!(result.shield_sound);
        assert_eq!(word(&warriors[3].bytes, 0x26).unwrap(), 238);
        assert_eq!(word(&warriors[3].bytes, 0x160).unwrap(), 238);
        // HP and the actor's AP/MOV belong to the cast caller, outside resolvedamage.
        let mut target = expected[3].bytes.clone();
        target[0x50..0x52].copy_from_slice(&party(before)[3].bytes[0x50..0x52]);
        assert_bytes(&warriors[3].bytes, &target, "complete damage target");
        for member in [1, 2, 4, 5] {
            assert_bytes(
                &warriors[member].bytes,
                &expected[member].bytes,
                "uninvolved warrior",
            );
        }
        let mut actor = expected[0].bytes.clone();
        for offset in [0x3c, 0x40] {
            actor[offset..offset + 2].copy_from_slice(&party(before)[0].bytes[offset..offset + 2]);
        }
        assert_bytes(&warriors[0].bytes, &actor, "actor outside caller AP/MOV");
        assert_eq!(actual, random(after));
        let mut receipt = random(before);
        for draw in draws {
            assert_eq!(
                receipt.bound(draw["bound"].as_i64().unwrap() as i16),
                draw["result"].as_i64().unwrap() as i16
            );
        }
        assert_eq!(actual, receipt);
        let calls = after["native_calls"].as_array().unwrap();
        let shield: Vec<_> = calls
            .iter()
            .filter(|call| call["name"] == "Sound" && call["return"] == "150c22")
            .map(|call| call["arguments"][0].as_i64().unwrap())
            .collect();
        assert_eq!(shield, vec![206]);
        let events = after["ranged_events"].as_array().unwrap();
        let guard = events
            .iter()
            .find(|event| event["address"] == "150a26")
            .unwrap();
        let physical = if source == 12 { 12 } else { 24 };
        assert_eq!(
            guard["damage_components"],
            serde_json::json!([physical, 0, 0, 0, 0, 0, 0, 0, 0])
        );
        let write = events
            .iter()
            .find(|event| event["address"] == "150b30")
            .unwrap();
        assert_eq!(write["d6"], 238);
        assert_eq!(
            write["damage_components"],
            serde_json::json!([physical - 1, 0, 0, 0, 0, 0, 0, 0, 0])
        );
        assert_eq!(
            write["draws"].as_u64().unwrap() - guard["draws"].as_u64().unwrap(),
            physical as u64
        );
        let inventory_write = events
            .iter()
            .find(|event| event["address"] == "150b44")
            .unwrap();
        assert_eq!(inventory_write["charge"], 238);
        assert_eq!(inventory_write["draws"], write["draws"]);
        for field in [
            "crate",
            "loot",
            "ammunition",
            "map",
            "fields",
            "party_buffs",
            "memory",
        ] {
            assert!(before[field] == after[field], "native {field} changed");
        }
    }
}

#[test]
fn native_pre_compaction_pistol_fit_restores_battery_cache_without_an_admission_guard() {
    let f = fixture();
    assert_eq!(f["restore_cases"].as_array().unwrap().len(), 1);
    let case = &f["restore_cases"][0];
    let before = &case["before"];
    let after = &case["after"];
    let content = content();
    let mut warriors = party(before);
    let slot = warriors[3]
        .inventory()
        .unwrap()
        .iter()
        .position(|item| item.id() == 2 && item.0[9] == 1)
        .unwrap();
    let item = warriors[3].inventory().unwrap()[slot].clone();
    assert_eq!(word(&warriors[3].bytes, 0x126).unwrap(), 307);
    assert_eq!(word(&item.0, 2).unwrap(), 8);
    assert_eq!(word(&item.0, 4).unwrap(), 201);
    assert_eq!(word(&item.0, 6).unwrap(), 12);
    let initial_stock = stock(before);
    assert_eq!(initial_stock[1], 132);
    let mut actual = random(before);
    let result = firearm::fit_ammunition(
        &warriors[3],
        slot,
        &content,
        &initial_stock,
        false,
        &mut actual,
    )
    .unwrap();
    assert!(result.loaded);
    assert_eq!(result.loaded_item_field32, 2);
    assert_eq!(result.sounds, vec![404]);
    let native_sounds: Vec<i16> = after["sounds"].as_array().unwrap()
        [before["sounds"].as_array().unwrap().len()..]
        .iter()
        .map(|sound| sound.as_i64().unwrap() as i16)
        .collect();
    assert_eq!(result.sounds, native_sounds);
    warriors[3] = result.warrior;
    let expected = party(after);
    for (actual, expected) in warriors.iter().zip(expected) {
        assert_bytes(
            &actual.bytes,
            &expected.bytes,
            "complete firearm cache restoration party",
        );
    }
    assert_eq!(word(&warriors[3].bytes, 0x126).unwrap(), 201);
    let item = warriors[3].inventory().unwrap()[slot].clone();
    assert_eq!(word(&item.0, 2).unwrap(), 12);
    assert_eq!(word(&item.0, 4).unwrap(), 201);
    assert_eq!(result.ammunition, stock(after));
    assert_eq!(result.ammunition[1], 128);
    assert_eq!(actual, random(after));
    let mut receipt = random(before);
    for draw in &after["draws"].as_array().unwrap()[before["draws"].as_array().unwrap().len()..] {
        assert_eq!(
            receipt.bound(draw["bound"].as_i64().unwrap() as i16),
            draw["result"].as_i64().unwrap() as i16
        );
    }
    assert_eq!(receipt, actual);
    assert_eq!(actual.draws - random(before).draws, 1);
    for field in [
        "crate",
        "loot",
        "map",
        "fields",
        "party_buffs",
        "memory",
        "packet",
    ] {
        assert!(before[field] == after[field], "native {field} changed");
    }
}
