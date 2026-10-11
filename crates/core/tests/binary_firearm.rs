use centurions_content::{Warrior, combat::MonsterRecord};
use centurions_core::{RandomState, combat::firearm};
fn decode(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn reload_matches_original_record_reserve_and_sound_order() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/reload.json")).unwrap();
    for (index, case) in cases["cases"].as_array().unwrap().iter().enumerate() {
        let mut warrior = Warrior::read(&decode(case["input"].as_str().unwrap())).unwrap();
        let mut reserve = case["reserve"].as_i64().unwrap() as i32;
        let mut random = RandomState::new(1);
        let (complete, sounds) = firearm::reload(
            &mut warrior,
            &mut reserve,
            case["silent"].as_i64().unwrap() != 0,
            &mut random,
        )
        .unwrap();
        assert_eq!(
            warrior.bytes,
            decode(case["after"].as_str().unwrap()),
            "warrior {index}"
        );
        assert_eq!(
            reserve,
            case["after_reserve"].as_i64().unwrap() as i32,
            "reserve {index}"
        );
        assert_eq!(
            u64::from(complete),
            case["returned"].as_u64().unwrap(),
            "complete {index}"
        );
        assert_eq!(
            random.seed,
            case["seed"].as_u64().unwrap() as u32,
            "seed {index}"
        );
        assert_eq!(
            random.draws,
            case["draws"].as_u64().unwrap(),
            "draws {index}"
        );
        assert_eq!(
            sounds,
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i16)
                .collect::<Vec<_>>(),
            "sounds {index}"
        );
    }
}
#[test]
fn firearm_preparation_and_resolution_match_original() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/gun-resolve.json")).unwrap();
    let attacks = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../NewCenturions/TheNewCenturions/Data Files/Data S"
    ))
    .unwrap();
    for (index, case) in cases["cases"].as_array().unwrap().iter().enumerate() {
        let mut warrior = Warrior::read(&decode(case["input"].as_str().unwrap())).unwrap();
        let prepared = firearm::prepare(
            &warrior,
            &decode(case["weapon"].as_str().unwrap()),
            &decode(case["ammo"].as_str().unwrap()),
            &attacks,
        )
        .unwrap();
        assert_eq!(
            prepared.packet,
            decode(case["packet"].as_str().unwrap()),
            "packet {index}"
        );
        let mut target = MonsterRecord {
            bytes: decode(case["target"].as_str().unwrap()),
        };
        let mut random = RandomState::new(case["initial_seed"].as_u64().unwrap() as u32);
        let result =
            firearm::resolve(&mut warrior, &mut target, &prepared, &mut random, 0).unwrap();
        assert_eq!(
            warrior.bytes,
            decode(case["after"].as_str().unwrap()),
            "warrior {index}"
        );
        assert_eq!(
            target.bytes,
            decode(case["after_target"].as_str().unwrap()),
            "target {index}"
        );
        assert_eq!(
            random.seed,
            case["seed"].as_u64().unwrap() as u32,
            "seed {index}"
        );
        assert_eq!(
            random.draws,
            case["draws"].as_u64().unwrap(),
            "draws {index}"
        );
        let sounds: Vec<i16> = case["sounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap() as i16)
            .collect();
        assert_eq!(result.sounds, sounds, "sounds {index}");
    }
}

#[test]
fn enemy_chemical_firearms_match_original_whole_records_and_randomness() {
    let fixtures: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/enemy-gun-resolve.json"
    ))
    .unwrap();
    let attacks = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../NewCenturions/TheNewCenturions/Data Files/Data S"
    ))
    .unwrap();
    let mut hits = 0;
    for (index, case) in fixtures["cases"].as_array().unwrap().iter().enumerate() {
        let mut warrior = Warrior::read(&decode(case["input"].as_str().unwrap())).unwrap();
        let mut enemy = MonsterRecord {
            bytes: decode(case["target"].as_str().unwrap()),
        };
        let prepared = firearm::prepare_enemy(
            &mut enemy,
            &decode(case["weapon"].as_str().unwrap()),
            &attacks,
        )
        .unwrap();
        assert_eq!(
            prepared.packet,
            decode(case["packet"].as_str().unwrap()),
            "packet {index}"
        );
        let mut random = RandomState::new(case["initial_seed"].as_u64().unwrap() as u32);
        let result =
            firearm::resolve_enemy(&mut warrior, &mut enemy, &prepared, &mut random, 0).unwrap();
        hits += usize::from(result.damage > 0);
        assert_eq!(
            warrior.bytes,
            decode(case["after"].as_str().unwrap()),
            "warrior {index}"
        );
        assert_eq!(
            enemy.bytes,
            decode(case["after_target"].as_str().unwrap()),
            "enemy {index}"
        );
        assert_eq!(
            random.seed,
            case["seed"].as_u64().unwrap() as u32,
            "seed {index}"
        );
        assert_eq!(
            random.draws,
            case["draws"].as_u64().unwrap(),
            "draws {index}"
        );
        let sounds: Vec<i16> = case["sounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap() as i16)
            .collect();
        assert_eq!(result.sounds, sounds, "sounds {index}");
    }
    assert!(hits > 0);
}
