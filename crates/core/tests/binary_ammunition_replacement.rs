use centurions_content::{Content, Warrior, digest};
use centurions_core::equipment;
use serde_json::Value;

fn fixture() -> Value {
    let text = include_str!("../../../evidence/fixtures/ammunition-management.json");
    assert_eq!(
        digest(text.as_bytes()),
        "4f2b282b7e6735dd3428d39bdaaabb160f10e02442ad81052032a604742bf9bb"
    );
    serde_json::from_str(text).unwrap()
}

fn content() -> Content {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
}

fn bytes(fixture: &Value, value: &Value) -> Vec<u8> {
    let text = if let Some(text) = value.as_str() {
        text
    } else {
        fixture["shared_rule_buffers"][value["shared_buffer_sha256"].as_str().unwrap()]
            .as_str()
            .unwrap()
    };
    let result: Vec<_> = (0..text.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).unwrap())
        .collect();
    if value.is_object() {
        assert_eq!(
            result.len(),
            value["byte_length"].as_u64().unwrap() as usize
        );
        assert_eq!(
            digest(&result),
            value["shared_buffer_sha256"].as_str().unwrap()
        );
    }
    result
}

#[test]
fn replacement_matches_native_full_warriors_retained_instances_costs_and_audio() {
    let fixture = fixture();
    let content = content();
    let mission = content.map(0).unwrap();
    let mut checked = 0;
    for (index, case) in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, case)| case["kind"] == "equip")
    {
        let before = &case["before"];
        let after = &case["after"];
        let member = case["member"].as_u64().unwrap() as usize;
        let warrior = Warrior::read(&bytes(&fixture, &before["member_record"])).unwrap();
        let result = equipment::wear(
            &warrior,
            &content,
            mission,
            case["slot"].as_u64().unwrap() as usize,
            case["combat"].as_i64().unwrap() != 0,
        )
        .unwrap_or_else(|error| panic!("replacement case {index}: {error}"));
        assert_eq!(
            result.warrior.bytes,
            bytes(&fixture, &after["member_record"]),
            "case {index}"
        );
        let mut party = bytes(&fixture, &before["party"]);
        party[member * 3138..(member + 1) * 3138].copy_from_slice(&result.warrior.bytes);
        assert_eq!(party, bytes(&fixture, &after["party"]), "case {index}");
        assert_eq!(
            result.changed,
            case["native_helper_return_word"].as_i64().unwrap() != 0
        );
        let expected = |key: &str| {
            case[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_i64().unwrap() as i16)
                .collect::<Vec<_>>()
        };
        assert_eq!(result.sounds, expected("sounds"), "case {index}");
        assert_eq!(result.warnings, expected("warnings"), "case {index}");
        checked += 1;
    }
    assert_eq!(checked, 11);
}
