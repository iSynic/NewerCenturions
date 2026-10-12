use crate::prelude::*;

use centurions_core::ammunition;

use ammunition::{AmmunitionResult, FitRequest, RefitFlags, WorkBuffers};
fn fixture() -> Value {
    crate::support::fixture("ammunition-management.json")
}

use crate::support::buffer as bytes;

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
use crate::native_common::random_state as random;

fn compare(fixture: &Value, case: &Value, actual: AmmunitionResult, actual_random: &RandomState) {
    let after = &case["after"];
    assert_eq!(
        actual.warrior.bytes,
        bytes(fixture, &after["member_record"])
    );
    let mut party = bytes(fixture, &case["before"]["party"]);
    let member = case.n::<usize>("member");
    party[member * 3138..(member + 1) * 3138].copy_from_slice(&actual.warrior.bytes);
    assert_eq!(party, bytes(fixture, &after["party"]));
    assert_eq!(actual.reserves, reserves(fixture, after));
    assert_eq!(actual.work, work(fixture, after));
    crate::support::native_expect! {
    scalar actual.native_return => &case["native_helper_return_word"], "binary_ammunition_management";
    scalar (actual.sounds) => &case["sounds"], "binary_ammunition_management";
    scalar (actual.warnings) => &case["warnings"], "binary_ammunition_management";
    }
    assert_eq!(actual_random, &random(after));
    let mut expected_random = random(&case["before"]);
    for draw in case.array("rng") {
        crate::support::expect(
            expected_random.bound(draw.n::<i16>("bound")),
            &draw["result"],
        );
    }
    assert_eq!(actual_random, &expected_random);
    assert_eq!(actual.changed_ammo, after.n::<i64>("changed_ammo") != 0);
}

#[test]
fn refit_matches_original_manual_and_automatic_records_caches_returns_and_audio() {
    let content = content();
    let fixture = fixture();
    let mut checked = 0;
    for (index, case) in fixture
        .array("cases")
        .iter()
        .enumerate()
        .filter(|(_, case)| matches!(case.text("kind"), "manual_refit" | "automatic_refit"))
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
                in_combat: case.n::<i64>("combat") != 0,
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
    for (index, case) in fixture
        .array("cases")
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
                slot: case.n::<usize>("slot"),
                reserve_index: accepted - 200,
                in_combat: case.n::<i64>("combat") != 0,
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
