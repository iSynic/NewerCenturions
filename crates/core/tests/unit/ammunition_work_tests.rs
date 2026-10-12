use super::*;
use serde_json::Value;

fn content() -> Content {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
}

fn bytes(value: &Value) -> Vec<u8> {
    let text = value.as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).unwrap())
        .collect()
}

#[test]
fn complete_npc_preparation_records_match_original_instructions() {
    let content = content();
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../../evidence/fixtures/final-enemy-weapons.json"
    ))
    .unwrap();
    let cases = fixture["preparation_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 324);
    for (index, case) in cases.iter().enumerate() {
        let input = bytes(&case["enemy_before"]);
        let weapon = word(&input, 0x26).unwrap();
        let ammo = if case["context"].as_i64().unwrap() == 0 {
            0
        } else {
            word(&input, 0x4c).unwrap()
        };
        let retained = definition(&content, 700).unwrap();
        let mut work = WorkBuffers {
            item: Some(retained.clone()),
            ..Default::default()
        };
        work.prepare_pair(
            &content.items[weapon.unsigned_abs() as usize],
            &content.items[ammo.unsigned_abs() as usize],
        )
        .unwrap();
        assert_eq!(
            work.weapon,
            Some(bytes(&case["weapon"])),
            "weapon case {index}"
        );
        assert_eq!(work.ammo, Some(bytes(&case["ammo"])), "ammo case {index}");
        assert_eq!(work.item, Some(retained), "retained item case {index}");
    }
}

#[test]
fn natural_and_melee_loaders_replace_only_their_original_work_records() {
    let content = content();
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../../evidence/fixtures/player-guard-reaction.json"
    ))
    .unwrap();
    let mut work = WorkBuffers::default();
    work.load_natural(&content.items).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        for snapshot in std::iter::once(&case["after"]).chain(
            case["attack_returns"]
                .as_array()
                .unwrap()
                .iter()
                .map(|entry| &entry["state"]),
        ) {
            assert_eq!(work.item, Some(bytes(&snapshot["item_work"])));
            assert_eq!(work.weapon, Some(bytes(&snapshot["weapon_work"])));
            assert_eq!(work.ammo, Some(bytes(&snapshot["ammo_work"])));
        }
    }
    let retained = work.clone();
    work.load_weapon(&content.items, 187).unwrap();
    assert_eq!(work.weapon, Some(definition(&content, 187).unwrap()));
    assert_eq!(work.item, retained.item);
    assert_eq!(work.ammo, retained.ammo);
    let mut unknown = WorkBuffers::default();
    unknown.load_weapon(&content.items, 187).unwrap();
    assert!(unknown.item.is_none() && unknown.ammo.is_none());
}
