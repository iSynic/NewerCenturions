use centurions_content::{Warrior, combat::MonsterRecord};
use centurions_core::{RandomState, combat::attack::natural_attack};
fn decode(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
fn assert_bytes(actual: &[u8], expected: &[u8], context: &str) {
    assert_eq!(actual.len(), expected.len(), "{context} length");
    if let Some(index) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "{context} differs at byte {index}: {} != {}",
            actual[index], expected[index]
        );
    }
}
#[test]
fn natural_slow_attack_matches_original_complete_records_and_random_consumption() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/natural-slow.json")).unwrap();
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let mut warrior = Warrior::read(&decode(case["input"].as_str().unwrap())).unwrap();
        let mut target = MonsterRecord {
            bytes: decode(case["target"].as_str().unwrap()),
        };
        let mut random = RandomState::new(case["initial_seed"].as_u64().unwrap() as u32);
        let result = natural_attack(
            &mut warrior,
            &mut target,
            case["attack"].as_u64().unwrap() as usize,
            &mut random,
        )
        .unwrap();
        assert_bytes(
            &warrior.bytes,
            &decode(case["after"].as_str().unwrap()),
            &format!("warrior case {index}"),
        );
        assert_bytes(
            &target.bytes,
            &decode(case["after_target"].as_str().unwrap()),
            &format!("enemy case {index}"),
        );
        assert_eq!(
            random.seed,
            case["seed"].as_u64().unwrap() as u32,
            "seed case {index}"
        );
        assert_eq!(
            random.draws,
            case["draws"].as_u64().unwrap(),
            "draws case {index}"
        );
        assert_eq!(
            result.sounds,
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_i64().unwrap() as i16)
                .collect::<Vec<_>>(),
            "sounds case {index}"
        );
    }
}

#[test]
fn slow_duration_reduction_matches_original_signed_counter_records() {
    use centurions_core::combat::{
        Combat, EnemySetup, SquadPlacement, TacticalMap, runtime::reduce_round,
    };
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/slow-reduction.json"
    ))
    .unwrap();
    let mut warriors = decode(fixture["before"].as_str().unwrap())
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|w| Warrior::read(w).unwrap())
        .collect::<Vec<_>>();
    let mut combat = Combat {
        battle: 8,
        map: TacticalMap {
            tiles: vec![],
            edge_word: 0,
            shift: (0, 0),
        },
        squad: SquadPlacement {
            positions: vec![],
            under_tiles: vec![],
        },
        enemies: EnemySetup {
            records: vec![],
            positions: vec![],
        },
        order: vec![],
        cursor: 0,
        round: 0,
        death_queue: Default::default(),
        spell_fields: vec![0; 1200],
        field_used: vec![0; 128],
        attack_packet: Some(vec![0; 58]),
        attack_identity: Some([0; 3]),
        weapon_work: centurions_core::combat::weapon_work::WeaponWork::zero(),
        unavailable: 0,
        round_scripts: Default::default(),
    };
    let mut random = RandomState::new(12);
    let events = reduce_round(&mut combat, &mut warriors, &mut random).unwrap();
    assert!(events.is_empty());
    assert_eq!(
        warriors
            .iter()
            .flat_map(|w| w.bytes.iter().copied())
            .collect::<Vec<_>>(),
        decode(fixture["after"].as_str().unwrap())
    );
    assert_eq!(random.draws, fixture["rng_draws"].as_u64().unwrap());
}
