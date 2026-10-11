use centurions_core::{RandomState, combat::encounter_battle};

#[test]
fn encounter_battle_selection_matches_original_signed_words_effects_and_random_consumption() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/encounter-battle.json"
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let hex = case["extra"].as_str().unwrap();
        let extra = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let result = encounter_battle(&extra, &mut random).unwrap();
        assert_eq!(result.battle, case["battle"].as_i64().unwrap() as i16);
        assert_eq!(
            result.treasure,
            i16::from_be_bytes(extra[8..10].try_into().unwrap())
        );
        let mut effects = vec![];
        if result.sound != 0 {
            effects.push(serde_json::json!(["sound", result.sound]));
        }
        if result.message != 0 {
            effects.push(serde_json::json!(["message", result.message]));
        }
        assert_eq!(serde_json::json!(effects), case["events"]);
        assert_eq!(
            (random.seed, random.draws),
            (
                case["seed_after"].as_u64().unwrap() as u32,
                case["draws"].as_u64().unwrap()
            )
        );
    }
}
