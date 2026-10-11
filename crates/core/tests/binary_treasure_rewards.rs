use centurions_content::{Content, Warrior};
use centurions_core::{RandomState, story_treasure};
use serde_json::Value;
use std::path::Path;

fn bytes(value: &Value, name: &str) -> Vec<u8> {
    let text = value[name].as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn original_treasure_eight_preserves_participation_and_awards_experience() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/treasure-rewards.json"
    ))
    .unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 12);
    for case in cases {
        let before = bytes(case, "before");
        let warriors: Vec<_> = before
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|record| Warrior::read(record).unwrap())
            .collect();
        let mut random = RandomState {
            seed: case["seed_before"].as_u64().unwrap() as u32,
            draws: 0,
        };
        let result = story_treasure::prepare(
            &warriors,
            &bytes(case, "table_before"),
            &content,
            &mut random,
        )
        .unwrap();
        assert_eq!(
            result
                .warriors
                .iter()
                .flat_map(|w| w.bytes.iter().copied())
                .collect::<Vec<_>>(),
            bytes(case, "after")
        );
        assert_eq!(result.share, case["share"].as_i64().unwrap() as i32);
        let mut crate_bytes = vec![0; 22000];
        for (slot, item) in result.items.iter().enumerate() {
            crate_bytes[slot * 22..(slot + 1) * 22].copy_from_slice(&item.0);
        }
        assert_eq!(crate_bytes, bytes(case, "crate_after"));
        assert_eq!(case["currency"], 0);
        assert_eq!(random.seed, case["seed_after"].as_u64().unwrap() as u32);
        assert_eq!(random.draws, case["rng_draws"].as_u64().unwrap());
    }
}

#[test]
fn unrecovered_treasure_currency_and_random_rewards_preserve_randomness() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let initial = RandomState { seed: 12, draws: 0 };
    for offset in [80, 84] {
        let mut record = vec![0; 88];
        record[offset..offset + 4].copy_from_slice(&(-1_i32).to_be_bytes());
        let mut random = initial.clone();
        assert!(story_treasure::prepare(&[], &record, &content, &mut random).is_err());
        assert_eq!(random, initial);
    }
}
