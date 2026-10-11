use centurions_content::{Warrior, word};
use centurions_core::story_skill::single_warrior;

fn bytes(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).unwrap())
        .collect()
}

#[test]
fn single_living_warrior_selection_matches_original_gate_and_mask() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/warrior-choice.json"
    ))
    .unwrap();
    for case in fixture["selection_cases"].as_array().unwrap() {
        let selected = case["selected"].as_u64().unwrap() as usize;
        let stamina = case["stamina"].as_i64().unwrap() as i16;
        let mut warriors = vec![
            Warrior {
                name: "Coriand".into(),
                original_sha256: String::new(),
                bytes: vec![0; 3138],
            };
            6
        ];
        warriors[selected].bytes[0x50..0x52].copy_from_slice(&stamina.to_be_bytes());
        let before = warriors.clone();
        let result = single_warrior(&warriors, selected);
        if case["accepted"].as_bool().unwrap() {
            let mask = bytes(case["mask"].as_str().unwrap());
            let expected = std::array::from_fn(|index| word(&mask, index * 2).unwrap());
            assert_eq!(result.unwrap(), expected);
            assert_eq!(&mask[12..], [0; 4]);
        } else {
            assert!(result.is_err());
        }
        assert_eq!(warriors, before);
    }
    let prefix = &fixture["prefix"];
    assert!(
        bytes(prefix["after"].as_str().unwrap())
            .iter()
            .all(|value| *value == 0)
    );
    assert_eq!(prefix["remaining"], 1);
    for case in fixture["cases"].as_array().unwrap() {
        assert_eq!(case["parameter"], 1);
        assert_eq!(case["count_minus_one"], 0);
        assert_eq!(case["presentation_mode"], 0);
        assert_eq!(case["include_dead"], 0);
        assert_eq!(case["local_mode"], 0);
        assert_eq!(case["calls"], 1);
        assert_eq!(case["cursor"], 3);
    }
    assert_eq!(fixture["rng_draws"], 0);
}
