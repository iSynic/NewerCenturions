use centurions_content::{Warrior, combat::MonsterRecord};
use centurions_core::{RandomState, combat::spell_conditions};

fn decode(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

fn patches(bytes: &mut [u8], values: &serde_json::Value) {
    for change in values.as_array().unwrap() {
        bytes[change[0].as_u64().unwrap() as usize] = change[1].as_u64().unwrap() as u8;
    }
}

fn assert_bytes(actual: &[u8], expected: &[u8], case: usize, name: &str) {
    assert_eq!(actual.len(), expected.len());
    if let Some(index) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "case {case} {name} byte {index}: {} != {}",
            actual[index], expected[index]
        );
    }
}

#[test]
fn effect_36_matches_original_all_body_slots_and_ordered_random_consumption() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/spell-conditions.json"
    ))
    .unwrap();
    let party = decode(fixture["party"].as_str().unwrap());
    let monsters = decode(fixture["monsters"].as_str().unwrap());
    let packet = decode(fixture["packet"].as_str().unwrap());
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let mut before = party.clone();
        let mut records = monsters.clone();
        patches(&mut before, &case["party_input"]);
        patches(&mut records, &case["monster_input"]);
        let warriors = before
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|w| Warrior::read(w).unwrap())
            .collect::<Vec<_>>();
        let enemies = records
            .as_chunks::<288>()
            .0
            .iter()
            .map(|m| MonsterRecord { bytes: m.to_vec() })
            .collect::<Vec<_>>();
        let mask = case["mask"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap() as i16)
            .collect::<Vec<_>>();
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let result = spell_conditions::resolve_1703(
            &warriors,
            &enemies,
            &packet,
            case["count"].as_i64().unwrap() as i16,
            &mask,
            &mut random,
        )
        .unwrap();
        let mut expected = before;
        patches(&mut expected, &case["party_changes"]);
        assert_bytes(
            &result
                .warriors
                .iter()
                .flat_map(|w| w.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &expected,
            index,
            "party",
        );
        let mut expected = records;
        patches(&mut expected, &case["monster_changes"]);
        assert_bytes(
            &result
                .enemies
                .iter()
                .flat_map(|m| m.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &expected,
            index,
            "roster",
        );
        assert_eq!(
            result.duration,
            case["duration"].as_i64().unwrap() as i16,
            "case {index} duration"
        );
        assert_eq!(
            random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "case {index} seed"
        );
        assert_eq!(
            random.draws,
            case["rng_draws"].as_u64().unwrap(),
            "case {index} draws"
        );
        assert_eq!(
            result.sounds,
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i16)
                .collect::<Vec<_>>(),
            "case {index} sounds"
        );
    }
}

#[test]
fn unsupported_conditions_preserve_target_and_randomness() {
    let mut record = vec![0x5a; 3138];
    let before = record.clone();
    let mut random = RandomState::new(12);
    let seed = random.clone();
    assert!(spell_conditions::apply(&mut record, false, 37, 3, &mut random).is_err());
    assert_eq!(record, before);
    assert_eq!(random, seed);
}
