use centurions_content::{Content, InventoryRecord, Warrior, word};
use centurions_core::{RandomState, story_inventory::*};
use std::path::Path;

fn bytes(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn warriors(s: &str) -> Vec<Warrior> {
    bytes(s)
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|record| Warrior::read(record).unwrap())
        .collect()
}

fn records(warriors: &[Warrior]) -> Vec<u8> {
    warriors
        .iter()
        .flat_map(|warrior| warrior.bytes.iter().copied())
        .collect()
}

fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/story-inventory.json"
    ))
    .unwrap()
}

fn assert_bytes(actual: &[u8], expected: &[u8], context: &str) {
    assert_eq!(actual.len(), expected.len(), "{context} record length");
    if let Some(index) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "{context} differs at byte {index}: {} != {}",
            actual[index], expected[index]
        );
    }
}

fn content() -> Content {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
}

#[test]
fn quest_consumption_matches_original_complete_warriors_and_drop_effects() {
    let fixture = fixture();
    let content = content();
    let race_movements: Vec<i8> = fixture["race_movements"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap() as i8)
        .collect();
    for case in fixture["consume_cases"].as_array().unwrap() {
        let before = warriors(case["before"].as_str().unwrap());
        let result = consume_party_items(
            &before,
            &content,
            &race_movements,
            &bytes(case["extra"].as_str().unwrap()),
        )
        .unwrap();
        assert_bytes(
            &records(&result.warriors),
            &bytes(case["after"].as_str().unwrap()),
            "consumption",
        );
        assert_eq!(result.consumed, case["consumed"].as_u64().unwrap() as usize);
        assert_eq!(
            result.sounds,
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i16)
                .collect::<Vec<_>>()
        );
        assert_eq!(records(&before), bytes(case["before"].as_str().unwrap()));
        assert_eq!(case["rng_draws"], 0);
    }
}

#[test]
fn item_prerequisites_match_original_first_party_holder_and_slot() {
    for case in fixture()["query_cases"].as_array().unwrap() {
        let before = warriors(case["before"].as_str().unwrap());
        let extra = bytes(case["extra"].as_str().unwrap());
        let found = query_party_item(&before, word(&extra, 0).unwrap()).unwrap();
        assert_eq!(
            item_prerequisite_matches(&before, &extra).unwrap(),
            case["matched"].as_bool().unwrap()
        );
        assert_eq!(
            found.as_ref().map_or(0, |item| item.warrior + 1),
            case["holder"].as_u64().unwrap() as usize
        );
        assert_eq!(
            found.as_ref().map_or(0, |item| item.slot),
            case["slot"].as_u64().unwrap() as usize
        );
        assert_eq!(records(&before), bytes(case["after"].as_str().unwrap()));
        assert_eq!(case["rng_draws"], 0);
    }
}

#[test]
fn story_experience_awards_match_original_complete_normalized_warriors() {
    for case in fixture()["award_cases"].as_array().unwrap() {
        let result = award_story_experience(
            &warriors(case["before"].as_str().unwrap()),
            case["amount"].as_i64().unwrap() as i32,
        )
        .unwrap();
        assert_bytes(
            &records(&result.warriors),
            &bytes(case["after"].as_str().unwrap()),
            "experience award",
        );
        assert_eq!(result.share, case["share"].as_i64().unwrap() as i32);
        assert_eq!(case["seed_before"], case["seed_after"]);
        assert_eq!(case["rng_draws"], 0);
    }
}

#[test]
fn authored_treasure_matches_original_full_loot_and_rng_for_both_seeds() {
    let content = content();
    for case in fixture()["treasure_cases"].as_array().unwrap() {
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let items = grant_story_treasure(
            &bytes(case["table_before"].as_str().unwrap()),
            &content,
            &mut random,
        )
        .unwrap();
        let mut loot = vec![InventoryRecord([0; 22]); 1000];
        loot[..items.len()].clone_from_slice(&items);
        assert_bytes(
            &loot.iter().flat_map(|item| item.0).collect::<Vec<_>>(),
            &bytes(case["crate_after"].as_str().unwrap()),
            &format!("treasure {} seed {}", case["treasure"], case["seed_before"]),
        );
        assert_eq!(random.seed, case["seed_after"].as_u64().unwrap() as u32);
        assert_eq!(random.draws, case["rng_draws"].as_u64().unwrap());
        assert_eq!(
            records(&normalize_story_loot(&warriors(case["before"].as_str().unwrap())).unwrap()),
            bytes(case["after"].as_str().unwrap())
        );
    }
}
