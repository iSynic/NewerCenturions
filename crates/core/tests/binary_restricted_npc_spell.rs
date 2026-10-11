use centurions_content::{Warrior, combat::MonsterRecord, word};
use centurions_core::{
    RandomState, RuleError,
    combat::spells::{
        BodySpellResult, DirectSpell, FieldUsage, resolve_bodies, resolve_bodies_with_item_cache,
        resolve_collision, resolve_collision_with_item_cache,
    },
};
use serde_json::Value;

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&hex[offset..offset + 2], 16).unwrap())
        .collect()
}

fn words(hex: &str) -> Vec<i16> {
    bytes(hex)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|value| i16::from_be_bytes(*value))
        .collect()
}

fn fixture() -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../evidence/fixtures/restricted-npc-spell.json"),
        )
        .unwrap(),
    )
    .unwrap()
}

fn party(hex: &str) -> Vec<Warrior> {
    bytes(hex)
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|record| Warrior::read(record).unwrap())
        .collect()
}

fn enemies(hex: &str) -> Vec<MonsterRecord> {
    bytes(hex)
        .as_chunks::<288>()
        .0
        .iter()
        .map(|record| MonsterRecord {
            bytes: record.to_vec(),
        })
        .collect()
}

fn random(snapshot: &Value) -> RandomState {
    RandomState {
        seed: snapshot["seed"].as_u64().unwrap() as u32,
        draws: snapshot["draws"].as_u64().unwrap(),
    }
}

fn assert_bytes(actual: &[u8], expected: &[u8], context: &str) {
    assert_eq!(actual.len(), expected.len(), "{context} length");
    if let Some(offset) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "{context} byte {offset:x}: {} != {}",
            actual[offset], expected[offset]
        );
    }
}

fn assert_records(result: &BodySpellResult, party_hex: &str, monster_hex: &str) {
    assert_bytes(
        &result
            .warriors
            .iter()
            .flat_map(|w| w.bytes.iter().copied())
            .collect::<Vec<_>>(),
        &bytes(party_hex),
        "complete native party",
    );
    assert_bytes(
        &result
            .enemies
            .iter()
            .flat_map(|m| m.bytes.iter().copied())
            .collect::<Vec<_>>(),
        &bytes(monster_hex),
        "complete native monsters",
    );
}

fn audio(result: &BodySpellResult) -> Vec<i16> {
    let mut ordered = result
        .targets
        .iter()
        .map(|(body, outcome)| (*body, outcome.sounds.clone()))
        .collect::<Vec<_>>();
    ordered.extend(result.race_excluded.iter().map(|body| (*body, vec![651])));
    ordered.sort_by_key(|(body, _)| *body);
    result
        .sounds
        .iter()
        .copied()
        .chain(ordered.into_iter().flat_map(|(_, sounds)| sounds))
        .collect()
}

