use centurions_content::{Content, Warrior, digest};
use centurions_core::{RandomState, ammunition};
use serde_json::Value;

use ammunition::{AmmunitionResult, FitRequest, RefitFlags, WorkBuffers};

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

fn work(fixture: &Value, snapshot: &Value) -> WorkBuffers {
    WorkBuffers {
        item: Some(bytes(fixture, &snapshot["item_work"])),
        ammo: Some(bytes(fixture, &snapshot["ammo_work"])),
        weapon: Some(bytes(fixture, &snapshot["weapon_work"])),
    }
}

fn reserves(fixture: &Value, snapshot: &Value) -> Vec<i32> {
    let bytes = bytes(fixture, &snapshot["reserves"]);
    assert_eq!(bytes.len(), 804);
    bytes[..800]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|value| i32::from_be_bytes(*value))
        .collect()
}

fn random(snapshot: &Value) -> RandomState {
    RandomState {
        seed: snapshot["seed"].as_u64().unwrap() as u32,
        draws: snapshot["draws"].as_u64().unwrap(),
    }
}

fn compare(fixture: &Value, case: &Value, actual: AmmunitionResult, actual_random: &RandomState) {
    let after = &case["after"];
    assert_eq!(
        actual.warrior.bytes,
        bytes(fixture, &after["member_record"])
    );
    let mut party = bytes(fixture, &case["before"]["party"]);
    let member = case["member"].as_u64().unwrap() as usize;
    party[member * 3138..(member + 1) * 3138].copy_from_slice(&actual.warrior.bytes);
    assert_eq!(party, bytes(fixture, &after["party"]));
    assert_eq!(actual.reserves, reserves(fixture, after));
    assert_eq!(actual.work, work(fixture, after));
    assert_eq!(
        actual.native_return,
        case["native_helper_return_word"].as_i64().unwrap() as i16
    );
    assert_eq!(
        actual.sounds,
        case["sounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_i64().unwrap() as i16)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        actual.warnings,
        case["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_i64().unwrap() as i16)
            .collect::<Vec<_>>()
    );
    assert_eq!(actual_random, &random(after));
    let mut expected_random = random(&case["before"]);
    for draw in case["rng"].as_array().unwrap() {
        assert_eq!(
            expected_random.bound(draw["bound"].as_i64().unwrap() as i16),
            draw["result"].as_i64().unwrap() as i16
        );
    }
    assert_eq!(actual_random, &expected_random);
    assert_eq!(
        actual.changed_ammo,
        after["changed_ammo"].as_i64().unwrap() != 0
    );
}

#[test]
fn refit_matches_original_manual_and_automatic_records_caches_returns_and_audio() {
    let content = content();
    let fixture = fixture();
    let mut checked = 0;
    for (index, case) in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, case)| {
            matches!(
                case["kind"].as_str().unwrap(),
                "manual_refit" | "automatic_refit"
            )
        })
    {
        let before = &case["before"];
        let warrior = Warrior::read(&bytes(&fixture, &before["member_record"])).unwrap();
        let mut rng = random(before);
        let automatic = case["kind"] == "automatic_refit";
        let actual = ammunition::refit(
            &warrior,
            &reserves(&fixture, before),
            &content,
            RefitFlags {
                in_combat: case["combat"].as_i64().unwrap() != 0,
                force: automatic,
                silent: automatic,
                warnings: !automatic,
            },
            &work(&fixture, before),
            &mut rng,
        )
        .unwrap_or_else(|error| panic!("case {index}: {error}"));
        compare(&fixture, case, actual, &rng);
        checked += 1;
    }
    assert_eq!(checked, 26);
}

#[test]
fn accepted_fit_matches_original_whole_warrior_reserves_work_and_rng() {
    let content = content();
    let fixture = fixture();
    let mut checked = 0;
    for (index, case) in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, case)| case["kind"] == "fit" && case["variant"] != "cancel_no_candidate")
    {
        let before = &case["before"];
        let warrior = Warrior::read(&bytes(&fixture, &before["member_record"])).unwrap();
        let mut rng = random(before);
        let adjacent = bytes(&fixture, &before["reserves"])[800..804]
            .try_into()
            .unwrap();
        let accepted = case["popup_inputs"][0]["accepted_ammo"].as_i64().unwrap() as usize;
        let actual = ammunition::fit(
            &warrior,
            &reserves(&fixture, before),
            &content,
            FitRequest {
                slot: case["slot"].as_u64().unwrap() as usize,
                reserve_index: accepted - 200,
                in_combat: case["combat"].as_i64().unwrap() != 0,
                loot_quality: Some(adjacent),
            },
            &work(&fixture, before),
            &mut rng,
        )
        .unwrap_or_else(|error| panic!("case {index}: {error}"));
        compare(&fixture, case, actual, &rng);
        checked += 1;
    }
    assert_eq!(checked, 12);
}

#[test]
fn unsupported_fit_contexts_and_candidates_preserve_inputs_and_rng() {
    let content = content();
    let fixture = fixture();
    let case = &fixture["cases"][13];
    let before = &case["before"];
    let warrior = Warrior::read(&bytes(&fixture, &before["member_record"])).unwrap();
    let reserves = reserves(&fixture, before);
    let work = work(&fixture, before);
    let request = FitRequest {
        slot: 0,
        reserve_index: 0,
        in_combat: true,
        loot_quality: Some([0; 4]),
    };
    let mut rng = random(before);
    let original = (warrior.clone(), reserves.clone(), work.clone(), rng.clone());
    assert!(ammunition::fit(&warrior, &reserves, &content, request, &work, &mut rng).is_err());
    assert_eq!(
        (warrior.clone(), reserves.clone(), work.clone(), rng),
        original
    );
    let request = FitRequest {
        slot: 0,
        reserve_index: 3,
        in_combat: true,
        loot_quality: None,
    };
    let mut unknown_rng = random(before);
    let unknown = ammunition::fit(
        &warrior,
        &reserves,
        &content,
        request,
        &work,
        &mut unknown_rng,
    )
    .unwrap();
    let mut known_rng = random(before);
    let known = ammunition::fit(
        &warrior,
        &reserves,
        &content,
        FitRequest {
            loot_quality: Some([1, 0, 0, 0]),
            ..request
        },
        &work,
        &mut known_rng,
    )
    .unwrap();
    assert_eq!(unknown, known);
    assert_eq!(unknown_rng, known_rng);
}
