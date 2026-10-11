use centurions_content::{Warrior, combat::MonsterRecord};
use centurions_core::{RandomState, combat::initialize_turn};
fn decode(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn first_turn_matches_whole_records_and_random_draws() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/turn.json")).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let mut warriors: Vec<_> = decode(case["warriors"].as_str().unwrap())
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|w| Warrior::read(w).unwrap())
            .collect();
        let mut enemies = vec![MonsterRecord {
            bytes: decode(case["enemy"].as_str().unwrap()),
        }];
        let mut random = RandomState::new(1);
        initialize_turn(
            case["body"].as_i64().unwrap() as i16,
            &mut warriors,
            &mut enemies,
            &[12, 14, 12, 14, 10, 12, 10, 10, 14, 16, 0, 12],
            &mut random,
        )
        .unwrap();
        assert_eq!(
            warriors
                .iter()
                .flat_map(|w| w.bytes.clone())
                .collect::<Vec<_>>(),
            decode(case["after_warriors"].as_str().unwrap()),
            "body {}",
            case["body"]
        );
        assert_eq!(
            enemies[0].bytes,
            decode(case["after_enemy"].as_str().unwrap())
        );
        assert_eq!(random.seed, case["seed"].as_u64().unwrap() as u32);
        assert_eq!(random.draws, case["draws"].as_u64().unwrap());
    }
}
