use centurions_content::{Content, Warrior, digest};
use centurions_core::{
    RandomState, story_inventory,
    story_promotion::{self, LibraryRandom},
    story_treasure,
};
use serde_json::Value;
use std::path::Path;

fn bytes(value: &Value, name: &str) -> Vec<u8> {
    let text = value[name].as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}
fn patched(mut input: Vec<u8>, patches: &Value) -> Vec<u8> {
    for p in patches.as_array().unwrap() {
        input[p[0].as_u64().unwrap() as usize] = p[1].as_u64().unwrap() as u8;
    }
    input
}
fn party(input: &[u8]) -> Vec<Warrior> {
    input
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|b| Warrior::read(b).unwrap())
        .collect()
}
fn flatten(input: &[Warrior]) -> Vec<u8> {
    input.iter().flat_map(|w| w.bytes.iter().copied()).collect()
}

#[test]
fn native_close_promotes_each_living_member_once_and_preserves_valid_spells() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/level-promotion.json"
    ))
    .unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let map = content.map(0).unwrap();
    let baseline = bytes(&fixture, "warriors");
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 107);
    for (index, case) in cases
        .iter()
        .enumerate()
        .filter(|(_, c)| c["kind"] == "close")
    {
        let before = patched(baseline.clone(), &case["before_changes"]);
        let warriors = party(&before);
        let initial = warriors.clone();
        let mut library = LibraryRandom {
            seed: case["library_seed_before"].as_u64().unwrap() as u32,
            draws: 0,
        };
        let result = story_promotion::close(
            &warriors,
            &map.races,
            &map.profiles,
            &map.combat.as_ref().unwrap().attacks,
            case["registered"].as_bool().unwrap(),
            &mut library,
        )
        .unwrap_or_else(|e| panic!("case{index}: {e:?}"));
        let expected = patched(before, &case["after_changes"]);
        assert_eq!(flatten(&result.warriors), expected, "case{index}");
        assert_eq!(digest(&expected), case["after_sha256"], "case{index}");
        assert_eq!(warriors, initial, "input party mutated");
        assert_eq!(
            library.seed,
            case["library_seed_after"].as_u64().unwrap() as u32,
            "case{index}"
        );
        assert_eq!(
            library.draws,
            case["library_draws"].as_u64().unwrap(),
            "case{index}"
        );
        assert_eq!(case["rng_draws"], 0);
        assert_eq!(case["seed_after"], 12);
        assert_eq!(case["currency_after"], 937);
    }
}

#[test]
fn original_reward_chain_crosses_level_thresholds_at_close() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/level-promotion.json"
    ))
    .unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let map = content.map(0).unwrap();
    let baseline = bytes(&fixture, "warriors");
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["kind"] == "chain")
    {
        let mut warriors = party(&patched(baseline.clone(), &case["before_changes"]));
        let mut library = LibraryRandom { seed: 1, draws: 0 };
        let mut random = RandomState {
            seed: case["seed_before"].as_u64().unwrap() as u32,
            draws: 0,
        };
        for stage in case["stages"].as_array().unwrap() {
            let n = stage["stage"].as_u64().unwrap();
            let table = bytes(stage, "table");
            let loot = if n == 1 {
                let prepared =
                    story_treasure::prepare(&warriors, &table, &content, &mut random).unwrap();
                warriors = prepared.warriors;
                prepared.items
            } else {
                let amount = i32::from_be_bytes(table[80..84].try_into().unwrap());
                warriors = story_inventory::award_story_experience(&warriors, amount)
                    .unwrap()
                    .warriors;
                vec![]
            };
            warriors = story_promotion::close(
                &warriors,
                &map.races,
                &map.profiles,
                &map.combat.as_ref().unwrap().attacks,
                true,
                &mut library,
            )
            .unwrap()
            .warriors;
            assert_eq!(
                flatten(&warriors),
                bytes(stage, "warriors"),
                "chain stage{n}"
            );
            let mut crate_bytes = vec![0; 22000];
            for (i, item) in loot.iter().enumerate() {
                crate_bytes[i * 22..i * 22 + 22].copy_from_slice(&item.0);
            }
            assert_eq!(crate_bytes, bytes(stage, "crate"), "chain stage{n}");
            assert_eq!(
                library.seed,
                stage["library_seed"].as_u64().unwrap() as u32,
                "chain stage{n}"
            );
            assert_eq!(
                library.draws,
                stage["library_draws"].as_u64().unwrap(),
                "chain stage{n}"
            );
            assert_eq!(
                random.seed,
                stage["seed"].as_u64().unwrap() as u32,
                "chain stage{n}"
            );
            assert_eq!(
                random.draws,
                stage["draws"].as_u64().unwrap(),
                "chain stage{n}"
            );
            assert_eq!(stage["currency"], 937);
        }
    }
}

#[test]
fn unsupported_promotion_preserves_library_randomness() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let map = content.map(0).unwrap();
    let mut warriors = content.warriors.clone();
    warriors[0].bytes[0x9ce..0x9d2].copy_from_slice(&(-1_i32).to_be_bytes());
    warriors[0].bytes[0x3a..0x3c].copy_from_slice(&65_i16.to_be_bytes());
    let initial = LibraryRandom { seed: 1, draws: 0 };
    let mut library = initial.clone();
    assert!(
        story_promotion::close(
            &warriors,
            &map.races,
            &map.profiles,
            &map.combat.as_ref().unwrap().attacks,
            true,
            &mut library
        )
        .is_err()
    );
    assert_eq!(library, initial);
}