#[test]
fn npc18_restriction_and_native_immunity_match_three_complete_resolver_cases() {
    let fixture = fixture();
    let source = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../NewCenturions/TheNewCenturions/Data Files/Data ID"),
    )
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 4);
    for (index, case) in cases.iter().enumerate().skip(1) {
        assert!(!case["complete_turn"].as_bool().unwrap());
        let before = &case["before"];
        let after = &case["after"];
        let warriors = party(before["party"].as_str().unwrap());
        let monsters = enemies(before["records"].as_str().unwrap());
        let packet = bytes(before["packet"].as_str().unwrap());
        let cache = bytes(before["first_item_cache"].as_str().unwrap());
        assert_bytes(&cache, &source[..30_000], "authentic first item cache");
        assert_eq!(
            centurions_content::digest(&cache),
            before["first_item_cache_digest"]
        );
        assert_eq!(word(&cache, 81 * 150 + 2).unwrap(), 0);
        assert_eq!(monsters[8].bytes[0x90], 101);
        let mask = words(before["mask"].as_str().unwrap());
        let context = DirectSpell {
            packet: &packet,
            level: 1,
            count: 3,
            armor_filter: false,
            weapon_sound: 0,
        };
        let mut actual = random(before);
        let result = resolve_bodies_with_item_cache(
            &warriors,
            &monsters,
            &mask,
            &context,
            &cache,
            &mut actual,
        )
        .unwrap();
        assert_records(
            &result,
            after["party"].as_str().unwrap(),
            after["records"].as_str().unwrap(),
        );
        assert!(result.targets.is_empty());
        assert_eq!(
            result.race_excluded,
            if index == 3 { vec![18] } else { vec![] }
        );
        assert_eq!(
            audio(&result),
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i16)
                .collect::<Vec<_>>()
        );
        assert_eq!(actual, random(after));
        let mut receipts = random(before);
        for receipt in case["rng"].as_array().unwrap() {
            assert_eq!(
                receipts.bound(receipt["bound"].as_i64().unwrap() as i16),
                receipt["result"].as_i64().unwrap() as i16
            );
        }
        assert_eq!(receipts, actual);
        assert_eq!(actual.draws - random(before).draws, 1);
        for (key, value) in before.as_object().unwrap() {
            if !matches!(key.as_str(), "seed" | "draws") {
                assert_eq!(value, &after[key], "native unchanged {key} case{index}");
            }
        }
        let mut usage = FieldUsage {
            slot: 7,
            used: bytes(before["used"].as_str().unwrap()),
        };
        usage.used[8] = 1;
        let original_usage = usage.used.clone();
        let mut collision_random = random(before);
        let collision = resolve_collision_with_item_cache(
            &warriors,
            &monsters,
            &mask,
            &context,
            &mut usage,
            &cache,
            &mut collision_random,
        )
        .unwrap();
        assert_records(
            &collision,
            after["party"].as_str().unwrap(),
            after["records"].as_str().unwrap(),
        );
        assert_eq!(collision.race_excluded, result.race_excluded);
        assert_eq!(audio(&collision), audio(&result));
        assert_eq!(collision_random, actual);
        assert_eq!(usage.used, original_usage);
    }
}

#[test]
fn reached_caller_all_four_resolutions_match_native_party_monsters_and_rng() {
    let fixture = fixture();
    let case = &fixture["cases"][0];
    assert!(case["complete_turn"].as_bool().unwrap());
    let before = &case["before"];
    let cache = bytes(before["first_item_cache"].as_str().unwrap());
    let mut warriors = party(before["party"].as_str().unwrap());
    let mut receipt_random = random(before);
    let receipts = case["rng"].as_array().unwrap();
    let mut receipt_index = 0;
    let mut impacts = vec![];
    let resolutions = case["resolutions"].as_array().unwrap();
    assert_eq!(resolutions.len(), 4);
    for resolution in resolutions {
        while receipt_random.draws < resolution["draws_before"].as_u64().unwrap() {
            let receipt = &receipts[receipt_index];
            receipt_index += 1;
            assert_eq!(
                receipt_random.bound(receipt["bound"].as_i64().unwrap() as i16),
                receipt["result"].as_i64().unwrap() as i16
            );
        }
        let mut actual_random = receipt_random.clone();
        let packet = bytes(resolution["packet"].as_str().unwrap());
        let mask = words(resolution["mask"].as_str().unwrap());
        // Caller charging has already supplied these actor records at this boundary.
        let monsters = enemies(resolution["records_after"].as_str().unwrap());
        let result = resolve_bodies_with_item_cache(
            &warriors,
            &monsters,
            &mask,
            &DirectSpell {
                packet: &packet,
                level: resolution["spell_id"].as_i64().unwrap() as i16 % 1000 / 100 - 1,
                count: resolution["count"].as_i64().unwrap() as i16,
                armor_filter: false,
                weapon_sound: 402,
            },
            &cache,
            &mut actual_random,
        )
        .unwrap();
        while receipt_random.draws < resolution["draws_after"].as_u64().unwrap() {
            let receipt = &receipts[receipt_index];
            receipt_index += 1;
            assert_eq!(
                receipt_random.bound(receipt["bound"].as_i64().unwrap() as i16),
                receipt["result"].as_i64().unwrap() as i16
            );
        }
        assert_eq!(actual_random, receipt_random);
        assert_records(
            &result,
            resolution["party_after"].as_str().unwrap(),
            resolution["records_after"].as_str().unwrap(),
        );
        let applied = words(resolution["applied_mask"].as_str().unwrap());
        assert_eq!(
            result
                .targets
                .iter()
                .map(|(body, _)| *body)
                .collect::<Vec<_>>(),
            applied
                .iter()
                .enumerate()
                .filter_map(|(body, flag)| (*flag > 0).then_some(body))
                .collect::<Vec<_>>()
        );
        assert!(result.race_excluded.is_empty());
        impacts.extend(audio(&result));
        warriors = result.warriors;
    }
    assert_eq!(impacts, [204, 204, 206]);
    assert_eq!(receipt_index, receipts.len());
    assert_eq!(receipt_random, random(&case["after"]));
    assert_bytes(
        &warriors
            .iter()
            .flat_map(|w| w.bytes.iter().copied())
            .collect::<Vec<_>>(),
        &bytes(case["after"]["party"].as_str().unwrap()),
        "final native resolver party",
    );
    assert_eq!(
        case["before"]["first_item_cache"],
        case["after"]["first_item_cache"]
    );
}

