use centurions_core::{RandomState, story_allies::add_ally};
fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn joined_biologists_match_complete_original_records_and_rng() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/story-allies.json")).unwrap();
    let definition = bytes(fixture["definition"].as_str().unwrap());
    for case in fixture["cases"].as_array().unwrap() {
        let mut records = bytes(case["before"].as_str().unwrap());
        let mut count = case["count_before"].as_u64().unwrap() as usize;
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let mut warnings = Vec::new();
        for _ in 0..case["calls"].as_u64().unwrap() {
            let result = add_ally(
                &records,
                count,
                &definition,
                case["difficulty"].as_i64().unwrap() as i16,
                &mut random,
            )
            .unwrap();
            records = result.records;
            count = result.count;
            if let Some(warning) = result.warning {
                warnings.push(warning);
            }
        }
        assert_eq!(
            records,
            bytes(case["after"].as_str().unwrap()),
            "case count {} seed {}",
            case["count_before"],
            case["seed_before"]
        );
        assert_eq!(count, case["count_after"].as_u64().unwrap() as usize);
        assert_eq!(random.seed, case["seed_after"].as_u64().unwrap() as u32);
        assert_eq!(random.draws, case["rng_draws"].as_u64().unwrap());
        assert_eq!(
            warnings,
            case["warnings"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i16)
                .collect::<Vec<_>>()
        );
    }
}
