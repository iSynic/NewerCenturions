use crate::prelude::*;

use centurions_core::story_inventory::*;

fn warriors(s: &str) -> Vec<Warrior> {
    crate::support::warriors(&bytes(s))
}
use crate::support::record_bytes as records;

fn fixture() -> serde_json::Value {
    crate::support::fixture("story-inventory.json")
}

#[test]
fn quest_consumption_matches_original_complete_warriors_and_drop_effects() {
    let fixture = fixture();
    let content = content();
    let race_movements: Vec<i8> = fixture
        .array("race_movements")
        .iter()
        .map(|v| v.as_i64().unwrap() as i8)
        .collect();
    for case in fixture.array("consume_cases") {
        let before = warriors(case.text("before"));
        let result =
            consume_party_items(&before, &content, &race_movements, &case.bytes("extra")).unwrap();
        crate::support::native_expect! {
        bytes records(&result.warriors) => &case["after"], "consumption";
        scalar result.consumed => &case["consumed"], "binary_story_inventory";
        scalar (result.sounds) => &case["sounds"], "binary_story_inventory";
        bytes records(&before) => &case["before"], "binary_story_inventory";
        }
    }
}

#[test]
fn item_prerequisites_match_original_first_party_holder_and_slot() {
    for case in fixture()["query_cases"].as_array().unwrap() {
        let before = warriors(case.text("before"));
        let extra = case.bytes("extra");
        let found = query_party_item(&before, word(&extra, 0).unwrap()).unwrap();
        crate::support::native_expect! {
        scalar item_prerequisite_matches(&before, &extra).unwrap() => &case["matched"], "binary_story_inventory";
        scalar found.as_ref().map_or(0, |item| item.warrior + 1) => &case["holder"], "binary_story_inventory";
        scalar found.as_ref().map_or(0, |item| item.slot) => &case["slot"], "binary_story_inventory";
        bytes records(&before) => &case["after"], "binary_story_inventory";
        }
    }
}

#[test]
fn story_experience_awards_match_original_complete_normalized_warriors() {
    for case in fixture()["award_cases"].as_array().unwrap() {
        let result =
            award_story_experience(&warriors(case.text("before")), case.n::<i32>("amount"))
                .unwrap();
        crate::support::native_expect! {
        bytes records(&result.warriors) => &case["after"], "experience award";
        scalar result.share => &case["share"], "binary_story_inventory";
        }
    }
}

#[test]
fn authored_treasure_matches_original_full_loot_and_rng_for_both_seeds() {
    let content = content();
    for case in fixture()["treasure_cases"].as_array().unwrap() {
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let items =
            grant_story_treasure(&case.bytes("table_before"), &content, &mut random, 0).unwrap();
        let mut loot = vec![InventoryRecord([0; 22]); 1000];
        loot[..items.len()].clone_from_slice(&items);
        crate::support::native_expect! {
        bytes loot.iter().flat_map(|item| item.0).collect::<Vec<_>>() => &case["crate_after"], &format!("treasure {} seed {}", case["treasure"], case["seed_before"]);
        scalar random.seed => &case["seed_after"], "binary_story_inventory";
        scalar random.draws => &case["rng_draws"], "binary_story_inventory";
        bytes records(&normalize_story_loot(&warriors(case.text("before"))).unwrap()) => &case["after"], "binary_story_inventory";
        }
    }
}