#[test]
fn absent_cache_unknown_aliases_and_unowned_bit_indices_remain_atomic_diagnostics() {
    let fixture = fixture();
    let before = &fixture["cases"][1]["before"];
    let warriors = party(before["party"].as_str().unwrap());
    let monsters = enemies(before["records"].as_str().unwrap());
    let packet = bytes(before["packet"].as_str().unwrap());
    let cache = bytes(before["first_item_cache"].as_str().unwrap());
    let base_mask = words(before["mask"].as_str().unwrap());
    let context = DirectSpell {
        packet: &packet,
        level: 1,
        count: 3,
        armor_filter: false,
        weapon_sound: 0,
    };
    let original_random = random(before);
    let mut actual = original_random.clone();
    assert!(matches!(
        resolve_bodies(&warriors, &monsters, &base_mask, &context, &mut actual),
        Err(RuleError::Unsupported { .. })
    ));
    assert_eq!(actual, original_random);
    let mut usage = FieldUsage {
        slot: 3,
        used: vec![0; 128],
    };
    let original_usage = usage.used.clone();
    assert!(matches!(
        resolve_collision(
            &warriors,
            &monsters,
            &base_mask,
            &context,
            &mut usage,
            &mut actual
        ),
        Err(RuleError::Unsupported { .. })
    ));
    assert_eq!(actual, original_random);
    assert_eq!(usage.used, original_usage);
    for malformed in [vec![], vec![0; 29_999], vec![0; 30_001]] {
        assert!(matches!(
            resolve_collision_with_item_cache(
                &warriors,
                &monsters,
                &base_mask,
                &context,
                &mut usage,
                &malformed,
                &mut actual
            ),
            Err(RuleError::Unsupported { .. })
        ));
        assert_eq!(actual, original_random);
        assert_eq!(usage.used, original_usage);
    }
    for body in [10, 14, 24, 109] {
        let mut mask = vec![0; 110];
        mask[0] = 1;
        mask[body] = 1;
        assert!(matches!(
            resolve_collision_with_item_cache(
                &warriors,
                &monsters,
                &mask,
                &context,
                &mut usage,
                &cache,
                &mut actual
            ),
            Err(RuleError::Unsupported { .. })
        ));
        assert_eq!(actual, original_random);
        assert_eq!(usage.used, original_usage);
    }
    for race in [-400, 65, i16::MIN, i16::MAX] {
        let mut invalid_cache = cache.clone();
        invalid_cache[81 * 150 + 2..81 * 150 + 4].copy_from_slice(&race.to_be_bytes());
        let mut mask = base_mask.clone();
        mask[0] = 1;
        assert!(matches!(
            resolve_collision_with_item_cache(
                &warriors,
                &monsters,
                &mask,
                &context,
                &mut usage,
                &invalid_cache,
                &mut actual
            ),
            Err(RuleError::Unsupported { .. })
        ));
        assert_eq!(actual, original_random);
        assert_eq!(usage.used, original_usage);
    }
    assert_eq!(cache, bytes(before["first_item_cache"].as_str().unwrap()));
    assert_bytes(
        &warriors
            .iter()
            .flat_map(|w| w.bytes.iter().copied())
            .collect::<Vec<_>>(),
        &bytes(before["party"].as_str().unwrap()),
        "unchanged rejected party",
    );
    assert_bytes(
        &monsters
            .iter()
            .flat_map(|m| m.bytes.iter().copied())
            .collect::<Vec<_>>(),
        &bytes(before["records"].as_str().unwrap()),
        "unchanged rejected monsters",
    );
}
