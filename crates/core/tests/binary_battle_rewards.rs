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
fn patched(mut input: Vec<u8>, patches: &Value) -> Vec<u8> {
    for patch in patches.as_array().unwrap() {
        input[patch[0].as_u64().unwrap() as usize] = patch[1].as_u64().unwrap() as u8;
    }
    input
}
fn party(input: &[u8]) -> Vec<Warrior> {
    input
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|w| Warrior::read(w).unwrap())
        .collect()
}
fn flatten(input: &[Warrior]) -> Vec<u8> {
    input.iter().flat_map(|w| w.bytes.iter().copied()).collect()
}

#[test]
fn complete_original_reward_table_cash_drops_and_ordinary_close() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/battle-rewards.json"
    ))
    .unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let baseline = bytes(&fixture, "warriors");
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 68);
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let before = patched(baseline.clone(), &case["before_changes"]);
        let enemy_bytes = bytes(case, "enemies");
        let enemies: Vec<_> = enemy_bytes
            .as_chunks::<288>()
            .0
            .iter()
            .map(|bytes| MonsterRecord {
                bytes: bytes.to_vec(),
            })
            .collect();
        let mut random = RandomState {
            seed: case["seed_before"].as_u64().unwrap() as u32,
            draws: 0,
        };
        let initial = random.clone();
        let result = rewards::prepare(
            &party(&before),
            &enemies,
            &content,
            case["difficulty"].as_i64().unwrap() as i16,
            case["npc_denominator"].as_i64().unwrap() as i16,
            case["currency_before"].as_i64().unwrap() as i32,
            &mut random,
        );
        let expected_crate = patched(vec![0; 22000], &case["crate_changes"]);
        if case["difficulty"].as_i64().unwrap() != 0
            && expected_crate.as_chunks::<22>().0.iter().any(|item| {
                i16::from_be_bytes([item[0], item[1]]) != 0 && !matches!(item[10], 23..=25)
            })
        {
            assert!(
                result.is_err(),
                "{name} requires difficulty-sensitive enhancements"
            );
            assert_eq!(random, initial, "{name} unsupported case must preserve RNG");
            continue;
        }
        let result = result.unwrap_or_else(|e| panic!("{name}: {e:?}"));
        let prep = patched(before, &case["prep_changes"]);
        assert_eq!(flatten(&result.warriors), prep, "{name} preparation");
        assert_eq!(
            digest(&prep),
            case["prep_sha256"].as_str().unwrap(),
            "{name}"
        );
        assert_eq!(
            result.share,
            case["share"].as_i64().unwrap() as i32,
            "{name}"
        );
        assert_eq!(
            result.total,
            case["total"].as_i64().unwrap() as i32,
            "{name}"
        );
        assert_eq!(
            result.currency,
            case["currency_after"].as_i64().unwrap() as i32,
            "{name}"
        );
        assert_eq!(
            result
                .enemies
                .iter()
                .flat_map(|m| m.bytes.iter().copied())
                .collect::<Vec<_>>(),
            patched(enemy_bytes, &case["enemy_changes"]),
            "{name} enemies"
        );
        let mut loot = vec![0; 22000];
        for (i, item) in result.items.iter().enumerate() {
            loot[i * 22..i * 22 + 22].copy_from_slice(&item.0);
        }
        assert_eq!(
            loot,
            patched(vec![0; 22000], &case["crate_changes"]),
            "{name} crate"
        );
        assert_eq!(
            digest(&loot),
            case["crate_sha256"].as_str().unwrap(),
            "{name}"
        );
        assert_eq!(
            random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "{name}"
        );
        assert_eq!(random.draws, case["rng_draws"].as_u64().unwrap(), "{name}");
        let closed = flatten(&rewards::close_without_promotion(&result.warriors).unwrap());
        assert_eq!(
            closed,
            patched(prep, &case["close_changes"]),
            "{name} Close"
        );
        assert_eq!(
            digest(&closed),
            case["after_sha256"].as_str().unwrap(),
            "{name}"
        );
        assert_eq!(case["enemy_count_after"], 0);
        assert_eq!(case["npc_after"], 0);
        assert_eq!(case["xp_after_close"], 0);
    }
}

#[test]
fn unsupported_item_or_sane_overflow_preserves_randomness() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let warriors = content.warriors.clone();
    let initial = RandomState { seed: 12, draws: 0 };
    let mut enemy = MonsterRecord {
        bytes: vec![0; 288],
    };
    enemy.bytes[0x26..0x28].copy_from_slice(&32767_i16.to_be_bytes());
    let mut random = initial.clone();
    assert!(rewards::prepare(&warriors, &[enemy], &content, 0, 0, 0, &mut random).is_err());
    assert_eq!(random, initial);
    let mut warriors = warriors;
    warriors[0].bytes[0x9d2..0x9d6].copy_from_slice(&i32::MAX.to_be_bytes());
    warriors[0].bytes[0x9df] = 1;
    let mut enemy = MonsterRecord {
        bytes: vec![0; 288],
    };
    enemy.bytes[0x112..0x114].copy_from_slice(&100_i16.to_be_bytes());
    assert!(rewards::prepare(&warriors, &[enemy], &content, 0, 0, 0, &mut random).is_err());
    assert_eq!(random, initial);
}
