use centurions_content::{Content, Warrior, combat::MonsterRecord, digest};
use centurions_core::{RandomState, combat::rewards};
use serde_json::Value;
use std::path::Path;

fn bytes(value: &Value, name: &str) -> Vec<u8> {
    let text = value[name].as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

fn flatten(warriors: &[Warrior]) -> Vec<u8> {
    warriors
        .iter()
        .flat_map(|warrior| warrior.bytes.iter().copied())
        .collect()
}

#[test]
fn original_booty_recounts_unavailable_warriors_before_normalizing_and_preserves_count_at_close() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/booty-availability.json"
    ))
    .unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 64);
    for (index, case) in cases.iter().enumerate() {
        let before = bytes(case, "before");
        let warriors: Vec<_> = before
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|input| Warrior::read(input).unwrap())
            .collect();
        let enemies: Vec<_> = bytes(case, "enemies")
            .as_chunks::<288>()
            .0
            .iter()
            .map(|input| MonsterRecord {
                bytes: input.to_vec(),
            })
            .collect();
        let mut random = RandomState {
            seed: case["seed_before"].as_u64().unwrap() as u32,
            draws: 0,
        };
        let result = rewards::prepare(&warriors, &enemies, &content, 0, 0, 37, &mut random)
            .unwrap_or_else(|error| panic!("case {index}: {error:?}"));
        assert_eq!(
            result.unavailable,
            case["unavailable_prepared"].as_i64().unwrap() as i8,
            "case {index}: recount"
        );
        let prepared = flatten(&result.warriors);
        assert_eq!(
            prepared,
            bytes(case, "prepared"),
            "case {index}: preparation"
        );
        assert_eq!(digest(&prepared), case["prep_sha256"].as_str().unwrap());
        assert_eq!(result.share, case["share"].as_i64().unwrap() as i32);
        assert_eq!(result.total, case["total"].as_i64().unwrap() as i32);
        assert_eq!(
            result.currency,
            case["currency_after"].as_i64().unwrap() as i32
        );
        assert!(
            result.items.is_empty(),
            "case {index}: controlled empty loot"
        );
        let native_enemy = bytes(case, "enemies");
        let mut expected_enemy = native_enemy;
        for change in case["enemy_changes"].as_array().unwrap() {
            expected_enemy[change[0].as_u64().unwrap() as usize] =
                change[1].as_u64().unwrap() as u8;
        }
        assert_eq!(
            result
                .enemies
                .iter()
                .flat_map(|enemy| enemy.bytes.iter().copied())
                .collect::<Vec<_>>(),
            expected_enemy,
            "case {index}: enemy normalization"
        );
        assert_eq!(random.seed, case["seed_after"].as_u64().unwrap() as u32);
        assert_eq!(random.draws, case["rng_draws"].as_u64().unwrap());
        let closed = flatten(&rewards::close_without_promotion(&result.warriors).unwrap());
        assert_eq!(closed, bytes(case, "after"), "case {index}: Close");
        assert_eq!(digest(&closed), case["after_sha256"].as_str().unwrap());
        assert_eq!(
            result.unavailable,
            case["unavailable_after"].as_i64().unwrap() as i8,
            "case {index}: native Close retains the recount"
        );
    }
}
