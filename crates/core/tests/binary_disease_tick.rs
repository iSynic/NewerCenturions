use centurions_core::combat::natural_conditions::disease_tick;

fn decode(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn disease_tick_matches_original_signed_guards_health_and_kill_decision() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/disease-tick.json")).unwrap();
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let mut warrior = decode(case["input"].as_str().unwrap());
        let tick = disease_tick(&warrior).unwrap();
        if let Some(tick) = tick {
            warrior[0x50..0x52].copy_from_slice(&tick.health.to_be_bytes());
        }
        assert_eq!(
            warrior,
            decode(case["after"].as_str().unwrap()),
            "case {index}"
        );
        assert_eq!(
            tick.is_some_and(|t| t.killed),
            case["killed"].as_bool().unwrap(),
            "kill case {index}"
        );
    }
}
