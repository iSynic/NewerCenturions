use centurions_content::{Warrior, combat::MonsterRecord};
use centurions_core::{
    RandomState,
    combat::attack::{natural_attack, natural_attack_with_packet},
};

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn required_parts_ranged_to_natural_fallback_matches_original_full_records_and_randomness() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/parts-natural.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 432);
    for (index, case) in cases.iter().enumerate() {
        let mut party = bytes(case["warriors_before"].as_str().unwrap());
        let member = case["member"].as_u64().unwrap() as usize;
        let mut warrior = Warrior::read(&party[member * 3138..(member + 1) * 3138]).unwrap();
        let mut enemy = MonsterRecord {
            bytes: bytes(case["enemy_before"].as_str().unwrap()),
        };
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let (result, packet) = natural_attack_with_packet(
            &mut warrior,
            &mut enemy,
            case["attack"].as_u64().unwrap() as usize,
            &mut random,
            &std::fs::read(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../NewCenturions/TheNewCenturions/Data Files/Data S"
            ))
            .unwrap(),
            &[0; 58],
        )
        .unwrap();
        assert_eq!(
            packet,
            bytes(case["packet"].as_str().unwrap()),
            "case{index} packet"
        );
        party[member * 3138..(member + 1) * 3138].copy_from_slice(&warrior.bytes);
        assert_eq!(
            party,
            bytes(case["warriors_after"].as_str().unwrap()),
            "case{index} party"
        );
        assert_eq!(
            enemy.bytes,
            bytes(case["enemy_after"].as_str().unwrap()),
            "case{index} enemy"
        );
        assert_eq!(
            random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "case{index} seed"
        );
        assert_eq!(
            random.draws,
            case["rng_draws"].as_u64().unwrap(),
            "case{index} draws"
        );
        assert_eq!(
            result.sounds,
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i16)
                .collect::<Vec<_>>(),
            "case{index} sounds"
        );
    }
}

#[test]
fn unrecovered_fallback_target_preserves_current_weapon_and_randomness() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/parts-natural.json"
    ))
    .unwrap();
    let case = &fixture["cases"][0];
    let raw = bytes(case["warriors_before"].as_str().unwrap());
    let mut warrior = Warrior::read(&raw[..3138]).unwrap();
    warrior.bytes[0x9c..0x9e].copy_from_slice(&1_i16.to_be_bytes());
    let mut enemy = MonsterRecord {
        bytes: bytes(case["enemy_before"].as_str().unwrap()),
    };
    let mut random = RandomState::new(12);
    let before = (warrior.clone(), enemy.clone(), random.clone());
    assert!(natural_attack(&mut warrior, &mut enemy, 0, &mut random).is_err());
    assert_eq!(warrior, before.0);
    assert_eq!(enemy, before.1);
    assert_eq!(random, before.2);
}
