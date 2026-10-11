use centurions_core::story::{
    ConditionalJump, add_mission_score, flag_condition_jump, set_mission_flag,
};

fn bytes(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!("../../../evidence/fixtures/story-flags.json")).unwrap()
}

#[test]
fn mission_flag_mutations_match_complete_original_flag_table() {
    for case in fixture()["set_cases"].as_array().unwrap() {
        let flags = bytes(case["before"].as_str().unwrap());
        let parameter = case["parameter"].as_i64().unwrap() as i16;
        assert_eq!(
            set_mission_flag(&flags, parameter).unwrap(),
            bytes(case["after"].as_str().unwrap())
        );
        assert!(set_mission_flag(&flags, i16::MIN).is_err());
    }
}

#[test]
fn mission_flag_condition_matches_original_branch_decision() {
    for case in fixture()["condition_cases"].as_array().unwrap() {
        let mut flags = vec![0; 128];
        let index = case["index"].as_i64().unwrap() as i16;
        flags[index as usize] = case["flag"].as_u64().unwrap() as u8;
        let extra: Vec<u8> = [index, case["mode"].as_i64().unwrap() as i16, 3, 0, 0]
            .into_iter()
            .flat_map(i16::to_be_bytes)
            .collect();
        assert_eq!(
            flag_condition_jump(&flags, &extra).unwrap(),
            case["matched"]
                .as_bool()
                .unwrap()
                .then_some(ConditionalJump::EndMenu)
        );
    }
}

#[test]
fn mission_score_addition_matches_original_word_arithmetic() {
    for case in fixture()["score_cases"].as_array().unwrap() {
        assert_eq!(
            add_mission_score(
                case["before"].as_i64().unwrap() as i16,
                case["parameter"].as_i64().unwrap() as i16,
            ),
            case["after"].as_i64().unwrap() as i16
        );
    }
}
